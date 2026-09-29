#![cfg(target_os = "linux")]
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Error, Extent, ExtentKind,
    NativeRequestIssue, RawDisk, Result, VirtualDisk,
};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionSelectionReason, ExecutionStrategy,
    IoUringExecutionOptions,
};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};
use std::{
    fs,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Source {
    file: LocalFileBlockDevice,
    extents: Vec<Extent>,
    offset_alignment: AtomicUsize,
}
impl AsFd for Source {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}
impl LinuxFdBackend for Source {
    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        let caps = self.file.linux_fd_capabilities();
        let alignment = self.offset_alignment.load(Ordering::Relaxed);
        LinuxFdCapabilities::new(
            caps.direct_io(),
            caps.memory_alignment(),
            if alignment == 0 {
                caps.offset_alignment()
            } else {
                alignment
            },
        )
    }
}
impl BlockDevice for Source {
    fn geometry(&self) -> DiskGeometry {
        self.file.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.file.capabilities()
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.file.copy_endpoint()
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.file.read_at(offset, buffer)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        Err(Error::Unsupported)
    }
    fn flush(&self) -> Result<()> {
        Err(Error::Unsupported)
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        assert_eq!((offset, length), (0, self.file.size()));
        Ok(self.extents.clone())
    }
}
fn fixture(
    size: usize,
    source_direct: bool,
    destination_direct: bool,
    layout: &[(u64, u64, ExtentKind)],
) -> (RawDisk<Source>, RawDisk<LocalFileBlockDevice>, Vec<u8>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::var_os("RVVDK_TEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "rvvdk-r24-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    let src = path.with_extension("src");
    let dst = path.with_extension("dst");
    let mut expected = vec![0; size];
    for &(offset, length, kind) in layout {
        if kind == ExtentKind::Data {
            expected[offset as usize..(offset + length) as usize].fill(0x5a);
        }
    }
    fs::write(&src, &expected).unwrap();
    fs::write(&dst, vec![0xa5; size]).unwrap();
    let file = if source_direct {
        LocalFileBlockDevice::open_direct_read_only(&src)
    } else {
        LocalFileBlockDevice::open_read_only(&src)
    }
    .unwrap();
    let destination = if destination_direct {
        LocalFileBlockDevice::open_direct_read_write(&dst)
    } else {
        LocalFileBlockDevice::open_read_write(&dst)
    }
    .unwrap();
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    (
        RawDisk::new(Source {
            file,
            extents: layout
                .iter()
                .map(|&(o, l, k)| Extent::new(o, l, k).unwrap())
                .collect(),
            offset_alignment: AtomicUsize::new(0),
        }),
        RawDisk::new(destination),
        expected,
    )
}
fn mover(auto: bool, block: usize, workers: usize) -> DataMover {
    let options = CopyOptions::with_execution(block, 4096, workers, 2).unwrap();
    let native = IoUringExecutionOptions::new(4).unwrap();
    DataMover::with_execution_strategy(
        options,
        if auto {
            ExecutionStrategy::Auto(native)
        } else {
            ExecutionStrategy::IoUring(native)
        },
    )
}
fn bytes(disk: &RawDisk<LocalFileBlockDevice>) -> Vec<u8> {
    let mut data = vec![0; disk.size() as usize];
    disk.read_exact_at(0, &mut data).unwrap();
    data
}

#[test]
fn odd_and_aligned_copies_select_once_and_preserve_bytes_across_endpoint_modes() {
    for size in [1, 4097, 12288] {
        for (sd, dd) in [(false, false), (true, false), (false, true), (true, true)] {
            for workers in [1, 4] {
                for observed in [false, true] {
                    let (source, destination, expected) =
                        fixture(size, sd, dd, &[(0, size as u64, ExtentKind::Data)]);
                    let mover = mover(true, 4096, workers);
                    let plan = mover
                        .plan_raw_with_destination(&source, &destination)
                        .unwrap();
                    let fallback = size != 12288 && (sd || dd);
                    assert_eq!(
                        plan.backend(),
                        if fallback {
                            ExecutionBackend::Threaded
                        } else {
                            ExecutionBackend::IoUring
                        }
                    );
                    if fallback {
                        assert!(matches!(
                            plan.execution_selection().reason(),
                            ExecutionSelectionReason::RawRequestsIncompatible(
                                NativeRequestIssue::DirectRange { .. }
                            )
                        ));
                    }
                    let callbacks = AtomicUsize::new(0);
                    let observer = |p: &rvvdk_datamover::ProgressSnapshot| {
                        assert_eq!(p.backend(), plan.backend());
                        callbacks.fetch_add(1, Ordering::Relaxed);
                    };
                    let report = if observed {
                        mover.execute_raw_plan_with_observer(
                            &plan,
                            &source,
                            &destination,
                            &observer,
                        )
                    } else {
                        mover.execute_raw_plan(&plan, &source, &destination)
                    }
                    .unwrap();
                    assert_eq!(report.backend(), plan.backend());
                    assert_eq!(report.stats().bytes_read(), size as u64);
                    assert_eq!(report.stats().bytes_written(), size as u64);
                    assert_eq!(bytes(&destination), expected);
                    assert_eq!(callbacks.load(Ordering::Relaxed) > 0, observed);
                }
            }
        }
    }
}

#[test]
fn unaligned_data_after_sparse_prefix_falls_back_or_rejects_before_mutation() {
    let layout = [
        (0, 1, ExtentKind::Zero),
        (1, 4096, ExtentKind::Data),
        (4097, 4095, ExtentKind::Hole),
    ];
    for (sd, dd) in [(true, false), (false, true), (true, true)] {
        let (source, destination, expected) = fixture(8192, sd, dd, &layout);
        let explicit = mover(false, 4096, 1);
        assert!(matches!(
            explicit.plan_raw_with_destination(&source, &destination),
            Err(Error::NativeRequestIncompatible(issue)) if matches!(*issue, NativeRequestIssue::DirectRange { offset: 1, .. })
        ));
        assert_eq!(bytes(&destination), vec![0xa5; 8192]);
        let auto = mover(true, 4096, 1);
        let plan = auto
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        assert_eq!(plan.backend(), ExecutionBackend::Threaded);
        let report = auto.execute_raw_plan(&plan, &source, &destination).unwrap();
        assert_eq!(report.stats().bytes_read(), 4096);
        assert_eq!(report.stats().bytes_zeroed(), 1);
        assert_eq!(report.stats().bytes_discarded(), 4095);
        assert_eq!(bytes(&destination), expected);
    }
}

#[test]
fn preparation_rechecks_requests_before_callbacks_and_sparse_prefix_then_allows_replanning() {
    for auto in [false, true] {
        let (source, destination, expected) = fixture(
            12288,
            true,
            false,
            &[(0, 4096, ExtentKind::Zero), (4096, 8192, ExtentKind::Data)],
        );
        let mover = mover(auto, 4096, 1);
        let plan = mover
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        source
            .device()
            .offset_alignment
            .store(8192, Ordering::Relaxed);
        let calls = AtomicUsize::new(0);
        let observer = |_: &rvvdk_datamover::ProgressSnapshot| {
            calls.fetch_add(1, Ordering::Relaxed);
        };
        for observed in [false, true] {
            let result = if observed {
                mover.execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
            } else {
                mover.execute_raw_plan(&plan, &source, &destination)
            };
            assert!(matches!(
                result,
                Err(Error::NativeRequestIncompatible(issue)) if matches!(*issue, NativeRequestIssue::DirectRange { offset: 4096, .. })
            ));
            assert_eq!(bytes(&destination), vec![0xa5; 12288]);
            assert_eq!(calls.load(Ordering::Relaxed), 0);
        }
        if auto {
            let replanned = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            assert_eq!(replanned.backend(), ExecutionBackend::Threaded);
            mover
                .execute_raw_plan(&replanned, &source, &destination)
                .unwrap();
            assert_eq!(bytes(&destination), expected);
        }
    }
}

#[test]
fn changed_direct_flag_is_rejected_before_observation() {
    let (source, destination, _) = fixture(8192, false, false, &[(0, 8192, ExtentKind::Data)]);
    let mover = mover(true, 4096, 1);
    let plan = mover
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    let fd = source.device().as_fd().as_raw_fd();
    // SAFETY: fd is live; F_GETFL/F_SETFL have no pointer arguments.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    assert!(flags >= 0);
    // SAFETY: changes only this owned source descriptor's flags during the test.
    assert_eq!(
        unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_DIRECT) },
        0
    );
    let result = mover.execute_raw_plan_with_observer(
        &plan,
        &source,
        &destination,
        &|_: &rvvdk_datamover::ProgressSnapshot| panic!("preflight must reject before callback"),
    );
    // SAFETY: restore the original flags on the still-live descriptor.
    assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFL, flags) }, 0);
    assert!(
        matches!(result,Err(Error::EndpointPreflight { endpoint:"source",source }) if matches!(*source,Error::EndpointChanged(_)))
    );
    assert_eq!(bytes(&destination), vec![0xa5; 8192]);
}

