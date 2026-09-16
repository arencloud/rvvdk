#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_local::LocalFileBlockDevice;

use rvvdk_platform::LinuxFdBackend;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-linux-backend-{name}-{unique}.img"))
}

#[test]
fn buffered_file_reports_buffered_capability() {
    let path = temporary_path("buffered");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let capabilities = device.linux_fd_capabilities();

    assert!(!capabilities.direct_io());

    assert_eq!(capabilities.memory_alignment(), 1,);

    assert_eq!(capabilities.offset_alignment(), 1,);

    assert!(device.raw_fd() >= 0);

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_file_reports_direct_capability() {
    let path = temporary_path("direct");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_only(&path).unwrap();

    let capabilities = device.linux_fd_capabilities();

    assert!(capabilities.direct_io());

    assert!(capabilities.memory_alignment() > 0);

    assert!(capabilities.offset_alignment() > 0);

    assert!(device.raw_fd() >= 0);

    fs::remove_file(path).unwrap();
}
