use rvvdk_core::{
    Capabilities, CopyOperation, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover, ProgressSnapshot};
use std::sync::Mutex;

const BLOCK: usize = 4096;
struct Disk {
    bytes: Mutex<Vec<u8>>,
    kind: ExtentKind,
    fail: Option<CopyOperation>,
}
impl Disk {
    fn new(kind: ExtentKind, fail: Option<CopyOperation>, fill: u8) -> Self {
        Self {
            bytes: Mutex::new(vec![fill; 2 * BLOCK]),
            kind,
            fail,
        }
    }
    fn denied() -> Error {
        Error::Io(std::io::Error::from_raw_os_error(13))
    }
}
impl VirtualDisk for Disk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new((2 * BLOCK) as u64, 512, BLOCK as u32).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        let base =
            Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | Capabilities::EXTENTS;
        if self.fail == Some(CopyOperation::Write) {
            base
        } else {
            base | Capabilities::WRITE_ZERO | Capabilities::DISCARD
        }
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        Ok(vec![
            Extent::new(0, BLOCK as u64, self.kind)?,
            Extent::new(BLOCK as u64, BLOCK as u64, self.kind)?,
        ])
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        if self.fail == Some(CopyOperation::Read) && offset >= BLOCK as u64 {
            return Err(Self::denied());
        }
        buffer.copy_from_slice(
            &self.bytes.lock().unwrap()[offset as usize..offset as usize + buffer.len()],
        );
        Ok(buffer.len())
    }
    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if self.fail == Some(CopyOperation::Write) && offset > BLOCK as u64 {
            return Err(Self::denied());
        }
        let n = if self.fail == Some(CopyOperation::Write) && offset == BLOCK as u64 {
            256.min(buffer.len())
        } else {
            buffer.len()
        };
        self.bytes.lock().unwrap()[offset as usize..offset as usize + n]
            .copy_from_slice(&buffer[..n]);
        Ok(n)
    }
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        if self.fail == Some(CopyOperation::WriteZero) && offset >= BLOCK as u64 {
            return Err(Self::denied());
        }
        self.bytes.lock().unwrap()[offset as usize..(offset + length) as usize].fill(0);
        Ok(())
    }
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        if self.fail == Some(CopyOperation::Discard) && offset >= BLOCK as u64 {
            return Err(Self::denied());
        }
        self.bytes.lock().unwrap()[offset as usize..(offset + length) as usize].fill(0);
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        if self.fail == Some(CopyOperation::Flush) {
            Err(Self::denied())
        } else {
            Ok(())
        }
    }
}

#[test]
fn sequential_failures_report_attempted_operation_and_confirmed_lower_bounds() {
    for operation in [
        CopyOperation::Read,
        CopyOperation::Write,
        CopyOperation::WriteZero,
        CopyOperation::Discard,
    ] {
        for observed in [false, true] {
            let kind = match operation {
                CopyOperation::WriteZero => ExtentKind::Zero,
                CopyOperation::Discard => ExtentKind::Hole,
                _ => ExtentKind::Data,
            };
            let source = Disk::new(
                kind,
                (operation == CopyOperation::Read).then_some(operation),
                0x5a,
            );
            let destination = Disk::new(
                ExtentKind::Data,
                (operation != CopyOperation::Read).then_some(operation),
                0xa5,
            );
            let mover = DataMover::new(CopyOptions::new(BLOCK).unwrap());
            let plan = mover.plan_with_destination(&source, &destination).unwrap();
            let snapshots = Mutex::new(Vec::new());
            let error = if observed {
                mover.execute_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|p: &ProgressSnapshot| snapshots.lock().unwrap().push(*p),
                )
            } else {
                mover.execute_plan(&plan, &source, &destination)
            }
            .unwrap_err();
            let failure = error.copy_failure().unwrap();
            assert_eq!(failure.backend, "threaded");
            assert_eq!(failure.operation, operation);
            assert_eq!(failure.range.unwrap().offset(), BLOCK as u64);
            assert_eq!(failure.range.unwrap().length(), BLOCK as u64);
            assert_eq!(failure.progress.extents_completed, Some(1));
            assert!(failure.progress.unconfirmed_io);
            match &failure.cause {
                Error::Io(io) => assert_eq!(io.raw_os_error(), Some(13)),
                e => panic!("{e}"),
            }
            let p = failure.progress;
            match operation {
                CopyOperation::Read => assert_eq!(
                    (p.bytes_read, p.bytes_written, p.blocks_completed),
                    (4096, 4096, 1)
                ),
                CopyOperation::Write => {
                    assert_eq!(
                        (p.bytes_read, p.bytes_written, p.blocks_completed),
                        (8192, 4096, 1)
                    );
                    // The failed write changed bytes beyond the confirmed lower bound.
                    assert_eq!(&destination.bytes.lock().unwrap()[4096..4352], &[0x5a; 256]);
                }
                CopyOperation::WriteZero => {
                    assert_eq!((p.bytes_zeroed, p.bytes_written), (4096, 0))
                }
                CopyOperation::Discard => {
                    assert_eq!((p.bytes_discarded, p.bytes_written), (4096, 0))
                }
                _ => unreachable!(),
            }
            // Initial + first completed extent, with no final-success snapshot.
            assert_eq!(
                snapshots.lock().unwrap().len(),
                if observed { 2 } else { 0 }
            );
            let mut cause: &dyn std::error::Error = &error;
            while let Some(next) = cause.source() {
                cause = next;
            }
            assert_eq!(
                cause
                    .downcast_ref::<std::io::Error>()
                    .unwrap()
                    .raw_os_error(),
                Some(13)
            );
        }
    }
}

