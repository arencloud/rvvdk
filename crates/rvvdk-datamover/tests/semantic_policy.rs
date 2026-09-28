use std::cell::RefCell;
use std::sync::Mutex;

use rvvdk_core::{Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ProgressSnapshot};

const MIB: u64 = 1024 * 1024;
type Event = (&'static str, u64, u64);

// Validate payloads without allocating the logical disk, including large progress
// boundaries. Non-Data payload reads and dirty fallback buffers fail immediately.
struct Disk {
    extents: Vec<Extent>,
    capabilities: Capabilities,
    events: Mutex<Vec<Event>>,
    failure: Option<Event>,
}
impl Disk {
    fn new(kinds: &[(ExtentKind, u64)], capabilities: Capabilities) -> Self {
        let mut offset = 0;
        let extents = kinds
            .iter()
            .map(|&(kind, length)| {
                let extent = Extent::new(offset, length, kind).unwrap();
                offset += length;
                extent
            })
            .collect();
        Self {
            extents,
            capabilities: capabilities
                | Capabilities::READ
                | Capabilities::WRITE
                | Capabilities::FLUSH,
            events: Mutex::new(Vec::new()),
            failure: None,
        }
    }
    fn record(&self, event: Event) -> Result<()> {
        self.events.lock().unwrap().push(event);
        if self.failure == Some(event) {
            Err(Error::Unsupported)
        } else {
            Ok(())
        }
    }
    fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }
}
impl VirtualDisk for Disk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(self.extents.last().unwrap().end(), 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        Ok(self.extents.clone())
    }
    fn read_at(&self, offset: u64, bytes: &mut [u8]) -> Result<usize> {
        let extent = self
            .extents
            .iter()
            .find(|e| e.offset() <= offset && e.end() >= offset + bytes.len() as u64)
            .unwrap();
        assert_eq!(extent.kind(), ExtentKind::Data, "read sparse payload");
        self.record(("read", offset, bytes.len() as u64))?;
        bytes.fill(0x5a);
        Ok(bytes.len())
    }
    fn write_at(&self, offset: u64, bytes: &[u8]) -> Result<usize> {
        let extent = self
            .extents
            .iter()
            .find(|e| e.offset() <= offset && e.end() >= offset + bytes.len() as u64)
            .unwrap();
        let expected = if extent.kind() == ExtentKind::Data {
            0x5a
        } else {
            0
        };
        assert!(
            bytes.iter().all(|&b| b == expected),
            "dirty fallback or incorrect data"
        );
        self.record(("write", offset, bytes.len() as u64))?;
        Ok(bytes.len())
    }
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        self.record(("zero", offset, length))
    }
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        self.record(("discard", offset, length))
    }
    fn flush(&self) -> Result<()> {
        self.record(("flush", 0, 0))
    }
}

#[test]
fn direct_planned_and_observed_calls_preserve_operation_trace_and_tail_accounting() {
    let layout = [
        (ExtentKind::Data, 4097),
        (ExtentKind::Zero, 8193),
        (ExtentKind::Hole, 8193),
        (ExtentKind::Data, 1),
    ];
    for capabilities in [
        Capabilities::empty(),
        Capabilities::WRITE_ZERO,
        Capabilities::DISCARD,
        Capabilities::WRITE_ZERO | Capabilities::DISCARD,
    ] {
        let mut reference = None;
        for mode in 0..3 {
            let source = Disk::new(&layout, Capabilities::empty());
            let destination = Disk::new(&layout, capabilities);
            let mover = DataMover::new(CopyOptions::new(4096).unwrap());
            let plan = mover.plan_with_destination(&source, &destination).unwrap();
            let events = RefCell::new(Vec::new());
            let stats = match mode {
                0 => mover.copy(&source, &destination).unwrap(),
                1 => *mover
                    .execute_plan(&plan, &source, &destination)
                    .unwrap()
                    .stats(),
                _ => *mover
                    .execute_plan_with_observer(
                        &plan,
                        &source,
                        &destination,
                        &|p: &ProgressSnapshot| events.borrow_mut().push(*p),
                    )
                    .unwrap()
                    .stats(),
            };
            assert_eq!(stats.bytes_read(), 4098);
            assert_eq!(stats.extents_processed(), 4);
            let zeroed = if capabilities.contains(Capabilities::WRITE_ZERO) {
                if capabilities.contains(Capabilities::DISCARD) {
                    8193
                } else {
                    16386
                }
            } else {
                0
            };
            let discarded = if capabilities.contains(Capabilities::DISCARD) {
                8193
            } else {
                0
            };
            assert_eq!(stats.bytes_zeroed(), zeroed);
            assert_eq!(stats.bytes_discarded(), discarded);
            assert_eq!(stats.bytes_written(), 20484 - zeroed - discarded);
            assert_eq!(
                stats.blocks_copied(),
                3 + 3 * ((16386 - zeroed - discarded) / 8193)
            );
            let trace = (source.events(), destination.events());
            assert_eq!(trace.1.last(), Some(&("flush", 0, 0)));
            if let Some(expected) = &reference {
                assert_eq!(&trace, expected);
            } else {
                reference = Some(trace);
            }
            if mode == 2 {
                let events = events.borrow();
                assert_eq!(
                    events
                        .iter()
                        .map(|p| p.logical_bytes_completed())
                        .collect::<Vec<_>>(),
                    [0, 4097, 12290, 20483, 20484]
                );
                assert_eq!(
                    events.last().unwrap().bytes_written(),
                    stats.bytes_written()
                );
                assert_eq!(events.last().unwrap().bytes_zeroed(), zeroed);
                assert_eq!(events.last().unwrap().bytes_discarded(), discarded);
            }
        }
    }
}

