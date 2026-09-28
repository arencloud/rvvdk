mod support;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{
    Capabilities, DiskGeometry, Error, Extent, ExtentKind, MemoryBlockDevice, RawDisk, Result,
    VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

use support::{
    MIB, benchmark_path, create_incompressible_file, ensure_benchmark_directory,
    incompressible_buffer, remove_file,
};

// Bit 8 is DISCARD_ZEROES. Use its literal value so this identical harness
// can also benchmark the pre-R2.1 baseline without changing its core API.
const SIZE: usize = 16 * MIB;
const BLOCK: usize = 64 * 1024;

fn benchmark_pair<S: VirtualDisk, D: VirtualDisk>(
    criterion: &mut Criterion,
    profile: &str,
    source: &S,
    destination: &D,
    payload: &[u8],
) {
    let mut group = criterion.benchmark_group(format!("scheduler_copy/{profile}"));
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(SIZE as u64));
    let reset = incompressible_buffer(SIZE, 0x5256_5644_4b32);
    let mut actual = vec![0; SIZE];
    let mut configurations = vec![(2, 1, BLOCK), (2, 16, BLOCK), (4, 1, BLOCK), (4, 16, BLOCK)];
    if profile == "memory" {
        configurations.push((4, 16, 4096));
    }
    for (workers, queue, block) in configurations {
        let mover =
            DataMover::new(CopyOptions::with_execution(block, 4096, workers, queue).unwrap());
        group.bench_function(format!("w{workers}_q{queue}_b{block}"), |bencher| {
            bencher.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    destination.write_all_at(0, &reset).unwrap();
                    destination.flush().unwrap();
                    let started = Instant::now();
                    let stats = mover.copy(source, destination).unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(stats.bytes_read(), SIZE as u64);
                    assert_eq!(stats.bytes_written(), SIZE as u64);
                    assert_eq!(stats.blocks_copied(), (SIZE / block) as u64);
                    destination.read_exact_at(0, &mut actual).unwrap();
                    assert_eq!(actual, payload);
                }
                elapsed
            });
        });
    }
    group.finish();
}

fn scheduler_copy(criterion: &mut Criterion) {
    let payload = incompressible_buffer(SIZE, 0x5256_5644_4b31);
    let source = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    let destination = RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap());
    source.write_all_at(0, &payload).unwrap();
    benchmark_pair(criterion, "memory", &source, &destination, &payload);

    ensure_benchmark_directory();
    let source_path = benchmark_path("scheduler-source");
    let destination_path = benchmark_path("scheduler-destination");
    create_incompressible_file(&source_path, SIZE, 0x5256_5644_4b31);
    std::fs::write(&destination_path, &payload).unwrap();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&source_path).unwrap());
    let destination =
        RawDisk::new(LocalFileBlockDevice::open_read_write(&destination_path).unwrap());
    destination.flush().unwrap();
    benchmark_pair(criterion, "buffered_file", &source, &destination, &payload);
    drop((source, destination));
    remove_file(source_path);
    remove_file(destination_path);
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Read,
    Write,
    Zero,
    Discard,
}

struct FaultDisk {
    operation: Operation,
    extent: ExtentKind,
    first_failure: Mutex<Option<Instant>>,
}

impl FaultDisk {
    fn new(operation: Operation, extent: ExtentKind) -> Self {
        Self {
            operation,
            extent,
            first_failure: Mutex::new(None),
        }
    }

    fn check(&self, operation: Operation) -> Result<()> {
        if self.operation == operation {
            self.first_failure
                .lock()
                .unwrap()
                .get_or_insert_with(Instant::now);
            return Err(Error::Io(std::io::Error::other(
                "scheduler benchmark fault",
            )));
        }
        Ok(())
    }
}

impl VirtualDisk for FaultDisk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(SIZE as u64, 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
            | Capabilities::WRITE
            | Capabilities::WRITE_ZERO
            | Capabilities::DISCARD
            | Capabilities::from_bits_retain(1 << 8)
    }
    fn read_at(&self, _: u64, buffer: &mut [u8]) -> Result<usize> {
        self.check(Operation::Read)?;
        buffer.fill(0x5a);
        Ok(buffer.len())
    }
    fn write_at(&self, _: u64, buffer: &[u8]) -> Result<usize> {
        self.check(Operation::Write)?;
        Ok(buffer.len())
    }
    fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
        self.check(Operation::Zero)
    }
    fn discard(&self, _: u64, _: u64) -> Result<()> {
        self.check(Operation::Discard)
    }
    fn flush(&self) -> Result<()> {
        panic!("failed copy must not flush")
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        Ok(vec![Extent::new(offset, length, self.extent)?])
    }
}

fn scheduler_failure(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("scheduler_failure");
    group.sampling_mode(SamplingMode::Flat);
    let mover = DataMover::new(CopyOptions::with_execution(4096, 4096, 4, 1).unwrap());
    for (name, operation, extent) in [
        ("read", Operation::Read, ExtentKind::Data),
        ("write", Operation::Write, ExtentKind::Data),
        ("zero", Operation::Zero, ExtentKind::Zero),
        ("discard", Operation::Discard, ExtentKind::Hole),
    ] {
        let source = FaultDisk::new(
            if operation == Operation::Read {
                operation
            } else {
                Operation::None
            },
            extent,
        );
        let destination = FaultDisk::new(
            if operation == Operation::Read {
                Operation::None
            } else {
                operation
            },
            extent,
        );
        let failing = if operation == Operation::Read {
            &source
        } else {
            &destination
        };
        for metric in ["call", "after_fault"] {
            group.bench_function(format!("{name}/{metric}"), |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        *failing.first_failure.lock().unwrap() = None;
                        let started = Instant::now();
                        let error = mover.copy(&source, &destination).unwrap_err();
                        let finished = Instant::now();
                        let first_failure = failing.first_failure.lock().unwrap().unwrap();
                        elapsed += finished.duration_since(if metric == "call" { started } else { first_failure });
                        assert!(matches!(error, Error::Io(ref cause) if cause.to_string() == "scheduler benchmark fault"));
                    }
                    elapsed
                });
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = scheduler_copy, scheduler_failure
}
criterion_main!(benches);
