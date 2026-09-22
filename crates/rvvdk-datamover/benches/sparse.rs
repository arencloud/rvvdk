mod support;

use std::fs;
use std::time::{Duration, Instant};

use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};

use rvvdk_core::{ExtentKind, RawDisk, VirtualDisk};

use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
};

use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_sparse_incompressible_file, ensure_benchmark_directory,
    materialize_file, remove_file,
};

const DISK_SIZE: usize = 256 * MIB;

const EXTENT_SIZE: usize = 4 * MIB;

const BLOCK_SIZE: usize = 64 * 1024;

const ALIGNMENT: usize = 4096;

const IO_URING_QUEUE_DEPTH: u32 = 8;

const THREADED_WORKERS: usize = 1;

const THREADED_QUEUE_CAPACITY: usize = 4;

const SOURCE_SEED: u64 = 0x5256_5644_4b19_e300;

#[derive(Debug, Clone, Copy)]
struct SparseProfile {
    name: &'static str,
    data_extents_per_group: usize,
    extents_per_group: usize,
    expected_data_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
struct ExtentSummary {
    data_bytes: u64,
    hole_bytes: u64,
    zero_bytes: u64,
    data_extents: usize,
    hole_extents: usize,
    zero_extents: usize,
}

fn profiles() -> [SparseProfile; 4] {
    [
        SparseProfile {
            name: "dense",
            data_extents_per_group: 1,
            extents_per_group: 1,
            expected_data_bytes: DISK_SIZE,
        },
        /*
         * D D D H
         *
         * 75% Data
         * 25% sparse
         */
        SparseProfile {
            name: "sparse25",
            data_extents_per_group: 3,
            extents_per_group: 4,
            expected_data_bytes: DISK_SIZE * 3 / 4,
        },
        /*
         * D H
         *
         * 50% Data
         * 50% sparse
         */
        SparseProfile {
            name: "sparse50",
            data_extents_per_group: 1,
            extents_per_group: 2,
            expected_data_bytes: DISK_SIZE / 2,
        },
        /*
         * D H H H
         *
         * 25% Data
         * 75% sparse
         */
        SparseProfile {
            name: "sparse75",
            data_extents_per_group: 1,
            extents_per_group: 4,
            expected_data_bytes: DISK_SIZE / 4,
        },
    ]
}

fn threaded_options() -> CopyOptions {
    CopyOptions::with_execution(
        BLOCK_SIZE,
        ALIGNMENT,
        THREADED_WORKERS,
        THREADED_QUEUE_CAPACITY,
    )
    .unwrap()
}

fn native_options() -> CopyOptions {
    CopyOptions::new(BLOCK_SIZE).unwrap()
}

fn summarize_extents(source_path: &std::path::Path) -> ExtentSummary {
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(source_path).unwrap());

    let extents = source.extents(0, DISK_SIZE as u64).unwrap();

    let mut summary = ExtentSummary {
        data_bytes: 0,
        hole_bytes: 0,
        zero_bytes: 0,
        data_extents: 0,
        hole_extents: 0,
        zero_extents: 0,
    };

    for extent in extents {
        match extent.kind() {
            ExtentKind::Data => {
                summary.data_bytes += extent.length();

                summary.data_extents += 1;
            }

            ExtentKind::Hole => {
                summary.hole_bytes += extent.length();

                summary.hole_extents += 1;
            }

            ExtentKind::Zero => {
                summary.zero_bytes += extent.length();

                summary.zero_extents += 1;
            }
        }
    }

    summary
}

fn benchmark_sparse_copy(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    for profile in profiles() {
        benchmark_profile(criterion, profile);
    }
}

