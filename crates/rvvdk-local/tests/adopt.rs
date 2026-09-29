#![cfg(target_os = "linux")]
use rvvdk_core::{BlockDevice, Capabilities};
use rvvdk_local::LocalFileBlockDevice;
use std::{
    fs::{self, File},
    os::unix::fs::OpenOptionsExt,
};
#[test]
fn adoption_derives_rights_and_retains_unlinked_file() {
    let path = std::env::temp_dir().join(format!("rvvdk-adopt-{}", std::process::id()));
    fs::write(&path, [0x5a; 4096]).unwrap();
    let ro = LocalFileBlockDevice::from_buffered_file(File::open(&path).unwrap()).unwrap();
    assert!(!ro.capabilities().contains(Capabilities::WRITE));
    assert!(ro.write_at(0, &[1]).is_err());
    for flags in [libc::O_APPEND, libc::O_DIRECT, libc::O_PATH] {
        let file = File::options()
            .read(true)
            .custom_flags(flags)
            .open(&path)
            .unwrap();
        assert!(LocalFileBlockDevice::from_buffered_file(file).is_err());
    }
    assert!(LocalFileBlockDevice::from_buffered_file(File::open("/dev/null").unwrap()).is_err());
    fs::remove_file(path).unwrap();
    let mut bytes = [0; 7];
    ro.read_exact_at(0, &mut bytes).unwrap();
    assert_eq!(bytes, [0x5a; 7]);
}
