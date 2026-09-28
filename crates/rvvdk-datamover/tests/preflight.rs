#![cfg(target_os = "linux")]

use rvvdk_core::{Capabilities, Extent, ExtentKind, MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::io_uring::{
    NativeExtentPlan, copy_extent_plan_with_destination, copy_file_range,
};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, ProgressObserver,
    ProgressSnapshot,
};
use rvvdk_local::LocalFileBlockDevice;
use std::fs::{self, File, OpenOptions};
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SIZE: usize = 8192;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    source: PathBuf,
    destination: PathBuf,
    link: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let stem = std::env::temp_dir().join(format!(
            "rvvdk-r05-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let this = Self {
            source: stem.with_extension("source"),
            destination: stem.with_extension("destination"),
            link: stem.with_extension("link"),
        };
        fs::write(&this.source, [0x5a; SIZE]).unwrap();
        fs::write(&this.destination, [0xff; SIZE]).unwrap();
        this
    }
    fn destination(&self) -> RawDisk<LocalFileBlockDevice> {
        RawDisk::new(LocalFileBlockDevice::open_read_write(&self.destination).unwrap())
    }
    fn unchanged(&self) {
        assert_eq!(fs::read(&self.destination).unwrap(), [0xff; SIZE]);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for p in [&self.source, &self.destination, &self.link] {
            let _ = fs::remove_file(p);
        }
    }
}
fn options() -> IoUringExecutionOptions {
    IoUringExecutionOptions::new(2).unwrap()
}
fn strategies() -> [ExecutionStrategy; 3] {
    [
        ExecutionStrategy::Threaded,
        ExecutionStrategy::IoUring(options()),
        ExecutionStrategy::Auto(options()),
    ]
}
fn mover(strategy: ExecutionStrategy) -> DataMover {
    DataMover::with_execution_strategy(CopyOptions::new(4096).unwrap(), strategy)
}
fn mixed() -> NativeExtentPlan {
    NativeExtentPlan::new(
        vec![
            Extent::new(0, 4096, ExtentKind::Zero).unwrap(),
            Extent::new(4096, 4096, ExtentKind::Data).unwrap(),
        ],
        SIZE as u64,
    )
    .unwrap()
}
#[derive(Default)]
struct Observer(AtomicUsize);
impl ProgressObserver for Observer {
    fn on_progress(&self, _: &ProgressSnapshot) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn missing_flush_rejected_before_portable_writes() {
    for workers in [1, 4] {
        let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
        source.write_all_at(0, &[0x5a; SIZE]).unwrap();
        let destination = RawDisk::new(
            MemoryBlockDevice::with_capabilities(SIZE, Capabilities::READ | Capabilities::WRITE)
                .unwrap(),
        );
        destination.write_all_at(0, &[0xff; SIZE]).unwrap();
        let mover = DataMover::new(CopyOptions::with_concurrency(4096, 4096, workers).unwrap());
        let error = mover.copy(&source, &destination).unwrap_err();
        assert!(
            error.to_string().contains("destination preflight"),
            "{error}"
        );
        assert!(error.to_string().contains("flush"));
        let mut actual = [0; SIZE];
        destination.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(actual, [0xff; SIZE]);
    }
}

#[test]
fn portable_access_requirements_are_explicit() {
    let unreadable =
        RawDisk::new(MemoryBlockDevice::with_capabilities(SIZE, Capabilities::WRITE).unwrap());
    let readable = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let readonly = RawDisk::new(MemoryBlockDevice::read_only(SIZE).unwrap());
    for (role, error) in [
        (
            "source",
            mover(ExecutionStrategy::Threaded)
                .copy(&unreadable, &readable)
                .unwrap_err(),
        ),
        (
            "destination",
            mover(ExecutionStrategy::Threaded)
                .copy(&readable, &readonly)
                .unwrap_err(),
        ),
    ] {
        assert!(
            error.to_string().contains(&format!("{role} preflight")),
            "{error}"
        );
    }
}

#[test]
fn aliases_rejected_for_memory_and_hard_linked_files() {
    let disk = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    assert!(
        mover(ExecutionStrategy::Threaded)
            .copy(&disk, &disk)
            .unwrap_err()
            .to_string()
            .contains("same backing object")
    );
    let f = Fixture::new();
    fs::hard_link(&f.source, &f.link).unwrap();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&f.source).unwrap());
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&f.link).unwrap());
    assert!(
        mover(ExecutionStrategy::Threaded)
            .copy(&source, &destination)
            .is_err()
    );
    for strategy in strategies() {
        assert!(
            mover(strategy)
                .copy_raw_with_report(&source, &destination)
                .unwrap_err()
                .to_string()
                .contains("same backing object")
        );
    }
    assert_eq!(fs::read(&f.source).unwrap(), [0x5a; SIZE]);
}

