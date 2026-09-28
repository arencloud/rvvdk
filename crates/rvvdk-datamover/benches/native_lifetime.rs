mod support;

use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{BlockDevice, RawDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions};
use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_incompressible_file, ensure_benchmark_directory,
    incompressible_buffer, remove_file,
};

const SIZE: usize = 16 * MIB;

fn fd_count() -> usize {
    std::fs::read_dir("/proc/self/fd").unwrap().count()
}

fn native_lifetime(criterion: &mut Criterion) {
    ensure_benchmark_directory();
    let source_path = benchmark_path("lifetime-source");
    let destination_path = benchmark_path("lifetime-destination");
    let expected = incompressible_buffer(SIZE, 0x5256_5644_4b33);
    let reset = incompressible_buffer(SIZE, 0x5256_5644_4b34);
    create_incompressible_file(&source_path, SIZE, 0x5256_5644_4b33);
    std::fs::write(&destination_path, &reset).unwrap();
    let reset_device = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();
    let mut actual = vec![0; SIZE];

    for direct in [false, true] {
        let source = if direct {
            LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap()
        } else {
            LocalFileBlockDevice::open_read_only(&source_path).unwrap()
        };
        let destination = if direct {
            LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap()
        } else {
            LocalFileBlockDevice::open_read_write(&destination_path).unwrap()
        };
        let source = RawDisk::new(source);
        let destination = RawDisk::new(destination);
        let profile = if direct { "direct" } else { "buffered" };
        let mut group = criterion.benchmark_group(format!("native_lifetime/{profile}"));
        group.sampling_mode(SamplingMode::Flat);
        group.throughput(Throughput::Bytes(SIZE as u64));
        for (queue, block) in [
            (1, 64 * 1024),
            (8, 64 * 1024),
            (8, 4096),
            (8, MIB),
            (0, 64 * 1024),
        ] {
            let native = queue != 0;
            let mover = if native {
                DataMover::with_execution_strategy(
                    CopyOptions::new(block).unwrap(),
                    ExecutionStrategy::IoUring(IoUringExecutionOptions::new(queue).unwrap()),
                )
            } else {
                DataMover::new(CopyOptions::with_concurrency(block, 4096, 4).unwrap())
            };
            let name = if native {
                format!("q{queue}_b{block}")
            } else {
                "threaded_control".into()
            };
            group.bench_function(name, |bencher| {
                bencher.iter_custom(|iterations| {
                    let before = fd_count();
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        reset_device.write_all_at(0, &reset).unwrap();
                        reset_device.flush().unwrap();
                        let started = Instant::now();
                        let (read, written) = if native {
                            let stats = mover
                                .copy_native(source.device(), destination.device(), 0, SIZE as u64)
                                .unwrap();
                            // copy_native does not flush; match the threaded boundary.
                            destination.device().flush().unwrap();
                            (stats.bytes_read(), stats.bytes_written())
                        } else {
                            let stats = mover.copy(&source, &destination).unwrap();
                            (stats.bytes_read(), stats.bytes_written())
                        };
                        elapsed += started.elapsed();
                        assert_eq!((read, written), (SIZE as u64, SIZE as u64));
                        reset_device.read_exact_at(0, &mut actual).unwrap();
                        assert_eq!(actual, expected);
                        assert_eq!(fd_count(), before, "copy leaked file descriptors");
                    }
                    elapsed
                });
            });
        }
        group.finish();
    }
    drop(reset_device);
    remove_file(source_path);
    remove_file(destination_path);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = native_lifetime
}
criterion_main!(benches);
