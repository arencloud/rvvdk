use rvvdk_core::{
    Capabilities, DiskGeometry, Extent, MemoryBlockDevice, RawDisk, Result, VirtualDisk,
};
use rvvdk_datamover::{CancellationToken, CopyEvent, CopyOptions, CopyPhase, DataMover, Verifier};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering},
};
const SIZE: usize = 2 * 1024 * 1024 + 7;
struct Disk {
    inner: RawDisk<MemoryBlockDevice>,
    writes: AtomicU64,
    flushes: AtomicU64,
    mode: u8,
    token: CancellationToken,
}
impl Disk {
    fn new(mode: u8, token: CancellationToken) -> Self {
        Self {
            inner: RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap()),
            writes: AtomicU64::new(0),
            flushes: AtomicU64::new(0),
            mode,
            token,
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
    fn read_at(&self, o: u64, b: &mut [u8]) -> Result<usize> {
        self.inner.read_at(o, b)
    }
    fn write_at(&self, o: u64, b: &[u8]) -> Result<usize> {
        let n = self.inner.write_at(o, b)?;
        self.writes.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
    fn write_zero_at(&self, o: u64, n: u64) -> Result<()> {
        self.inner.write_zero_at(o, n)
    }
    fn discard(&self, o: u64, n: u64) -> Result<()> {
        self.inner.discard(o, n)
    }
    fn flush(&self) -> Result<()> {
        self.flushes.fetch_add(1, Ordering::Relaxed);
        if self.mode == 1 {
            return Err(std::io::Error::other("flush fault").into());
        }
        if self.mode == 2 {
            self.token.cancel();
        }
        Ok(())
    }
    fn extents(&self, o: u64, n: u64) -> Result<Vec<Extent>> {
        self.inner.extents(o, n)
    }
}
fn source() -> RawDisk<MemoryBlockDevice> {
    let d = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    d.write_all_at(0, &vec![0x5a; SIZE]).unwrap();
    d
}
#[test]
fn sequential_and_concurrent_callbacks_are_coordinated_and_complete_after_flush() {
    for workers in [1, 4] {
        let token = CancellationToken::new();
        let source = source();
        let dest = Disk::new(0, token.clone());
        let mover = DataMover::new(CopyOptions::with_concurrency(4096, 4096, workers).unwrap());
        let plan = mover.plan_with_destination(&source, &dest).unwrap();
        let events = RefCell::new(Vec::new());
        let coordinator = std::thread::current().id();
        mover
            .execute_plan_controlled(&plan, &source, &dest, &token, &|e: &CopyEvent| {
                assert_eq!(std::thread::current().id(), coordinator);
                if e.phase == CopyPhase::Completed {
                    assert_eq!(dest.flushes.load(Ordering::Relaxed), 1);
                }
                events.borrow_mut().push(*e);
            })
            .unwrap();
        let events = events.into_inner();
        assert_eq!(events.first().unwrap().phase, CopyPhase::Preparing);
        assert_eq!(events.last().unwrap().phase, CopyPhase::Completed);
        assert!(events.iter().any(|e| e.phase == CopyPhase::Transferring
            && e.progress.bytes_written > 0
            && e.progress.bytes_written < SIZE as u64));
        for pair in events.windows(2) {
            assert!(pair[0].progress.bytes_written <= pair[1].progress.bytes_written);
        }
        let mut bytes = vec![0; SIZE];
        dest.read_exact_at(0, &mut bytes).unwrap();
        assert_eq!(bytes, vec![0x5a; SIZE]);
    }
}
#[test]
fn cancellation_at_start_transfer_and_flush_joins_workers_and_never_completes() {
    for workers in [1, 4] {
        for phase in [
            CopyPhase::Preparing,
            CopyPhase::Started,
            CopyPhase::Transferring,
            CopyPhase::Flushing,
        ] {
            let token = CancellationToken::new();
            let source = source();
            let dest = Disk::new(0, token.clone());
            let mover = DataMover::new(CopyOptions::with_concurrency(4096, 4096, workers).unwrap());
            let plan = mover.plan_with_destination(&source, &dest).unwrap();
            let events = RefCell::new(Vec::new());
            let error = mover
                .execute_plan_controlled(&plan, &source, &dest, &token, &|e: &CopyEvent| {
                    events.borrow_mut().push(e.phase);
                    if e.phase == phase {
                        token.cancel();
                    }
                })
                .unwrap_err();
            assert!(error.is_cancelled(), "{error}");
            assert_eq!(events.borrow().last(), Some(&CopyPhase::Cancelled));
            assert!(!events.borrow().contains(&CopyPhase::Completed));
            assert_eq!(dest.flushes.load(Ordering::Relaxed), 0);
            if phase == CopyPhase::Preparing || phase == CopyPhase::Started {
                assert_eq!(dest.writes.load(Ordering::Relaxed), 0);
            } else {
                assert_eq!(
                    error.copy_failure().unwrap().progress.bytes_written,
                    dest.writes.load(Ordering::Relaxed)
                );
            }
        }
    }
}
#[test]
fn flush_failure_and_cancellation_during_flush_are_terminal_without_success() {
    for workers in [1, 4] {
        for mode in [1, 2] {
            let token = CancellationToken::new();
            let source = source();
            let dest = Disk::new(mode, token.clone());
            let mover = DataMover::new(CopyOptions::with_concurrency(4096, 4096, workers).unwrap());
            let plan = mover.plan_with_destination(&source, &dest).unwrap();
            let events = RefCell::new(Vec::new());
            let error = mover
                .execute_plan_controlled(&plan, &source, &dest, &token, &|e: &CopyEvent| {
                    events.borrow_mut().push(e.phase)
                })
                .unwrap_err();
            assert_eq!(error.is_cancelled(), mode == 2);
            assert_eq!(
                error.copy_failure().unwrap().progress.bytes_written,
                SIZE as u64
            );
            assert_eq!(
                events.borrow().last(),
                Some(&if mode == 2 {
                    CopyPhase::Cancelled
                } else {
                    CopyPhase::Failed
                })
            );
            assert!(!events.borrow().contains(&CopyPhase::Completed));
        }
    }
}
#[test]
fn verification_stops_at_matching_prefix_and_pre_cancelled_input_is_untouched() {
    let source = source();
    let token = CancellationToken::new();
    let mut v = Verifier::new(SIZE as u64, 4096, 8192).unwrap();
    let dest = Disk::new(0, token.clone());
    dest.inner.write_all_at(0, &vec![0x5a; SIZE]).unwrap();
    let progress = RefCell::new(Vec::new());
    let error = v
        .verify_controlled(&source, &dest, &token, |n| {
            progress.borrow_mut().push(n);
            token.cancel();
        })
        .unwrap_err();
    assert!(error.is_cancelled());
    assert_eq!(&*progress.borrow(), &[4096]);
    assert_eq!(dest.writes.load(Ordering::Relaxed), 0);
    assert!(
        v.verify_controlled(&source, &dest, &token, |_| panic!("pre-cancelled callback"))
            .unwrap_err()
            .is_cancelled()
    );
}
#[cfg(target_os = "linux")]
#[test]
fn native_progress_cancellation_drains_or_reports_uncertainty_then_can_reuse_files() {
    use rvvdk_datamover::{ExecutionStrategy, IoUringExecutionOptions};
    use rvvdk_local::LocalFileBlockDevice;
    let root = std::env::var_os("RVVDK_TEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("rvvdk-controlled-native-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let a = root.join("a");
    let b = root.join("b");
    std::fs::write(&a, vec![0x5a; SIZE]).unwrap();
    std::fs::write(&b, vec![0xa5; SIZE]).unwrap();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&a).unwrap());
    let dest = RawDisk::new(LocalFileBlockDevice::open_read_write(&b).unwrap());
    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(4096).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );
    let plan = mover.plan_raw_with_destination(&source, &dest).unwrap();
    for phase in [
        CopyPhase::Started,
        CopyPhase::Transferring,
        CopyPhase::Flushing,
    ] {
        let token = CancellationToken::new();
        let events = RefCell::new(Vec::new());
        let error = mover
            .execute_raw_plan_controlled(&plan, &source, &dest, &token, &|e: &CopyEvent| {
                events.borrow_mut().push(*e);
                if e.phase == phase {
                    token.cancel();
                }
            })
            .unwrap_err();
        assert!(error.is_cancelled(), "{error}");
        assert_eq!(events.borrow().last().unwrap().phase, CopyPhase::Cancelled);
        assert!(
            !events
                .borrow()
                .iter()
                .any(|e| e.phase == CopyPhase::Completed)
        );
        let done = RefCell::new(Vec::new());
        mover
            .execute_raw_plan_controlled(
                &plan,
                &source,
                &dest,
                &CancellationToken::new(),
                &|e: &CopyEvent| done.borrow_mut().push(*e),
            )
            .unwrap();
        assert!(
            done.borrow()
                .iter()
                .any(|e| e.phase == CopyPhase::Transferring
                    && e.progress.bytes_written > 0
                    && e.progress.bytes_written < SIZE as u64)
        );
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    }
    drop(source);
    drop(dest);
    std::fs::remove_dir_all(root).unwrap();
}
