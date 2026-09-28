use rvvdk_core::{MemoryBlockDevice, RawDisk};
#[cfg(target_os = "linux")]
use rvvdk_datamover::ExecutionBackend;
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionSelectionReason, ExecutionStrategy};

#[test]
fn portable_plan_records_explicit_threaded_selection() {
    let source = RawDisk::new(MemoryBlockDevice::new(4096).unwrap());
    let plan = DataMover::new(CopyOptions::new(4096).unwrap())
        .plan(&source)
        .unwrap();
    let selection = plan.execution_selection();
    assert_eq!(selection.requested(), ExecutionStrategy::Threaded);
    assert_eq!(selection.selected(), plan.backend());
    assert_eq!(
        selection.reason(),
        ExecutionSelectionReason::RequestedThreaded
    );
}

#[cfg(target_os = "linux")]
#[test]
fn portable_auto_records_the_api_boundary_without_changing_logical_intent() {
    use rvvdk_datamover::IoUringExecutionOptions;
    let source = RawDisk::new(MemoryBlockDevice::new(4096).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(4096).unwrap());
    let options = CopyOptions::new(4096).unwrap();
    let requested = ExecutionStrategy::Auto(IoUringExecutionOptions::new(8).unwrap());
    let explicit = DataMover::new(options).plan(&source).unwrap();
    let automatic = DataMover::with_execution_strategy(options, requested)
        .plan_with_destination(&source, &destination)
        .unwrap();
    assert_eq!(automatic.extents(), explicit.extents());
    assert_eq!(automatic.summary(), explicit.summary());
    assert_eq!(
        automatic.extent_fingerprint(),
        explicit.extent_fingerprint()
    );
    assert_eq!(automatic.execution_selection().requested(), requested);
    assert_eq!(
        automatic.execution_selection().selected(),
        ExecutionBackend::Threaded
    );
    assert_eq!(
        automatic.execution_selection().reason(),
        ExecutionSelectionReason::PortableApi
    );
    // Planning provenance is retained even when another mover executes the plan.
    let before = *automatic.execution_selection();
    let report = DataMover::new(options)
        .execute_plan(&automatic, &source, &destination)
        .unwrap();
    assert_eq!(report.backend(), ExecutionBackend::Threaded);
    assert_eq!(*automatic.execution_selection(), before);
}

#[cfg(target_os = "linux")]
#[test]
fn raw_planning_records_request_and_reason_separately_from_logical_intent() {
    use rvvdk_datamover::IoUringExecutionOptions;
    use rvvdk_local::LocalFileBlockDevice;
    let stem = std::env::temp_dir().join(format!("rvvdk-r14-selection-{}", std::process::id()));
    let src = stem.with_extension("source");
    let dst = stem.with_extension("destination");
    std::fs::write(&src, [0x5a; 4096]).unwrap();
    std::fs::write(&dst, [0xa5; 4096]).unwrap();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&src).unwrap());
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&dst).unwrap());
    std::fs::remove_file(src).unwrap();
    std::fs::remove_file(dst).unwrap();
    let options = CopyOptions::new(4096).unwrap();
    let io = IoUringExecutionOptions::new(2).unwrap();
    let threaded = DataMover::new(options)
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    for (strategy, reason) in [
        (
            ExecutionStrategy::IoUring(io),
            ExecutionSelectionReason::RequestedNative,
        ),
        (
            ExecutionStrategy::Auto(io),
            ExecutionSelectionReason::RawDescriptorsCompatible,
        ),
    ] {
        let native = DataMover::with_execution_strategy(options, strategy)
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        assert_eq!(native.extents(), threaded.extents());
        assert_eq!(native.summary(), threaded.summary());
        assert_eq!(native.extent_fingerprint(), threaded.extent_fingerprint());
        assert_eq!(native.execution_selection().requested(), strategy);
        assert_eq!(
            native.execution_selection().selected(),
            ExecutionBackend::IoUring
        );
        assert_eq!(native.execution_selection().reason(), reason);
    }
    // Existing dispatch policy keeps the plan's Threaded choice authoritative.
    let report = DataMover::with_execution_strategy(options, ExecutionStrategy::IoUring(io))
        .execute_raw_plan(&threaded, &source, &destination)
        .unwrap();
    assert_eq!(report.backend(), ExecutionBackend::Threaded);
}
