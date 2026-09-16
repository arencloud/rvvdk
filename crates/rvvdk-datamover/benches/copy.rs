mod support;

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_incompressible_file, create_zero_file, ensure_benchmark_directory,
    incompressible_buffer, remove_file,
};

const DISK_SIZE: usize = 256 * MIB;

fn benchmark_dense_copy(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("dense-source");

    let destination_path = benchmark_path("dense-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b02);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("dense_raw_copy");

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for block_size in [64 * 1024, 256 * 1024, MIB, 4 * MIB, 8 * MIB] {
        group.bench_with_input(
            BenchmarkId::from_parameter(block_size),
            &block_size,
            |bencher, &block_size| {
                bencher.iter(|| {
                    let source_device = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

                    let destination_device =
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

                    let source = RawDisk::new(source_device);

                    let destination = RawDisk::new(destination_device);

                    let options = CopyOptions::new(block_size).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

fn benchmark_sparse_copy(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("sparse-source");

    let destination_path = benchmark_path("sparse-destination");

    let mut source_file = File::create(&source_path).unwrap();

    source_file.set_len(DISK_SIZE as u64).unwrap();

    let data_block = incompressible_buffer(MIB, 0x5256_5644_4b05);

    for offset in [16 * MIB, 64 * MIB, 128 * MIB, 192 * MIB] {
        source_file.seek(SeekFrom::Start(offset as u64)).unwrap();

        source_file.write_all(&data_block).unwrap();
    }

    source_file.sync_all().unwrap();

    drop(source_file);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("sparse_raw_copy");

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for block_size in [64 * 1024, 256 * 1024, MIB, 4 * MIB] {
        group.bench_with_input(
            BenchmarkId::from_parameter(block_size),
            &block_size,
            |bencher, &block_size| {
                bencher.iter(|| {
                    let source_device = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

                    let destination_device =
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

                    let source = RawDisk::new(source_device);

                    let destination = RawDisk::new(destination_device);

                    let options = CopyOptions::new(block_size).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

fn benchmark_concurrent_copy(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("concurrent-source");

    let destination_path = benchmark_path("concurrent-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b03);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("concurrent_raw_copy");

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for concurrency in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |bencher, &concurrency| {
                bencher.iter(|| {
                    let source_device = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

                    let destination_device =
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

                    let source = RawDisk::new(source_device);

                    let destination = RawDisk::new(destination_device);

                    let options = CopyOptions::with_concurrency(MIB, 4096, concurrency).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    remove_file(source_path);

    remove_file(destination_path);
}

fn benchmark_queue_capacity(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("queue-source");

    let destination_path = benchmark_path("queue-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b04);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("queue_capacity");

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for queue_capacity in [1, 2, 4, 8, 16, 32, 64, 128] {
        group.bench_with_input(
            BenchmarkId::from_parameter(queue_capacity),
            &queue_capacity,
            |bencher, &queue_capacity| {
                bencher.iter(|| {
                    let source_device = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

                    let destination_device =
                        LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

                    let source = RawDisk::new(source_device);

                    let destination = RawDisk::new(destination_device);

                    let options =
                        CopyOptions::with_execution(MIB, 4096, 2, queue_capacity).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
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
    benchmark_dense_copy,
    benchmark_sparse_copy,
    benchmark_concurrent_copy,
    benchmark_queue_capacity
);

criterion_main!(benches);
