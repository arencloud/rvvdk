use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Error, Extent, ExtentKind,
    MemoryBlockDevice, RawDisk, Result, VirtualDisk,
};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ProgressObserver, ProgressSnapshot,
};

const BLOCK: usize = 4096;
const SIZE: usize = 4 * BLOCK;

struct TranslatedDisk {
    backing: Arc<MemoryBlockDevice>,
    mixed: AtomicBool,
    invalid: AtomicBool,
    capabilities: Capabilities,
    flushes: AtomicUsize,
    fail_flush: bool,
}
impl TranslatedDisk {
    fn new(mixed: bool, extra: Capabilities) -> Self {
        let backing = Arc::new(MemoryBlockDevice::new(SIZE + 2 * BLOCK).unwrap());
        backing
            .write_all_at(0, &vec![0xa5; SIZE + 2 * BLOCK])
            .unwrap();
        backing.write_all_at(BLOCK as u64, &[0x31; BLOCK]).unwrap();
        backing
            .write_all_at((4 * BLOCK) as u64, &[0x42; BLOCK])
            .unwrap();
        Self {
            backing,
            mixed: AtomicBool::new(mixed),
            invalid: AtomicBool::new(false),
            capabilities: Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | extra,
            flushes: AtomicUsize::new(0),
            fail_flush: false,
        }
    }
    fn expected() -> Vec<u8> {
        let mut bytes = vec![0; SIZE];
        bytes[..BLOCK].fill(0x31);
        bytes[3 * BLOCK..].fill(0x42);
        bytes
    }
    fn check_output(&self) {
        let mut actual = vec![0; SIZE + 2 * BLOCK];
        self.backing.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(&actual[..BLOCK], &[0xa5; BLOCK]);
        assert_eq!(&actual[BLOCK..BLOCK + SIZE], Self::expected());
        assert_eq!(&actual[BLOCK + SIZE..], &[0xa5; BLOCK]);
    }
}
impl VirtualDisk for TranslatedDisk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(SIZE as u64, 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        let mut endpoint = self.backing.copy_endpoint()?;
        endpoint.size = SIZE as u64;
        endpoint.capabilities = self.capabilities;
        Ok(endpoint)
    }
    fn read_at(&self, offset: u64, bytes: &mut [u8]) -> Result<usize> {
        if self.mixed.load(Ordering::Relaxed) {
            assert!(
                offset + bytes.len() as u64 <= BLOCK as u64 || offset >= 3 * BLOCK as u64,
                "Zero/Hole payload must not be read"
            );
        }
        self.backing.read_at(offset + BLOCK as u64, bytes)
    }
    fn write_at(&self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.backing.write_at(offset + BLOCK as u64, bytes)
    }
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        self.backing.write_zero_at(offset + BLOCK as u64, length)
    }
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        self.backing.discard(offset + BLOCK as u64, length)
    }
    fn flush(&self) -> Result<()> {
        self.flushes.fetch_add(1, Ordering::Relaxed);
        if self.fail_flush {
            return Err(Error::Io(std::io::Error::other("injected flush failure")));
        }
        self.backing.flush()
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        if self.invalid.load(Ordering::Relaxed) {
            return Ok(vec![Extent::new(1, SIZE as u64 - 1, ExtentKind::Data)?]);
        }
        if !self.mixed.load(Ordering::Relaxed) {
            return Ok(vec![Extent::new(0, SIZE as u64, ExtentKind::Data)?]);
        }
        [
            ExtentKind::Data,
            ExtentKind::Zero,
            ExtentKind::Hole,
            ExtentKind::Data,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, kind)| Extent::new((i * BLOCK) as u64, BLOCK as u64, kind))
        .collect()
    }
}
// Even a translated object that also exposes Linux traits must stay on logical
// methods through the portable API. Any accidental FD probing panics immediately.
#[cfg(target_os = "linux")]
impl std::os::fd::AsFd for TranslatedDisk {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        panic!("portable API accessed a physical FD")
    }
}
#[cfg(target_os = "linux")]
impl rvvdk_platform::LinuxFdBackend for TranslatedDisk {
    fn linux_fd_capabilities(&self) -> rvvdk_platform::LinuxFdCapabilities {
        panic!("portable API queried native capabilities")
    }
}
fn mover(workers: usize) -> DataMover {
    DataMover::new(CopyOptions::with_concurrency(1024, 4096, workers).unwrap())
}

