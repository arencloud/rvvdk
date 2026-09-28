#![cfg(target_os = "linux")]

use std::fs::{File, OpenOptions};
use std::os::fd::AsFd;
use std::os::unix::fs::FileExt;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use rvvdk_core::{
    BufferPool, Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk,
};
use rvvdk_datamover::IoUringExecutionOptions;
use rvvdk_datamover::io_uring::{
    IoUringEngine, IoUringFile, NativeExtentPlan, copy_extent_plan,
    copy_extent_plan_with_destination, copy_file_range, copy_file_range_with_options,
};

const SIZE: usize = 8192;
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn file(fill: u8) -> File {
    let path = std::env::temp_dir().join(format!(
        "rvvdk-r04-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::fs::remove_file(path).unwrap();
    file.write_all_at(&[fill; SIZE], 0).unwrap();
    file
}

fn unchanged(file: &File) {
    let mut actual = [0; SIZE];
    file.read_exact_at(&mut actual, 0).unwrap();
    assert_eq!(actual, [0xff; SIZE]);
}

fn options() -> IoUringExecutionOptions {
    IoUringExecutionOptions::new(2).unwrap()
}

fn bounded(name: &str, test: impl FnOnce()) {
    const CHILD: &str = "RVVDK_VALIDATION_TEST_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        test();
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(CHILD, name)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{name}: child failed");
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("{name}: exceeded five-second deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

// Counts every backend interaction, including querying capabilities. Invalid
// public inputs must be rejected before entering destination-specific work.
struct UntouchedDisk {
    calls: AtomicUsize,
    capabilities: Capabilities,
}
impl VirtualDisk for UntouchedDisk {
    fn geometry(&self) -> DiskGeometry {
        self.calls.fetch_add(1, Ordering::Relaxed);
        DiskGeometry::new(SIZE as u64, 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.capabilities
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> Result<usize> {
        panic!("unexpected read")
    }
    fn write_at(&self, _: u64, bytes: &[u8]) -> Result<usize> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(bytes.len())
    }
    fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn discard(&self, _: u64, _: u64) -> Result<()> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        panic!("unexpected flush")
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        panic!("unexpected extent query")
    }
}

#[test]
fn invalid_configuration_precedes_every_extent_path() {
    bounded("invalid_configuration_precedes_every_extent_path", || {
        let fd = file(0xff);
        let empty = NativeExtentPlan::new(vec![], 0).unwrap();
        for (block, alignment) in [
            (0, 4096),
            (4096, 0),
            (4096, 3),
            (usize::MAX, 4096),
            (4096, 1usize << (usize::BITS - 1)),
        ] {
            assert!(
                copy_extent_plan(fd.as_fd(), fd.as_fd(), &empty, block, alignment, options())
                    .is_err()
            );
            for kind in [
                None,
                Some(ExtentKind::Zero),
                Some(ExtentKind::Hole),
                Some(ExtentKind::Data),
            ] {
                let plan = match kind {
                    None => NativeExtentPlan::new(vec![], 0).unwrap(),
                    Some(kind) => NativeExtentPlan::new(
                        vec![Extent::new(0, SIZE as u64, kind).unwrap()],
                        SIZE as u64,
                    )
                    .unwrap(),
                };
                for capabilities in [
                    Capabilities::WRITE,
                    Capabilities::WRITE | Capabilities::WRITE_ZERO,
                    Capabilities::WRITE | Capabilities::DISCARD,
                ] {
                    let disk = UntouchedDisk {
                        calls: AtomicUsize::new(0),
                        capabilities,
                    };
                    assert!(
                        copy_extent_plan_with_destination(
                            fd.as_fd(),
                            fd.as_fd(),
                            &disk,
                            &plan,
                            block,
                            alignment,
                            options()
                        )
                        .is_err(),
                        "block={block} alignment={alignment} kind={kind:?}"
                    );
                    assert_eq!(disk.calls.load(Ordering::Relaxed), 0);
                    unchanged(&fd);
                }
            }
        }
    });
}

#[test]
fn zero_block_fallback_is_bounded() {
    bounded("zero_block_fallback_is_bounded", || {
        let fd = file(0xff);
        for kind in [ExtentKind::Zero, ExtentKind::Hole] {
            let plan = NativeExtentPlan::new(
                vec![Extent::new(0, SIZE as u64, kind).unwrap()],
                SIZE as u64,
            )
            .unwrap();
            let disk = UntouchedDisk {
                calls: AtomicUsize::new(0),
                capabilities: Capabilities::WRITE,
            };
            assert!(matches!(
                copy_extent_plan_with_destination(
                    fd.as_fd(),
                    fd.as_fd(),
                    &disk,
                    &plan,
                    0,
                    4096,
                    options()
                ),
                Err(Error::InvalidAlignment { value: 0, .. })
            ));
            assert_eq!(disk.calls.load(Ordering::Relaxed), 0);
        }
    });
}

#[test]
fn unsupported_later_extent_precedes_data_writes() {
    bounded("unsupported_later_extent_precedes_data_writes", || {
        let source = file(0x5a);
        let destination = file(0xff);
        for (kind, label) in [(ExtentKind::Zero, "zero"), (ExtentKind::Hole, "hole")] {
            let plan = NativeExtentPlan::new(
                vec![
                    Extent::new(0, 4096, ExtentKind::Data).unwrap(),
                    Extent::new(4096, 4096, kind).unwrap(),
                ],
                SIZE as u64,
            )
            .unwrap();
            assert!(
                matches!(copy_extent_plan(source.as_fd(), destination.as_fd(), &plan, 4096, 4096, options()), Err(Error::UnsupportedNativeExtent { kind, offset: 4096, length: 4096 }) if kind == label)
            );
            unchanged(&destination);
        }
    });
}

#[test]
fn invalid_alignment_precedes_zero_then_data() {
    let fd = file(0xff);
    let disk = UntouchedDisk {
        calls: AtomicUsize::new(0),
        capabilities: Capabilities::WRITE_ZERO,
    };
    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, 4096, ExtentKind::Zero).unwrap(),
            Extent::new(4096, 4096, ExtentKind::Data).unwrap(),
        ],
        SIZE as u64,
    )
    .unwrap();
    assert!(matches!(
        copy_extent_plan_with_destination(fd.as_fd(), fd.as_fd(), &disk, &plan, 4096, 3, options()),
        Err(Error::InvalidBufferAlignment { alignment: 3 })
    ));
    assert_eq!(disk.calls.load(Ordering::Relaxed), 0);
    unchanged(&fd);
}

#[test]
fn range_validation_includes_empty_copies() {
    let fd = file(0xff);
    assert!(matches!(
        copy_file_range(fd.as_fd(), fd.as_fd(), 0, 0, 4096, 0, 4096),
        Err(Error::InvalidIoUringQueueDepth)
    ));
    for length in [0, 1] {
        for (block, alignment) in [(0, 4096), (4096, 0), (4096, 3), (usize::MAX, 4096)] {
            assert!(
                copy_file_range_with_options(
                    fd.as_fd(),
                    fd.as_fd(),
                    0,
                    length,
                    block,
                    alignment,
                    options()
                )
                .is_err()
            );
        }
    }
    for (offset, length) in [
        (u64::MAX, 0),
        (u64::MAX, 1),
        (u64::MAX - 1, 4),
        (i64::MAX as u64, 1),
    ] {
        assert!(matches!(
            copy_file_range_with_options(
                fd.as_fd(),
                fd.as_fd(),
                offset,
                length,
                4096,
                4096,
                options()
            ),
            Err(Error::RangeOverflow { .. })
        ));
    }
    assert_eq!(
        copy_file_range_with_options(
            fd.as_fd(),
            fd.as_fd(),
            i64::MAX as u64,
            0,
            4096,
            4096,
            options()
        )
        .unwrap()
        .bytes_written(),
        0
    );
    unchanged(&fd);
}

#[test]
fn empty_plans_are_valid_no_ops() {
    let fd = file(0xff);
    let disk = UntouchedDisk {
        calls: AtomicUsize::new(0),
        capabilities: Capabilities::WRITE,
    };
    let plan = NativeExtentPlan::new(vec![], 0).unwrap();
    assert_eq!(
        copy_extent_plan(fd.as_fd(), fd.as_fd(), &plan, 4096, 4096, options())
            .unwrap()
            .extents_processed(),
        0
    );
    assert_eq!(
        copy_extent_plan_with_destination(
            fd.as_fd(),
            fd.as_fd(),
            &disk,
            &plan,
            4096,
            4096,
            options()
        )
        .unwrap()
        .extents_processed(),
        0
    );
    assert_eq!(disk.calls.load(Ordering::Relaxed), 0);
}

#[test]
fn unrepresentable_plan_range_precedes_destination_calls() {
    let fd = file(0xff);
    let disk = UntouchedDisk {
        calls: AtomicUsize::new(0),
        capabilities: Capabilities::WRITE_ZERO,
    };
    let plan = NativeExtentPlan::new(
        vec![
            Extent::new(0, 4096, ExtentKind::Zero).unwrap(),
            Extent::new(4096, i64::MAX as u64, ExtentKind::Data).unwrap(),
        ],
        i64::MAX as u64 + 4096,
    )
    .unwrap();
    assert!(matches!(
        copy_extent_plan_with_destination(
            fd.as_fd(),
            fd.as_fd(),
            &disk,
            &plan,
            4096,
            4096,
            options()
        ),
        Err(Error::RangeOverflow { .. })
    ));
    assert_eq!(disk.calls.load(Ordering::Relaxed), 0);
}

#[test]
fn engine_rejects_invalid_offsets_without_publishing_or_poisoning() {
    bounded(
        "engine_rejects_invalid_offsets_without_publishing_or_poisoning",
        || {
            let fd = file(0xff);
            let endpoint = IoUringFile::new(fd.as_fd()).unwrap();
            let pool = BufferPool::new(1, 4096, 4096).unwrap();
            let mut engine = IoUringEngine::new(1).unwrap();
            for offset in [u64::MAX, i64::MAX as u64, u64::MAX - 4095] {
                assert!(matches!(
                    engine.submit_owned_read(&endpoint, offset, 4096, pool.acquire()),
                    Err(Error::RangeOverflow { .. })
                ));
                assert!(matches!(
                    engine.submit_owned_write(&endpoint, offset, 4096, pool.acquire()),
                    Err(Error::RangeOverflow { .. })
                ));
                assert_eq!(engine.in_flight(), 0);
                assert_eq!(pool.available(), 1);
            }
            engine
                .submit_owned_read(&endpoint, 0, 4096, pool.acquire())
                .unwrap();
            engine.submit().unwrap();
            let completed = engine.wait_owned_completion().unwrap();
            drop(completed);
            engine.shutdown().unwrap();
            assert_eq!(pool.available(), 1);
            unchanged(&fd);
        },
    );
}
