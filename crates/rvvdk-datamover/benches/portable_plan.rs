use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover};

// Candidate-only baseline for the newly portable API. Reset/readback are untimed;
// allocation, validation, execution, worker creation, and flush are timed.
fn portable_plan(criterion: &mut Criterion) {
    const SIZE: usize = 16 * 1024 * 1024;
    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let expected: Vec<u8> = (0..SIZE)
        .map(|i| (i.wrapping_mul(31) ^ (i >> 8)) as u8)
        .collect();
    let reset = vec![0xa5; SIZE];
    let mut actual = vec![0; SIZE];
    source.write_all_at(0, &expected).unwrap();
    for workers in [1, 4] {
        let mover =
            DataMover::new(CopyOptions::with_concurrency(64 * 1024, 4096, workers).unwrap());
        let plan = mover.plan_with_destination(&source, &destination).unwrap();
        let mut group = criterion.benchmark_group(format!("portable_memory/workers{workers}"));
        group.sampling_mode(SamplingMode::Flat);
        group.throughput(Throughput::Bytes(SIZE as u64));
        for dynamic in [false, true] {
            group.bench_function(if dynamic { "dyn" } else { "static" }, |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        destination.write_all_at(0, &reset).unwrap();
                        let started = Instant::now();
                        let report = if dynamic {
                            let source: &dyn VirtualDisk = black_box(&source);
                            let destination: &dyn VirtualDisk = black_box(&destination);
                            mover.execute_plan(&plan, source, destination)
                        } else {
                            mover.execute_plan(&plan, &source, &destination)
                        }
                        .unwrap();
                        elapsed += started.elapsed();
                        assert_eq!(report.stats().bytes_read(), SIZE as u64);
                        assert_eq!(report.stats().bytes_written(), SIZE as u64);
                        destination.read_exact_at(0, &mut actual).unwrap();
                        assert_eq!(actual, expected);
                    }
                    elapsed
                });
            });
        }
        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = portable_plan
}
criterion_main!(benches);
