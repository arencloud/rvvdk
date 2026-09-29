#![cfg(target_os = "linux")]
use serde_json::Value;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::Write,
    os::unix::{
        ffi::OsStringExt,
        fs::{FileExt, MetadataExt, PermissionsExt, symlink},
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
                "rvddk-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source.raw");
        let destination = root.join("destination.raw");
        fs::write(&source, vec![0x5a; size]).unwrap();
        Self {
            root,
            source,
            destination,
        }
    }
    fn args(&self, plan: bool, extra: &[&str]) -> Vec<OsString> {
        let mut args = vec![
            "rvddk".into(),
            if plan {
                "plan".into()
            } else {
                "inspect".into()
            },
            self.source.clone().into(),
        ];
        if plan {
            args.push(self.destination.clone().into());
        }
        args.extend(["--format".into(), "raw".into(), "--json".into()]);
        args.extend(extra.iter().map(OsString::from));
        args
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn run(args: Vec<OsString>) -> (i32, Vec<u8>, Vec<u8>) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = rvvdk_cli::run(args, &mut out, &mut err);
    (code, out, err)
}
fn ok(args: Vec<OsString>) -> Value {
    let (code, out, err) = run(args);
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&err));
    assert!(err.is_empty());
    serde_json::from_slice(&out).unwrap()
}
fn error(args: Vec<OsString>, expected: &str, exit: i32) {
    let (code, out, err) = run(args);
    assert_eq!(code, exit, "{}", String::from_utf8_lossy(&err));
    assert!(out.is_empty());
    let value: Value = serde_json::from_slice(&err).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["error"]["code"], expected);
}
#[test]
fn inspect_dense_empty_and_odd_images_with_explicit_extent_output() {
    for size in [0, 7, 4096, 4103] {
        let f = Fixture::new(size);
        let report = ok(f.args(false, &["--extents"]));
        assert_eq!(report["schema_version"], 1);
        assert_eq!(report["status"], "inspected");
        assert_eq!(report["summary"]["logical_bytes"], size);
        let extents = report["extents"].as_array().unwrap();
        let mut offset = 0;
        for extent in extents {
            assert_eq!(extent["offset"], offset);
            offset += extent["length"].as_u64().unwrap();
        }
        assert_eq!(offset, size as u64);
        assert!(report.get("execution").is_none());
        assert_eq!(fs::read(&f.source).unwrap(), vec![0x5a; size]);
        assert!(ok(f.args(false, &[])).get("extents").is_none());
    }
}
#[test]
fn sparse_map_matches_library_topology_without_reading_payload() {
    use rvvdk_core::{RawDisk, VirtualDisk};
    let f = Fixture::new(0);
    let file = File::options().write(true).open(&f.source).unwrap();
    file.set_len(65536).unwrap();
    file.write_all_at(&[0x77; 4096], 16384).unwrap();
    file.sync_all().unwrap();
    let source =
        RawDisk::new(rvvdk_local::LocalFileBlockDevice::open_read_only(&f.source).unwrap());
    let map = source.extents(0, source.size()).unwrap();
    let report = ok(f.args(false, &["--extents"]));
    assert_eq!(report["extents"].as_array().unwrap().len(), map.len());
    for (e, expected) in report["extents"].as_array().unwrap().iter().zip(map) {
        assert_eq!(e["offset"], expected.offset());
        assert_eq!(e["length"], expected.length());
    }
}
#[test]
fn new_destination_preview_never_creates_output_and_native_is_deferred() {
    let f = Fixture::new(4103);
    for backend in ["threaded", "auto", "io-uring"] {
        let report = ok(f.args(
            true,
            &[
                "--backend",
                backend,
                "--block-size",
                "4096",
                "--workers",
                "4",
            ],
        ));
        assert_eq!(report["status"], "preview");
        assert_eq!(report["destination"]["policy"], "create_new_no_clobber");
        assert_eq!(report["destination"]["write_access_checked"], false);
        assert_eq!(report["execution"]["runtime_prepared"], false);
        assert_eq!(report["execution"]["workers"], 4);
        assert_eq!(report["execution"]["requested_backend"], backend);
        if backend == "threaded" {
            assert_eq!(report["execution"]["selected_backend"], "threaded");
        } else {
            assert!(report["execution"]["selected_backend"].is_null());
        }
        assert!(!f.destination.exists());
    }
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
}
#[test]
fn overwrite_is_explicit_preserves_bytes_and_reports_untouched_tail() {
    let f = Fixture::new(4096);
    fs::write(&f.destination, vec![0xa5; 8192]).unwrap();
    fs::set_permissions(&f.destination, fs::Permissions::from_mode(0o444)).unwrap();
    let before = fs::metadata(&f.destination).unwrap();
    error(f.args(true, &[]), "destination_exists", 1);
    let report = ok(f.args(true, &["--overwrite"]));
    assert_eq!(report["destination"]["policy"], "overwrite_in_place");
    assert_eq!(report["destination"]["preserved_tail_bytes"], 4096);
    assert_eq!(fs::read(&f.destination).unwrap(), vec![0xa5; 8192]);
    let after = fs::metadata(&f.destination).unwrap();
    assert_eq!(
        (
            before.ino(),
            before.len(),
            before.mtime(),
            before.mtime_nsec()
        ),
        (after.ino(), after.len(), after.mtime(), after.mtime_nsec())
    );
    assert_eq!(fs::read(&f.source).unwrap(), vec![0x5a; 4096]);
}
#[test]
fn aliases_symlinks_and_small_destinations_reject_without_changes() {
    let f = Fixture::new(4096);
    fs::hard_link(&f.source, &f.destination).unwrap();
    error(f.args(true, &["--overwrite"]), "same_file", 1);
    fs::remove_file(&f.destination).unwrap();
    for target in [&f.source, &f.root.join("missing.raw")] {
        symlink(target, &f.destination).unwrap();
        error(f.args(true, &["--overwrite"]), "invalid_destination", 1);
        fs::remove_file(&f.destination).unwrap();
    }
    fs::write(&f.destination, [0xa5; 7]).unwrap();
    error(f.args(true, &["--overwrite"]), "destination_too_small", 1);
    assert_eq!(fs::read(&f.destination).unwrap(), [0xa5; 7]);
}
#[test]
fn missing_parent_and_non_file_paths_fail_before_output() {
    let mut f = Fixture::new(4096);
    f.destination = f.root.join("missing/target");
    error(f.args(true, &[]), "io", 1);
    f.destination = f.root.join("new/");
    error(f.args(true, &[]), "invalid_destination", 1);
    f.destination = f.root.clone();
    error(f.args(true, &["--overwrite"]), "invalid_destination", 1);
    f.source = f.root.clone();
    error(f.args(false, &[]), "not_regular_file", 1);
}
#[test]
fn usage_errors_are_json_and_tunables_are_checked() {
    let f = Fixture::new(7);
    for extra in [
        &["--backend", "bogus"][..],
        &["--block-size", "0"],
        &["--workers", "0"],
        &["--queue-depth", "0"],
        &["--queue-depth", "4294967296"],
        &["--workers", "-1"],
        &["--unknown"],
    ] {
        error(f.args(true, extra), "usage", 2);
    }
    error(
        vec![
            "rvddk".into(),
            "inspect".into(),
            f.source.clone().into(),
            "--json".into(),
        ],
        "usage",
        2,
    );
    error(
        vec!["rvddk".into(), "copy".into(), "--json".into()],
        "usage",
        2,
    );
    let (code, _, _) = run(vec!["rvddk".into(), "--help".into()]);
    assert_eq!(code, 0);
    let (code, out, _) = run(vec!["rvddk".into(), "--version".into()]);
    assert_eq!(code, 0);
    assert!(String::from_utf8(out).unwrap().starts_with("rvddk "));
}
#[test]
fn metadata_budget_failure_and_unadmitted_execution_estimate_are_distinct() {
    let f = Fixture::new(4096);
    error(f.args(false, &["--memory-budget", "0"]), "memory_budget", 1);
    let report = ok(f.args(true, &["--memory-budget", "1024"]));
    assert_eq!(report["execution"]["threaded_payload_fits_budget"], false);
    assert!(!f.destination.exists());
}
#[test]
fn non_utf8_paths_round_trip_and_control_characters_are_escaped() {
    let mut f = Fixture::new(7);
    let old = f.source.clone();
    f.source = f
        .root
        .join(OsString::from_vec(b"source-\xff\n.raw".to_vec()));
    fs::rename(old, &f.source).unwrap();
    let report = ok(f.args(false, &[]));
    assert!(
        report["source"]["path"]["bytes_hex"]
            .as_str()
            .unwrap()
            .ends_with("736f757263652dff0a2e726177")
    );
    let args = f
        .args(false, &[])
        .into_iter()
        .filter(|a| a != "--json")
        .collect();
    let (code, out, _) = run(args);
    assert_eq!(code, 0);
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("\\n.raw"));
    assert_eq!(text.lines().count(), 3);
}
#[test]
fn output_write_and_flush_failures_return_failure() {
    struct FailsFlush;
    impl Write for FailsFlush {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
    }
    let f = Fixture::new(7);
    let mut full = File::options().write(true).open("/dev/full").unwrap();
    assert_eq!(
        rvvdk_cli::run(f.args(false, &[]), &mut full, &mut Vec::new()),
        1
    );
    assert_eq!(
        rvvdk_cli::run(f.args(false, &[]), &mut FailsFlush, &mut Vec::new()),
        1
    );
}
#[test]
fn actual_binary_handles_help_errors_and_special_files_without_blocking() {
    let mut f = Fixture::new(7);
    let fifo = f.root.join("fifo");
    let path = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: the path is NUL terminated and local to this test.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    f.source = fifo;
    let output = bounded(Command::new(env!("CARGO_BIN_EXE_rvddk")).args(&f.args(false, &[])[1..]));
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "not_regular_file");
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
fn commands_succeed_when_writable_opens_and_ring_setup_are_denied() {
    use std::os::unix::process::CommandExt;
    let f = Fixture::new(4103);
    for existing in [false, true] {
        if existing {
            fs::write(&f.destination, vec![0xa5; 8192]).unwrap();
        }
        for backend in ["threaded", "auto", "io-uring"] {
            let mut args = f.args(true, &["--backend", backend]);
            if existing {
                args.push("--overwrite".into());
            }
            let instruction = |code, jt, jf, k| libc::sock_filter { code, jt, jf, k };
            let mut filter = Vec::new();
            // Filter only the disposable exec child; never change host policy.
            for (call, argument) in [(libc::SYS_openat, 2), (libc::SYS_open, 1)] {
                let word = 16 + argument * 8 + if cfg!(target_endian = "big") { 4 } else { 0 };
                filter.extend([
                    instruction(0x20, 0, 0, 0),
                    instruction(0x15, 0, 4, call as u32),
                    instruction(0x20, 0, 0, word),
                    instruction(
                        0x54,
                        0,
                        0,
                        (libc::O_ACCMODE | libc::O_CREAT | libc::O_TRUNC | libc::O_APPEND) as u32,
                    ),
                    instruction(0x15, 1, 0, 0),
                    instruction(0x06, 0, 0, libc::SECCOMP_RET_ERRNO | libc::EPERM as u32),
                ]);
            }
            for syscall in [
                libc::SYS_creat,
                libc::SYS_truncate,
                libc::SYS_ftruncate,
                libc::SYS_fallocate,
                libc::SYS_io_uring_setup,
            ] {
                filter.extend([
                    instruction(0x20, 0, 0, 0),
                    instruction(0x15, 0, 1, syscall as u32),
                    instruction(0x06, 0, 0, libc::SECCOMP_RET_ERRNO | libc::EPERM as u32),
                ]);
            }
            filter.push(instruction(0x06, 0, 0, libc::SECCOMP_RET_ALLOW));
            let mut command = Command::new(env!("CARGO_BIN_EXE_rvddk"));
            command.args(&args[1..]);
            // SAFETY: after fork this closure only calls prctl using live filter
            // storage. It does not allocate, lock, or access shared mutable state.
            unsafe {
                command.pre_exec(move || {
                    let program = libc::sock_fprog {
                        len: filter.len() as u16,
                        filter: filter.as_mut_ptr(),
                    };
                    if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
                        || libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program)
                            != 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let output = bounded(&mut command);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(report["status"], "preview");
            assert_eq!(report["execution"]["runtime_prepared"], false);
            assert_eq!(f.destination.exists(), existing);
        }
    }
    assert_eq!(fs::read(&f.source).unwrap(), vec![0x5a; 4103]);
    assert_eq!(fs::read(&f.destination).unwrap(), vec![0xa5; 8192]);
}