#[test]
fn trait_object_plans_preserve_translated_data_zero_and_hole_semantics() {
    for workers in [1, 4] {
        for capabilities in [
            Capabilities::empty(),
            Capabilities::WRITE_ZERO,
            Capabilities::DISCARD,
            Capabilities::WRITE_ZERO | Capabilities::DISCARD,
        ] {
            for observed in [false, true] {
                let source = TranslatedDisk::new(true, Capabilities::empty());
                let destination = TranslatedDisk::new(false, capabilities);
                let events = RefCell::new(Vec::new());
                let observer = |p: &ProgressSnapshot| {
                    events.borrow_mut().push(*p);
                };
                let observer: &dyn ProgressObserver = &observer;
                let s: &dyn VirtualDisk = &source;
                let d: &dyn VirtualDisk = &destination;
                let mover = mover(workers);
                let plan = mover.plan_with_destination(s, d).unwrap();
                assert_eq!(plan.backend(), ExecutionBackend::Threaded);
                assert_eq!(
                    (plan.data_bytes(), plan.zero_bytes(), plan.hole_bytes()),
                    (2 * BLOCK as u64, BLOCK as u64, BLOCK as u64)
                );
                let report = if observed {
                    mover.execute_plan_with_observer(&plan, s, d, observer)
                } else {
                    mover.execute_plan(&plan, s, d)
                }
                .unwrap();
                assert_eq!(report.stats().bytes_read(), 2 * BLOCK as u64);
                assert_eq!(report.stats().extents_processed(), 4);
                let zeroed = if capabilities.contains(Capabilities::WRITE_ZERO) {
                    BLOCK as u64
                        * if capabilities.contains(Capabilities::DISCARD) {
                            1
                        } else {
                            2
                        }
                } else {
                    0
                };
                let discarded = if capabilities.contains(Capabilities::DISCARD) {
                    BLOCK as u64
                } else {
                    0
                };
                assert_eq!(report.stats().bytes_zeroed(), zeroed);
                assert_eq!(report.stats().bytes_discarded(), discarded);
                assert_eq!(
                    report.stats().bytes_written(),
                    SIZE as u64 - zeroed - discarded
                );
                assert_eq!(destination.flushes.load(Ordering::Relaxed), 1);
                destination.check_output();
                if observed {
                    let events = events.borrow();
                    assert_eq!(events.first().unwrap().logical_bytes_completed(), 0);
                    assert_eq!(
                        events.last().unwrap().logical_bytes_completed(),
                        SIZE as u64
                    );
                    assert!(
                        events
                            .windows(2)
                            .all(|p| p[0].logical_bytes_completed()
                                <= p[1].logical_bytes_completed())
                    );
                    assert_eq!(events.len(), if workers == 1 { 5 } else { 2 });
                }
            }
        }
    }
}

#[test]
fn memory_disks_support_the_same_plan_and_copy_report_api() {
    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    source.write_all_at(0, &[0x7b; SIZE]).unwrap();
    for workers in [1, 4] {
        let mover = mover(workers);
        let plan = mover.plan(&source).unwrap();
        mover.execute_plan(&plan, &source, &destination).unwrap();
        mover
            .copy_with_report(
                &source as &dyn VirtualDisk,
                &destination as &dyn VirtualDisk,
            )
            .unwrap();
        let mut actual = [0; SIZE];
        destination.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(actual, [0x7b; SIZE]);
    }
}

