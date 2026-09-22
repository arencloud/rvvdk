#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{Extent, ExtentKind};

use rvvdk_datamover::IoUringExecutionOptions;

use rvvdk_datamover::io_uring::{NativeExtentPlan, copy_extent_plan};

const BLOCK_SIZE: usize = 64 * 1024;

const EXTENT_SIZE: usize = 2 * 1024 * 1024;

const FILE_SIZE: usize = 3 * EXTENT_SIZE;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-extent-copy-{name}-{unique}.img"))
}

fn source_data() -> Vec<u8> {
    let mut data = vec![0_u8; FILE_SIZE];

    for (index, value) in data.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    data
}

#[test]
fn copies_multiple_data_extents() {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let extents = vec![
        Extent::new(0, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
        Extent::new(EXTENT_SIZE as u64, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
        Extent::new(
            (2 * EXTENT_SIZE) as u64,
            EXTENT_SIZE as u64,
            ExtentKind::Data,
        )
        .unwrap(),
    ];

    let plan = NativeExtentPlan::new(extents, FILE_SIZE as u64).unwrap();

    let options = IoUringExecutionOptions::new(8).unwrap();

    let stats = copy_extent_plan(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        &plan,
        BLOCK_SIZE,
        4096,
        options,
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.extents_processed(), 3,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn rejects_zero_extent_for_now() {
    let source_path = temporary_path("zero-source");

    let destination_path = temporary_path("zero-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let extents = vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Zero).unwrap()];

    let plan = NativeExtentPlan::new(extents, FILE_SIZE as u64).unwrap();

    let result = copy_extent_plan(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    );

    assert!(matches!(
        result,
        Err(rvvdk_core::Error::UnsupportedNativeExtent { kind: "zero", .. })
    ));

    drop(destination);
    drop(source);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn rejects_hole_extent_for_now() {
    let source_path = temporary_path("hole-source");

    let destination_path = temporary_path("hole-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let extents = vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Hole).unwrap()];

    let plan = NativeExtentPlan::new(extents, FILE_SIZE as u64).unwrap();

    let result = copy_extent_plan(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    );

    assert!(matches!(
        result,
        Err(rvvdk_core::Error::UnsupportedNativeExtent { kind: "hole", .. })
    ));

    drop(destination);
    drop(source);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}
