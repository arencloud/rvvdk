mod support;

use std::fs::File;
use std::hint::black_box;
use std::os::fd::AsFd;
use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_core::{Capabilities, Extent, ExtentKind, MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::IoUringExecutionOptions;
use rvvdk_datamover::io_uring::{
    NativeExtentPlan, copy_extent_plan, copy_extent_plan_with_destination,
    copy_file_range_with_options,
};
use support::MIB;

fn native_validation(criterion: &mut Criterion) {
    let fd = File::open("/dev/null").unwrap();
    let options = IoUringExecutionOptions::new(8).unwrap();
    let empty = NativeExtentPlan::new(vec![], 0).unwrap();
    support::ensure_benchmark_directory();
    let source_path = support::benchmark_path("validation-source");
    let destination_path = support::benchmark_path("validation-destination");
    support::create_incompressible_file(&source_path, MIB, 0x0052_5604);
    let expected = support::incompressible_buffer(MIB, 0x0052_5604);
    let reset = support::incompressible_buffer(MIB, 0x0052_5605);
    std::fs::write(&destination_path, &reset).unwrap();
    let source = File::open(&source_path).unwrap();
    let destination = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();
    let mut group = criterion.benchmark_group("native_validation");
    group.sampling_mode(SamplingMode::Flat);
    group.bench_function("empty_range", |b| {
        b.iter(|| {
            black_box(
                copy_file_range_with_options(
                    fd.as_fd(),
                    fd.as_fd(),
                    black_box(0),
                    black_box(0),
                    black_box(65536),
                    black_box(4096),
                    black_box(options),
                )
                .unwrap(),
            );
        })
    });
    group.bench_function("empty_plan", |b| {
        b.iter(|| {
            black_box(
                copy_extent_plan(
                    fd.as_fd(),
                    fd.as_fd(),
                    black_box(&empty),
                    black_box(65536),
                    black_box(4096),
                    black_box(options),
                )
                .unwrap(),
            );
        })
    });
    let unsupported = NativeExtentPlan::new(
        vec![Extent::new(0, MIB as u64, ExtentKind::Zero).unwrap()],
        MIB as u64,
    )
    .unwrap();
    group.bench_function("unsupported_first_extent", |b| {
        b.iter(|| {
            black_box(
                copy_extent_plan(
                    fd.as_fd(),
                    fd.as_fd(),
                    black_box(&unsupported),
                    black_box(65536),
                    black_box(4096),
                    black_box(options),
                )
                .unwrap_err(),
            );
        })
    });
    // Each iteration actually zeros nonzero bytes; setup/readback are untimed.
    let disk = RawDisk::new(
        MemoryBlockDevice::with_capabilities(
            MIB,
            Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        )
        .unwrap(),
    );
    let reset = vec![0xff; MIB];
    let mut actual = vec![0; MIB];
    let plan = NativeExtentPlan::new(
        (0..16)
            .map(|i| Extent::new(i * 65536, 65536, ExtentKind::Zero).unwrap())
            .collect(),
        MIB as u64,
    )
    .unwrap();
    group.bench_function("zero_fallback_1mib_16extents", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                disk.write_all_at(0, &reset).unwrap();
                let start = Instant::now();
                let stats = copy_extent_plan_with_destination(
                    source.as_fd(),
                    fd.as_fd(),
                    &disk,
                    &plan,
                    65536,
                    4096,
                    options,
                )
                .unwrap();
                disk.flush().unwrap();
                elapsed += start.elapsed();
                assert_eq!(stats.bytes_written(), MIB as u64);
                assert_eq!(stats.extents_processed(), 16);
                disk.read_exact_at(0, &mut actual).unwrap();
                assert!(actual.iter().all(|byte| *byte == 0));
            }
            elapsed
        })
    });
    let plan = NativeExtentPlan::new(
        (0..16)
            .map(|i| Extent::new(i * 65536, 65536, ExtentKind::Data).unwrap())
            .collect(),
        MIB as u64,
    )
    .unwrap();
    group.bench_function("data_1mib_16extents", |b| {
        b.iter_custom(|iterations| {
            use std::os::unix::fs::FileExt;
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                destination.write_all_at(&reset, 0).unwrap();
                destination.sync_all().unwrap();
                let start = Instant::now();
                let stats = copy_extent_plan(
                    source.as_fd(),
                    destination.as_fd(),
                    &plan,
                    65536,
                    4096,
                    options,
                )
                .unwrap();
                destination.sync_all().unwrap();
                elapsed += start.elapsed();
                assert_eq!(stats.bytes_written(), MIB as u64);
                assert_eq!(stats.extents_processed(), 16);
                destination.read_exact_at(&mut actual, 0).unwrap();
                assert_eq!(actual, expected);
            }
            elapsed
        })
    });
    drop(source);
    drop(destination);
    support::remove_file(source_path);
    support::remove_file(destination_path);
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = native_validation
}
criterion_main!(benches);
