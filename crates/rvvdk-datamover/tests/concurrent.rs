use rvvdk_core::{MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover};

#[test]
fn concurrent_copy_matches_source() {
    let size = 8 * 1024 * 1024;

    let source = RawDisk::new(MemoryBlockDevice::new(size).unwrap());

    let destination = RawDisk::new(MemoryBlockDevice::new(size).unwrap());

    for offset in [0, 1024 * 1024, 3 * 1024 * 1024, 7 * 1024 * 1024] {
        source.write_all_at(offset as u64, b"rvvdk").unwrap();
    }

    let options = CopyOptions::with_concurrency(256 * 1024, 4096, 4).unwrap();

    let mover = DataMover::new(options);

    let stats = mover.copy(&source, &destination).unwrap();

    assert_eq!(stats.bytes_read(), size as u64,);

    assert_eq!(stats.bytes_written(), size as u64,);

    let mut source_buffer = vec![0_u8; size];

    let mut destination_buffer = vec![0_u8; size];

    source.read_exact_at(0, &mut source_buffer).unwrap();

    destination
        .read_exact_at(0, &mut destination_buffer)
        .unwrap();

    assert_eq!(destination_buffer, source_buffer,);
}

#[test]
fn different_concurrency_levels_produce_identical_results() {
    const SIZE: usize = 4 * 1024 * 1024;

    for concurrency in [1, 2, 4, 8] {
        let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());

        let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());

        source.write_all_at(12345, b"concurrent-rvvdk").unwrap();

        source
            .write_all_at(2 * 1024 * 1024, b"second-region")
            .unwrap();

        let options = CopyOptions::with_concurrency(128 * 1024, 4096, concurrency).unwrap();

        DataMover::new(options).copy(&source, &destination).unwrap();

        let mut expected = vec![0_u8; SIZE];

        let mut actual = vec![0_u8; SIZE];

        source.read_exact_at(0, &mut expected).unwrap();

        destination.read_exact_at(0, &mut actual).unwrap();

        assert_eq!(
            actual, expected,
            "copy mismatch at concurrency {concurrency}",
        );
    }
}

#[test]
fn concurrent_copy_preserves_block_boundaries() {
    const BLOCK_SIZE: usize = 64 * 1024;

    const BLOCKS: usize = 64;

    const SIZE: usize = BLOCK_SIZE * BLOCKS;

    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());

    let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());

    for block in 0..BLOCKS {
        let value = u8::try_from(block).unwrap();

        let data = vec![value; BLOCK_SIZE];

        source
            .write_all_at((block * BLOCK_SIZE) as u64, &data)
            .unwrap();
    }

    let options = CopyOptions::with_concurrency(BLOCK_SIZE, 4096, 8).unwrap();

    DataMover::new(options).copy(&source, &destination).unwrap();

    let mut buffer = vec![0_u8; BLOCK_SIZE];

    for block in 0..BLOCKS {
        destination
            .read_exact_at((block * BLOCK_SIZE) as u64, &mut buffer)
            .unwrap();

        let expected = u8::try_from(block).unwrap();

        assert!(
            buffer.iter().all(|value| { *value == expected },),
            "block {block} was corrupted",
        );
    }
}