fn benchmark_profile(criterion: &mut Criterion, profile: SparseProfile) {
    let source_path = benchmark_path(&format!("{}-source", profile.name,));

    create_sparse_incompressible_file(
        &source_path,
        DISK_SIZE,
        EXTENT_SIZE,
        profile.data_extents_per_group,
        profile.extents_per_group,
        SOURCE_SEED,
    );

    let summary = summarize_extents(&source_path);

    /*
     * Validate that the filesystem extent map agrees with the logical
     * benchmark profile before measuring anything.
     *
     * LocalFileBlockDevice reports real filesystem Data/Hole extents,
     * so this also detects unexpected allocation behavior.
     */
    assert_eq!(
        summary.data_bytes, profile.expected_data_bytes as u64,
        "{}: filesystem Data bytes differ from benchmark profile",
        profile.name,
    );

    assert_eq!(
        summary.data_bytes + summary.hole_bytes + summary.zero_bytes,
        DISK_SIZE as u64,
        "{}: extent map does not cover the complete disk",
        profile.name,
    );

    println!(
        concat!(
            "{} extent map: ",
            "logical={} MiB, ",
            "data={} MiB ({} extents), ",
            "hole={} MiB ({} extents), ",
            "zero={} MiB ({} extents)"
        ),
        profile.name,
        DISK_SIZE / MIB,
        summary.data_bytes / MIB as u64,
        summary.data_extents,
        summary.hole_bytes / MIB as u64,
        summary.hole_extents,
        summary.zero_bytes / MIB as u64,
        summary.zero_extents,
    );

    let mut group = criterion.benchmark_group(format!("sparse_copy/{}", profile.name,));

    /*
     * iter_custom controls its own iteration count and timing, but
     * these settings still keep the surrounding Criterion benchmark
     * configuration consistent with the rest of the suite.
     */
    group.sample_size(10);

    group.sampling_mode(SamplingMode::Flat);

    group.measurement_time(Duration::from_secs(20));

    group.warm_up_time(Duration::from_secs(3));

    /*
     * Criterion displays logical disk throughput.
     *
     * Example:
     *
     * sparse75:
     *     logical bytes = 256 MiB
     *     actual Data   =  64 MiB
     *
     * Data throughput can be derived from the elapsed time and
     * summary.data_bytes.
     */
    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    group.bench_function(BenchmarkId::new("threaded", THREADED_WORKERS), |bencher| {
        bencher.iter_custom(|iterations| {
            let mut measured = Duration::ZERO;

            for iteration in 0..iterations {
                let destination_path =
                    benchmark_path(&format!("{}-threaded-{}", profile.name, iteration,));

                /*
                 * NOT TIMED:
                 *
                 * Materialize the complete destination with
                 * non-zero contents.
                 */
                materialize_file(&destination_path, DISK_SIZE, 0xff);

                /*
                 * NOT TIMED:
                 *
                 * Open source and destination.
                 */
                let source =
                    RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

                let destination =
                    RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());

                let mover = DataMover::new(threaded_options());

                /*
                 * TIMING STARTS HERE.
                 */
                let started = Instant::now();

                let stats = mover.copy(&source, &destination).unwrap();

                measured += started.elapsed();

                /*
                 * TIMING HAS STOPPED.
                 */

                assert_eq!(stats.bytes_read(), profile.expected_data_bytes as u64,);

                /*
                 * Close descriptors before deleting the
                 * destination.
                 *
                 * NOT TIMED.
                 */
                drop(destination);
                drop(source);

                fs::remove_file(destination_path).unwrap();
            }

            measured
        });
    });

    group.bench_function(
        BenchmarkId::new("io_uring", IO_URING_QUEUE_DEPTH),
        |bencher| {
            bencher.iter_custom(|iterations| {
                let mut measured = Duration::ZERO;

                for iteration in 0..iterations {
                    let destination_path =
                        benchmark_path(&format!("{}-io-uring-{}", profile.name, iteration,));

                    /*
                     * NOT TIMED.
                     */
                    materialize_file(&destination_path, DISK_SIZE, 0xff);

                    /*
                     * NOT TIMED.
                     */
                    let source =
                        RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

                    let destination = RawDisk::new(
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap(),
                    );

                    let mover = DataMover::with_execution_strategy(
                        native_options(),
                        ExecutionStrategy::IoUring(
                            IoUringExecutionOptions::new(IO_URING_QUEUE_DEPTH).unwrap(),
                        ),
                    );

                    /*
                     * TIMING STARTS HERE.
                     */
                    let started = Instant::now();

                    let report = mover.copy_with_report(&source, &destination).unwrap();

                    measured += started.elapsed();

                    /*
                     * TIMING HAS STOPPED.
                     */

                    assert_eq!(report.backend(), ExecutionBackend::IoUring,);

                    assert_eq!(
                        report.stats().bytes_read(),
                        profile.expected_data_bytes as u64,
                    );

                    drop(destination);
                    drop(source);

                    fs::remove_file(destination_path).unwrap();
                }

                measured
            });
        },
    );

    group.finish();

    println!(
        concat!(
            "{} summary: ",
            "logical={} MiB, ",
            "data={} MiB, ",
            "sparse={} MiB, ",
            "data_extents={}, ",
            "hole_extents={}"
        ),
        profile.name,
        DISK_SIZE / MIB,
        profile.expected_data_bytes / MIB,
        (DISK_SIZE - profile.expected_data_bytes) / MIB,
        summary.data_extents,
        summary.hole_extents,
    );

    remove_file(source_path);
}

criterion_group!(benches, benchmark_sparse_copy);

criterion_main!(benches);
