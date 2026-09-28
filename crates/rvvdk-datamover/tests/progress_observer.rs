#![cfg(target_os = "linux")]

use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionBackend, ProgressSnapshot};
use rvvdk_local::LocalFileBlockDevice;

const MIB: usize = 1024 * 1024;
const FILE_SIZE: usize = 8 * MIB;
const BLOCK_SIZE: usize = 64 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("rvvdk-progress-{name}-{unique}.img"))
}

fn create_sparse_source(path: &PathBuf) {
    let mut file = File::create(path).unwrap();
    file.write_all(&vec![0x5a_u8; MIB]).unwrap();
    file.seek(SeekFrom::Start((4 * MIB) as u64)).unwrap();
    file.write_all(&vec![0xa5_u8; MIB]).unwrap();
    file.set_len(FILE_SIZE as u64).unwrap();
    file.sync_all().unwrap();
}

fn assert_monotonic(snapshots: &[ProgressSnapshot]) {
    for pair in snapshots.windows(2) {
        assert!(
            pair[1].logical_bytes_completed() >= pair[0].logical_bytes_completed(),
            "progress regressed: {} -> {}",
            pair[0].logical_bytes_completed(),
            pair[1].logical_bytes_completed(),
        );
    }
}

#[test]
fn sparse_threaded_execution_emits_intermediate_progress() {
    let source_path = temporary_path("source");
    let destination_path = temporary_path("destination");

    create_sparse_source(&source_path);
    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());
    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());
    let plan = mover
        .plan_raw_with_destination(&source, &destination)
        .unwrap();

    assert!(
        plan.extent_count() > 1,
        "source must contain sparse extents"
    );

    let snapshots = RefCell::new(Vec::<ProgressSnapshot>::new());
    let observer = |snapshot: &ProgressSnapshot| {
        snapshots.borrow_mut().push(*snapshot);
    };

    let report = mover
        .execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
        .unwrap();

    assert_eq!(report.backend(), ExecutionBackend::Threaded);

    let snapshots = snapshots.into_inner();

    assert!(
        snapshots.len() > 2,
        "expected initial, intermediate, and final snapshots; got {}",
        snapshots.len(),
    );

    assert_eq!(snapshots.first().unwrap().logical_bytes_completed(), 0);
    assert_eq!(
        snapshots.last().unwrap().logical_bytes_completed(),
        FILE_SIZE as u64,
    );
    assert_eq!(snapshots.last().unwrap().logical_completion(), 1.0);

    assert!(
        snapshots[1..snapshots.len() - 1].iter().any(|snapshot| {
            snapshot.logical_bytes_completed() > 0
                && snapshot.logical_bytes_completed() < FILE_SIZE as u64
        }),
        "expected at least one intermediate logical progress snapshot",
    );

    assert_monotonic(&snapshots);

    drop(destination);
    drop(source);

    let expected = fs::read(&source_path).unwrap();
    let actual = fs::read(&destination_path).unwrap();
    assert_eq!(actual, expected);

    fs::remove_file(source_path).unwrap();
    fs::remove_file(destination_path).unwrap();
}
