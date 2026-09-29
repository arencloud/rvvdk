#![cfg(target_os = "linux")]
use serde_json::Value;
use std::{
    ffi::OsString,
    fs::{self, File},
    os::unix::{
        fs::{FileExt, MetadataExt, symlink},
        process::CommandExt,
    },
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
struct Fixture {
    root: PathBuf,
    source: PathBuf,
    destination: PathBuf,
}
impl Fixture {
    fn new(size: usize) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::var_os("RVVDK_TEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!(
                "rvddk-transfer-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source");
        let destination = root.join("destination");
        fs::write(&source, vec![0x5a; size]).unwrap();
        Self {
            root,
            source,
            destination,
        }
    }
    fn args(&self, command: &str, extras: &[&str]) -> Vec<OsString> {
        let mut a = vec![
            "rvddk".into(),
            command.into(),
            self.source.clone().into_os_string(),
            self.destination.clone().into_os_string(),
            "--format".into(),
            "raw".into(),
            "--json".into(),
        ];
        a.extend(extras.iter().map(OsString::from));
        a
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn run(args: Vec<OsString>) -> (i32, Value) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = rvvdk_cli::run(args, &mut out, &mut err);
    let value = serde_json::from_slice(if code == 0 {
        &out
    } else {
        assert!(out.is_empty());
        &err
    })
    .unwrap();
    (code, value)
}
#[test]
fn new_and_overwrite_copies_verify_empty_odd_and_sparse_images_across_backends() {
    for backend in ["threaded", "auto", "io-uring"] {
        for size in [0, 7, 16391] {
            let f = Fixture::new(size);
            if size > 4096 {
                let file = File::options().write(true).open(&f.source).unwrap();
                file.set_len(65543).unwrap();
                file.write_all_at(&[0x33; 7], 65536).unwrap();
            }
            let expected = fs::read(&f.source).unwrap();
            let (code, report) = run(f.args(
                "copy",
                &[
                    "--backend",
                    backend,
                    "--verify",
                    "--block-size",
                    "4096",
                    "--workers",
                    "4",
                ],
            ));
            assert_eq!(code, 0, "{report}");
            assert_eq!(report["status"], "completed");
            assert_eq!(report["verification"]["bytes_verified"], expected.len());
            assert_eq!(report["durability"], "file_and_directory_synced");
            assert_eq!(fs::read(&f.destination).unwrap(), expected);
            assert_eq!(fs::metadata(&f.destination).unwrap().mode() & 0o077, 0);
            let (code, report) = run(f.args("verify", &["--block-size", "3"]));
            assert_eq!(code, 0, "{report}");
            fs::write(&f.destination, vec![0xa5; expected.len() + 9]).unwrap();
            let inode = fs::metadata(&f.destination).unwrap().ino();
            let (code, report) =
                run(f.args("copy", &["--overwrite", "--verify", "--backend", backend]));
            assert_eq!(code, 0, "{report}");
            assert_eq!(report["destination_tail_bytes"], 9);
            assert_eq!(report["durability"], "file_synced");
            let actual = fs::read(&f.destination).unwrap();
            assert_eq!(&actual[..expected.len()], &expected);
            assert_eq!(&actual[expected.len()..], &[0xa5; 9]);
            assert_eq!(fs::metadata(&f.destination).unwrap().ino(), inode);
        }
    }
}
#[test]
fn no_clobber_alias_symlink_and_capacity_rejections_preserve_contents() {
    let f = Fixture::new(4096);
    fs::write(&f.destination, [0xa5; 7]).unwrap();
    assert_eq!(
        run(f.args("copy", &[])).1["error"]["code"],
        "destination_exists"
    );
    assert_eq!(
        run(f.args("copy", &["--overwrite"])).1["error"]["code"],
        "destination_too_small"
    );
    assert_eq!(fs::read(&f.destination).unwrap(), [0xa5; 7]);
    fs::remove_file(&f.destination).unwrap();
    fs::hard_link(&f.source, &f.destination).unwrap();
    assert_eq!(
        run(f.args("copy", &["--overwrite"])).1["error"]["code"],
        "same_file"
    );
    fs::remove_file(&f.destination).unwrap();
    symlink(&f.source, &f.destination).unwrap();
    assert_eq!(
        run(f.args("copy", &["--overwrite"])).1["error"]["code"],
        "invalid_destination"
    );
    assert_eq!(fs::read(&f.source).unwrap(), vec![0x5a; 4096]);
}
#[test]
fn verification_checks_holes_and_first_mismatch_without_mutation() {
    let f = Fixture::new(8199);
    File::options()
        .write(true)
        .open(&f.source)
        .unwrap()
        .set_len(16391)
        .unwrap();
    fs::copy(&f.source, &f.destination).unwrap();
    File::options()
        .write(true)
        .open(&f.destination)
        .unwrap()
        .write_all_at(&[0x44], 12000)
        .unwrap();
    let before = fs::read(&f.destination).unwrap();
    let (code, error) = run(f.args("verify", &["--block-size", "4096"]));
    assert_eq!(code, 1);
    assert_eq!(error["error"]["code"], "verification_mismatch");
    assert_eq!(error["error"]["details"]["mismatch_offset"], 12000);
    assert_eq!(fs::read(&f.destination).unwrap(), before);
}
#[test]
fn verification_buffers_and_copy_budget_are_jointly_admitted() {
    let f = Fixture::new(16384);
    let (code, error) = run(f.args(
        "copy",
        &[
            "--verify",
            "--block-size",
            "4096",
            "--memory-budget",
            "8192",
        ],
    ));
    assert_eq!(code, 1, "{error}");
    assert!(!f.destination.exists());
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
    fs::write(&f.destination, vec![0xa5; 16384]).unwrap();
    let (code, error) = run(f.args(
        "copy",
        &[
            "--overwrite",
            "--verify",
            "--block-size",
            "4096",
            "--memory-budget",
            "10000",
        ],
    ));
    assert_eq!(code, 1, "{error}");
    assert_eq!(fs::read(&f.destination).unwrap(), vec![0xa5; 16384]);
}
#[test]
fn runtime_ring_denial_auto_falls_back_but_explicit_native_does_not_publish() {
    for backend in ["auto", "io-uring"] {
        let f = Fixture::new(8199);
        let args = f.args("copy", &["--backend", backend, "--verify"]);
        let mut command = Command::new(env!("CARGO_BIN_EXE_rvddk"));
        command.args(&args[1..]);
        // SAFETY: child-only fixed filter; no allocation or locks after fork.
        unsafe {
            command.pre_exec(|| {
                let mut filter = [
                    libc::sock_filter {
                        code: 0x20,
                        jt: 0,
                        jf: 0,
                        k: 0,
                    },
                    libc::sock_filter {
                        code: 0x15,
                        jt: 0,
                        jf: 1,
                        k: libc::SYS_io_uring_setup as u32,
                    },
                    libc::sock_filter {
                        code: 0x06,
                        jt: 0,
                        jf: 0,
                        k: libc::SECCOMP_RET_ERRNO | libc::EPERM as u32,
                    },
                    libc::sock_filter {
                        code: 0x06,
                        jt: 0,
                        jf: 0,
                        k: libc::SECCOMP_RET_ALLOW,
                    },
                ];
                let program = libc::sock_fprog {
                    len: filter.len() as u16,
                    filter: filter.as_mut_ptr(),
                };
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
                    || libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let output = bounded(&mut command);
        if backend == "auto" {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(report["backend"], "threaded");
            assert_eq!(report["runtime_fallback"]["os_error"], libc::EPERM);
            assert_eq!(
                fs::read(&f.destination).unwrap(),
                fs::read(&f.source).unwrap()
            );
        } else {
            assert!(!output.status.success());
            assert!(!f.destination.exists());
        }
    }
}

fn bounded(command: &mut Command) -> std::process::Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("CLI timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn report_failure_retains_completed_operation_and_published_bytes() {
    struct BrokenOutput;
    impl std::io::Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(libc::ENOSPC))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let f = Fixture::new(8199);
    let mut err = Vec::new();
    assert_eq!(
        rvvdk_cli::run(f.args("copy", &["--verify"]), &mut BrokenOutput, &mut err),
        1
    );
    let report: Value = serde_json::from_slice(&err).unwrap();
    assert_eq!(report["error"]["details"]["operation_completed"], true);
    assert_eq!(
        report["error"]["details"]["result"]["durability"],
        "file_and_directory_synced"
    );
    assert_eq!(
        fs::read(&f.destination).unwrap(),
        fs::read(&f.source).unwrap()
    );
}

struct CancellingProgress {
    bytes: Vec<u8>,
    token: rvvdk_datamover::CancellationToken,
    phase: &'static str,
}
impl std::io::Write for CancellingProgress {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(b);
        if String::from_utf8_lossy(&self.bytes).contains(&format!("\"phase\":\"{}\"", self.phase)) {
            self.token.cancel();
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn progress_cancellation_preserves_private_and_published_destination_states() {
    for phase in [
        "planning",
        "copy_started",
        "copying",
        "copy_flushing",
        "verifying",
        "file_sync",
        "publication",
        "directory_sync",
    ] {
        let f = Fixture::new(1024 * 1024 + 7);
        let token = rvvdk_datamover::CancellationToken::new();
        let mut err = CancellingProgress {
            bytes: Vec::new(),
            token: token.clone(),
            phase,
        };
        let mut out = Vec::new();
        let code = rvvdk_cli::run_with_cancellation(
            f.args("copy", &["--verify", "--progress", "--block-size", "4096"]),
            &mut out,
            &mut err,
            &token,
        );
        assert_eq!(
            code,
            130,
            "{phase}: {}",
            String::from_utf8_lossy(&err.bytes)
        );
        assert!(out.is_empty());
        let events: Vec<Value> = String::from_utf8(err.bytes)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(events.last().unwrap()["error"]["code"], "cancelled");
        assert!(!events.iter().any(|e| e["phase"] == "completed"));
        assert_eq!(f.destination.exists(), phase == "directory_sync");
        if phase == "directory_sync" {
            assert_eq!(
                events.last().unwrap()["error"]["details"]["destination_state"],
                "published"
            );
            assert_eq!(
                fs::read(&f.source).unwrap(),
                fs::read(&f.destination).unwrap()
            );
        }
        if phase == "verifying" {
            assert!(
                events.last().unwrap()["error"]["details"]["bytes_verified"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
        }
    }
}
#[test]
fn lifecycle_json_is_separate_from_success_and_actual_backend_is_reported() {
    for backend in ["threaded", "io-uring"] {
        let f = Fixture::new(1024 * 1024 + 7);
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            rvvdk_cli::run(
                f.args(
                    "copy",
                    &[
                        "--progress",
                        "--verify",
                        "--backend",
                        backend,
                        "--block-size",
                        "4096",
                        "--workers",
                        "4"
                    ]
                ),
                &mut out,
                &mut err
            ),
            0
        );
        let report: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(report["backend"], backend);
        let events: Vec<Value> = String::from_utf8(err)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(events.last().unwrap()["phase"], "completed");
        let mut previous = 0;
        for (i, e) in events.iter().enumerate() {
            assert_eq!(e["sequence"], i + 1);
            let n = e["logical_bytes_processed"].as_u64().unwrap();
            assert!(n >= previous);
            previous = n;
        }
        assert!(
            events
                .iter()
                .position(|e| e["phase"] == "copy_flushed")
                .unwrap()
                < events
                    .iter()
                    .position(|e| e["phase"] == "publication")
                    .unwrap()
        );
        assert_eq!(events.last().unwrap()["bytes_verified"], 1024 * 1024 + 7);
    }
}
#[test]
fn cancelled_overwrite_keeps_inode_tail_and_reports_partial_effects() {
    let f = Fixture::new(1024 * 1024 + 7);
    fs::write(&f.destination, vec![0xa5; 1024 * 1024 + 16]).unwrap();
    let inode = fs::metadata(&f.destination).unwrap().ino();
    let token = rvvdk_datamover::CancellationToken::new();
    let mut err = CancellingProgress {
        bytes: Vec::new(),
        token: token.clone(),
        phase: "copying",
    };
    assert_eq!(
        rvvdk_cli::run_with_cancellation(
            f.args(
                "copy",
                &["--overwrite", "--progress", "--block-size", "4096"]
            ),
            &mut Vec::new(),
            &mut err,
            &token
        ),
        130
    );
    let bytes = fs::read(&f.destination).unwrap();
    assert_eq!(&bytes[1024 * 1024 + 7..], &[0xa5; 9]);
    assert_eq!(fs::metadata(&f.destination).unwrap().ino(), inode);
    assert!(bytes[..4096].iter().all(|b| *b == 0x5a));
    let text = String::from_utf8(err.bytes).unwrap();
    let error: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(
        error["error"]["details"]["destination_state"],
        "existing_may_be_modified"
    );
}
#[test]
fn progress_output_failure_stops_before_publication() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(libc::ENOSPC))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let f = Fixture::new(8199);
    assert_eq!(
        rvvdk_cli::run(
            f.args("copy", &["--progress"]),
            &mut Vec::new(),
            &mut Broken
        ),
        1
    );
    assert!(!f.destination.exists());
}
#[test]
fn sigint_and_sigterm_cancel_binary_copy_and_verify_with_bounded_shutdown() {
    for (command, backend, signal, code, phase) in [
        ("copy", "threaded", libc::SIGINT, 130, "copying"),
        ("copy", "io-uring", libc::SIGTERM, 143, "copying"),
        ("verify", "threaded", libc::SIGTERM, 143, "verifying"),
    ] {
        let f = Fixture::new(16 * 1024 * 1024 + 7);
        if command == "verify" {
            fs::copy(&f.source, &f.destination).unwrap();
        }
        let stdout = f.root.join("out");
        let stderr = f.root.join("err");
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_rvddk"));
        let mut args = f.args(command, &["--progress", "--block-size", "512"]);
        if command == "copy" {
            args.extend(["--backend".into(), backend.into()]);
        }
        let mut child = cmd
            .args(&args[1..])
            .stdout(File::create(&stdout).unwrap())
            .stderr(File::create(&stderr).unwrap())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut sent = false;
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if !sent
                && fs::read_to_string(&stderr)
                    .unwrap()
                    .contains(&format!("\"phase\":\"{phase}\""))
            {
                // SAFETY: send a cancellation signal to this owned, live child.
                assert_eq!(unsafe { libc::kill(child.id() as i32, signal) }, 0);
                sent = true;
            }
            if Instant::now() > deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("signal cancellation timed out");
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        assert!(
            sent,
            "child exited before signal: {}",
            fs::read_to_string(&stderr).unwrap()
        );
        assert_eq!(
            status.code(),
            Some(code),
            "{}",
            fs::read_to_string(&stderr).unwrap()
        );
        assert!(fs::read(&stdout).unwrap().is_empty());
        let text = fs::read_to_string(&stderr).unwrap();
        let error: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
        assert_eq!(error["error"]["code"], "cancelled");
        assert_eq!(f.destination.exists(), command == "verify");
    }
}
#[test]
fn sparse_copy_cancellation_after_full_logical_processing_is_not_completion() {
    for backend in ["threaded", "io-uring"] {
        let f = Fixture::new(0);
        File::options()
            .write(true)
            .open(&f.source)
            .unwrap()
            .set_len(1024 * 1024 + 7)
            .unwrap();
        let token = rvvdk_datamover::CancellationToken::new();
        let mut err = CancellingProgress {
            bytes: Vec::new(),
            token: token.clone(),
            phase: "copy_flushing",
        };
        assert_eq!(
            rvvdk_cli::run_with_cancellation(
                f.args(
                    "copy",
                    &[
                        "--progress",
                        "--backend",
                        backend,
                        "--workers",
                        "4",
                        "--block-size",
                        "4096"
                    ]
                ),
                &mut Vec::new(),
                &mut err,
                &token
            ),
            130
        );
        assert!(!f.destination.exists());
        let text = String::from_utf8(err.bytes).unwrap();
        let events: Vec<Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert!(
            events
                .iter()
                .any(|e| e["logical_bytes_processed"] == 1024 * 1024 + 7)
        );
        assert!(!events.iter().any(|e| e["phase"] == "completed"));
    }
}
