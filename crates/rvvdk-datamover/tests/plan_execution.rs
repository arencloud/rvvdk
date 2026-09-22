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

    std::env::temp_dir().join(format!("rvvdk-plan-execution-{name}-{unique}.img"))
}

fn source_contents() -> Vec<u8> {
    let mut contents = vec![0_u8; FILE_SIZE];

    for (index, value) in contents.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    contents
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

    let expected = source_contents();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    (source_path, destination_path, expected, source, destination)
}

fn cleanup(source_path: PathBuf, destination_path: PathBuf) {
    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn threaded_plan_executes_successfully() {
    let (source_path, destination_path, expected, source, destination) = create_disks("threaded");

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::Threaded,);

    let report = mover.execute_plan(&plan, &source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::Threaded,);

    assert_eq!(report.stats().bytes_read(), FILE_SIZE as u64,);

    assert_eq!(report.stats().bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    cleanup(source_path, destination_path);
}

#[test]
fn io_uring_plan_executes_successfully() {
    let (source_path, destination_path, expected, source, destination) = create_disks("io-uring");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::IoUring,);

    let report = mover.execute_plan(&plan, &source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::IoUring,);

    assert_eq!(report.stats().bytes_read(), FILE_SIZE as u64,);

    assert_eq!(report.stats().bytes_written(), FILE_SIZE as u64,);

    drop(destination);
    drop(source);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    cleanup(source_path, destination_path);
}

#[test]
fn auto_plan_executes_selected_backend() {
    let (source_path, destination_path, expected, source, destination) = create_disks("auto");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    assert_eq!(plan.backend(), ExecutionBackend::IoUring,);

    let report = mover.execute_plan(&plan, &source, &destination).unwrap();

    assert_eq!(report.backend(), plan.backend(),);

    drop(destination);
    drop(source);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    cleanup(source_path, destination_path);
}

#[test]
fn planned_execution_rejects_changed_source_size() {
    let (source_path, destination_path, _, source, destination) = create_disks("changed-source");

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let plan = mover.plan_with_destination(&source, &destination).unwrap();

    drop(source);

    /*
     * Change source geometry after planning.
     *
     * M20D performs basic source-size stale-plan detection.
     */
    let source_file = fs::OpenOptions::new()
        .write(true)
        .open(&source_path)
        .unwrap();

    source_file.set_len((FILE_SIZE / 2) as u64).unwrap();

    drop(source_file);

    let changed_source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

    let result = mover.execute_plan(&plan, &changed_source, &destination);

    assert!(result.is_err());

    drop(destination);
    drop(changed_source);

    cleanup(source_path, destination_path);
}

#[test]
fn copy_with_report_uses_plan_execution_path() {
    let (source_path, destination_path, expected, source, destination) =
        create_disks("copy-with-report");

    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    let report = mover.copy_with_report(&source, &destination).unwrap();

    assert_eq!(report.backend(), ExecutionBackend::IoUring,);

    drop(destination);
    drop(source);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    cleanup(source_path, destination_path);
}