#[test]
fn native_alias_rejected_before_zeroing_source() {
    let f = Fixture::new();
    fs::hard_link(&f.source, &f.link).unwrap();
    let source = File::open(&f.source).unwrap();
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&f.link).unwrap());
    for kind in [ExtentKind::Zero, ExtentKind::Hole] {
        let plan = NativeExtentPlan::new(
            vec![Extent::new(0, SIZE as u64, kind).unwrap()],
            SIZE as u64,
        )
        .unwrap();
        assert!(
            copy_extent_plan_with_destination(
                source.as_fd(),
                destination.device().as_fd(),
                &destination,
                &plan,
                4096,
                4096,
                options()
            )
            .unwrap_err()
            .to_string()
            .contains("same backing object")
        );
        assert_eq!(fs::read(&f.source).unwrap(), [0x5a; SIZE]);
    }
    assert!(
        copy_file_range(
            source.as_fd(),
            destination.device().as_fd(),
            0,
            SIZE as u64,
            4096,
            2,
            4096
        )
        .is_err()
    );
}

#[test]
fn source_write_only_rejected_before_zero_extent() {
    let f = Fixture::new();
    let source = OpenOptions::new().write(true).open(&f.source).unwrap();
    let destination = f.destination();
    let error = copy_extent_plan_with_destination(
        source.as_fd(),
        destination.device().as_fd(),
        &destination,
        &mixed(),
        4096,
        4096,
        options(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("source preflight"), "{error}");
    f.unchanged();
}

#[test]
fn native_readonly_descriptor_rejected_before_zero_extent() {
    let f = Fixture::new();
    let source = File::open(&f.source).unwrap();
    let destination = f.destination();
    let readonly = File::open(&f.destination).unwrap();
    let error = copy_extent_plan_with_destination(
        source.as_fd(),
        readonly.as_fd(),
        &destination,
        &mixed(),
        4096,
        4096,
        options(),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("destination preflight"),
        "{error}"
    );
    f.unchanged();
}

#[test]
fn append_descriptor_cannot_redirect_explicit_writes() {
    let f = Fixture::new();
    let source = File::open(&f.source).unwrap();
    let destination = OpenOptions::new()
        .append(true)
        .open(&f.destination)
        .unwrap();
    let error = copy_file_range(
        source.as_fd(),
        destination.as_fd(),
        0,
        SIZE as u64,
        4096,
        2,
        4096,
    )
    .unwrap_err();
    assert!(error.to_string().contains("append mode"), "{error}");
    f.unchanged();
}

#[test]
fn undersized_native_destination_is_not_extended() {
    let f = Fixture::new();
    let source = File::open(&f.source).unwrap();
    let destination = OpenOptions::new().write(true).open(&f.destination).unwrap();
    destination.set_len(4096).unwrap();
    let error = copy_file_range(
        source.as_fd(),
        destination.as_fd(),
        0,
        SIZE as u64,
        4096,
        2,
        4096,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("destination preflight"),
        "{error}"
    );
    assert_eq!(fs::read(&f.destination).unwrap(), [0xff; 4096]);
}

#[test]
fn changed_file_sizes_rejected_before_observer_notification() {
    for strategy in strategies() {
        for change in ["source_shrink", "source_grow", "destination_shrink"] {
            let f = Fixture::new();
            let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&f.source).unwrap());
            let destination = f.destination();
            let mover = mover(strategy);
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let path = if change.starts_with("source") {
                &f.source
            } else {
                &f.destination
            };
            OpenOptions::new()
                .write(true)
                .open(path)
                .unwrap()
                .set_len(if change == "source_grow" {
                    2 * SIZE as u64
                } else {
                    4096
                })
                .unwrap();
            let before = fs::read(&f.destination).unwrap();
            let observer = Observer::default();
            assert!(
                mover
                    .execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
                    .is_err(),
                "{strategy:?}: {change}"
            );
            assert_eq!(observer.0.load(Ordering::Relaxed), 0);
            assert_eq!(fs::read(&f.destination).unwrap(), before);
        }
    }
}

