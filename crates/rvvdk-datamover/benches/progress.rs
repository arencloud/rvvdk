mod support;

use std::cell::Cell;
use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{ExtentKind, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, NoopProgressObserver, ProgressSnapshot};
use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_sparse_incompressible_file, ensure_benchmark_directory,
    incompressible_buffer, materialize_file, remove_file,
};

const SIZE: usize = 16 * MIB;
const BLOCK_SIZE: usize = 64 * 1024;
const SEED: u64 = 0x5256_5644_4b52_3031;

fn benchmark_progress(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    for (profile, group_size) in [("dense", 1), ("fragmented50", 2)] {
        let source_path = benchmark_path(&format!("progress-{profile}-source"));
        let destination_path = benchmark_path(&format!("progress-{profile}-destination"));
        create_sparse_incompressible_file(&source_path, SIZE, BLOCK_SIZE, 1, group_size, SEED);
        materialize_file(&destination_path, SIZE, 0xff);

        let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());
        let destination =
            RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());
        let mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());
        let plan = mover.plan_with_destination(&source, &destination).unwrap();
        assert_eq!(plan.data_bytes(), (SIZE / group_size) as u64);

        // Warm buffered source reads are intentional: this measures API/observer
        // overhead, not a device's cold-read bandwidth. Setup and verification
        // stay outside the timed interval for both execution entry points.
        // Reading holes can materialize tmpfs pages and change SEEK_DATA results.
        // Build expected logical bytes from Data extents without touching holes.
        let mut expected = vec![0_u8; SIZE];
        for extent in plan.extents() {
            if extent.kind() == ExtentKind::Data {
                source
                    .read_exact_at(
                        extent.offset(),
                        &mut expected[extent.offset() as usize..extent.end() as usize],
                    )
                    .unwrap();
            }
        }
        let prefill = incompressible_buffer(SIZE, SEED + 1);
        let mut actual = vec![0_u8; SIZE];
        println!(
            "progress/{profile}: logical={}, data={}, holes={}, extents={}, block={}, workers=1, buffered, flush=included",
            plan.logical_bytes(),
            plan.data_bytes(),
            plan.hole_bytes(),
            plan.extent_count(),
            BLOCK_SIZE,
        );

        let mut planning = criterion.benchmark_group(format!("progress_plan/{profile}"));
        planning.sampling_mode(SamplingMode::Flat);
        planning.throughput(Throughput::Elements(plan.extent_count() as u64));
        planning.bench_function("plan", |bencher| {
            bencher.iter(|| black_box(mover.plan_with_destination(&source, &destination).unwrap()));
        });
        planning.finish();

        let mut execution = criterion.benchmark_group(format!("progress_copy/{profile}"));
        execution.sampling_mode(SamplingMode::Flat);
        execution.throughput(Throughput::Bytes(SIZE as u64));

        for mode in ["unobserved", "noop", "counter"] {
            execution.bench_function(mode, |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        destination.write_all_at(0, &prefill).unwrap();
                        destination.flush().unwrap();

                        let callbacks = Cell::new(0_u64);
                        let observer = |snapshot: &ProgressSnapshot| {
                            callbacks.set(callbacks.get() + 1);
                            black_box(snapshot.logical_bytes_completed());
                        };

                        let started = Instant::now();
                        let report = match mode {
                            "unobserved" => mover.execute_plan(&plan, &source, &destination),
                            "noop" => mover.execute_plan_with_observer(
                                &plan,
                                &source,
                                &destination,
                                &NoopProgressObserver,
                            ),
                            "counter" => mover.execute_plan_with_observer(
                                &plan,
                                &source,
                                &destination,
                                &observer,
                            ),
                            _ => unreachable!(),
                        }
                        .unwrap();
                        elapsed += started.elapsed();

                        assert_eq!(report.stats().bytes_read(), plan.data_bytes());
                        let discarded = if destination.capabilities().contains(
                            rvvdk_core::Capabilities::DISCARD
                                | rvvdk_core::Capabilities::DISCARD_ZEROES,
                        ) {
                            plan.hole_bytes()
                        } else {
                            0
                        };
                        assert_eq!(report.stats().bytes_discarded(), discarded);
                        assert_eq!(report.stats().bytes_written(), SIZE as u64 - discarded);
                        destination.read_exact_at(0, &mut actual).unwrap();
                        assert_eq!(actual, expected, "{profile}/{mode}: output differs");
                        if mode == "counter" {
                            assert_eq!(callbacks.get(), plan.extent_count() as u64 + 1);
                        }
                    }
                    elapsed
                });
            });
        }
        execution.finish();

        drop(destination);
        drop(source);
        remove_file(source_path);
        remove_file(destination_path);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1));
    targets = benchmark_progress
}
criterion_main!(benches);