#[test]
fn byte_threshold_and_flush_order_are_preserved_for_data_and_sparse_operations() {
    for kind in [ExtentKind::Data, ExtentKind::Zero, ExtentKind::Hole] {
        for capabilities in [
            Capabilities::empty(),
            Capabilities::WRITE_ZERO | Capabilities::DISCARD,
        ] {
            let source = Disk::new(&[(kind, 65 * MIB)], Capabilities::empty());
            let destination = Disk::new(&[(kind, 65 * MIB)], capabilities);
            let mover = DataMover::new(CopyOptions::new(MIB as usize).unwrap());
            let plan = mover.plan_with_destination(&source, &destination).unwrap();
            let snapshots = RefCell::new(Vec::new());
            mover
                .execute_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|p: &ProgressSnapshot| {
                        let flushed = destination.events().last() == Some(&("flush", 0, 0));
                        snapshots.borrow_mut().push((
                            p.logical_bytes_completed(),
                            flushed,
                            p.extents_processed(),
                        ));
                    },
                )
                .unwrap();
            let mut expected = vec![(0, false, 0)];
            if kind == ExtentKind::Data || capabilities.is_empty() {
                expected.push((64 * MIB, false, 0));
            }
            expected.push((65 * MIB, true, 1));
            assert_eq!(*snapshots.borrow(), expected);
        }
    }
}

#[test]
fn threshold_snapshot_is_not_a_durability_event_on_failed_flush() {
    let source = Disk::new(&[(ExtentKind::Data, 64 * MIB)], Capabilities::empty());
    let mut destination = Disk::new(&[(ExtentKind::Data, 64 * MIB)], Capabilities::empty());
    destination.failure = Some(("flush", 0, 0));
    let mover = DataMover::new(CopyOptions::new(MIB as usize).unwrap());
    let plan = mover.plan_with_destination(&source, &destination).unwrap();
    let snapshots = RefCell::new(Vec::new());
    assert!(
        mover
            .execute_plan_with_observer(&plan, &source, &destination, &|p: &ProgressSnapshot| {
                snapshots
                    .borrow_mut()
                    .push((p.logical_bytes_completed(), p.extents_processed()))
            })
            .is_err()
    );
    assert_eq!(*snapshots.borrow(), [(0, 0), (64 * MIB, 0)]);
    // The outer post-flush snapshot (with one completed extent) must not appear.
}

#[test]
fn advertised_sparse_operation_failure_is_not_retried_as_a_write() {
    for (kind, operation) in [(ExtentKind::Zero, "zero"), (ExtentKind::Hole, "discard")] {
        for workers in [1, 4] {
            for mode in 0..3 {
                let source = Disk::new(&[(kind, 4096)], Capabilities::empty());
                let mut destination = Disk::new(
                    &[(kind, 4096)],
                    Capabilities::WRITE_ZERO | Capabilities::DISCARD,
                );
                destination.failure = Some((operation, 0, 4096));
                let mover =
                    DataMover::new(CopyOptions::with_concurrency(4096, 4096, workers).unwrap());
                let plan = mover.plan_with_destination(&source, &destination).unwrap();
                let snapshots = RefCell::new(Vec::new());
                let result = match mode {
                    0 => mover.copy(&source, &destination).map(|_| ()),
                    1 => mover.execute_plan(&plan, &source, &destination).map(|_| ()),
                    _ => mover
                        .execute_plan_with_observer(
                            &plan,
                            &source,
                            &destination,
                            &|p: &ProgressSnapshot| snapshots.borrow_mut().push(*p),
                        )
                        .map(|_| ()),
                };
                assert!(matches!(result, Err(Error::Unsupported)));
                assert_eq!(destination.events(), [(operation, 0, 4096)]);
                assert!(source.events().is_empty());
                assert_eq!(snapshots.borrow().len(), usize::from(mode == 2));
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn native_sparse_failure_keeps_the_same_no_retry_policy() {
    use rvvdk_datamover::{
        IoUringExecutionOptions,
        io_uring::{NativeExtentPlan, copy_extent_plan_with_destination},
    };
    use std::os::fd::AsFd;
    let path = std::env::temp_dir().join(format!("rvvdk-r12-policy-{}", std::process::id()));
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(4096).unwrap();
    drop(file);
    let source = std::fs::File::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    for (kind, operation) in [(ExtentKind::Zero, "zero"), (ExtentKind::Hole, "discard")] {
        let mut destination = Disk::new(
            &[(kind, 4096)],
            Capabilities::WRITE_ZERO | Capabilities::DISCARD,
        );
        destination.failure = Some((operation, 0, 4096));
        let plan = NativeExtentPlan::new(destination.extents.clone(), 4096).unwrap();
        assert!(matches!(
            copy_extent_plan_with_destination(
                source.as_fd(),
                source.as_fd(),
                &destination,
                &plan,
                4096,
                4096,
                IoUringExecutionOptions::new(2).unwrap()
            ),
            Err(Error::Unsupported)
        ));
        assert_eq!(destination.events(), [(operation, 0, 4096)]);
    }
}