#[test]
fn fallback_uses_threaded_budget_and_fd_only_native_api_never_silently_falls_back() {
    let (source, destination, expected) = fixture(4097, true, true, &[(0, 4097, ExtentKind::Data)]);
    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(4096).unwrap().with_memory_budget(5000),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap()),
    );
    let plan = mover
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    assert_eq!(plan.backend(), ExecutionBackend::Threaded);
    mover
        .execute_raw_plan(&plan, &source, &destination)
        .unwrap();
    assert_eq!(bytes(&destination), expected);
    assert!(matches!(
        mover.copy_native_with_report(source.device(), destination.device(), 1, 4096),
        Err(Error::NativeRequestIncompatible(issue)) if matches!(*issue, NativeRequestIssue::DirectRange { offset: 1, .. })
    ));
}

#[test]
fn unaligned_split_block_falls_back_but_unused_block_boundary_is_allowed() {
    for (size, fallback) in [(4096, false), (8192, true)] {
        let (source, destination, expected) =
            fixture(size, true, true, &[(0, size as u64, ExtentKind::Data)]);
        let mover = mover(true, 4097, 1);
        let plan = mover
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        assert_eq!(
            plan.backend(),
            if fallback {
                ExecutionBackend::Threaded
            } else {
                ExecutionBackend::IoUring
            }
        );
        if fallback {
            assert!(matches!(
                plan.execution_selection().reason(),
                ExecutionSelectionReason::RawRequestsIncompatible(
                    NativeRequestIssue::DirectBlockSize {
                        block_size: 4097,
                        ..
                    }
                )
            ));
        }
        mover
            .execute_raw_plan(&plan, &source, &destination)
            .unwrap();
        assert_eq!(bytes(&destination), expected);
    }
}
