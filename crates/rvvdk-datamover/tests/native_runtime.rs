#![cfg(target_os = "linux")]
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, CopyOperation, DiskGeometry, Error, Extent,
    ExtentKind, RawDisk, Result, VirtualDisk,
};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
    NativeRuntimeFallback, ProgressSnapshot,
};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};
use std::{
    cell::Cell,
    fs,
    os::fd::{AsFd, BorrowedFd},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Disk {
    file: LocalFileBlockDevice,
    map: Vec<Extent>,
    reads: AtomicUsize,
}
impl AsFd for Disk {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}
impl LinuxFdBackend for Disk {
    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        self.file.linux_fd_capabilities()
    }
}
impl BlockDevice for Disk {
    fn geometry(&self) -> DiskGeometry {
        self.file.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.file.capabilities()
            & (Capabilities::READ
                | Capabilities::WRITE
                | Capabilities::FLUSH
                | Capabilities::EXTENTS)
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.file.copy_endpoint()
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        Ok(self.map.clone())
    }
    fn read_at(&self, offset: u64, bytes: &mut [u8]) -> Result<usize> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.file.read_at(offset, bytes)
    }
    fn write_at(&self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.file.write_at(offset, bytes)
    }
    fn flush(&self) -> Result<()> {
        self.file.flush()
    }
}
fn fixture(kinds: &[ExtentKind]) -> (RawDisk<Disk>, RawDisk<Disk>, Vec<u8>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::var_os("RVVDK_TEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "rvvdk-r25-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    let src = path.with_extension("src");
    let dst = path.with_extension("dst");
    let expected: Vec<_> = kinds
        .iter()
        .flat_map(|k| vec![if *k == ExtentKind::Data { 0x5a } else { 0 }; 4096])
        .collect();
    fs::write(&src, &expected).unwrap();
    fs::write(&dst, vec![0xa5; expected.len()]).unwrap();
    let source = LocalFileBlockDevice::open_read_only(&src).unwrap();
    let destination = LocalFileBlockDevice::open_read_write(&dst).unwrap();
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    let wrap = |file| {
        RawDisk::new(Disk {
            file,
            map: kinds
                .iter()
                .enumerate()
                .map(|(i, k)| Extent::new(i as u64 * 4096, 4096, *k).unwrap())
                .collect(),
            reads: AtomicUsize::new(0),
        })
    };
    (wrap(source), wrap(destination), expected)
}
fn mover(auto: bool, options: CopyOptions) -> DataMover {
    let native = IoUringExecutionOptions::new(2).unwrap();
    DataMover::with_execution_strategy(
        options,
        if auto {
            ExecutionStrategy::Auto(native)
        } else {
            ExecutionStrategy::IoUring(native)
        },
    )
}
fn bytes(disk: &RawDisk<Disk>) -> Vec<u8> {
    let mut bytes = vec![0; disk.size() as usize];
    disk.read_exact_at(0, &mut bytes).unwrap();
    bytes
}

// Irreversible syscall filtering is isolated in a bounded child process.
fn isolated(name: &str, test: impl FnOnce()) {
    const CHILD: &str = "RVVDK_NATIVE_RUNTIME_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        test();
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(CHILD, name)
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() > until {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("runtime test timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn deny(syscall: libc::c_long, errno: i32) {
    let instruction = |code, jt, jf, k| libc::sock_filter { code, jt, jf, k };
    let mut filter = [
        instruction(0x20, 0, 0, 0),
        instruction(0x15, 0, 1, syscall as u32),
        instruction(0x06, 0, 0, libc::SECCOMP_RET_ERRNO | errno as u32),
        instruction(0x06, 0, 0, libc::SECCOMP_RET_ALLOW),
    ];
    let program = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_mut_ptr(),
    };
    // SAFETY: valid filter pointers live through prctl; only this disposable test thread is filtered.
    unsafe {
        assert_eq!(libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0), 0);
        assert_eq!(
            libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program),
            0
        );
    }
}
fn fd_count() -> usize {
    fs::read_dir("/proc/self/fd").unwrap().count()
}

