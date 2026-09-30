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
#[path = "../../rvvdk-vmdk/tests/support/sparse_metadata.rs"]
#[allow(dead_code)]
mod sparse;

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
                "rvddk-sparse-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        let source = root.join("disk.vmdk");
        let destination = root.join("output.raw");
        let text = sparse::descriptor(
            "twoGbMaxExtentSparse",
            "RW 2048 SPARSE \"a\"\nRDONLY 2048 SPARSE \"b\"",
        );
        fs::write(&source, text).unwrap();
        fs::write(root.join("a"), sparse::bytes(None)).unwrap();
        fs::write(root.join("b"), sparse::bytes(None)).unwrap();
        let mut expected = vec![0; 2 * 1048576];
        for offset in [0, 1048576] {
            expected[offset..offset + 65536].fill(0x5a);
            expected[offset + 65536..offset + 131072].fill(0xa5);
        }
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
fn split_inspect_plan_copy_verify_and_tail_preservation() {
    for backend in ["threaded", "auto"] {
        for overwrite in [false, true] {
            let f = Fixture::new();
            let before = fs::read(&f.source).unwrap();
            let (code, r) = f.run("inspect", &["--extents"]);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["source"]["logical_bytes"], 2097152);
            assert_eq!(r["source"]["vmdk"]["layout"], "hosted_sparse");
            assert_eq!(r["source"]["vmdk"]["backing_file_count"], 2);
            assert_eq!(r["summary"]["data_bytes"], 262144);
            assert_eq!(r["summary"]["zero_bytes"], 1835008);
            assert_eq!(r["extents"].as_array().unwrap().len(), 4);
            let (code, r) = f.run("plan", &["--backend", backend]);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["execution"]["selected_backend"], "threaded");
            assert!(!f.destination.exists());
            if overwrite {
                fs::write(&f.destination, vec![0xcc; f.expected.len() + 513]).unwrap();
            }
            let mut extras = vec![
                "--verify",
                "--backend",
                backend,
                "--workers",
                "4",
                "--block-size",
                "65537",
            ];
            if overwrite {
                extras.push("--overwrite");
            }
            let (code, r) = f.run("copy", &extras);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["backend"], "threaded");
            assert_eq!(r["verification"]["bytes_verified"], 2097152);
            let actual = fs::read(&f.destination).unwrap();
            assert_eq!(&actual[..f.expected.len()], f.expected);
            assert!(actual[f.expected.len()..].iter().all(|&b| b == 0xcc));
            assert_eq!(f.run("verify", &["--block-size", "65537"]).0, 0);
            assert_eq!(fs::read(&f.source).unwrap(), before);
            fs::OpenOptions::new()
                .write(true)
                .open(&f.destination)
                .unwrap()
                .write_all_at(&[7], 140000)
                .unwrap();
            let (code, r) = f.run("verify", &[]);
            assert_eq!(code, 1);
            assert_eq!(r["error"]["details"]["mismatch_offset"], 140000);
        }
    }
}
fn monolithic(f: &Fixture) -> Vec<u8> {
    let t = sparse::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"");
    let raw = sparse::bytes(Some(&t));
    fs::write(&f.source, &raw).unwrap();
    raw
}
#[test]
fn monolithic_container_uses_logical_capacity_and_raw_remains_explicit() {
    let f = Fixture::new();
    let original = monolithic(&f);
    for command in ["inspect", "plan", "copy", "verify"] {
        let extra = if command == "copy" {
            vec!["--verify"]
        } else {
            vec![]
        };
        let (code, r) = f.run(command, &extra);
        assert_eq!(code, 0, "{r}");
        assert_eq!(fs::read(&f.source).unwrap(), original);
    }
    assert_eq!(fs::read(&f.destination).unwrap(), f.expected[..1048576]);
    let mut args = f.args("inspect", &[]);
    let index = args.iter().position(|a| a == "vmdk").unwrap();
    args[index] = "raw".into();
    let mut out = Vec::new();
    assert_eq!(rvvdk_cli::run(args, &mut out, &mut Vec::new()), 0);
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(r["source"]["logical_bytes"], original.len());
}
#[test]
fn container_cannot_redirect_to_another_inode_or_external_monolithic_descriptor() {
    let f = Fixture::new();
    let t = sparse::descriptor("monolithicSparse", "RW 2048 SPARSE \"other.vmdk\"");
    let raw = sparse::bytes(Some(&t));
    fs::write(&f.source, &raw).unwrap();
    fs::write(f.root.join("other.vmdk"), &raw).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
    fs::remove_file(f.root.join("other.vmdk")).unwrap();
    fs::hard_link(&f.source, f.root.join("other.vmdk")).unwrap();
    assert_eq!(f.run("copy", &["--verify"]).0, 0);
    fs::remove_file(&f.destination).unwrap();
    fs::write(&f.source, t).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
}
#[test]
fn sparse_descriptor_and_every_backing_hardlink_reject_before_mutation() {
    for mono in [false, true] {
        for alias in if mono {
            vec!["disk.vmdk"]
        } else {
            vec!["disk.vmdk", "a", "b"]
        } {
            for command in ["plan", "copy", "verify"] {
                let f = Fixture::new();
                if mono {
                    monolithic(&f);
                }
                let path = f.root.join(alias);
                let before = fs::read(&path).unwrap();
                fs::hard_link(&path, &f.destination).unwrap();
                let extra = if command == "verify" {
                    vec![]
                } else {
                    vec!["--overwrite"]
                };
                let (code, r) = f.run(command, &extra);
                assert_eq!(code, 1, "{r}");
                assert_eq!(r["error"]["code"], "same_file");
                assert_eq!(fs::read(&path).unwrap(), before);
            }
        }
    }
}
#[test]
fn malformed_features_confinement_and_size_bounds_fail_without_output() {
    for mutation in 0..8 {
        let f = Fixture::new();
        let mut raw = monolithic(&f);
        match mutation {
            0 => raw[4..8].copy_from_slice(&2u32.to_le_bytes()),
            1 => raw[72] = 1,
            2 => raw[28..36].copy_from_slice(&u64::MAX.to_le_bytes()),
            3 => raw[36..44].copy_from_slice(&4096u64.to_le_bytes()),
            4 => raw.truncate(500),
            5 => raw[22 * 512] = 0,
            6 => raw[512..512 + 7].copy_from_slice(b"garbage"),
            _ => raw[21 * 512..21 * 512 + 4].fill(0),
        }
        fs::write(&f.source, raw).unwrap();
        let (code, r) = f.run("copy", &[]);
        assert_eq!(code, 1, "{r}");
        assert_eq!(r["error"]["code"], "vmdk");
        assert!(!f.destination.exists());
    }
    for name in ["../escape", "/absolute", "link"] {
        let f = Fixture::new();
        let t = sparse::descriptor(
            "twoGbMaxExtentSparse",
            &format!("RW 2048 SPARSE \"{name}\""),
        );
        fs::write(&f.source, t).unwrap();
        symlink("a", f.root.join("link")).unwrap();
        assert_eq!(f.run("copy", &[]).0, 1);
        assert!(!f.destination.exists());
    }
    let f = Fixture::new();
    let mut text = fs::read(&f.source).unwrap();
    text.resize(1048577, 0);
    fs::write(&f.source, text).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
}
#[test]
fn sparse_native_and_budget_failures_do_not_publish() {
    let mut f = Fixture::new();
    f.destination = f.root.join("missing/out");
    for command in ["plan", "copy"] {
        let (code, r) = f.run(command, &["--backend", "io-uring"]);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "unsupported_backend");
    }
    f.destination = f.root.join("output.raw");
    assert_eq!(f.run("copy", &["--memory-budget", "1"]).0, 1);
    assert!(!f.destination.exists());
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

