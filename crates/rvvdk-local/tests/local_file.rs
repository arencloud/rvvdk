use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{BlockDevice, Capabilities, Error};
use rvvdk_local::LocalFileBlockDevice;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-{name}-{unique}.img"))
}

#[test]
fn opens_file_read_only() {
    let path = temporary_path("readonly");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    assert_eq!(device.size(), 4096);

    assert!(device.capabilities().contains(Capabilities::READ));

    assert!(!device.capabilities().contains(Capabilities::WRITE));

    fs::remove_file(path).unwrap();
}

#[test]
fn reads_at_offset() {
    let path = temporary_path("read");

    let mut data = vec![0_u8; 4096];

    data[1024..1029].copy_from_slice(b"rvvdk");

    fs::write(&path, data).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let mut buffer = [0_u8; 5];

    let bytes_read = device.read_at(1024, &mut buffer).unwrap();

    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b"rvvdk");

    fs::remove_file(path).unwrap();
}

#[test]
fn writes_at_offset() {
    let path = temporary_path("write");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_write(&path).unwrap();

    let bytes_written = device.write_at(2048, b"rvvdk").unwrap();

    assert_eq!(bytes_written, 5);

    device.flush().unwrap();

    let data = fs::read(&path).unwrap();

    assert_eq!(&data[2048..2053], b"rvvdk",);

    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_out_of_bounds_read() {
    let path = temporary_path("bounds");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let mut buffer = [0_u8; 512];

    let result = device.read_at(4000, &mut buffer);

    assert!(matches!(result, Err(Error::OutOfBounds { .. })));

    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_write_on_read_only_device() {
    let path = temporary_path("readonly-write");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let result = device.write_at(0, b"rvvdk");

    assert!(matches!(result, Err(Error::Unsupported)));

    fs::remove_file(path).unwrap();
}

#[test]
fn does_not_depend_on_file_cursor() {
    let path = temporary_path("positional");

    let mut file = File::create(&path).unwrap();

    file.write_all(&vec![0_u8; 4096]).unwrap();

    drop(file);

    let device = LocalFileBlockDevice::open_read_write(&path).unwrap();

    device.write_at(100, b"first").unwrap();

    device.write_at(2000, b"second").unwrap();

    let mut first = [0_u8; 5];
    let mut second = [0_u8; 6];

    device.read_at(100, &mut first).unwrap();

    device.read_at(2000, &mut second).unwrap();

    assert_eq!(&first, b"first");
    assert_eq!(&second, b"second");

    fs::remove_file(path).unwrap();
}
