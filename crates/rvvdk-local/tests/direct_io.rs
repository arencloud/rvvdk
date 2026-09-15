use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{AlignedBuffer, BlockDevice, Capabilities, Error};
use rvvdk_local::LocalFileBlockDevice;

const ALIGNMENT: usize = 4096;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-direct-{name}-{unique}.img"))
}

#[test]
fn opens_direct_io_device() {
    let path = temporary_path("open");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();

    assert!(device.is_direct_io());

    assert_eq!(device.io_alignment(), ALIGNMENT,);

    assert!(device.capabilities().contains(Capabilities::DIRECT_IO,));

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_reads_and_writes_aligned_buffer() {
    let path = temporary_path("aligned");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();

    let mut write_buffer = AlignedBuffer::new(ALIGNMENT, ALIGNMENT).unwrap();

    write_buffer.fill(0x5a);

    device.write_all_at(0, write_buffer.as_slice()).unwrap();

    device.flush().unwrap();

    let mut read_buffer = AlignedBuffer::new(ALIGNMENT, ALIGNMENT).unwrap();

    device.read_exact_at(0, read_buffer.as_mut_slice()).unwrap();

    assert_eq!(read_buffer.as_slice(), write_buffer.as_slice(),);

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_rejects_unaligned_offset() {
    let path = temporary_path("offset");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_only(&path).unwrap();

    let mut buffer = AlignedBuffer::new(ALIGNMENT, ALIGNMENT).unwrap();

    let result = device.read_at(1, buffer.as_mut_slice());

    assert!(matches!(result, Err(Error::DirectIoAlignment { .. })));

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_rejects_unaligned_length() {
    let path = temporary_path("length");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_only(&path).unwrap();

    let mut buffer = AlignedBuffer::new(ALIGNMENT, ALIGNMENT).unwrap();

    let result = device.read_at(0, &mut buffer.as_mut_slice()[..2048]);

    assert!(matches!(result, Err(Error::DirectIoAlignment { .. })));

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_rejects_unaligned_buffer_address() {
    let path = temporary_path("address");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_only(&path).unwrap();

    let mut buffer = AlignedBuffer::new(ALIGNMENT * 2, ALIGNMENT).unwrap();

    let slice = &mut buffer.as_mut_slice()[1..ALIGNMENT + 1];

    let result = device.read_at(0, slice);

    assert!(matches!(result, Err(Error::DirectIoAlignment { .. })));

    fs::remove_file(path).unwrap();
}