#[test]
fn portable_validation_rejects_stale_and_invalid_maps_before_observation() {
    for workers in [1, 4] {
        for invalid in [false, true] {
            let source = TranslatedDisk::new(true, Capabilities::empty());
            let destination = TranslatedDisk::new(false, Capabilities::empty());
            let mover = mover(workers);
            let plan = mover.plan_with_destination(&source, &destination).unwrap();
            if invalid {
                source.invalid.store(true, Ordering::Relaxed);
            } else {
                source.mixed.store(false, Ordering::Relaxed);
            }
            let calls = RefCell::new(0);
            assert!(
                mover
                    .execute_plan_with_observer(
                        &plan,
                        &source as &dyn VirtualDisk,
                        &destination,
                        &|_: &ProgressSnapshot| *calls.borrow_mut() += 1
                    )
                    .is_err()
            );
            assert_eq!(*calls.borrow(), 0);
            assert_eq!(destination.flushes.load(Ordering::Relaxed), 0);
            let mut bytes = [0; BLOCK];
            destination
                .backing
                .read_exact_at((2 * BLOCK) as u64, &mut bytes)
                .unwrap();
            assert_eq!(bytes, [0xa5; BLOCK]);
        }
    }
}

#[test]
fn portable_plan_checks_block_alignment_capacity_and_aliases() {
    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let plan = mover(1)
        .plan_with_destination(&source, &destination)
        .unwrap();
    for options in [
        CopyOptions::new(2048).unwrap(),
        CopyOptions::with_alignment(1024, 512).unwrap(),
    ] {
        let calls = RefCell::new(0);
        assert!(
            DataMover::new(options)
                .execute_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|_: &ProgressSnapshot| *calls.borrow_mut() += 1
                )
                .is_err()
        );
        assert_eq!(*calls.borrow(), 0);
    }
    let small = RawDisk::new(MemoryBlockDevice::new(BLOCK).unwrap());
    assert!(
        mover(1)
            .plan_with_destination(&source as &dyn VirtualDisk, &small)
            .is_err()
    );
    assert!(mover(1).execute_plan(&plan, &source, &small).is_err());
    assert!(
        mover(1)
            .plan_with_destination(&source as &dyn VirtualDisk, &source as &dyn VirtualDisk)
            .is_err()
    );
}

#[test]
fn portable_observation_does_not_finish_after_flush_failure() {
    for workers in [1, 4] {
        let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
        let mut destination = TranslatedDisk::new(false, Capabilities::empty());
        destination.fail_flush = true;
        let mover = mover(workers);
        let plan = mover.plan_with_destination(&source, &destination).unwrap();
        let events = RefCell::new(Vec::new());
        let error = mover
            .execute_plan_with_observer(&plan, &source, &destination, &|p: &ProgressSnapshot| {
                events.borrow_mut().push(*p)
            })
            .unwrap_err();
        assert!(error.to_string().contains("injected flush failure"));
        assert_eq!(events.borrow().len(), 1);
        assert_eq!(destination.flushes.load(Ordering::Relaxed), 1);
    }
}