#[test]
fn denied_setup_falls_back_before_progress_for_auto_only() {
    isolated(
        "denied_setup_falls_back_before_progress_for_auto_only",
        || {
            deny(libc::SYS_io_uring_setup, libc::EPERM);
            for auto in [false, true] {
                for workers in [1, 4] {
                    for observed in [false, true] {
                        let (source, destination, expected) = fixture(&[
                            ExtentKind::Zero,
                            ExtentKind::Data,
                            ExtentKind::Hole,
                            ExtentKind::Data,
                        ]);
                        let mover = mover(
                            auto,
                            CopyOptions::with_execution(4096, 4096, workers, 2).unwrap(),
                        );
                        let plan = mover
                            .plan_raw_with_destination(&source, &destination)
                            .unwrap();
                        assert_eq!(plan.backend(), ExecutionBackend::IoUring);
                        let callbacks = Cell::new(0);
                        let observer = |s: &ProgressSnapshot| {
                            callbacks.set(callbacks.get() + 1);
                            assert_eq!(s.backend(), ExecutionBackend::Threaded);
                        };
                        let before = fd_count();
                        let result = if observed {
                            mover.execute_raw_plan_with_observer(
                                &plan,
                                &source,
                                &destination,
                                &observer,
                            )
                        } else {
                            mover.execute_raw_plan(&plan, &source, &destination)
                        };
                        assert_eq!(fd_count(), before);
                        if auto {
                            let report = result.unwrap();
                            assert_eq!(report.backend(), ExecutionBackend::Threaded);
                            assert_eq!(
                                report.runtime_fallback(),
                                Some(NativeRuntimeFallback::RingUnavailable {
                                    os_error: libc::EPERM
                                })
                            );
                            assert_eq!(bytes(&destination), expected);
                            assert_eq!(report.stats().bytes_read(), 8192);
                            assert_eq!(report.stats().bytes_written(), 16384);
                            assert!(source.device().reads.load(Ordering::Relaxed) > 0);
                            assert_eq!(observed, callbacks.get() > 0);
                        } else {
                            let error = result.unwrap_err();
                            let failure = error.copy_failure().unwrap();
                            assert_eq!(failure.operation, CopyOperation::NativeSetup);
                            assert_eq!(failure.progress.bytes_written, 0);
                            assert!(!failure.progress.unconfirmed_io);
                            assert!(
                                matches!(&failure.cause,Error::Io(e) if e.raw_os_error()==Some(libc::EPERM))
                            );
                            assert_eq!(callbacks.get(), 0);
                            assert_eq!(bytes(&destination), vec![0xa5; expected.len()]);
                        }
                        assert_eq!(
                            plan.backend(),
                            ExecutionBackend::IoUring,
                            "planning provenance is immutable"
                        );
                    }
                }
            }
        },
    );
}

#[test]
fn prepared_ring_is_reused_across_data_and_zero_fallback_extents() {
    isolated(
        "prepared_ring_is_reused_across_data_and_zero_fallback_extents",
        || {
            let (source, destination, expected) = fixture(&[
                ExtentKind::Zero,
                ExtentKind::Data,
                ExtentKind::Hole,
                ExtentKind::Data,
                ExtentKind::Zero,
            ]);
            let mover = mover(false, CopyOptions::new(4096).unwrap());
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let before = fd_count();
            let calls = Cell::new(0);
            let report = mover
                .execute_raw_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|s: &ProgressSnapshot| {
                        assert_eq!(s.backend(), ExecutionBackend::IoUring);
                        if calls.replace(calls.get() + 1) == 0 {
                            deny(libc::SYS_io_uring_setup, libc::EPERM);
                        }
                    },
                )
                .unwrap();
            assert_eq!(calls.get(), 2);
            assert_eq!(fd_count(), before);
            assert_eq!(report.runtime_fallback(), None);
            assert_eq!(report.stats().bytes_read(), 8192);
            assert_eq!(report.stats().bytes_written(), 20480);
            assert_eq!(source.device().reads.load(Ordering::Relaxed), 0);
            assert_eq!(bytes(&destination), expected);
        },
    );
}

#[test]
fn sparse_only_and_empty_plans_do_not_require_a_ring() {
    isolated("sparse_only_and_empty_plans_do_not_require_a_ring", || {
        deny(libc::SYS_io_uring_setup, libc::EPERM);
        for kinds in [&[][..], &[ExtentKind::Zero, ExtentKind::Hole][..]] {
            let (source, destination, expected) = fixture(kinds);
            let mover = mover(false, CopyOptions::new(4096).unwrap());
            let report = mover.copy_raw_with_report(&source, &destination).unwrap();
            assert_eq!(report.backend(), ExecutionBackend::IoUring);
            assert_eq!(report.runtime_fallback(), None);
            assert_eq!(bytes(&destination), expected);
        }
    });
}

#[test]
fn invalid_or_exhausted_ring_setup_does_not_fallback() {
    isolated("invalid_or_exhausted_ring_setup_does_not_fallback", || {
        for errno in [libc::EINVAL, libc::ENOMEM, libc::EMFILE] {
            deny(libc::SYS_io_uring_setup, errno);
            let (source, destination, expected) = fixture(&[ExtentKind::Zero, ExtentKind::Data]);
            let mover = mover(true, CopyOptions::new(4096).unwrap());
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let calls = Cell::new(0);
            let error = mover
                .execute_raw_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|_: &ProgressSnapshot| calls.set(calls.get() + 1),
                )
                .unwrap_err();
            assert!(
                matches!(&error.copy_failure().unwrap().cause,Error::Io(e) if e.raw_os_error()==Some(errno))
            );
            assert_eq!(calls.get(), 0);
            assert_eq!(bytes(&destination), vec![0xa5; expected.len()]);
        }
    });
}

