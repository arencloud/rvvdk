#![cfg(target_os = "linux")]
mod support {
    pub mod parents;
}
use serde_json::Value;
use std::{
    fs,
    io::Write,
    os::unix::fs::{FileExt, symlink},
};
use support::parents::Fixture;
#[test]
fn chain_commands_resolve_inheritance_zeros_and_preserve_tail() {
    for split in [[false; 3], [true; 3], [false, true, false]] {
        for backend in ["auto", "threaded"] {
            let f = Fixture::new(split);
            let originals: Vec<_> = fs::read_dir(&f.root)
                .unwrap()
                .map(|e| {
                    let p = e.unwrap().path();
                    let b = fs::read(&p).unwrap();
                    (p, b)
                })
                .collect();
            let (c, r) = f.run("inspect", &["--extents"]);
            assert_eq!(c, 0, "{r}");
            assert_eq!(r["source"]["vmdk"]["layer_count"], 3);
            assert_eq!(r["source"]["vmdk"]["layout"], "hosted_sparse_chain");
            assert_eq!(r["source"]["logical_bytes"], 1048576);
            let (c, r) = f.run("plan", &["--backend", backend]);
            assert_eq!(c, 0, "{r}");
            assert!(!f.destination.exists());
            for overwrite in [false, true] {
                if overwrite {
                    fs::write(&f.destination, vec![0xcc; 1048576 + 513]).unwrap();
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
                let (c, r) = f.run("copy", &extras);
                assert_eq!(c, 0, "{r}");
                let b = fs::read(&f.destination).unwrap();
                assert_eq!(b[..1048576], f.expected);
                assert!(b[1048576..].iter().all(|b| *b == 0xcc));
                assert_eq!(f.run("verify", &[]).0, 0);
            }
            fs::OpenOptions::new()
                .write(true)
                .open(&f.destination)
                .unwrap()
                .write_all_at(&[7], 140000)
                .unwrap();
            let (c, r) = f.run("verify", &[]);
            assert_eq!(c, 1);
            assert_eq!(r["error"]["details"]["mismatch_offset"], 140000);
            for (p, b) in originals {
                assert_eq!(fs::read(p).unwrap(), b);
            }
        }
    }
}
#[test]
fn opt_in_requires_vmdk_and_default_still_rejects_parents() {
    let f = Fixture::new([true; 3]);
    for command in ["inspect", "plan", "copy", "verify"] {
        let mut a = f.args(command, &[]);
        a.retain(|v| v != "--allow-parents");
        assert_eq!(rvvdk_cli::run(a, &mut Vec::new(), &mut Vec::new()), 1);
        let mut a = f.args(command, &[]);
        let i = a.iter().position(|v| v == "vmdk").unwrap();
        a[i] = "raw".into();
        assert_eq!(rvvdk_cli::run(a, &mut Vec::new(), &mut Vec::new()), 1);
    }
    assert!(!f.destination.exists());
}
#[test]
fn all_ancestor_descriptors_and_backings_reject_destination_aliases() {
    for name in [
        "layer0.vmdk",
        "layer0-s001.vmdk",
        "layer1.vmdk",
        "layer1-s001.vmdk",
        "layer2.vmdk",
        "layer2-s001.vmdk",
    ] {
        let f = Fixture::new([true; 3]);
        let p = f.root.join(name);
        let before = fs::read(&p).unwrap();
        fs::hard_link(&p, &f.destination).unwrap();
        for command in ["plan", "copy", "verify"] {
            assert_eq!(
                f.run(
                    command,
                    if command == "verify" {
                        &[]
                    } else {
                        &["--overwrite"]
                    }
                )
                .0,
                1
            );
            assert_eq!(fs::read(&p).unwrap(), before);
        }
    }
}
#[test]
fn unsafe_missing_or_symlink_parents_never_publish() {
    for hint in [
        "missing.vmdk",
        "../layer1.vmdk",
        "/layer1.vmdk",
        "dir/layer1.vmdk",
        "a\\b",
        "file:x",
        "link.vmdk",
    ] {
        let f = Fixture::new([true; 3]);
        symlink("layer1.vmdk", f.root.join("link.vmdk")).unwrap();
        f.replace("layer2.vmdk", "layer1.vmdk", hint);
        assert_eq!(f.run("copy", &[]).0, 1, "{hint}");
        assert!(!f.destination.exists());
    }
}
#[test]
fn invalid_chain_metadata_never_publishes() {
    for (name, from, to) in [
        ("layer1.vmdk", "CID=00000002", "CID=00000009"),
        ("layer2.vmdk", "RW 2048", "RW 2047"),
        ("layer2.vmdk", "layer1.vmdk", "layer2.vmdk"),
        ("layer0.vmdk", "twoGbMaxExtentSparse", "streamOptimized"),
    ] {
        let f = Fixture::new([true; 3]);
        f.replace(name, from, to);
        assert_eq!(f.run("copy", &[]).0, 1);
        assert!(!f.destination.exists());
    }
}
#[test]
fn parent_backing_symlinks_and_oversized_descriptors_rejected() {
    for oversized in [false, true] {
        let f = Fixture::new([true; 3]);
        if oversized {
            let p = f.root.join("layer0.vmdk");
            let mut b = fs::read(&p).unwrap();
            b.resize(1048577, 0);
            fs::write(p, b).unwrap();
        } else {
            fs::rename(f.root.join("layer0-s001.vmdk"), f.root.join("saved")).unwrap();
            symlink("saved", f.root.join("layer0-s001.vmdk")).unwrap();
        }
        assert_eq!(f.run("copy", &[]).0, 1);
        assert!(!f.destination.exists());
    }
}
struct Events<F> {
    buffer: Vec<u8>,
    hook: F,
}
impl<F: FnMut(&Value)> Write for Events<F> {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend(b);
        while let Some(end) = self.buffer.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.buffer.drain(..=end).collect();
            (self.hook)(&serde_json::from_slice(&line).unwrap());
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn ancestor_descriptor_metadata_and_payload_mutations_prevent_publication() {
    for (name, offset) in [
        ("layer0.vmdk", 0),
        ("layer1.vmdk", 0),
        ("layer0-s001.vmdk", 0),
        ("layer0-s001.vmdk", 27 * 512),
        ("layer0-s001.vmdk", 65536),
        ("layer1-s001.vmdk", 65536),
    ] {
        let f = Fixture::new([true; 3]);
        let mut error = Value::Null;
        let mut events = Events {
            buffer: vec![],
            hook: |v: &Value| {
                if v["phase"] == "verification" {
                    fs::OpenOptions::new()
                        .write(true)
                        .open(f.root.join(name))
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
fn parent_chain_publication_cancellation_and_collision() {
    let f = Fixture::new([false; 3]);
    let token = rvvdk_datamover::CancellationToken::new();
    let mut events = Events {
        buffer: vec![],
        hook: |v: &Value| {
            if v["phase"] == "publication" {
                token.cancel();
            }
        },
    };
    assert_eq!(
        rvvdk_cli::run_with_cancellation(
            f.args("copy", &["--verify", "--progress"]),
            &mut Vec::new(),
            &mut events,
            &token
        ),
        130
    );
    assert!(!f.destination.exists());
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
fn native_and_budget_rejections_precede_publication() {
    let f = Fixture::new([true; 3]);
    for extras in [
        &["--backend", "io-uring"][..],
        &["--memory-budget", "1"][..],
    ] {
        assert_eq!(f.run("copy", extras).0, 1);
        assert!(!f.destination.exists());
    }
}
#[test]
fn base_entry_and_padded_parent_descriptors_supported() {
    let mut f = Fixture::new([true; 3]);
    let p = f.root.join("layer0.vmdk");
    let mut b = fs::read(&p).unwrap();
    b.resize(8192, 0);
    fs::write(&p, b).unwrap();
    assert_eq!(f.run("copy", &["--verify"]).0, 0);
    f.source = p;
    let (c, r) = f.run("inspect", &[]);
    assert_eq!(c, 0, "{r}");
    assert_eq!(r["source"]["vmdk"]["layer_count"], 1);
}
#[test]
fn depth_limit_is_enforced_before_output_creation() {
    let mut f = Fixture::new([true; 3]);
    let template = fs::read_to_string(f.root.join("layer2.vmdk")).unwrap();
    for i in 3..17 {
        let text = template
            .replace("CID=00000003", &format!("CID={:08x}", i + 1))
            .replace("parentCID=00000002", &format!("parentCID={i:08x}"))
            .replace("layer1.vmdk", &format!("layer{}.vmdk", i - 1))
            .replace("layer2-s001.vmdk", &format!("layer{i}-s001.vmdk"));
        fs::write(f.root.join(format!("layer{i}.vmdk")), text).unwrap();
        fs::copy(
            f.root.join("layer2-s001.vmdk"),
            f.root.join(format!("layer{i}-s001.vmdk")),
        )
        .unwrap();
    }
    f.source = f.root.join("layer15.vmdk");
    let (c, r) = f.run("inspect", &[]);
    assert_eq!(c, 0, "{r}");
    assert_eq!(r["source"]["vmdk"]["layer_count"], 16);
    f.source = f.root.join("layer16.vmdk");
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
}
#[test]
fn embedded_identity_binding_and_external_mirrors() {
    let mut f = Fixture::new([false; 3]);
    let original = fs::read(&f.source).unwrap();
    let end = original[512..].iter().position(|b| *b == 0).unwrap() + 512;
    fs::write(f.root.join("mirror.vmdk"), &original[512..end]).unwrap();
    f.source = f.root.join("mirror.vmdk");
    let (c, r) = f.run("copy", &["--verify"]);
    assert_eq!(c, 0, "{r}");
    assert_eq!(fs::read(&f.destination).unwrap(), f.expected);
    fs::remove_file(&f.destination).unwrap();
    // An embedded descriptor may name a hard link to its own container, but
    // cannot redirect to a byte-identical second inode.
    f.source = f.root.join("layer2.vmdk");
    fs::rename(&f.source, f.root.join("saved2.vmdk")).unwrap();
    fs::hard_link(f.root.join("saved2.vmdk"), &f.source).unwrap();
    f.source = f.root.join("saved2.vmdk");
    assert_eq!(f.run("inspect", &[]).0, 0);
    fs::remove_file(f.root.join("layer2.vmdk")).unwrap();
    fs::write(f.root.join("layer2.vmdk"), original).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
}
#[test]
fn parent_opt_in_rejects_flat_sources() {
    let f = Fixture::new([true; 3]);
    fs::write(&f.source,b"# Disk DescriptorFile\nversion=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"monolithicFlat\"\nRW 2048 FLAT \"flat.raw\" 0\n").unwrap();
    fs::write(f.root.join("flat.raw"), vec![0; 1048576]).unwrap();
    assert_eq!(f.run("copy", &[]).0, 1);
    assert!(!f.destination.exists());
}
