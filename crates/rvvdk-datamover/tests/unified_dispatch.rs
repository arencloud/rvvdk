#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::RawDisk;

use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
};

use rvvdk_local::LocalFileBlockDevice;

const BLOCK_SIZE: usize = 64 * 1024;

const FILE_SIZE: usize = 8 * 1024 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-unified-{name}-{unique}.img"))
}

fn source_data() -> Vec<u8> {
    let mut data = vec![0_u8; FILE_SIZE];

    for (index, value) in data.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    data
}

fn create_disks(
    name: &str,
) -> (
    PathBuf,
    PathBuf,
    Vec<u8>,
    RawDisk<LocalFileBlockDevice>,
    RawDisk<LocalFileBlockDevice>,
) {
    let source_path = temporary_path(&format!("{name}-source"));

    let destination_path = temporary_path(&format!("{name}-destination"));

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    (source_path, destination_path, expected, source, destination)
}

fn verify_and_cleanup(source_path: PathBuf, destination_path: PathBuf, expected: &[u8]) {
    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn threaded_strategy_reports_threaded_backend() {
    let (source_path, destination_path, expected, source, destination) = create_disks("threaded");

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let report = mover.copy_with_report(&source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::Threaded,);

    assert_eq!(report.stats().bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    verify_and_cleanup(source_path, destination_path, &expected);
}

#[test]
fn explicit_io_uring_reports_io_uring_backend() {
    let (source_path, destination_path, expected, source, destination) = create_disks("io-uring");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );

    let report = mover.copy_with_report(&source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::IoUring,);

    assert_eq!(report.stats().bytes_read(), FILE_SIZE as u64,);

    assert_eq!(report.stats().bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    verify_and_cleanup(source_path, destination_path, &expected);
}

#[test]
fn auto_selects_io_uring_for_linux_fd_backends() {
    let (source_path, destination_path, expected, source, destination) = create_disks("auto");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    let report = mover.copy_with_report(&source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::IoUring,);

    assert_eq!(report.stats().bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    verify_and_cleanup(source_path, destination_path, &expected);
}
