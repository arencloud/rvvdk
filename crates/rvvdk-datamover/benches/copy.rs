use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use std::io::{Seek, SeekFrom, Write};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

const MIB: usize = 1024 * 1024;
const DISK_SIZE: usize = 256 * MIB;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-bench-{name}-{unique}.img"))
}

fn benchmark_dense_copy(criterion: &mut Criterion) {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let source_data = vec![0x5a_u8; DISK_SIZE];

    fs::write(&source_path, &source_data).unwrap();

    fs::write(&destination_path, vec![0_u8; DISK_SIZE]).unwrap();

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

                    let mover = DataMover::new(CopyOptions::new(block_size).unwrap());

                    mover.copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

fn benchmark_sparse_copy(criterion: &mut Criterion) {
    let source_path = temporary_path("sparse-source");

    let destination_path = temporary_path("sparse-destination");

    let mut source_file = fs::File::create(&source_path).unwrap();

    source_file.set_len(DISK_SIZE as u64).unwrap();

    for offset in [16 * MIB, 64 * MIB, 128 * MIB, 192 * MIB] {
        source_file.seek(SeekFrom::Start(offset as u64)).unwrap();

        source_file.write_all(&vec![0xa5_u8; MIB]).unwrap();
    }

    source_file.sync_all().unwrap();

    drop(source_file);

    fs::write(&destination_path, vec![0_u8; DISK_SIZE]).unwrap();

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

                    let mover = DataMover::new(CopyOptions::new(block_size).unwrap());

                    mover.copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

criterion_group!(benches, benchmark_dense_copy, benchmark_sparse_copy);

criterion_main!(benches);