#[test]
fn flush_failure_keeps_full_completed_counters_without_claiming_durability() {
    for workers in [1, 4] {
        let source = Disk::new(ExtentKind::Data, None, 0x5a);
        let destination = Disk::new(ExtentKind::Data, Some(CopyOperation::Flush), 0xa5);
        let mover = DataMover::new(CopyOptions::with_concurrency(BLOCK, BLOCK, workers).unwrap());
        let error = mover.copy(&source, &destination).unwrap_err();
        let failure = error.copy_failure().unwrap();
        assert_eq!(failure.operation, CopyOperation::Flush);
        assert_eq!(failure.range, None);
        assert_eq!(
            (failure.progress.bytes_read, failure.progress.bytes_written),
            (8192, 8192)
        );
        assert_eq!(failure.progress.extents_completed, Some(2));
        assert!(!failure.progress.unconfirmed_io);
        assert_eq!(*destination.bytes.lock().unwrap(), vec![0x5a; 8192]);
    }
}

#[test]
fn preparation_rejections_are_typed_and_do_not_claim_execution_progress() {
    let source = Disk::new(ExtentKind::Data, None, 0x5a);
    let destination = Disk::new(ExtentKind::Data, None, 0xa5);
    let plan = DataMover::new(CopyOptions::new(BLOCK).unwrap())
        .plan(&source)
        .unwrap();
    let error = DataMover::new(CopyOptions::new(BLOCK / 2).unwrap())
        .execute_plan(&plan, &source, &destination)
        .unwrap_err();
    assert!(matches!(error, Error::InvalidCopyConfiguration(_)));
    assert!(error.copy_failure().is_none());
    let changed = Disk::new(ExtentKind::Zero, None, 0x5a);
    let error = DataMover::new(CopyOptions::new(BLOCK).unwrap())
        .execute_plan(&plan, &changed, &destination)
        .unwrap_err();
    assert!(matches!(error, Error::StaleCopyPlan(_)));
    assert!(error.copy_failure().is_none());
}

#[cfg(target_os = "linux")]
#[test]
fn native_sparse_failures_keep_prior_extents_and_fallback_write_context() {
    use rvvdk_datamover::IoUringExecutionOptions;
    use rvvdk_datamover::io_uring::{NativeExtentPlan, copy_extent_plan_with_destination};
    use std::os::fd::AsFd;
    let path = std::env::temp_dir().join(format!("rvvdk-r15-sparse-{}", std::process::id()));
    std::fs::write(&path, [0x5a; 8192]).unwrap();
    let source = std::fs::File::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    for (kind, operation) in [
        (ExtentKind::Zero, CopyOperation::WriteZero),
        (ExtentKind::Hole, CopyOperation::Discard),
        (ExtentKind::Zero, CopyOperation::Write),
        (ExtentKind::Hole, CopyOperation::Write),
    ] {
        let destination = Disk::new(kind, Some(operation), 0xa5);
        let plan = NativeExtentPlan::new(destination.extents(0, 8192).unwrap(), 8192).unwrap();
        let error = copy_extent_plan_with_destination(
            source.as_fd(),
            source.as_fd(),
            &destination,
            &plan,
            BLOCK,
            BLOCK,
            IoUringExecutionOptions::new(2).unwrap(),
        )
        .unwrap_err();
        let f = error.copy_failure().unwrap();
        assert_eq!(f.backend, "io_uring");
        assert_eq!(f.operation, operation);
        assert_eq!(f.progress.extents_completed, Some(1));
        assert_eq!(f.range.unwrap().offset(), 4096);
        assert!(f.progress.unconfirmed_io);
        match operation {
            CopyOperation::WriteZero => assert_eq!(f.progress.bytes_zeroed, 4096),
            CopyOperation::Discard => assert_eq!(f.progress.bytes_discarded, 4096),
            CopyOperation::Write => {
                assert_eq!(f.progress.bytes_written, 4096);
                assert_eq!(f.progress.bytes_read, 0);
                assert_eq!(f.progress.blocks_completed, 1);
            }
            _ => unreachable!(),
        }
    }
}
