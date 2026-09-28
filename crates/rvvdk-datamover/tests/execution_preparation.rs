#![cfg(target_os = "linux")]

use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Extent, RawDisk, Result};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, ProgressSnapshot,
};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};
use std::cell::Cell;
use std::fs;
use std::os::fd::{AsFd, BorrowedFd};
use std::sync::atomic::{AtomicUsize, Ordering};

struct ChangingAlignment {
    inner: LocalFileBlockDevice,
    alignment: AtomicUsize,
}
impl AsFd for ChangingAlignment {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}
impl LinuxFdBackend for ChangingAlignment {
    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        let alignment = self.alignment.load(Ordering::Relaxed);
        LinuxFdCapabilities::new(false, alignment, alignment)
    }
}
impl BlockDevice for ChangingAlignment {
    fn geometry(&self) -> DiskGeometry {
        self.inner.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.inner.copy_endpoint()
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        self.inner.extents(offset, length)
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        self.inner.write_at(offset, buffer)
    }
    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
}

fn fixture() -> (RawDisk<ChangingAlignment>, RawDisk<ChangingAlignment>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let stem = std::env::temp_dir().join(format!(
        "rvvdk-r14-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    let src = stem.with_extension("source");
    let dst = stem.with_extension("destination");
    fs::write(&src, [0x5a; 16384]).unwrap();
    fs::write(&dst, [0xa5; 16384]).unwrap();
    let source = LocalFileBlockDevice::open_read_only(&src).unwrap();
    let destination = LocalFileBlockDevice::open_read_write(&dst).unwrap();
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    let wrap = |inner| {
        RawDisk::new(ChangingAlignment {
            inner,
            alignment: AtomicUsize::new(1),
        })
    };
    (wrap(source), wrap(destination))
}
fn native(options: CopyOptions) -> DataMover {
    DataMover::with_execution_strategy(
        options,
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(2).unwrap()),
    )
}
fn assert_rejected(
    executor: &DataMover,
    plan: &rvvdk_datamover::CopyPlan,
    source: &RawDisk<ChangingAlignment>,
    destination: &RawDisk<ChangingAlignment>,
    message: &str,
) {
    let plain = executor
        .execute_raw_plan(plan, source, destination)
        .unwrap_err();
    let callbacks = Cell::new(0);
    let observed = executor
        .execute_raw_plan_with_observer(plan, source, destination, &|_: &ProgressSnapshot| {
            callbacks.set(callbacks.get() + 1)
        })
        .unwrap_err();
    assert!(plain.to_string().contains(message), "{plain}");
    assert_eq!(plain.to_string(), observed.to_string());
    assert_eq!(
        callbacks.get(),
        0,
        "preparation rejection must precede observation"
    );
    let mut actual = [0; 16384];
    destination.device().read_exact_at(0, &mut actual).unwrap();
    assert_eq!(actual, [0xa5; 16384]);
}

#[test]
fn native_strategy_rejection_precedes_observation() {
    let (source, destination) = fixture();
    let planner = native(CopyOptions::new(4096).unwrap());
    let plan = planner
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    assert_rejected(
        &DataMover::new(CopyOptions::new(4096).unwrap()),
        &plan,
        &source,
        &destination,
        "not selected",
    );
}

#[test]
fn native_configuration_alignment_rejection_precedes_observation() {
    let (source, destination) = fixture();
    let planner = native(CopyOptions::new(4096).unwrap());
    let plan = planner
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    assert_rejected(
        &native(CopyOptions::with_alignment(4096, 8192).unwrap()),
        &plan,
        &source,
        &destination,
        "alignment mismatch",
    );
}

#[test]
fn native_preparation_refreshes_both_backend_alignments_on_every_execution() {
    let (source, destination) = fixture();
    let mover = native(CopyOptions::new(4096).unwrap());
    let plan = mover
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    for endpoint in [source.device(), destination.device()] {
        endpoint.alignment.store(8192, Ordering::Relaxed);
        assert_rejected(&mover, &plan, &source, &destination, "alignment mismatch");
        endpoint.alignment.store(1, Ordering::Relaxed);
    }
    let callbacks = Cell::new(0);
    let report = mover
        .execute_raw_plan_with_observer(&plan, &source, &destination, &|_: &ProgressSnapshot| {
            callbacks.set(callbacks.get() + 1)
        })
        .unwrap();
    assert_eq!(callbacks.get(), 2);
    assert_eq!(report.stats().bytes_written(), 16384);
    let mut actual = [0; 16384];
    destination.device().read_exact_at(0, &mut actual).unwrap();
    assert_eq!(actual, [0x5a; 16384]);
}
