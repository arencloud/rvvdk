mod support;

use std::os::fd::AsRawFd;
use std::time::Duration;

use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};

use rvvdk_core::RawDisk;

use rvvdk_datamover::{CopyOptions, DataMover};

use rvvdk_datamover::io_uring::copy_file_range;

use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_incompressible_file, create_zero_file, ensure_benchmark_directory,
    remove_file,
};

const DISK_SIZE: usize = 2 * 1024 * MIB;

const BLOCK_SIZE: usize = MIB;

const ALIGNMENT: usize = 4096;

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);

    group.sampling_mode(SamplingMode::Flat);

    group.measurement_time(Duration::from_secs(30));

    group.warm_up_time(Duration::from_secs(3));

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));
}

fn benchmark_threaded_direct(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("threaded-direct-source");

    let destination_path = benchmark_path("threaded-direct-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b18);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("threaded_direct_raw_copy");

    configure_group(&mut group);

    for workers in [1_usize, 2, 4] {
        group.bench_with_input(
            BenchmarkId::from_parameter(workers),
            &workers,
            |bencher, &workers| {
                bencher.iter(|| {
                    let source = RawDisk::new(
                        LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap(),
                    );

                    let destination = RawDisk::new(
                        LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap(),
                    );

                    let options =
                        CopyOptions::with_execution(BLOCK_SIZE, ALIGNMENT, workers, 4).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

fn benchmark_io_uring_direct(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("io-uring-direct-source");

    let destination_path = benchmark_path("io-uring-direct-destination");

    /*
     * Source data is generated through the buffered setup path before
     * the benchmark starts.
     *
     * create_incompressible_file() synchronizes the file before
     * returning, so the benchmark starts with fully materialized
     * deterministic source data.
     */
    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b18);

    /*
     * The destination is created and sized before it is reopened
     * through LocalFileBlockDevice with O_DIRECT.
     */
    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("io_uring_direct_raw_copy");

    configure_group(&mut group);

    for queue_depth in [1_u32, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(queue_depth),
            &queue_depth,
            |bencher, &queue_depth| {
                bencher.iter(|| {
                    /*
                     * Both descriptors are opened through the same
                     * rvvdk local backend used by the threaded direct
                     * benchmark.
                     *
                     * The primary descriptor therefore carries
                     * O_DIRECT.
                     */
                    let source = LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap();

                    let destination =
                        LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap();

                    let stats = copy_file_range(
                        source.as_raw_fd(),
                        destination.as_raw_fd(),
                        0,
                        DISK_SIZE as u64,
                        BLOCK_SIZE,
                        queue_depth,
                        ALIGNMENT,
                    )
                    .unwrap();

                    assert_eq!(stats.bytes_read(), DISK_SIZE as u64,);

                    assert_eq!(stats.bytes_written(), DISK_SIZE as u64,);

                    assert_eq!(stats.blocks_completed(), (DISK_SIZE / BLOCK_SIZE) as u64,);
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

criterion_group!(
    benches,
    benchmark_threaded_direct,
    benchmark_io_uring_direct
);

criterion_main!(benches);