#[test]
fn monolithic_metadata_and_payload_changes_prevent_publication() {
    for offset in [0, 512, 26 * 512, 27 * 512, 65536] {
        let f = Fixture::new();
        monolithic(&f);
        let mut error = Value::Null;
        let mut events = Events {
            buffer: vec![],
            hook: |v: &Value| {
                if v["phase"] == "verification" {
                    fs::OpenOptions::new()
                        .write(true)
                        .open(&f.source)
                        .unwrap()
                        .write_all_at(&[0xff], offset)
                        .unwrap();
                }
                if !v["error"].is_null() {
                    error = v.clone();
                }
            },
        };
        assert_eq!(
            rvvdk_cli::run(
                f.args("copy", &["--progress"]),
                &mut Vec::new(),
                &mut events
            ),
            1
        );
        assert_eq!(error["error"]["code"], "source_changed");
        assert!(!f.destination.exists());
    }
}
#[test]
fn padded_split_descriptor_is_preserved() {
    let f = Fixture::new();
    let mut text = fs::read(&f.source).unwrap();
    text.resize(8192, 0);
    fs::write(&f.source, &text).unwrap();
    assert_eq!(f.run("copy", &["--verify"]).0, 0);
    assert_eq!(fs::read(&f.source).unwrap(), text);
}
