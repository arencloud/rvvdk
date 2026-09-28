use std::cell::Cell;
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

use rvvdk_core::{
    BlockDevice, Capabilities, DiskGeometry, Error, Extent, ExtentKind, MemoryBlockDevice, Result,
    VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DEFAULT_MEMORY_BUDGET, DataMover, ProgressSnapshot};

struct Disk {
    inner: MemoryBlockDevice,
    capacity: AtomicUsize,
    queries: AtomicUsize,
    io: AtomicUsize,
}
impl Disk {
    fn new(capacity: usize) -> Self {
        Self {
            inner: MemoryBlockDevice::new(4096).unwrap(),
            capacity: AtomicUsize::new(capacity),
            queries: AtomicUsize::new(0),
            io: AtomicUsize::new(0),
        }
    }
}
impl VirtualDisk for Disk {
    fn geometry(&self) -> DiskGeometry {
        self.inner.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.io.fetch_add(1, Ordering::Relaxed);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        self.io.fetch_add(1, Ordering::Relaxed);
        self.inner.write_at(offset, buffer)
    }
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        self.io.fetch_add(1, Ordering::Relaxed);
        self.inner.write_zero_at(offset, length)
    }
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        self.io.fetch_add(1, Ordering::Relaxed);
        self.inner.discard(offset, length)
    }
    fn flush(&self) -> Result<()> {
        self.io.fetch_add(1, Ordering::Relaxed);
        self.inner.flush()
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        self.queries.fetch_add(1, Ordering::Relaxed);
        let mut extents = Vec::with_capacity(self.capacity.load(Ordering::Relaxed));
        extents.push(Extent::new(0, 4096, ExtentKind::Data)?);
        Ok(extents)
    }
}
fn assert_budget(error: Error, phase: &str, expected: usize, limit: usize) {
    assert!(error.copy_failure().is_none());
    assert!(
        matches!(error, Error::MemoryBudgetExceeded { phase: actual, required, budget }
        if actual == phase && required == expected && budget == limit)
    );
}

#[test]
fn planning_charges_unused_extent_capacity_and_accepts_exact_limit() {
    assert_eq!(
        CopyOptions::default().memory_budget(),
        DEFAULT_MEMORY_BUDGET
    );
    let source = Disk::new(128);
    let required = 128 * size_of::<Extent>();
    let options = CopyOptions::new(512).unwrap();
    assert_budget(
        DataMover::new(options.with_memory_budget(required - 1))
            .plan(&source)
            .unwrap_err(),
        "planning",
        required,
        required - 1,
    );
    let plan = DataMover::new(options.with_memory_budget(required))
        .plan(&source)
        .unwrap();
    assert_eq!(plan.extent_count(), 1);
    assert_eq!(plan.extent_capacity(), 128);
    assert_eq!(source.io.load(Ordering::Relaxed), 0);
}

#[test]
fn execution_boundary_precedes_observation_and_io_for_all_portable_paths() {
    for workers in [1, 4] {
        let source = Disk::new(1);
        let destination = Disk::new(1);
        source.inner.write_all_at(0, &[0x5a; 4096]).unwrap();
        let options = CopyOptions::with_execution(512, 512, workers, 3).unwrap();
        let planner = DataMover::new(options);
        let plan = planner.plan(&source).unwrap();
        let usage = planner.execution_memory(&plan).unwrap();
        assert_eq!(usage.extent_bytes(), size_of::<Extent>());
        assert_eq!(usage.queue_bytes() == 0, workers == 1);
        let required = usage.total_bytes();
        let limited = DataMover::new(options.with_memory_budget(required - 1));
        let callbacks = Cell::new(0);
        let queries = source.queries.load(Ordering::Relaxed);
        assert_budget(
            limited
                .execute_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|_: &ProgressSnapshot| callbacks.set(callbacks.get() + 1),
                )
                .unwrap_err(),
            "execution",
            required,
            required - 1,
        );
        assert_eq!(
            source.queries.load(Ordering::Relaxed),
            queries,
            "known excess rejected before another extent query"
        );
        assert_budget(
            limited
                .execute_plan(&plan, &source, &destination)
                .unwrap_err(),
            "execution",
            required,
            required - 1,
        );
        assert_budget(
            limited.copy(&source, &destination).unwrap_err(),
            "execution",
            required,
            required - 1,
        );
        assert_budget(
            limited.copy_with_report(&source, &destination).unwrap_err(),
            "execution",
            required,
            required - 1,
        );
        assert_eq!(callbacks.get(), 0);
        assert_eq!(source.io.load(Ordering::Relaxed), 0);
        assert_eq!(destination.io.load(Ordering::Relaxed), 0);
        let exact = DataMover::new(options.with_memory_budget(required));
        exact.execute_plan(&plan, &source, &destination).unwrap();
        exact.copy(&source, &destination).unwrap();
        let mut bytes = [0; 4096];
        destination.inner.read_exact_at(0, &mut bytes).unwrap();
        assert_eq!(bytes, [0x5a; 4096]);
    }
}

#[test]
fn live_revalidation_charges_both_vecs_and_releases_them_before_execution() {
    let source = Disk::new(128);
    let destination = Disk::new(1);
    let options = CopyOptions::new(512).unwrap();
    let plan = DataMover::new(options).plan(&source).unwrap();
    // Revalidation, not the buffer pool, determines the peak for this plan.
    let limit = 256 * size_of::<Extent>();
    let mover = DataMover::new(options.with_memory_budget(limit));
    assert!(mover.execution_memory(&plan).unwrap().total_bytes() < limit);
    mover.execute_plan(&plan, &source, &destination).unwrap();
    source.capacity.store(129, Ordering::Relaxed);
    let before = destination.io.load(Ordering::Relaxed);
    assert_budget(
        mover
            .execute_plan_with_observer(&plan, &source, &destination, &|_: &ProgressSnapshot| {
                panic!("budget rejection notified observer")
            })
            .unwrap_err(),
        "revalidation",
        257 * size_of::<Extent>(),
        limit,
    );
    assert_eq!(destination.io.load(Ordering::Relaxed), before);
}

#[test]
fn oversized_configuration_rejects_without_allocation_or_io() {
    let source = Disk::new(1);
    let destination = Disk::new(1);
    for options in [
        CopyOptions::new(usize::MAX).unwrap(),
        CopyOptions::with_buffer_pool(4096, 4096, usize::MAX).unwrap(),
        CopyOptions::with_execution(4096, 4096, 2, usize::MAX).unwrap(),
    ] {
        let mover = DataMover::new(options.with_memory_budget(usize::MAX));
        assert!(matches!(
            mover.copy(&source, &destination),
            Err(Error::MemoryAccountingOverflow)
        ));
    }
    assert_eq!(source.io.load(Ordering::Relaxed), 0);
    assert_eq!(destination.io.load(Ordering::Relaxed), 0);
}