#[test]
fn execution_rechecks_aliasing_instead_of_trusting_planning() {
    let f = Fixture::new();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&f.source).unwrap());
    let destination = f.destination();
    for strategy in strategies() {
        let mover = mover(strategy);
        let plan = mover
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        let alias = RawDisk::new(LocalFileBlockDevice::open_read_write(&f.source).unwrap());
        let observer = Observer::default();
        assert!(
            mover
                .execute_raw_plan_with_observer(&plan, &source, &alias, &observer)
                .unwrap_err()
                .to_string()
                .contains("same backing object")
        );
        assert_eq!(observer.0.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn native_backend_descriptor_mismatch_precedes_zero_writes() {
    let f = Fixture::new();
    let source = File::open(&f.source).unwrap();
    let destination = f.destination();
    fs::write(&f.link, [0xaa; SIZE]).unwrap();
    let other = OpenOptions::new().write(true).open(&f.link).unwrap();
    let error = copy_extent_plan_with_destination(
        source.as_fd(),
        other.as_fd(),
        &destination,
        &mixed(),
        4096,
        4096,
        options(),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("different backing objects"),
        "{error}"
    );
    f.unchanged();
    assert_eq!(fs::read(&f.link).unwrap(), [0xaa; SIZE]);
}

#[test]
fn valid_native_range_preserves_prefix_and_tail() {
    let f = Fixture::new();
    let source = File::open(&f.source).unwrap();
    let destination = OpenOptions::new().write(true).open(&f.destination).unwrap();
    copy_file_range(
        source.as_fd(),
        destination.as_fd(),
        1024,
        4096,
        4096,
        2,
        4096,
    )
    .unwrap();
    let mut expected = [0xff; SIZE];
    expected[1024..5120].fill(0x5a);
    assert_eq!(fs::read(&f.destination).unwrap(), expected);
}

#[test]
fn readonly_replacement_rejected_before_observer_notification() {
    let f = Fixture::new();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&f.source).unwrap());
    let destination = f.destination();
    let readonly = RawDisk::new(LocalFileBlockDevice::open_read_only(&f.destination).unwrap());
    for strategy in strategies() {
        let mover = mover(strategy);
        let plan = mover
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        let observer = Observer::default();
        let error = mover
            .execute_raw_plan_with_observer(&plan, &source, &readonly, &observer)
            .unwrap_err();
        assert!(
            error.to_string().contains("destination preflight"),
            "{error}"
        );
        assert_eq!(observer.0.load(Ordering::Relaxed), 0);
        f.unchanged();
    }
}

#[test]
fn nonregular_native_endpoint_is_rejected() {
    let f = Fixture::new();
    let source = File::open("/dev/zero").unwrap();
    let destination = OpenOptions::new().write(true).open(&f.destination).unwrap();
    let error = copy_file_range(
        source.as_fd(),
        destination.as_fd(),
        0,
        SIZE as u64,
        4096,
        2,
        4096,
    )
    .unwrap_err();
    assert!(error.to_string().contains("source preflight"));
    assert!(error.to_string().contains("regular file"));
    f.unchanged();
}
