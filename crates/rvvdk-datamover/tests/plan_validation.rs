#![cfg(target_os = "linux")]

mod support;

use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rvvdk_core::{Capabilities, Extent, ExtentKind, RawDisk, VirtualDisk};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, ProgressSnapshot,
};

use support::test_block_device::TestExtentBlockDevice;

const SIZE: usize = 16 * 1024;
const BLOCK: usize = 4096;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "rvvdk-plan-validation-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("source"), vec![0x5a; SIZE]).unwrap();
        Self(path)
    }

    fn source(&self, size: usize, kind: ExtentKind) -> RawDisk<TestExtentBlockDevice> {
        RawDisk::new(
            TestExtentBlockDevice::open_read_only(
                self.0.join("source"),
                size as u64,
                Capabilities::READ | Capabilities::EXTENTS,
                vec![Extent::new(0, size as u64, kind).unwrap()],
            )
            .unwrap(),
        )
    }

    fn destination(&self, size: usize) -> RawDisk<TestExtentBlockDevice> {
        let destination = RawDisk::new(
            TestExtentBlockDevice::create(
                self.0.join("destination"),
                size as u64,
                Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
                vec![],
            )
            .unwrap(),
        );
        destination.write_all_at(0, &vec![0xff; size]).unwrap();
        destination
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_rejected_without_side_effects(
    execution_options: CopyOptions,
    source_size: usize,
    destination_size: usize,
    planned_kind: ExtentKind,
    current_kind: ExtentKind,
    expected_error: &str,
) {
    let fixture = Fixture::new();
    let planner = DataMover::new(CopyOptions::new(BLOCK).unwrap());
    let original_source = fixture.source(SIZE, planned_kind);
    let original_destination = fixture.destination(SIZE);
    let plan = planner
        .plan_raw_with_destination(&original_source, &original_destination)
        .unwrap();
    drop(original_destination);

    let source = fixture.source(source_size, current_kind);
    let destination = fixture.destination(destination_size);
    let executor = DataMover::new(execution_options);
    let before = fs::read(fixture.0.join("destination")).unwrap();

    let ordinary_error = executor
        .execute_raw_plan(&plan, &source, &destination)
        .unwrap_err();
    assert!(ordinary_error.to_string().contains(expected_error));
    assert_eq!(fs::read(fixture.0.join("destination")).unwrap(), before);

    let callbacks = Cell::new(0);
    let observer = |_: &ProgressSnapshot| callbacks.set(callbacks.get() + 1);
    let observed_error = executor
        .execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
        .unwrap_err();

    assert_eq!(observed_error.to_string(), ordinary_error.to_string());
    assert_eq!(fs::read(fixture.0.join("destination")).unwrap(), before);
    assert_eq!(callbacks.get(), 0, "invalid plans must not emit progress");
}

#[test]
fn both_entry_points_reject_smaller_execution_blocks() {
    assert_configuration_rejected(CopyOptions::new(BLOCK / 2).unwrap(), "block size mismatch");
}

#[test]
fn both_entry_points_reject_larger_execution_blocks_without_panicking() {
    assert_configuration_rejected(CopyOptions::new(BLOCK * 2).unwrap(), "block size mismatch");
}

#[test]
fn both_entry_points_reject_changed_alignment() {
    assert_configuration_rejected(
        CopyOptions::with_alignment(BLOCK, 8192).unwrap(),
        "alignment mismatch",
    );
}

fn assert_configuration_rejected(options: CopyOptions, message: &str) {
    assert_rejected_without_side_effects(
        options,
        SIZE,
        SIZE,
        ExtentKind::Data,
        ExtentKind::Data,
        message,
    );
}

#[test]
fn both_entry_points_reject_changed_source_size_before_partial_copy() {
    assert_rejected_without_side_effects(
        CopyOptions::new(BLOCK).unwrap(),
        SIZE / 2,
        SIZE,
        ExtentKind::Data,
        ExtentKind::Data,
        "source size mismatch",
    );
}

#[test]
fn both_entry_points_reject_smaller_destination_before_partial_copy() {
    assert_rejected_without_side_effects(
        CopyOptions::new(BLOCK).unwrap(),
        SIZE,
        SIZE / 2,
        ExtentKind::Data,
        ExtentKind::Data,
        "range outside device bounds",
    );
}

#[test]
fn stale_hole_plan_must_not_clear_new_source_data() {
    assert_rejected_without_side_effects(
        CopyOptions::new(BLOCK).unwrap(),
        SIZE,
        SIZE,
        ExtentKind::Hole,
        ExtentKind::Data,
        "extent map changed",
    );
}

#[test]
fn concurrent_observer_rejects_invalid_plan_before_emitting_progress() {
    assert_configuration_rejected(
        CopyOptions::with_concurrency(BLOCK / 2, 4096, 2).unwrap(),
        "block size mismatch",
    );
}

#[test]
fn native_observer_rejects_invalid_plan_before_emitting_progress() {
    let fixture = Fixture::new();
    let source = fixture.source(SIZE, ExtentKind::Data);
    let destination = fixture.destination(SIZE);
    let strategy = ExecutionStrategy::IoUring(IoUringExecutionOptions::new(2).unwrap());
    let planner = DataMover::with_execution_strategy(CopyOptions::new(BLOCK).unwrap(), strategy);
    let plan = planner
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    let executor =
        DataMover::with_execution_strategy(CopyOptions::new(BLOCK * 2).unwrap(), strategy);
    let callbacks = Cell::new(0);
    let observer = |_: &ProgressSnapshot| callbacks.set(callbacks.get() + 1);

    let ordinary_error = executor
        .execute_raw_plan(&plan, &source, &destination)
        .unwrap_err();
    let observed_error = executor
        .execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
        .unwrap_err();

    assert!(ordinary_error.to_string().contains("block size mismatch"));
    assert_eq!(observed_error.to_string(), ordinary_error.to_string());
    assert_eq!(callbacks.get(), 0);
    assert_eq!(
        fs::read(fixture.0.join("destination")).unwrap(),
        vec![0xff; SIZE]
    );
}
