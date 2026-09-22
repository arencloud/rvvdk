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

const FILE_SIZE: usize = 16 * MIB;

const BLOCK_SIZE: usize = 64 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-plan-parity-{name}-{unique}.img"))
}

fn source_contents() -> Vec<u8> {
    let mut contents = vec![0_u8; FILE_SIZE];

    for (index, value) in contents.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    contents
}

fn create_source(path: &PathBuf) {
    fs::write(path, source_contents()).unwrap();
}

fn create_destination(path: &PathBuf) {
    fs::write(path, vec![0xff_u8; FILE_SIZE]).unwrap();
}

fn open_source(path: &PathBuf) -> RawDisk<LocalFileBlockDevice> {
    RawDisk::new(LocalFileBlockDevice::open_read_only(path).unwrap())
}

fn open_destination(path: &PathBuf) -> RawDisk<LocalFileBlockDevice> {
    RawDisk::new(LocalFileBlockDevice::open_read_write(path).unwrap())
}

fn cleanup(paths: &[PathBuf]) {
    for path in paths {
        fs::remove_file(path).unwrap();
    }
}

fn run_parity(name: &str, mover: DataMover, expected_backend: ExecutionBackend) {
    let source_path = temporary_path(&format!("{name}-source"));

    let direct_path = temporary_path(&format!("{name}-direct"));

    let planned_path = temporary_path(&format!("{name}-planned"));

    create_source(&source_path);

    create_destination(&direct_path);

    create_destination(&planned_path);

    /*
     * Direct workflow.
     */
    let direct_source = open_source(&source_path);

    let direct_destination = open_destination(&direct_path);

    let direct_report = mover
        .copy_with_report(&direct_source, &direct_destination)
        .unwrap();

    assert_eq!(direct_report.backend(), expected_backend,);

    let direct_stats = *direct_report.stats();

    drop(direct_destination);
    drop(direct_source);

    /*
     * Explicit plan -> execute workflow.
     */
    let planned_source = open_source(&source_path);

    let planned_destination = open_destination(&planned_path);

    let plan = mover
        .plan_with_destination(&planned_source, &planned_destination)
        .unwrap();

    assert_eq!(plan.backend(), expected_backend,);

    let planned_report = mover
        .execute_plan(&plan, &planned_source, &planned_destination)
        .unwrap();

    assert_eq!(planned_report.backend(), expected_backend,);

    let planned_stats = *planned_report.stats();

    drop(planned_destination);
    drop(planned_source);

    /*
     * Logical output parity.
     */
    let expected = fs::read(&source_path).unwrap();

    let direct_contents = fs::read(&direct_path).unwrap();

    let planned_contents = fs::read(&planned_path).unwrap();

    assert_eq!(direct_contents, expected,);

    assert_eq!(planned_contents, expected,);

    assert_eq!(direct_contents, planned_contents,);

    /*
     * Deterministic statistics parity.
     *
     * elapsed is deliberately excluded.
     */
    assert_eq!(direct_stats.bytes_read(), planned_stats.bytes_read(),);

    assert_eq!(direct_stats.bytes_written(), planned_stats.bytes_written(),);

    assert_eq!(direct_stats.bytes_zeroed(), planned_stats.bytes_zeroed(),);

    assert_eq!(
        direct_stats.bytes_discarded(),
        planned_stats.bytes_discarded(),
    );

    assert_eq!(direct_stats.blocks_copied(), planned_stats.blocks_copied(),);

    assert_eq!(
        direct_stats.extents_processed(),
        planned_stats.extents_processed(),
    );

    cleanup(&[source_path, direct_path, planned_path]);
}

#[test]
fn threaded_direct_and_planned_execution_are_equivalent() {
    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    run_parity("threaded", mover, ExecutionBackend::Threaded);
}

#[test]
fn io_uring_direct_and_planned_execution_are_equivalent() {
    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );

    run_parity("io-uring", mover, ExecutionBackend::IoUring);
}

#[test]
fn auto_direct_and_planned_execution_are_equivalent() {
    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );

    run_parity("auto", mover, ExecutionBackend::IoUring);
}
