mod support;

use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::time::Duration;

use rvvdk_core::RawDisk;

use rvvdk_datamover::{CopyOptions, DataMover};

use rvvdk_local::LocalFileBlockDevice;

use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};

use rvvdk_datamover::io_uring::copy_file_range;

use support::{
    MIB, benchmark_path, create_incompressible_file, create_zero_file, ensure_benchmark_directory,
    remove_file,
};

const DISK_SIZE: usize = 256 * MIB;

const BLOCK_SIZE: usize = MIB;

const ALIGNMENT: usize = 4096;

fn benchmark_io_uring(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("io-uring-source");

    let destination_path = benchmark_path("io-uring-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b17);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("io_uring_raw_copy");

    group.sample_size(20);

    group.sampling_mode(SamplingMode::Flat);

    group.measurement_time(Duration::from_secs(20));

    group.warm_up_time(Duration::from_secs(3));

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for queue_depth in [1_u32, 2, 4, 8, 16, 32] {
        group.bench_with_input(
            BenchmarkId::from_parameter(queue_depth),
            &queue_depth,
            |bencher, &queue_depth| {
                bencher.iter(|| {
                    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

                    let destination = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&destination_path)
                        .unwrap();

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
                    destination.sync_data().unwrap();

                    assert_eq!(stats.bytes_written(), DISK_SIZE as u64,);
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

fn benchmark_threaded(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("threaded-source");

    let destination_path = benchmark_path("threaded-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b17);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("threaded_raw_copy");

    group.sample_size(20);

    group.sampling_mode(SamplingMode::Flat);

    group.measurement_time(Duration::from_secs(20));

    group.warm_up_time(Duration::from_secs(3));

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for workers in [1_usize, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(workers),
            &workers,
            |bencher, &workers| {
                bencher.iter(|| {
                    let source =
                        RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());

                    let destination = RawDisk::new(
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap(),
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

criterion_group!(benches, benchmark_threaded, benchmark_io_uring);

criterion_main!(benches);
