use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

    std::env::temp_dir().join(format!("rvvdk-direct-bench-{name}-{unique}.img"))
}

fn benchmark_direct_copy(criterion: &mut Criterion) {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    fs::write(&source_path, vec![0x5a_u8; DISK_SIZE]).unwrap();

    fs::File::open(&source_path).unwrap().sync_all().unwrap();

    fs::write(&destination_path, vec![0_u8; DISK_SIZE]).unwrap();

    fs::OpenOptions::new()
        .write(true)
        .open(&destination_path)
        .unwrap()
        .sync_all()
        .unwrap();

    let mut group = criterion.benchmark_group("direct_raw_copy");

    group.throughput(Throughput::Bytes(DISK_SIZE as u64));

    for concurrency in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |bencher, &concurrency| {
                bencher.iter(|| {
                    let source_device =
                        LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap();

                    let destination_device =
                        LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap();

                    let source = RawDisk::new(source_device);

                    let destination = RawDisk::new(destination_device);

                    let options = CopyOptions::with_execution(MIB, 4096, concurrency, 4).unwrap();

                    DataMover::new(options).copy(&source, &destination).unwrap();
                });
            },
        );
    }

    group.finish();

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

criterion_group!(benches, benchmark_direct_copy);

criterion_main!(benches);
