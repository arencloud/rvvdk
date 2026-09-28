#![cfg(target_os = "linux")]
mod support;
use rvvdk_core::{BlockDevice, Capabilities, Extent, ExtentKind, RawDisk, VirtualDisk};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions, NoopProgressObserver,
};
use rvvdk_local::LocalFileBlockDevice;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use support::test_block_device::TestExtentBlockDevice;

#[test]
fn local_sparse_destination_preserves_bytes_and_counters_across_executors() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    for direct in [false, true] {
        for executor in [0, 1, 2] {
            for observed in [false, true] {
                let stem = std::env::temp_dir().join(format!(
                    "rvvdk-r22-copy-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                let src = stem.with_extension("src");
                let dst = stem.with_extension("dst");
                let layout = [
                    (0, 4096, ExtentKind::Data),
                    (4096, 7, ExtentKind::Zero),
                    (4103, 16370, ExtentKind::Hole),
                    (20473, 7, ExtentKind::Zero),
                    (20480, 4096, ExtentKind::Data),
                ];
                let extents: Vec<_> = layout
                    .into_iter()
                    .map(|(o, l, k)| Extent::new(o, l, k).unwrap())
                    .collect();
                let device = TestExtentBlockDevice::create(
                    &src,
                    24576,
                    Capabilities::READ
                        | Capabilities::WRITE
                        | Capabilities::FLUSH
                        | Capabilities::EXTENTS,
                    extents.clone(),
                )
                .unwrap();
                let mut expected = vec![0; 24576];
                expected[..4096].fill(0x5a);
                expected[20480..].fill(0x3c);
                device.write_all_at(0, &expected).unwrap();
                device.flush().unwrap();
                drop(device);
                let source = RawDisk::new(
                    TestExtentBlockDevice::open_read_only(
                        &src,
                        24576,
                        Capabilities::READ | Capabilities::EXTENTS,
                        extents,
                    )
                    .unwrap(),
                );
                fs::write(&dst, vec![0xa5; 24576]).unwrap();
                let destination = RawDisk::new(
                    if direct {
                        LocalFileBlockDevice::open_direct_read_write(&dst)
                    } else {
                        LocalFileBlockDevice::open_read_write(&dst)
                    }
                    .unwrap(),
                );
                let options =
                    CopyOptions::with_execution(4096, 4096, if executor == 1 { 4 } else { 1 }, 1)
                        .unwrap();
                let mover = DataMover::with_execution_strategy(
                    options,
                    if executor == 2 {
                        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(4).unwrap())
                    } else {
                        ExecutionStrategy::Threaded
                    },
                );
                let plan = mover
                    .plan_raw_with_destination(&source, &destination)
                    .unwrap();
                let report = if observed {
                    mover.execute_raw_plan_with_observer(
                        &plan,
                        &source,
                        &destination,
                        &NoopProgressObserver,
                    )
                } else {
                    mover.execute_raw_plan(&plan, &source, &destination)
                }
                .unwrap();
                let stats = report.stats();
                assert_eq!(stats.bytes_read(), 8192);
                assert_eq!(stats.bytes_written(), 8192);
                assert_eq!(stats.bytes_zeroed(), 14);
                assert_eq!(stats.bytes_discarded(), 16370);
                assert_eq!(stats.extents_processed(), 5);
                let mut actual = vec![0; 24576];
                destination.read_exact_at(0, &mut actual).unwrap();
                assert_eq!(actual, expected);
                assert_eq!(fs::read(&dst).unwrap(), expected);
                fs::remove_file(src).unwrap();
                fs::remove_file(dst).unwrap();
            }
        }
    }
}
