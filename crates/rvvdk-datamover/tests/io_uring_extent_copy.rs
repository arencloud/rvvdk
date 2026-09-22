#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{Extent, ExtentKind, RawDisk};

use rvvdk_datamover::IoUringExecutionOptions;

use rvvdk_datamover::io_uring::{
    NativeExtentPlan, copy_extent_plan, copy_extent_plan_with_destination,
};

use rvvdk_local::LocalFileBlockDevice;

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
    let source_path = temporary_path("data-source");

    let destination_path = temporary_path("data-destination");

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
            Extent::new(EXTENT_SIZE as u64, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
            Extent::new(
                (2 * EXTENT_SIZE) as u64,
                EXTENT_SIZE as u64,
                ExtentKind::Data,
            )
            .unwrap(),
        ],
        FILE_SIZE as u64,
    )
    .unwrap();

    let stats = copy_extent_plan(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    assert_eq!(stats.extents_processed(), 3,);

    drop(destination);
    drop(source);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_zero_extent_with_destination_semantics() {
    let source_path = temporary_path("zero-source");

    let destination_path = temporary_path("zero-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source_backend = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let plan = NativeExtentPlan::new(
        vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Zero).unwrap()],
        FILE_SIZE as u64,
    )
    .unwrap();

    let stats = copy_extent_plan_with_destination(
        source_backend.as_raw_fd(),
        destination.device().as_raw_fd(),
        &destination,
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    )
    .unwrap();

    /*
     * LocalFileBlockDevice does not advertise WRITE_ZERO here.
     *
     * Zero therefore uses ordinary zero-filled fallback writes.
     */
    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    assert_eq!(stats.extents_processed(), 1,);

    drop(destination);
    drop(source_backend);

    assert_eq!(fs::read(&destination_path,).unwrap(), vec![0_u8; FILE_SIZE],);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_data_zero_data_extent_plan() {
    let source_path = temporary_path("mixed-zero-source");

    let destination_path = temporary_path("mixed-zero-destination");

    let source_contents = source_data();

    fs::write(&source_path, &source_contents).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source_backend = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
            Extent::new(EXTENT_SIZE as u64, EXTENT_SIZE as u64, ExtentKind::Zero).unwrap(),
            Extent::new(
                (2 * EXTENT_SIZE) as u64,
                EXTENT_SIZE as u64,
                ExtentKind::Data,
            )
            .unwrap(),
        ],
        FILE_SIZE as u64,
    )
    .unwrap();

    let stats = copy_extent_plan_with_destination(
        source_backend.as_raw_fd(),
        destination.device().as_raw_fd(),
        &destination,
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    )
    .unwrap();

    /*
     * Two Data extents are read from the source.
     */
    assert_eq!(stats.bytes_read(), (2 * EXTENT_SIZE) as u64,);

    /*
     * Data writes = 4 MiB.
     * Zero fallback writes = 2 MiB.
     * Total ordinary writes = 6 MiB.
     */
    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    assert_eq!(stats.extents_processed(), 3,);

    let mut expected = source_contents;

    expected[EXTENT_SIZE..(2 * EXTENT_SIZE)].fill(0);

    drop(destination);
    drop(source_backend);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_hole_extent_with_destination_semantics() {
    let source_path = temporary_path("hole-source");

    let destination_path = temporary_path("hole-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source_backend = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let plan = NativeExtentPlan::new(
        vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Hole).unwrap()],
        FILE_SIZE as u64,
    )
    .unwrap();

    let stats = copy_extent_plan_with_destination(
        source_backend.as_raw_fd(),
        destination.device().as_raw_fd(),
        &destination,
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    )
    .unwrap();

    /*
     * LocalFileBlockDevice advertises neither DISCARD nor WRITE_ZERO
     * for this destination.
     *
     * Hole therefore uses ordinary zero-filled fallback writes.
     */
    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    assert_eq!(stats.extents_processed(), 1,);

    drop(destination);
    drop(source_backend);

    assert_eq!(fs::read(&destination_path,).unwrap(), vec![0_u8; FILE_SIZE],);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_data_hole_data_extent_plan() {
    let source_path = temporary_path("mixed-hole-source");

    let destination_path = temporary_path("mixed-hole-destination");

    let source_contents = source_data();

    fs::write(&source_path, &source_contents).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source_backend = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
            Extent::new(EXTENT_SIZE as u64, EXTENT_SIZE as u64, ExtentKind::Hole).unwrap(),
            Extent::new(
                (2 * EXTENT_SIZE) as u64,
                EXTENT_SIZE as u64,
                ExtentKind::Data,
            )
            .unwrap(),
        ],
        FILE_SIZE as u64,
    )
    .unwrap();

    let stats = copy_extent_plan_with_destination(
        source_backend.as_raw_fd(),
        destination.device().as_raw_fd(),
        &destination,
        &plan,
        BLOCK_SIZE,
        4096,
        IoUringExecutionOptions::new(8).unwrap(),
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), (2 * EXTENT_SIZE) as u64,);

    /*
     * Data writes = 4 MiB.
     * Hole fallback writes = 2 MiB.
     * Total ordinary writes = 6 MiB.
     */
    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    assert_eq!(stats.extents_processed(), 3,);

    let mut expected = source_contents;

    expected[EXTENT_SIZE..(2 * EXTENT_SIZE)].fill(0);

    drop(destination);
    drop(source_backend);

    assert_eq!(fs::read(&destination_path,).unwrap(), expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn low_level_extent_copy_still_rejects_zero() {
    let source_path = temporary_path("low-level-zero-source");

    let destination_path = temporary_path("low-level-zero-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let plan = NativeExtentPlan::new(
        vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Zero).unwrap()],
        FILE_SIZE as u64,
    )
    .unwrap();

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
fn low_level_extent_copy_still_rejects_hole() {
    let source_path = temporary_path("low-level-hole-source");

    let destination_path = temporary_path("low-level-hole-destination");

    fs::write(&source_path, vec![0_u8; FILE_SIZE]).unwrap();

    fs::write(&destination_path, vec![0xff_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let plan = NativeExtentPlan::new(
        vec![Extent::new(0, FILE_SIZE as u64, ExtentKind::Hole).unwrap()],
        FILE_SIZE as u64,
    )
    .unwrap();

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
