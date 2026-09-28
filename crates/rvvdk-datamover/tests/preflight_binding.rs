#![cfg(target_os = "linux")]

use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Extent, RawDisk, Result};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, ProgressObserver,
    ProgressSnapshot,
};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};
use std::fs::{self, File};
use std::os::fd::{AsFd, BorrowedFd};
use std::sync::atomic::{AtomicUsize, Ordering};

// A backend accidentally exposing another file's descriptor must not bypass
// logical endpoint checks when the execution strategy selects native I/O.
struct Mismatched {
    logical: LocalFileBlockDevice,
    exposed: File,
}
impl AsFd for Mismatched {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.exposed.as_fd()
    }
}
impl LinuxFdBackend for Mismatched {
    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        LinuxFdCapabilities::new(false, 1, 1)
    }
}
impl BlockDevice for Mismatched {
    fn geometry(&self) -> DiskGeometry {
        self.logical.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.logical.capabilities()
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.logical.copy_endpoint()
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> Result<usize> {
        panic!("preflight must reject before read")
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        panic!("preflight must reject before write")
    }
    fn flush(&self) -> Result<()> {
        panic!("preflight must reject before flush")
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        self.logical.extents(offset, length)
    }
}
struct Observer(AtomicUsize);
impl ProgressObserver for Observer {
    fn on_progress(&self, _: &ProgressSnapshot) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn descriptor_binding_is_validated_before_planning_or_observation() {
    let paths: Vec<_> = (0..3)
        .map(|i| std::env::temp_dir().join(format!("rvvdk-r05-binding-{}-{i}", std::process::id())))
        .collect();
    for path in &paths {
        fs::write(path, [0x5a; 8192]).unwrap();
    }
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&paths[0]).unwrap());
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&paths[1]).unwrap());
    let mismatch_source = RawDisk::new(Mismatched {
        logical: LocalFileBlockDevice::open_read_only(&paths[0]).unwrap(),
        exposed: File::open(&paths[2]).unwrap(),
    });
    let mismatch_destination = RawDisk::new(Mismatched {
        logical: LocalFileBlockDevice::open_read_write(&paths[1]).unwrap(),
        exposed: File::options()
            .read(true)
            .write(true)
            .open(&paths[2])
            .unwrap(),
    });
    for path in &paths {
        fs::remove_file(path).unwrap();
    }
    for strategy in [
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(2).unwrap()),
        ExecutionStrategy::Auto(IoUringExecutionOptions::new(2).unwrap()),
    ] {
        let mover = DataMover::with_execution_strategy(CopyOptions::new(4096).unwrap(), strategy);
        let plan = mover.plan_with_destination(&source, &destination).unwrap();
        assert!(
            mover
                .plan_with_destination(&mismatch_source, &destination)
                .is_err()
        );
        assert!(
            mover
                .plan_with_destination(&source, &mismatch_destination)
                .is_err()
        );
        let observer = Observer(AtomicUsize::new(0));
        let error = mover
            .execute_plan_with_observer(&plan, &mismatch_source, &destination, &observer)
            .unwrap_err();
        assert!(error.to_string().contains("source preflight"));
        let error = mover
            .execute_plan_with_observer(&plan, &source, &mismatch_destination, &observer)
            .unwrap_err();
        assert!(error.to_string().contains("destination preflight"));
        assert_eq!(observer.0.load(Ordering::Relaxed), 0);
    }
}
