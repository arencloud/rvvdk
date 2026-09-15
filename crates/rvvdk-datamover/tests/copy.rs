use rvvdk_core::{MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover};

#[test]
fn copies_memory_disk() {
    let source_device = MemoryBlockDevice::new(4096).unwrap();

    let destination_device = MemoryBlockDevice::new(4096).unwrap();

    let source = RawDisk::new(source_device);
    let destination = RawDisk::new(destination_device);

    source.write_at(0, b"rvvdk").unwrap();

    source.write_at(2048, b"datamover").unwrap();

    let mover = DataMover::new(CopyOptions::new(512).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    assert_eq!(stats.bytes_read(), 4096);
    assert_eq!(stats.bytes_written(), 4096);
    assert_eq!(stats.blocks_copied(), 8);

    let mut first = [0_u8; 5];
    let mut second = [0_u8; 9];

    destination.read_at(0, &mut first).unwrap();

    destination.read_at(2048, &mut second).unwrap();

    assert_eq!(&first, b"rvvdk");
    assert_eq!(&second, b"datamover");
}

#[test]
fn copies_final_partial_block() {
    let source_device = MemoryBlockDevice::new(1000).unwrap();

    let destination_device = MemoryBlockDevice::new(1000).unwrap();

    let source = RawDisk::new(source_device);
    let destination = RawDisk::new(destination_device);

    source.write_at(995, b"rvvdk").unwrap();

    let mover = DataMover::new(CopyOptions::new(256).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    assert_eq!(stats.bytes_written(), 1000);
    assert_eq!(stats.blocks_copied(), 4);

    let mut buffer = [0_u8; 5];

    destination.read_at(995, &mut buffer).unwrap();

    assert_eq!(&buffer, b"rvvdk");
}

#[test]
fn rejects_destination_smaller_than_source() {
    let source = RawDisk::new(MemoryBlockDevice::new(4096).unwrap());

    let destination = RawDisk::new(MemoryBlockDevice::new(2048).unwrap());

    let mover = DataMover::new(CopyOptions::default());

    assert!(mover.copy(&source, &destination).is_err());
}
