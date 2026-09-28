use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use rvvdk_core::{Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ProgressSnapshot};

const SIZE: usize = 8212;
const HOLE: usize = 8193;
const GUARD: usize = 19;

struct Disk {
    bytes: Mutex<Vec<u8>>,
    extents: Vec<Extent>,
    capabilities: Capabilities,
    source: bool,
    fail_discard: bool,
    reads: AtomicUsize,
    writes: AtomicUsize,
    zeroes: AtomicUsize,
    discards: AtomicUsize,
    flushes: AtomicUsize,
}
impl Disk {
    fn new(extra: Capabilities, source: bool) -> Self {
        Self {
            bytes: Mutex::new(vec![if source { 0x5a } else { 0xa5 }; SIZE + GUARD]),
            extents: vec![
                Extent::new(0, 7, ExtentKind::Data).unwrap(),
                Extent::new(7, 3, ExtentKind::Zero).unwrap(),
                Extent::new(10, HOLE as u64, ExtentKind::Hole).unwrap(),
                Extent::new(8203, 9, ExtentKind::Data).unwrap(),
            ],
            capabilities: Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | extra,
            source,
            fail_discard: false,
            reads: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
            zeroes: AtomicUsize::new(0),
            discards: AtomicUsize::new(0),
            flushes: AtomicUsize::new(0),
        }
    }
    fn fill(&self, offset: u64, length: u64, value: u8) {
        self.bytes.lock().unwrap()[offset as usize..(offset + length) as usize].fill(value);
    }
    fn check_bytes(&self, mixed: bool) {
        let actual = self.bytes.lock().unwrap();
        let mut expected = vec![0; SIZE];
        if mixed {
            expected[..7].fill(0x5a);
            expected[8203..].fill(0x5a);
        }
        assert_eq!(&actual[..SIZE], expected);
        assert_eq!(
            &actual[SIZE..],
            &[0xa5; GUARD],
            "writes escaped logical size"
        );
    }
}
impl VirtualDisk for Disk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(SIZE as u64, 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        Ok(self.extents.clone())
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.reads.fetch_add(buffer.len(), Ordering::Relaxed);
        buffer.copy_from_slice(
            &self.bytes.lock().unwrap()[offset as usize..offset as usize + buffer.len()],
        );
        if self.source {
            // Resolve logical zeroes even though physical backing is nonzero.
            for extent in &self.extents {
                if extent.kind() != ExtentKind::Data {
                    let start = offset.max(extent.offset());
                    let end = (offset + buffer.len() as u64).min(extent.end());
                    if start < end {
                        buffer[(start - offset) as usize..(end - offset) as usize].fill(0);
                    }
                }
            }
        }
        Ok(buffer.len())
    }
    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        self.writes.fetch_add(1, Ordering::Relaxed);
        self.bytes.lock().unwrap()[offset as usize..offset as usize + buffer.len()]
            .copy_from_slice(buffer);
        Ok(buffer.len())
    }
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        assert!(self.capabilities.contains(Capabilities::WRITE_ZERO));
        self.zeroes.fetch_add(1, Ordering::Relaxed);
        self.fill(offset, length, 0);
        Ok(())
    }
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        assert!(self.capabilities.contains(Capabilities::DISCARD));
        self.discards.fetch_add(1, Ordering::Relaxed);
        if self.fail_discard {
            self.fill(offset, 1, 0);
            return Err(Error::Unsupported);
        }
        // Ordinary discard deliberately returns success with nonzero contents.
        let value = if self.capabilities.contains(Capabilities::DISCARD_ZEROES) {
            0
        } else {
            0xdd
        };
        self.fill(offset, length, value);
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        self.flushes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
fn variants() -> impl Iterator<Item = Capabilities> {
    (0..8).map(|bits| {
        let mut caps = Capabilities::empty();
        if bits & 1 != 0 {
            caps |= Capabilities::DISCARD;
        }
        if bits & 2 != 0 {
            caps |= Capabilities::DISCARD_ZEROES;
        }
        if bits & 4 != 0 {
            caps |= Capabilities::WRITE_ZERO;
        }
        caps
    })
}