#[test]
fn empty_trait_object_plan_is_flushed_and_observed() {
    let source = RawDisk::new(MemoryBlockDevice::new(0).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(0).unwrap());
    for workers in [1, 4] {
        let mover = mover(workers);
        let plan = mover
            .plan_with_destination(
                &source as &dyn VirtualDisk,
                &destination as &dyn VirtualDisk,
            )
            .unwrap();
        let events = RefCell::new(Vec::new());
        let report = mover
            .execute_plan_with_observer(
                &plan,
                &source as &dyn VirtualDisk,
                &destination as &dyn VirtualDisk,
                &|p: &ProgressSnapshot| events.borrow_mut().push(*p),
            )
            .unwrap();
        assert_eq!(report.stats().bytes_written(), 0);
        assert_eq!(events.borrow().len(), 2);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn auto_stays_logical_and_explicit_native_is_rejected() {
    use rvvdk_datamover::{ExecutionStrategy, IoUringExecutionOptions};
    let source = TranslatedDisk::new(true, Capabilities::empty());
    let destination = TranslatedDisk::new(false, Capabilities::empty());
    let options = IoUringExecutionOptions::new(2).unwrap();
    let auto = DataMover::with_execution_strategy(
        CopyOptions::new(1024).unwrap(),
        ExecutionStrategy::Auto(options),
    );
    let report = auto.copy_with_report(&source, &destination).unwrap();
    assert_eq!(report.backend(), ExecutionBackend::Threaded);
    destination.check_output();
    let native = DataMover::with_execution_strategy(
        CopyOptions::new(1024).unwrap(),
        ExecutionStrategy::IoUring(options),
    );
    assert!(matches!(
        native.plan(&source),
        Err(Error::NativeExecutionUnsupported)
    ));
    assert!(matches!(
        native.plan_with_destination(&source, &destination),
        Err(Error::NativeExecutionUnsupported)
    ));
    assert!(matches!(
        native.copy(&source, &destination),
        Err(Error::NativeExecutionUnsupported)
    ));
    let plan = auto.plan_with_destination(&source, &destination).unwrap();
    let calls = RefCell::new(0);
    assert!(matches!(
        native.execute_plan_with_observer(&plan, &source, &destination, &|_: &ProgressSnapshot| {
            *calls.borrow_mut() += 1
        }),
        Err(Error::NativeExecutionUnsupported)
    ));
    assert_eq!(*calls.borrow(), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn local_raw_disks_share_the_portable_api_and_native_plans_stay_explicit() {
    use rvvdk_datamover::{ExecutionStrategy, IoUringExecutionOptions};
    use rvvdk_local::LocalFileBlockDevice;
    let source_path = std::env::temp_dir().join(format!("rvvdk-r11-source-{}", std::process::id()));
    let destination_path =
        std::env::temp_dir().join(format!("rvvdk-r11-destination-{}", std::process::id()));
    std::fs::write(&source_path, [0x5a; SIZE]).unwrap();
    std::fs::write(&destination_path, [0xff; SIZE]).unwrap();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());
    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());
    std::fs::remove_file(source_path).unwrap();
    std::fs::remove_file(destination_path).unwrap();
    for workers in [1, 4] {
        let mover = mover(workers);
        let plan = mover
            .plan_with_destination(
                &source as &dyn VirtualDisk,
                &destination as &dyn VirtualDisk,
            )
            .unwrap();
        let report = mover
            .execute_plan(
                &plan,
                &source as &dyn VirtualDisk,
                &destination as &dyn VirtualDisk,
            )
            .unwrap();
        assert_eq!(report.backend(), ExecutionBackend::Threaded);
        let mut actual = [0; SIZE];
        destination.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(actual, [0x5a; SIZE]);
    }
    let native = DataMover::with_execution_strategy(
        CopyOptions::new(1024).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(2).unwrap()),
    );
    let native_plan = native
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    let translated = TranslatedDisk::new(true, Capabilities::empty());
    let calls = RefCell::new(0);
    assert!(matches!(
        mover(1).execute_plan_with_observer(
            &native_plan,
            &translated,
            &destination,
            &|_: &ProgressSnapshot| *calls.borrow_mut() += 1
        ),
        Err(Error::NativeExecutionUnsupported)
    ));
    assert_eq!(*calls.borrow(), 0);
    let report = native
        .execute_raw_plan(&native_plan, &source, &destination)
        .unwrap();
    assert_eq!(report.backend(), ExecutionBackend::IoUring);
}

#[test]
fn portable_preflight_checks_flush_support_before_any_callback() {
    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let destination = RawDisk::new(
        MemoryBlockDevice::with_capabilities(SIZE, Capabilities::READ | Capabilities::WRITE)
            .unwrap(),
    );
    destination.write_all_at(0, &[0xff; SIZE]).unwrap();
    let mover = mover(4);
    let plan = mover.plan(&source).unwrap();
    let calls = RefCell::new(0);
    assert!(mover.plan_with_destination(&source, &destination).is_err());
    assert!(
        mover
            .execute_plan_with_observer(&plan, &source, &destination, &|_: &ProgressSnapshot| {
                *calls.borrow_mut() += 1
            })
            .is_err()
    );
    assert_eq!(*calls.borrow(), 0);
    let mut actual = [0; SIZE];
    destination.read_exact_at(0, &mut actual).unwrap();
    assert_eq!(actual, [0xff; SIZE]);
}
