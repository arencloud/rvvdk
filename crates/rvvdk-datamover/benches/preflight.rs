mod support;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_core::{RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions};
use rvvdk_local::LocalFileBlockDevice;
use std::hint::black_box;
use std::time::{Duration, Instant};

fn preflight(criterion: &mut Criterion) {
    support::ensure_benchmark_directory();
    let source_path = support::benchmark_path("preflight-source");
    let destination_path = support::benchmark_path("preflight-destination");
    support::create_incompressible_file(&source_path, support::MIB, 0x0052_5605);
    support::create_incompressible_file(&destination_path, support::MIB, 0x0052_5606);
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());
    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());
    let before = std::fs::read(&destination_path).unwrap();
    let mut group = criterion.benchmark_group("preflight");
    group.sampling_mode(SamplingMode::Flat);
    for (name, strategy) in [
        ("plan_threaded", ExecutionStrategy::Threaded),
        (
            "plan_native",
            ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
        ),
    ] {
        let mover = DataMover::with_execution_strategy(CopyOptions::new(65536).unwrap(), strategy);
        group.bench_function(name, |b| {
            b.iter(|| {
                let plan = mover
                    .plan_raw_with_destination(black_box(&source), black_box(&destination))
                    .unwrap();
                assert_eq!(plan.logical_bytes(), support::MIB as u64);
                black_box(plan);
            })
        });
    }
    group.finish();
    assert_eq!(std::fs::read(&destination_path).unwrap(), before);

    // Full RAW public-call boundary: fresh planning + execution preflight,
    // payload transfer, and final flush. Reset/readback are deliberately untimed.
    let expected = std::fs::read(&source_path).unwrap();
    let mut actual = vec![0; support::MIB];
    let mut group = criterion.benchmark_group("preflight_copy");
    group.sampling_mode(SamplingMode::Flat);
    for (name, strategy) in [
        ("threaded", ExecutionStrategy::Threaded),
        (
            "native",
            ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
        ),
    ] {
        let mover = DataMover::with_execution_strategy(CopyOptions::new(65536).unwrap(), strategy);
        group.bench_function(name, |b| {
            b.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    destination.write_all_at(0, &before).unwrap();
                    destination.flush().unwrap();
                    let started = Instant::now();
                    let report = mover.copy_raw_with_report(&source, &destination).unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(report.stats().bytes_read(), support::MIB as u64);
                    assert_eq!(report.stats().bytes_written(), support::MIB as u64);
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
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = preflight
}
criterion_main!(benches);
