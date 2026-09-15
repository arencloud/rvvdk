use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{RawDisk, VirtualDisk};
use rvvdk_local::LocalFileBlockDevice;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-{name}-{unique}.img"))
}

#[test]
fn raw_disk_works_with_local_file_backend() {
    let path = temporary_path("raw");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_write(&path).unwrap();

    let disk = RawDisk::new(device);

    disk.write_at(512, b"rvvdk").unwrap();

    disk.flush().unwrap();

    let mut buffer = [0_u8; 5];

    disk.read_at(512, &mut buffer).unwrap();

    assert_eq!(&buffer, b"rvvdk");

    fs::remove_file(path).unwrap();
}