#[test]
fn runtime_fallback_checks_the_threaded_budget_before_observation() {
    isolated(
        "runtime_fallback_checks_the_threaded_budget_before_observation",
        || {
            let (source, destination, expected) = fixture(&[ExtentKind::Data]);
            let options = CopyOptions::with_execution(4096, 4096, 16, 2).unwrap();
            let planner = mover(true, options);
            let plan = planner
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let budget = planner.execution_memory(&plan).unwrap().total_bytes();
            let mover = mover(true, options.with_memory_budget(budget));
            deny(libc::SYS_io_uring_setup, libc::ENOSYS);
            let calls = Cell::new(0);
            let error = mover
                .execute_raw_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|_: &ProgressSnapshot| calls.set(calls.get() + 1),
                )
                .unwrap_err();
            assert!(matches!(
                error,
                Error::MemoryBudgetExceeded {
                    phase: "runtime fallback",
                    ..
                }
            ));
            assert_eq!(calls.get(), 0);
            assert_eq!(bytes(&destination), vec![0xa5; expected.len()]);
        },
    );
}

#[test]
fn observer_panic_releases_unsubmitted_native_resources() {
    isolated(
        "observer_panic_releases_unsubmitted_native_resources",
        || {
            let (source, destination, expected) = fixture(&[ExtentKind::Data]);
            let mover = mover(false, CopyOptions::new(4096).unwrap());
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let before = fd_count();
            for _ in 0..4 {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mover.execute_raw_plan_with_observer(
                        &plan,
                        &source,
                        &destination,
                        &|_: &ProgressSnapshot| panic!("observer"),
                    )
                }));
                assert!(result.is_err());
                assert_eq!(fd_count(), before);
                assert_eq!(bytes(&destination), vec![0xa5; expected.len()]);
            }
        },
    );
}

#[test]
fn submission_failure_after_sparse_prefix_never_retries_threaded() {
    isolated(
        "submission_failure_after_sparse_prefix_never_retries_threaded",
        || {
            let (source, destination, _) = fixture(&[ExtentKind::Zero, ExtentKind::Data]);
            let mover = mover(true, CopyOptions::new(4096).unwrap());
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            deny(libc::SYS_io_uring_enter, libc::EIO);
            let calls = Cell::new(0);
            let error = mover
                .execute_raw_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|s: &ProgressSnapshot| {
                        calls.set(calls.get() + 1);
                        assert_eq!(s.backend(), ExecutionBackend::IoUring);
                    },
                )
                .unwrap_err();
            let failure = error.copy_failure().unwrap();
            assert_eq!(failure.progress.bytes_written, 4096);
            assert_eq!(failure.progress.extents_completed, Some(1));
            assert_eq!(calls.get(), 1);
            assert_eq!(source.device().reads.load(Ordering::Relaxed), 0);
            assert_eq!(&bytes(&destination)[..4096], &[0; 4096]);
        },
    );
}

#[test]
fn descriptor_duplication_denial_is_not_ring_unavailability() {
    isolated(
        "descriptor_duplication_denial_is_not_ring_unavailability",
        || {
            let (source, destination, expected) = fixture(&[ExtentKind::Zero, ExtentKind::Data]);
            let mover = mover(true, CopyOptions::new(4096).unwrap());
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let instruction = |code, jt, jf, k| libc::sock_filter { code, jt, jf, k };
            let mut filter = [
                instruction(0x20, 0, 0, 0), // seccomp_data.nr
                instruction(0x15, 0, 3, libc::SYS_fcntl as u32),
                instruction(0x20, 0, 0, 24), // low word of args[1]: fcntl command (little endian)
                instruction(0x15, 0, 1, libc::F_DUPFD_CLOEXEC as u32),
                instruction(0x06, 0, 0, libc::SECCOMP_RET_ERRNO | libc::EPERM as u32),
                instruction(0x06, 0, 0, libc::SECCOMP_RET_ALLOW),
            ];
            // Adjust the command word for big-endian Linux targets.
            if cfg!(target_endian = "big") {
                filter[2].k = 28;
            }
            let program = libc::sock_fprog {
                len: filter.len() as u16,
                filter: filter.as_mut_ptr(),
            };
            // SAFETY: filter storage remains live during prctl, in an isolated child.
            unsafe {
                assert_eq!(libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0), 0);
                assert_eq!(
                    libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program),
                    0
                );
            }
            let before = fd_count();
            let calls = Cell::new(0);
            let error = mover
                .execute_raw_plan_with_observer(
                    &plan,
                    &source,
                    &destination,
                    &|_: &ProgressSnapshot| calls.set(calls.get() + 1),
                )
                .unwrap_err();
            let failure = error.copy_failure().unwrap();
            assert_eq!(failure.operation, CopyOperation::NativeSetup);
            assert!(matches!(
                &failure.cause,
                Error::EndpointPreflight {
                    endpoint: "source descriptor",
                    ..
                }
            ));
            assert_eq!(calls.get(), 0);
            assert_eq!(fd_count(), before);
            assert_eq!(bytes(&destination), vec![0xa5; expected.len()]);
        },
    );
}
