#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::RawDisk;

use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
};

use rvvdk_local::LocalFileBlockDevice;

const MIB: usize = 1024 * 1024;

const FILE_SIZE: usize = 8 * MIB;

const BLOCK_SIZE: usize = 64 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-copy-plan-{name}-{unique}.img"))
}

fn create_disks(
    name: &str,
) -> (
    PathBuf,
    PathBuf,
    RawDisk<LocalFileBlockDevice>,
    RawDisk<LocalFileBlockDevice>,
) {
    let source_path = temporary_path(&format!("{name}-source"));

    let destination_path = temporary_path(&format!("{name}-destination"));

    fs::write(&source_path, vec![0x5a_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    (source_path, destination_path, source, destination)
}

fn cleanup(source_path: PathBuf, destination_path: PathBuf) {
    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn portable_plan_reports_threaded_backend() {
    let (source_path, destination_path, source, destination) = create_disks("portable");

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let plan = mover.plan(&source).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::Threaded,);

    assert_eq!(plan.logical_bytes(), FILE_SIZE as u64,);

    assert_eq!(plan.data_bytes(), FILE_SIZE as u64,);

    assert_eq!(plan.zero_bytes(), 0,);

    assert_eq!(plan.hole_bytes(), 0,);

    assert_eq!(plan.extent_count(), 1,);

    drop(destination);
    drop(source);

    cleanup(source_path, destination_path);
}

#[test]
fn explicit_io_uring_plan_reports_io_uring_backend() {
    let (source_path, destination_path, source, destination) = create_disks("io-uring");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::IoUring,);

    assert_eq!(plan.logical_bytes(), FILE_SIZE as u64,);

    assert_eq!(plan.data_bytes(), FILE_SIZE as u64,);

    assert_eq!(plan.block_size(), BLOCK_SIZE,);

    /*
     * Local buffered files have no stronger requirement than the
     * configured CopyOptions alignment in the resulting plan.
     */
    assert!(plan.alignment() >= 4096);

    drop(destination);
    drop(source);

    cleanup(source_path, destination_path);
}

#[test]
fn auto_plan_selects_io_uring_for_compatible_linux_backends() {
    let (source_path, destination_path, source, destination) = create_disks("auto");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::IoUring,);

    assert_eq!(plan.logical_bytes(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    cleanup(source_path, destination_path);
}

#[test]
fn destination_smaller_than_source_is_rejected_during_planning() {
    let source_path = temporary_path("small-source");

    let destination_path = temporary_path("small-destination");

    fs::write(&source_path, vec![0x5a_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE / 2]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    let result = mover.plan_with_destination(&source, &destination);

    assert!(result.is_err());

    drop(destination);
    drop(source);

    cleanup(source_path, destination_path);
}
