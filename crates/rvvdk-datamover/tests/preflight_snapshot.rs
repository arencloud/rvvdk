#![cfg(target_os = "linux")]

use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Extent, RawDisk, Result};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, ProgressObserver,
    ProgressSnapshot,
};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};
use std::fs;
use std::os::fd::{AsFd, BorrowedFd};
use std::sync::atomic::{AtomicUsize, Ordering};

// This wrapper intentionally uses the default inspection hook: access to an
// underlying writable FD must never grant operations denied by its logical API.
struct Restricted {
    inner: LocalFileBlockDevice,
    denied: AtomicUsize,
    calls: AtomicUsize,
}
impl Restricted {
    fn new(inner: LocalFileBlockDevice) -> Self {
        Self {
            inner,
            denied: AtomicUsize::new(0),
            calls: AtomicUsize::new(0),
        }
    }
}
impl AsFd for Restricted {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}
impl LinuxFdBackend for Restricted {
    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        self.inner.linux_fd_capabilities()
    }
}
impl BlockDevice for Restricted {
    fn geometry(&self) -> DiskGeometry {
        self.inner.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let mut info = self.inner.copy_endpoint()?;
        match self.denied.load(Ordering::Relaxed) {
            1 => info.capabilities.remove(Capabilities::READ),
            2 => info.capabilities.remove(Capabilities::WRITE),
            3 => info.capabilities.remove(Capabilities::FLUSH),
            4 => info.identity = None,
            _ => {}
        }
        Ok(info)
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        self.inner.extents(offset, length)
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> Result<usize> {
        panic!("rejection must precede payload I/O")
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        panic!("rejection must precede payload I/O")
    }
    fn flush(&self) -> Result<()> {
        panic!("rejection must precede flush")
    }
}
#[derive(Default)]
struct Observer(AtomicUsize);
impl ProgressObserver for Observer {
    fn on_progress(&self, _: &ProgressSnapshot) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
fn fixture() -> (RawDisk<Restricted>, RawDisk<Restricted>, std::fs::File) {
    let stem = std::env::temp_dir().join(format!("rvvdk-r13-restrict-{}", std::process::id()));
    let src = stem.with_extension("source");
    let dst = stem.with_extension("destination");
    fs::write(&src, [0x5a; 8192]).unwrap();
    fs::write(&dst, [0xa5; 8192]).unwrap();
    let source = LocalFileBlockDevice::open_read_only(&src).unwrap();
    let destination = LocalFileBlockDevice::open_read_write(&dst).unwrap();
    let verify = std::fs::File::open(&dst).unwrap();
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    (
        RawDisk::new(Restricted::new(source)),
        RawDisk::new(Restricted::new(destination)),
        verify,
    )
}

#[test]
fn custom_endpoint_checks_remain_fresh_and_authoritative() {
    use std::os::unix::fs::FileExt;
    let (source, destination, verify) = fixture();
    for strategy in [
        ExecutionStrategy::Threaded,
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(2).unwrap()),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(2).unwrap()),
    ] {
        let mover = DataMover::with_execution_strategy(CopyOptions::new(4096).unwrap(), strategy);
        for (role, denied, message) in [
            ("source", 1, "read"),
            ("destination", 2, "write"),
            ("destination", 3, "flush"),
            ("source", 4, "known backing identity"),
            ("destination", 4, "known backing identity"),
        ] {
            if denied == 4 && strategy == ExecutionStrategy::Threaded {
                continue;
            }
            let plan = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap();
            let endpoint = if role == "source" {
                source.device()
            } else {
                destination.device()
            };
            endpoint.denied.store(denied, Ordering::Relaxed);
            let calls_before = endpoint.calls.load(Ordering::Relaxed);
            let error = mover
                .plan_raw_with_destination(&source, &destination)
                .unwrap_err();
            assert!(
                error.to_string().contains(&format!("{role} preflight")),
                "{error}"
            );
            assert!(error.to_string().contains(message), "{error}");
            let observer = Observer::default();
            let error = mover
                .execute_raw_plan_with_observer(&plan, &source, &destination, &observer)
                .unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
            assert_eq!(observer.0.load(Ordering::Relaxed), 0);
            assert_eq!(endpoint.calls.load(Ordering::Relaxed) - calls_before, 2);
            endpoint.denied.store(0, Ordering::Relaxed);
            let mut actual = [0; 8192];
            verify.read_exact_at(&mut actual, 0).unwrap();
            assert_eq!(actual, [0xa5; 8192]);
        }
    }
}
