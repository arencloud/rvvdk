#![allow(dead_code)]
#[path = "../../../rvvdk-vmdk/tests/support/sparse_metadata.rs"]
mod sparse;
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
pub struct Fixture {
    pub root: PathBuf,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub expected: Vec<u8>,
}
impl Fixture {
    pub fn new(split: [bool; 3]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::var_os("RVVDK_TEST_DIR")
            .or_else(|| std::env::var_os("RVVDK_BENCH_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!(
                "rvddk-parents-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        for (i, is_split) in split.into_iter().enumerate() {
            let name = format!("layer{i}.vmdk");
            let backing = if is_split {
                format!("layer{i}-s001.vmdk")
            } else {
                name.clone()
            };
            let kind = if is_split {
                "twoGbMaxExtentSparse"
            } else {
                "monolithicSparse"
            };
            let mut text = sparse::descriptor(kind, &format!("RW 2048 SPARSE \"{backing}\""))
                .replace("CID=12345678", &format!("CID={:08x}", i + 1));
            if i > 0 {
                text = text.replace(
                    "parentCID=ffffffff",
                    &format!(
                        "parentCID={i:08x}\nparentFileNameHint=\"layer{}.vmdk\"",
                        i - 1
                    ),
                );
            }
            let mut bytes = sparse::bytes(if is_split { None } else { Some(&text) });
            // Base owns grains 0/1. Middle overrides grain 1. Leaf allocates
            // an all-zero grain 2: allocated zeros must mask ancestor data.
            if i > 0 {
                for table in [22 * 512, 27 * 512] {
                    bytes[table..table + 12].fill(0);
                    bytes[table + i * 4..table + i * 4 + 4].copy_from_slice(&128u32.to_le_bytes());
                }
                bytes[65536..131072].fill(if i == 1 { 0x33 } else { 0 });
            } else {
                // Grain 2 aliases no physical allocation: allocate third grain.
                bytes.resize(262144, 0x77);
                for table in [22 * 512, 27 * 512] {
                    bytes[table + 8..table + 12].copy_from_slice(&384u32.to_le_bytes());
                }
            }
            fs::write(root.join(&backing), bytes).unwrap();
            if is_split {
                fs::write(root.join(name), text).unwrap();
            }
        }
        let mut expected = vec![0; 1048576];
        expected[..65536].fill(0x5a);
        expected[65536..131072].fill(0x33);
        Self {
            source: root.join("layer2.vmdk"),
            destination: root.join("output.raw"),
            root,
            expected,
        }
    }
    pub fn args(&self, command: &str, extras: &[&str]) -> Vec<OsString> {
        let mut args = vec![
            "rvddk".into(),
            command.into(),
            self.source.clone().into_os_string(),
        ];
        if command != "inspect" {
            args.push(self.destination.clone().into_os_string());
        }
        args.extend([
            "--format".into(),
            "vmdk".into(),
            "--json".into(),
            "--allow-parents".into(),
        ]);
        args.extend(extras.iter().map(OsString::from));
        args
    }
    pub fn run(&self, command: &str, extras: &[&str]) -> (i32, serde_json::Value) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = rvvdk_cli::run(self.args(command, extras), &mut out, &mut err);
        (
            code,
            serde_json::from_slice(if code == 0 {
                &out
            } else {
                assert!(out.is_empty());
                &err
            })
            .unwrap(),
        )
    }
    pub fn replace(&self, name: &str, from: &str, to: &str) {
        let p = self.root.join(name);
        let t = fs::read_to_string(&p).unwrap();
        assert!(t.contains(from));
        fs::write(p, t.replace(from, to)).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
