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
#[path = "../../rvvdk-vmdk/tests/support/stream_disk.rs"]
mod stream;

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
                "rvddk-stream-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        let source = root.join("disk.vmdk");
        let destination = root.join("output.raw");
        let mut expected = vec![0; 4 * 65536];
        expected[65536..131072].copy_from_slice(&stream::bytes(1));
        expected[196608..].copy_from_slice(&stream::bytes(3));
        fs::write(
            &source,
            stream::image(
                true,
                4,
                &[
                    (1, stream::stored(&stream::bytes(1))),
                    (3, stream::stored(&stream::bytes(3))),
                ],
            ),
        )
        .unwrap();
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
fn logical_commands_cover_both_layouts_cross_grain_reads_and_overwrite_tail() {
    for footer in [false, true] {
        for backend in ["threaded", "auto"] {
            let f = Fixture::new();
            fs::write(
                &f.source,
                stream::image(
                    footer,
                    4,
                    &[
                        (1, stream::stored(&stream::bytes(1))),
                        (3, stream::stored(&stream::bytes(3))),
                    ],
                ),
            )
            .unwrap();
            let (code, r) = f.run("inspect", &["--extents"]);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["source"]["logical_bytes"], f.expected.len());
            assert_eq!(r["source"]["vmdk"]["layout"], "stream_optimized");
            assert_eq!(r["source"]["vmdk"]["payload_validation"], "on_read");
            assert_eq!(r["source"]["vmdk"]["allocated_grains"], 2);
            assert!(
                r["source"]["vmdk"]["decode_memory_reservation_bytes"]
                    .as_u64()
                    .unwrap()
                    < 524288
            );
            assert!(r["source"]["identity"].is_null());
            assert_eq!(r["summary"]["data_bytes"], 131072);
            let (code, r) = f.run("plan", &["--backend", backend]);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["execution"]["selected_backend"], "threaded");
            assert!(!f.destination.exists());
            for overwrite in [false, true] {
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
                let actual = fs::read(&f.destination).unwrap();
                assert_eq!(&actual[..f.expected.len()], f.expected);
                assert!(actual[f.expected.len()..].iter().all(|b| *b == 0xcc));
                assert_eq!(f.run("verify", &["--block-size", "65537"]).0, 0);
                fs::remove_file(&f.destination).unwrap();
            }
        }
    }
}
#[test]
fn payload_validation_is_lazy_but_corruption_never_publishes() {
    let f = Fixture::new();
    let mut bad = stream::stored(&stream::bytes(1));
    *bad.last_mut().unwrap() ^= 1;
    fs::write(&f.source, stream::image(true, 4, &[(1, bad)])).unwrap();
    assert_eq!(f.run("inspect", &[]).0, 0);
    assert_eq!(f.run("plan", &[]).0, 0);
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
    fs::write(&f.destination, &f.expected).unwrap();
    assert_eq!(f.run("verify", &[]).0, 1);
    let (code, r) = f.run("copy", &["--overwrite"]);
    assert_eq!(code, 1);
    assert_eq!(
        r["error"]["details"]["destination_state"],
        "existing_may_be_modified"
    );
}
#[test]
fn unsupported_options_and_limits_reject_before_destination_preparation() {
    let mut f = Fixture::new();
    f.destination = f.root.join("missing/output");
    for command in ["plan", "copy", "verify"] {
        let (code, r) = f.run(command, &["--block-size", "67108865"]);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "arguments");
    }
    for command in ["plan", "copy"] {
        let (code, r) = f.run(command, &["--backend", "io-uring"]);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "unsupported_backend");
    }
    assert_eq!(f.run("inspect", &["--allow-parents"]).0, 1);
    f.destination = f.root.join("output.raw");
    assert_eq!(f.run("copy", &["--memory-budget", "1"]).0, 1);
    assert!(!f.destination.exists());
}
#[test]
fn aliases_and_symlinks_reject_but_embedded_filename_is_not_followed() {
    let f = Fixture::new();
    // The descriptor names self.vmdk, deliberately a dangling symlink.
    symlink("missing", f.root.join("self.vmdk")).unwrap();
    assert_eq!(f.run("copy", &["--verify"]).0, 0);
    fs::remove_file(&f.destination).unwrap();
    fs::hard_link(&f.source, &f.destination).unwrap();
    for command in ["plan", "copy", "verify"] {
        let extras = if command == "verify" {
            vec![]
        } else {
            vec!["--overwrite"]
        };
        let (code, r) = f.run(command, &extras);
        assert_eq!(code, 1);
        assert_eq!(r["error"]["code"], "same_file");
    }
    fs::remove_file(&f.source).unwrap();
    symlink(&f.destination, &f.source).unwrap();
    assert_eq!(f.run("inspect", &[]).0, 1);
}
#[test]
fn unsupported_versions_flags_and_parent_descriptors_stay_rejected() {
    for (at, bytes) in [
        (4, 2u32.to_le_bytes().to_vec()),
        (8, 0u32.to_le_bytes().to_vec()),
        (77, vec![2, 0]),
        (72, vec![1]),
    ] {
        let f = Fixture::new();
        let file = fs::OpenOptions::new().write(true).open(&f.source).unwrap();
        file.write_all_at(&bytes, at).unwrap();
        assert_eq!(f.run("copy", &[]).0, 1);
        assert!(!f.destination.exists());
    }
    let f = Fixture::new();
    let mut b = fs::read(&f.source).unwrap();
    let at = b
        .windows(18)
        .position(|v| v == b"parentCID=ffffffff")
        .unwrap();
    b[at + 10..at + 18].copy_from_slice(b"12345678");
    fs::write(&f.source, b).unwrap();
    assert_eq!(f.run("inspect", &[]).0, 1);
}
#[test]
fn mutations_cancel_and_publication_races_preserve_output_policy() {
    for action in ["mutate", "cancel", "collision"] {
        let f = Fixture::new();
        let token = rvvdk_datamover::CancellationToken::new();
        let mut error = Value::Null;
        let mut events = Events {
            buffer: vec![],
            hook: |v: &Value| {
                if action == "mutate" && v["phase"] == "verification" {
                    fs::OpenOptions::new()
                        .write(true)
                        .open(&f.source)
                        .unwrap()
                        .write_all_at(&[0xff], 65560)
                        .unwrap();
                }
                if v["phase"] == "publication" {
                    if action == "cancel" {
                        token.cancel();
                    }
                    if action == "collision" {
                        fs::write(&f.destination, b"winner").unwrap();
                    }
                }
                if !v["error"].is_null() {
                    error = v.clone();
                }
            },
        };
        let mut out = Vec::new();
        let code = rvvdk_cli::run_with_cancellation(
            f.args("copy", &["--progress"]),
            &mut out,
            &mut events,
            &token,
        );
        assert_eq!(code, if action == "cancel" { 130 } else { 1 });
        assert!(out.is_empty());
        if action == "collision" {
            assert_eq!(fs::read(&f.destination).unwrap(), b"winner");
        } else {
            assert!(!f.destination.exists());
        }
        if action == "mutate" {
            assert_eq!(error["error"]["code"], "source_changed");
        }
    }
}
