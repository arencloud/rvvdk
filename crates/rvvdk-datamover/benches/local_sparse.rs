mod support;
#[path = "../tests/support/test_block_device.rs"]
mod test_block_device;
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{BlockDevice, Capabilities, Extent, ExtentKind, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions};
use rvvdk_local::LocalFileBlockDevice;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, Instant};
use support::{
    MIB, benchmark_path, ensure_benchmark_directory, incompressible_buffer, remove_file,
};
use test_block_device::TestExtentBlockDevice;

fn local_sparse(c: &mut Criterion) {
    ensure_benchmark_directory();
    let src = benchmark_path("local-sparse-src");
    let dst = benchmark_path("local-sparse-dst");
    let size = 32 * MIB;
    let extents: Vec<_> = [
        (0, 4, ExtentKind::Data),
        (4, 4, ExtentKind::Zero),
        (8, 20, ExtentKind::Hole),
        (28, 4, ExtentKind::Data),
    ]
    .into_iter()
    .map(|(o, l, k)| Extent::new((o * MIB) as u64, (l * MIB) as u64, k).unwrap())
    .collect();
    let mut expected = incompressible_buffer(size, 0x5256523232000011);
    expected[4 * MIB..28 * MIB].fill(0);
    let device = TestExtentBlockDevice::create(
        &src,
        size as u64,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | Capabilities::EXTENTS,
        extents.clone(),
    )
    .unwrap();
    device.write_all_at(0, &expected).unwrap();
    device.flush().unwrap();
    drop(device);
    let source = RawDisk::new(
        TestExtentBlockDevice::open_read_only(
            &src,
            size as u64,
            Capabilities::READ | Capabilities::EXTENTS,
            extents,
        )
        .unwrap(),
    );
    let reset = incompressible_buffer(size, 0x5256523232000022);
    std::fs::write(&dst, &reset).unwrap();
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&dst).unwrap());
    let mut actual = vec![0; size];
    let caps = destination.capabilities();
    let zeroed = if caps.contains(Capabilities::WRITE_ZERO) {
        4 * MIB
    } else {
        0
    };
    let discarded = if caps.contains(Capabilities::DISCARD | Capabilities::DISCARD_ZEROES) {
        20 * MIB
    } else {
        0
    };
    println!(
        "local_sparse: logical={size}, data={}, zero={}, hole={}, block=65536, flush=included, expected_zeroed={zeroed}, expected_discarded={discarded}",
        8 * MIB,
        4 * MIB,
        20 * MIB
    );
    let mut group = c.benchmark_group("local_sparse");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(size as u64));
    for (name, workers, strategy) in [
        ("threaded1", 1, ExecutionStrategy::Threaded),
        ("threaded4", 4, ExecutionStrategy::Threaded),
        (
            "native",
            1,
            ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
        ),
    ] {
        let mover = DataMover::with_execution_strategy(
            CopyOptions::with_execution(65536, 4096, workers, 4).unwrap(),
            strategy,
        );
        let plan = mover
            .plan_raw_with_destination(&source, &destination)
            .unwrap();
        group.bench_function(name, |b| {
            b.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    destination.write_all_at(0, &reset).unwrap();
                    destination.flush().unwrap();
                    let start = Instant::now();
                    let report = mover
                        .execute_raw_plan(&plan, &source, &destination)
                        .unwrap();
                    elapsed += start.elapsed();
                    let stats = report.stats();
                    assert_eq!(stats.bytes_read(), (8 * MIB) as u64);
                    assert_eq!(stats.bytes_written(), (size - zeroed - discarded) as u64);
                    assert_eq!(stats.bytes_zeroed(), zeroed as u64);
                    assert_eq!(stats.bytes_discarded(), discarded as u64);
                    destination.read_exact_at(0, &mut actual).unwrap();
                    assert_eq!(actual, expected);
                }
                elapsed
            })
        });
        println!(
            "{name}: allocated_bytes_after_copy={}",
            std::fs::metadata(&dst).unwrap().blocks() * 512
        );
    }
    group.finish();
    drop(destination);
    drop(source);
    remove_file(src);
    remove_file(dst);
}
criterion_group! { name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=local_sparse }
criterion_main!(benches);
