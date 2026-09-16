#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions};

use rvvdk_local::LocalFileBlockDevice;

const BLOCK_SIZE: usize = 64 * 1024;

const FILE_SIZE: usize = 8 * 1024 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-native-copy-{name}-{unique}.img"))
}

#[test]
fn datamover_copies_with_io_uring_native_execution() {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let mut expected = vec![0_u8; FILE_SIZE];

    for (index, value) in expected.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

    let copy_options = CopyOptions::new(BLOCK_SIZE).unwrap();

    let io_uring_options = IoUringExecutionOptions::new(8).unwrap();

    let mover = DataMover::with_execution_strategy(
        copy_options,
        ExecutionStrategy::IoUring(io_uring_options),
    );

    let stats = mover
        .copy_native(&source, &destination, 0, FILE_SIZE as u64)
        .unwrap();

    assert_eq!(stats.bytes_read(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn native_copy_requires_native_strategy() {
    let source_path = temporary_path("threaded-source");

    let destination_path = temporary_path("threaded-destination");

    fs::write(&source_path, vec![0_u8; BLOCK_SIZE]).unwrap();

    fs::write(&destination_path, vec![0_u8; BLOCK_SIZE]).unwrap();

    let source = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let result = mover.copy_native(&source, &destination, 0, BLOCK_SIZE as u64);

    assert!(matches!(
        result,
        Err(rvvdk_core::Error::NativeExecutionNotSelected)
    ));

    drop(destination);
    drop(source);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn datamover_copies_with_io_uring_direct_execution() {
    const DIRECT_BLOCK_SIZE: usize = 1024 * 1024;

    const DIRECT_FILE_SIZE: usize = 8 * DIRECT_BLOCK_SIZE;

    let source_path = temporary_path("direct-source");

    let destination_path = temporary_path("direct-destination");

    let mut expected = vec![0_u8; DIRECT_FILE_SIZE];

    for (index, value) in expected.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; DIRECT_FILE_SIZE]).unwrap();

    let source = LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap();

    let copy_options = CopyOptions::new(DIRECT_BLOCK_SIZE).unwrap();

    let io_uring_options = IoUringExecutionOptions::new(8).unwrap();

    let mover = DataMover::with_execution_strategy(
        copy_options,
        ExecutionStrategy::IoUring(io_uring_options),
    );

    let stats = mover
        .copy_native(&source, &destination, 0, DIRECT_FILE_SIZE as u64)
        .unwrap();

    assert_eq!(stats.bytes_read(), DIRECT_FILE_SIZE as u64,);

    assert_eq!(stats.bytes_written(), DIRECT_FILE_SIZE as u64,);

    assert_eq!(
        stats.blocks_completed(),
        (DIRECT_FILE_SIZE / DIRECT_BLOCK_SIZE) as u64,
    );

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}
