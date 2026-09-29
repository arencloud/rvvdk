#![cfg(target_os = "linux")]
use rvvdk_core::{Error, RawDisk};
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_vmdk::{Limits, LocalResolver, ResolutionLimits, ResolvedDescriptor, VmdkDisk};
use std::{
    fs::{self, File},
    path::PathBuf,
};
#[test]
fn local_second_backing_hardlink_and_late_truncation_fail_before_mutation() {
    let root = std::env::var_os("RVVDK_TEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = root.join(format!("rvvdk-logical-local-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("a"), [7; 4096]).unwrap();
    fs::write(path.join("b"), [9; 4096]).unwrap();
    fs::hard_link(path.join("b"), path.join("alias")).unwrap();
    fs::write(path.join("disk"),"version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 2 FLAT \"a\" 1\nRW 1 ZERO\nRW 1 FLAT \"b\" 0\n").unwrap();
    let (text, resolver) =
        LocalResolver::open_descriptor(path.join("disk"), Limits::default()).unwrap();
    let disk = VmdkDisk::new(
        ResolvedDescriptor::resolve(
            &text.parse().unwrap(),
            &resolver,
            ResolutionLimits::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let alias = RawDisk::new(LocalFileBlockDevice::open_read_write(path.join("alias")).unwrap());
    let mover = DataMover::new(CopyOptions::with_concurrency(512, 4096, 4).unwrap());
    assert!(matches!(
        mover.copy(&disk, &alias),
        Err(Error::AliasedEndpoints)
    ));
    assert_eq!(fs::read(path.join("b")).unwrap(), [9; 4096]);
    fs::write(path.join("out"), [0xa5; 4096]).unwrap();
    let out = RawDisk::new(LocalFileBlockDevice::open_read_write(path.join("out")).unwrap());
    mover.copy(&disk, &out).unwrap();
    let bytes = fs::read(path.join("out")).unwrap();
    assert_eq!(&bytes[..1024], &[7; 1024]);
    assert_eq!(&bytes[1024..1536], &[0; 512]);
    assert_eq!(&bytes[1536..2048], &[9; 512]);
    assert_eq!(&bytes[2048..], &[0xa5; 2048]);
    let plan = mover.plan_with_destination(&disk, &out).unwrap();
    File::options()
        .write(true)
        .open(path.join("b"))
        .unwrap()
        .set_len(511)
        .unwrap();
    assert!(mover.execute_plan(&plan, &disk, &out).is_err());
    assert_eq!(fs::read(path.join("out")).unwrap(), bytes);
    drop(out);
    drop(alias);
    drop(disk);
    drop(resolver);
    fs::remove_dir_all(path).unwrap();
}
