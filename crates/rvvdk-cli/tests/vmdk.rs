#![cfg(target_os = "linux")]
use serde_json::Value;
use std::{
    ffi::OsString,
    fs,
    io::Write,
    os::unix::fs::{FileExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture {
    root: PathBuf,
    source: PathBuf,
    destination: PathBuf,
    expected: Vec<u8>,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::var_os("RVVDK_TEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!(
                "rvddk-vmdk-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        let source = root.join("disk.vmdk");
        let destination = root.join("output.raw");
        let a: Vec<u8> = (0..4096).map(|n| (n * 29 + n / 251) as u8).collect();
        let b: Vec<u8> = a.iter().copied().rev().collect();
        fs::write(root.join("a"), &a).unwrap();
        fs::write(root.join("b"), &b).unwrap();
        fs::write(&source, "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 3 FLAT \"a\" 1\nRW 4 ZERO\nRDONLY 2 FLAT \"b\" 3\nRW 1 FLAT \"a\" 0\n").unwrap();
        let mut expected = a[512..2048].to_vec();
        expected.extend([0; 2048]);
        expected.extend(&b[1536..2560]);
        expected.extend(&a[..512]);
        Self {
            root,
            source,
            destination,
            expected,
        }
    }
    fn args(&self, command: &str, extras: &[&str]) -> Vec<OsString> {
        let mut a = vec![
            "rvddk".into(),
            command.into(),
            self.source.clone().into_os_string(),
        ];
        if command != "inspect" {
            a.push(self.destination.clone().into_os_string());
        }
        a.extend(["--format".into(), "vmdk".into(), "--json".into()]);
        a.extend(extras.iter().map(OsString::from));
        a
    }
    fn run(&self, command: &str, extras: &[&str]) -> (i32, Value) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = rvvdk_cli::run(self.args(command, extras), &mut out, &mut err);
        if code != 0 {
            assert!(out.is_empty());
        }
        (
            code,
            serde_json::from_slice(if code == 0 { &out } else { &err }).unwrap(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn logical_inspection_and_preview_use_capacity_and_portable_selection() {
    let f = Fixture::new();
    let (code, r) = f.run("inspect", &["--extents"]);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["format"], "vmdk");
    assert_eq!(r["source"]["logical_bytes"], 5120);
    assert!(r["source"].get("identity").is_none());
    assert_eq!(r["source"]["vmdk"]["backing_file_count"], 2);
    assert_eq!(r["summary"]["data_bytes"], 3072);
    assert_eq!(r["summary"]["zero_bytes"], 2048);
    assert_eq!(r["summary"]["hole_bytes"], 0);
    assert_eq!(r["extents"].as_array().unwrap().len(), 3);
    let (code, r) = f.run("plan", &["--backend", "auto"]);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["execution"]["selected_backend"], "threaded");
    assert_eq!(r["execution"]["selection_reason"], "portable_api");
    assert_eq!(r["destination"]["copy_range_bytes"], 5120);
    assert!(!f.destination.exists());
}
#[test]
fn copies_verify_logical_bytes_and_preserve_overwrite_tail() {
    for backend in ["threaded", "auto"] {
        for overwrite in [false, true] {
            let f = Fixture::new();
            if overwrite {
                fs::write(&f.destination, vec![0xa5; 6000]).unwrap();
            }
            let mut extra = vec![
                "--verify",
                "--backend",
                backend,
                "--workers",
                "4",
                "--block-size",
                "777",
            ];
            if overwrite {
                extra.push("--overwrite");
            }
            let (code, r) = f.run("copy", &extra);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["backend"], "threaded");
            assert_eq!(r["format"], "vmdk");
            assert_eq!(r["verification"]["bytes_verified"], 5120);
            let bytes = fs::read(&f.destination).unwrap();
            assert_eq!(&bytes[..5120], f.expected);
            assert!(bytes[5120..].iter().all(|b| *b == 0xa5));
            let (code, r) = f.run("verify", &["--block-size", "513"]);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["logical_bytes"], 5120);
            fs::OpenOptions::new()
                .write(true)
                .open(&f.destination)
                .unwrap()
                .write_all_at(&[1], 2000)
                .unwrap();
            let (code, r) = f.run("verify", &[]);
            assert_eq!(code, 1);
            assert_eq!(r["error"]["code"], "verification_mismatch");
            assert_eq!(r["error"]["details"]["mismatch_offset"], 2000);
        }
    }
}
#[test]
fn descriptor_and_every_backing_hardlink_are_rejected_without_mutation() {
    for alias in ["disk.vmdk", "a", "b"] {
        for command in ["plan", "copy", "verify"] {
            let f = Fixture::new();
            let path = f.root.join(alias);
            let before = fs::read(&path).unwrap();
            fs::hard_link(&path, &f.destination).unwrap();
            let extras = if command == "verify" {
                vec![]
            } else {
                vec!["--overwrite"]
            };
            let (code, r) = f.run(command, &extras);
            assert_eq!(code, 1, "{r}");
            assert_eq!(r["error"]["code"], "same_file");
            assert_eq!(fs::read(path).unwrap(), before);
        }
    }
}
#[test]
fn native_rejection_precedes_destination_open_and_budget_failure_does_not_publish() {
    let mut f = Fixture::new();
    f.destination = f.root.join("missing-parent/out");
    for command in ["plan", "copy"] {
        let (code, r) = f.run(command, &["--backend", "io-uring"]);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "unsupported_backend");
    }
    f.destination = f.root.join("output.raw");
    let (code, r) = f.run("copy", &["--memory-budget", "1"]);
    assert_eq!(code, 1, "{r}");
    assert!(!f.destination.exists());
}
#[test]
fn zero_only_source_and_explicit_raw_selection() {
    let f = Fixture::new();
    fs::write(
        &f.source,
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 10 ZERO\n",
    )
    .unwrap();
    let (code, r) = f.run("copy", &["--verify"]);
    assert_eq!(code, 0, "{r}");
    assert_eq!(fs::read(&f.destination).unwrap(), vec![0; 5120]);
    let mut args = f.args("inspect", &[]);
    let pos = args.iter().position(|a| a == "vmdk").unwrap();
    args[pos] = "raw".into();
    let mut out = Vec::new();
    assert_eq!(rvvdk_cli::run(args, &mut out, &mut Vec::new()), 0);
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        r["source"]["logical_bytes"],
        fs::metadata(&f.source).unwrap().len()
    );
}
#[test]
fn malformed_unsupported_and_unconfined_sources_fail_without_output() {
    for text in [
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"monolithicSparse\"\nRW 1 SPARSE \"a\"\n",
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 1 FLAT \"../a\" 0\n",
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 1 ZERO\n\0",
    ] {
        let f = Fixture::new();
        fs::write(&f.source, text).unwrap();
        let (code, r) = f.run("copy", &[]);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "vmdk");
        assert!(!f.destination.exists());
    }
    let f = Fixture::new();
    fs::rename(f.root.join("a"), f.root.join("actual")).unwrap();
    symlink("actual", f.root.join("a")).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
    fs::remove_file(f.root.join("a")).unwrap();
    fs::rename(f.root.join("actual"), f.root.join("a")).unwrap();
    fs::rename(&f.source, f.root.join("actual.vmdk")).unwrap();
    symlink("actual.vmdk", &f.source).unwrap();
    assert_eq!(f.run("inspect", &[]).0, 1);
}
struct Events<F> {
    buffer: Vec<u8>,
    hook: F,
}
impl<F: FnMut(&Value)> Write for Events<F> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend(bytes);
        while let Some(end) = self.buffer.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.buffer.drain(..=end).collect();
            (self.hook)(&serde_json::from_slice(&line).unwrap());
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn same_size_backing_or_descriptor_mutation_prevents_publication() {
    for name in ["a", "b", "disk.vmdk"] {
        let f = Fixture::new();
        let path = f.root.join(name);
        let mut error = Value::Null;
        let mut events = Events {
            buffer: vec![],
            hook: |v: &Value| {
                if v["phase"] == "verification" {
                    let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
                    file.write_all_at(b"#", 0).unwrap();
                }
                if !v["error"].is_null() {
                    error = v.clone();
                }
            },
        };
        let mut out = Vec::new();
        let code = rvvdk_cli::run(f.args("copy", &["--progress"]), &mut out, &mut events);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert!(!f.destination.exists());
        assert_eq!(error["error"]["code"], "source_changed");
    }
}
#[test]
fn cancellation_and_publication_collision_preserve_output_policy() {
    let f = Fixture::new();
    let token = rvvdk_datamover::CancellationToken::new();
    let mut events = Events {
        buffer: vec![],
        hook: |v: &Value| {
            if v["phase"] == "publication" {
                token.cancel();
            }
        },
    };
    let mut out = Vec::new();
    assert_eq!(
        rvvdk_cli::run_with_cancellation(
            f.args("copy", &["--verify", "--progress"]),
            &mut out,
            &mut events,
            &token
        ),
        130
    );
    assert!(!f.destination.exists());
    assert!(out.is_empty());
    let mut events = Events {
        buffer: vec![],
        hook: |v: &Value| {
            if v["phase"] == "publication" {
                fs::write(&f.destination, b"winner").unwrap();
            }
        },
    };
    assert_eq!(
        rvvdk_cli::run(
            f.args("copy", &["--progress", "--overwrite"]),
            &mut Vec::new(),
            &mut events
        ),
        1
    );
    assert_eq!(fs::read(&f.destination).unwrap(), b"winner");
}
