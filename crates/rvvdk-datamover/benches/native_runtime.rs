mod support;
#[path = "../tests/support/test_block_device.rs"]
mod test_block_device;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_core::{BlockDevice, Capabilities, Extent, ExtentKind, RawDisk, VirtualDisk};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
    NoopProgressObserver,
};
use rvvdk_local::LocalFileBlockDevice;
use std::time::{Duration, Instant};
use test_block_device::TestExtentBlockDevice;

fn native_runtime(c: &mut Criterion) {
    support::ensure_benchmark_directory();
    let unavailable = std::env::var_os("RVVDK_BENCH_NATIVE_UNAVAILABLE").is_some();
    let size = 16 * support::MIB;
    let source_path = support::benchmark_path("native-runtime-source");
    let destination_path = support::benchmark_path("native-runtime-destination");
    let extents: Vec<_> = (0..256)
        .map(|i| {
            Extent::new(
                i * 65536,
                65536,
                if unavailable || i % 2 == 0 {
                    ExtentKind::Data
                } else {
                    ExtentKind::Hole
                },
            )
            .unwrap()
        })
        .collect();
    let mut expected = support::incompressible_buffer(size, 0x52562501);
    for extent in &extents {
        if extent.kind() == ExtentKind::Hole {
            expected[extent.offset() as usize..extent.end() as usize].fill(0);
        }
    }
    let device = TestExtentBlockDevice::create(
        &source_path,
        size as u64,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | Capabilities::EXTENTS,
        extents.clone(),
    )
    .unwrap();
    device.write_all_at(0, &expected).unwrap();
    device.flush().unwrap();
    drop(device);
    let source = RawDisk::new(
        TestExtentBlockDevice::open_read_only(
            &source_path,
            size as u64,
            Capabilities::READ | Capabilities::EXTENTS,
            extents,
        )
        .unwrap(),
    );
    let reset = support::incompressible_buffer(size, 0x52562502);
    std::fs::write(&destination_path, &reset).unwrap();
    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());
    let options = IoUringExecutionOptions::new(8).unwrap();
    let mover = DataMover::with_execution_strategy(
        CopyOptions::new(65536).unwrap(),
        if unavailable {
            ExecutionStrategy::Auto(options)
        } else {
            ExecutionStrategy::IoUring(options)
        },
    );
    let plan = mover
        .plan_raw_with_destination(&source, &destination)
        .unwrap();
    let mut actual = vec![0; size];
    let mut group = c.benchmark_group("native_runtime");
    group.sampling_mode(SamplingMode::Flat);
    let cases: &[(&str, bool)] = if unavailable {
        &[("unavailable_fallback", false)]
    } else {
        &[("fragmented_unobserved", false), ("fragmented_noop", true)]
    };
    for &(name, observed) in cases {
        group.bench_function(name, |b| {
            b.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    destination.write_all_at(0, &reset).unwrap();
                    destination.flush().unwrap();
                    let started = Instant::now();
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
                    elapsed += started.elapsed();
                    assert_eq!(
                        report.backend(),
                        if unavailable {
                            ExecutionBackend::Threaded
                        } else {
                            ExecutionBackend::IoUring
                        }
                    );
                    assert_eq!(
                        report.stats().bytes_read(),
                        if unavailable { size } else { size / 2 } as u64
                    );
                    assert_eq!(
                        report.stats().bytes_written()
                            + report.stats().bytes_zeroed()
                            + report.stats().bytes_discarded(),
                        size as u64
                    );
                    destination.read_exact_at(0, &mut actual).unwrap();
                    assert_eq!(actual, expected);
                }
                elapsed
            })
        });
    }
    group.finish();
    drop(source);
    drop(destination);
    support::remove_file(source_path);
    support::remove_file(destination_path);
}
criterion_group! { name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=native_runtime }
criterion_main!(benches);
