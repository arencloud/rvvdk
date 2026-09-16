mod support;

use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_incompressible_file, create_zero_file, ensure_benchmark_directory,
    remove_file,
};

const DISK_SIZE: usize = 256 * MIB;

fn benchmark_direct_copy(criterion: &mut Criterion) {
    ensure_benchmark_directory();

    let source_path = benchmark_path("direct-source");

    let destination_path = benchmark_path("direct-destination");

    create_incompressible_file(&source_path, DISK_SIZE, 0x5256_5644_4b01);

    create_zero_file(&destination_path, DISK_SIZE);

    let mut group = criterion.benchmark_group("direct_raw_copy");

    group.sample_size(20);

    group.sampling_mode(SamplingMode::Flat);

    group.measurement_time(std::time::Duration::from_secs(20));

    group.warm_up_time(std::time::Duration::from_secs(3));

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

    remove_file(source_path);

    remove_file(destination_path);
}

criterion_group!(benches, benchmark_direct_copy);

criterion_main!(benches);
