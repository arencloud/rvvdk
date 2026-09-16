use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{AlignedBuffer, BlockDevice, Capabilities};
use rvvdk_local::LocalFileBlockDevice;

fn aligned_transfer_size(memory_alignment: usize, offset_alignment: usize) -> usize {
    let mut size = memory_alignment.max(offset_alignment);

    while !size.is_multiple_of(memory_alignment) || !size.is_multiple_of(offset_alignment) {
        size += memory_alignment.min(offset_alignment);
    }

    size
}

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

    let alignment = device
        .direct_io_alignment()
        .expect("direct device must expose alignment");

    assert!(alignment.memory_alignment() > 0);

    assert!(alignment.offset_alignment() > 0);

    assert!(device.capabilities().contains(Capabilities::DIRECT_IO,));

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_reads_and_writes_aligned_buffer() {
    let path = temporary_path("aligned");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();

    let alignment = device
        .direct_io_alignment()
        .expect("direct device must expose alignment");

    let memory_alignment = alignment.memory_alignment();

    let offset_alignment = alignment.offset_alignment();

    let transfer_size = aligned_transfer_size(memory_alignment, offset_alignment);

    let mut write_buffer = AlignedBuffer::new(transfer_size, memory_alignment).unwrap();

    write_buffer.fill(0x5a);

    device.write_all_at(0, write_buffer.as_slice()).unwrap();

    device.flush().unwrap();

    let mut read_buffer = AlignedBuffer::new(transfer_size, memory_alignment).unwrap();

    device.read_exact_at(0, read_buffer.as_mut_slice()).unwrap();

    assert_eq!(read_buffer.as_slice(), write_buffer.as_slice(),);

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_falls_back_for_unaligned_offset() {
    let path = temporary_path("offset-fallback");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();

    device.write_all_at(1, b"rvvdk").unwrap();

    device.flush().unwrap();

    let mut buffer = [0_u8; 5];

    device.read_exact_at(1, &mut buffer).unwrap();

    assert_eq!(&buffer, b"rvvdk",);

    fs::remove_file(path).unwrap();
}

#[test]
fn direct_io_handles_aligned_bulk_and_unaligned_tail() {
    const ALIGNMENT: usize = 4096;
    const TAIL: usize = 123;

    let size = ALIGNMENT * 4 + TAIL;

    let path = temporary_path("tail");

    fs::write(&path, vec![0_u8; size]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();

    let mut bulk = AlignedBuffer::new(ALIGNMENT * 4, ALIGNMENT).unwrap();

    bulk.fill(0xaa);

    device.write_all_at(0, bulk.as_slice()).unwrap();

    let tail = vec![0xbb_u8; TAIL];

    device.write_all_at((ALIGNMENT * 4) as u64, &tail).unwrap();

    device.flush().unwrap();

    let data = fs::read(&path).unwrap();

    assert!(data[..ALIGNMENT * 4].iter().all(|value| *value == 0xaa));

    assert!(data[ALIGNMENT * 4..].iter().all(|value| *value == 0xbb));

    fs::remove_file(path).unwrap();
}

#[test]
fn discovers_direct_io_alignment() {
    let path = temporary_path("alignment-discovery");

    fs::write(&path, vec![0_u8; 1024 * 1024]).unwrap();

    let device = LocalFileBlockDevice::open_direct_read_only(&path).unwrap();

    let alignment = device
        .direct_io_alignment()
        .expect("direct device must expose alignment");

    //println!(
    //    "memory_alignment={}, offset_alignment={}",
    //    alignment.memory_alignment(),
    //    alignment.offset_alignment(),
    //);

    //println!(
    //    "memory_alignment={}, offset_alignment={}, source={:?}",
    //    alignment.memory_alignment(),
    //    alignment.offset_alignment(),
    //    alignment.source(),
    //);

    assert!(alignment.memory_alignment() > 0);

    assert!(alignment.offset_alignment() > 0);

    fs::remove_file(path).unwrap();
}

#[test]
fn buffered_device_has_no_direct_io_alignment() {
    let path = temporary_path("buffered-alignment");

    fs::write(&path, vec![0_u8; 4096]).unwrap();

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    assert!(device.direct_io_alignment().is_none());

    fs::remove_file(path).unwrap();
}