#[test]
fn every_capability_combination_preserves_mixed_logical_bytes_across_portable_paths() {
    for caps in variants() {
        for workers in [1, 4] {
            for mode in 0..4 {
                let source = Disk::new(Capabilities::empty(), true);
                let destination = Disk::new(caps, false);
                let mover =
                    DataMover::new(CopyOptions::with_execution(4096, 4096, workers, 1).unwrap());
                let plan = mover.plan(&source).unwrap();
                let callbacks = AtomicUsize::new(0);
                let stats = match mode {
                    0 => mover.copy(&source, &destination).unwrap(),
                    1 => *mover
                        .copy_with_report(&source, &destination)
                        .unwrap()
                        .stats(),
                    2 => *mover
                        .execute_plan(&plan, &source, &destination)
                        .unwrap()
                        .stats(),
                    _ => *mover
                        .execute_plan_with_observer(
                            &plan,
                            &source,
                            &destination,
                            &|_: &ProgressSnapshot| {
                                callbacks.fetch_add(1, Ordering::Relaxed);
                            },
                        )
                        .unwrap()
                        .stats(),
                };
                let discarded =
                    if caps.contains(Capabilities::DISCARD | Capabilities::DISCARD_ZEROES) {
                        HOLE
                    } else {
                        0
                    };
                let zeroed = if caps.contains(Capabilities::WRITE_ZERO) {
                    HOLE + 3 - discarded
                } else {
                    0
                };
                assert_eq!(stats.bytes_read(), 16);
                assert_eq!(
                    source.reads.load(Ordering::Relaxed),
                    16,
                    "read logical sparse bytes"
                );
                assert_eq!(stats.bytes_discarded(), discarded as u64);
                assert_eq!(stats.bytes_zeroed(), zeroed as u64);
                assert_eq!(stats.bytes_written(), (SIZE - zeroed - discarded) as u64);
                assert_eq!(stats.extents_processed(), 4);
                assert_eq!(
                    destination.discards.load(Ordering::Relaxed) == 0,
                    discarded == 0
                );
                assert_eq!(destination.flushes.load(Ordering::Relaxed), 1);
                assert_eq!(callbacks.load(Ordering::Relaxed) > 0, mode == 3);
                destination.check_bytes(true);
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn native_sparse_dispatch_requires_the_same_zero_read_guarantee() {
    use rvvdk_datamover::{
        IoUringExecutionOptions,
        io_uring::{NativeExtentPlan, copy_extent_plan_with_destination},
    };
    use std::os::fd::AsFd;
    let path = std::env::temp_dir().join(format!("rvvdk-r21-hole-{}", std::process::id()));
    std::fs::write(&path, vec![0x5a; SIZE]).unwrap();
    let source = std::fs::File::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, 10, ExtentKind::Zero).unwrap(),
            Extent::new(10, HOLE as u64, ExtentKind::Hole).unwrap(),
            Extent::new(8203, 9, ExtentKind::Zero).unwrap(),
        ],
        SIZE as u64,
    )
    .unwrap();
    for caps in variants() {
        let destination = Disk::new(caps, false);
        // Sparse-only dispatch never accesses the destination FD or submits Data.
        let stats = copy_extent_plan_with_destination(
            source.as_fd(),
            source.as_fd(),
            &destination,
            &plan,
            4096,
            4096,
            IoUringExecutionOptions::new(2).unwrap(),
        )
        .unwrap();
        let discarded = if caps.contains(Capabilities::DISCARD | Capabilities::DISCARD_ZEROES) {
            HOLE
        } else {
            0
        };
        let zeroed = if caps.contains(Capabilities::WRITE_ZERO) {
            SIZE - discarded
        } else {
            0
        };
        assert_eq!(stats.bytes_read(), 0);
        assert_eq!(stats.bytes_discarded(), discarded as u64);
        assert_eq!(stats.bytes_zeroed(), zeroed as u64);
        assert_eq!(stats.bytes_written(), (SIZE - zeroed - discarded) as u64);
        assert_eq!(stats.extents_processed(), 3);
        assert_eq!(
            destination.discards.load(Ordering::Relaxed) == 0,
            discarded == 0
        );
        assert_eq!(
            destination.flushes.load(Ordering::Relaxed),
            0,
            "low-level API leaves flush to caller"
        );
        destination.check_bytes(false);
    }
}

#[test]
fn partial_guaranteed_discard_failure_is_not_retried_or_reported_complete() {
    for workers in [1, 4] {
        let mut source = Disk::new(Capabilities::empty(), true);
        source.extents = vec![Extent::new(0, SIZE as u64, ExtentKind::Hole).unwrap()];
        let mut destination = Disk::new(
            Capabilities::DISCARD | Capabilities::DISCARD_ZEROES | Capabilities::WRITE_ZERO,
            false,
        );
        destination.fail_discard = true;
        // One work item makes failure progress deterministic even with four workers.
        let mover = DataMover::new(CopyOptions::with_concurrency(SIZE, 4096, workers).unwrap());
        let plan = mover.plan(&source).unwrap();
        let callbacks = AtomicUsize::new(0);
        let error = mover
            .execute_plan_with_observer(&plan, &source, &destination, &|_: &ProgressSnapshot| {
                callbacks.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap_err();
        let failure = error.copy_failure().unwrap();
        assert_eq!(failure.operation, rvvdk_core::CopyOperation::Discard);
        assert!(matches!(failure.cause, Error::Unsupported));
        assert!(failure.progress.unconfirmed_io);
        assert_eq!(failure.progress.bytes_discarded, 0);
        assert_eq!(destination.discards.load(Ordering::Relaxed), 1);
        assert_eq!(destination.zeroes.load(Ordering::Relaxed), 0);
        assert_eq!(destination.writes.load(Ordering::Relaxed), 0);
        assert_eq!(destination.flushes.load(Ordering::Relaxed), 0);
        assert_eq!(callbacks.load(Ordering::Relaxed), 1);
        let bytes = destination.bytes.lock().unwrap();
        assert_eq!(bytes[0], 0);
        assert!(bytes[1..].iter().all(|&b| b == 0xa5));
    }
}
