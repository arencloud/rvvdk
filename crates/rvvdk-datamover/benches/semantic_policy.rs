use std::time::{Duration, Instant};

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{
    Capabilities, DiskGeometry, Extent, ExtentKind, MemoryBlockDevice, RawDisk, Result, VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover, NoopProgressObserver};

const SIZE: usize = 1024 * 1024;
const BLOCK: usize = 64 * 1024;

struct Source {
    disk: RawDisk<MemoryBlockDevice>,
    extents: Vec<Extent>,
}
impl VirtualDisk for Source {
    fn geometry(&self) -> DiskGeometry {
        self.disk.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.disk.capabilities()
    }
    fn read_at(&self, offset: u64, bytes: &mut [u8]) -> Result<usize> {
        self.disk.read_at(offset, bytes)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        unreachable!()
    }
    fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
        unreachable!()
    }
    fn discard(&self, _: u64, _: u64) -> Result<()> {
        unreachable!()
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
    fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
        Ok(self.extents.clone())
    }
}

fn semantic_policy(criterion: &mut Criterion) {
    for (profile, kinds, accelerated) in [
        ("zero_fallback", vec![ExtentKind::Zero], false),
        ("hole_fallback", vec![ExtentKind::Hole], false),
        (
            "mixed_accelerated",
            vec![
                ExtentKind::Data,
                ExtentKind::Zero,
                ExtentKind::Hole,
                ExtentKind::Data,
            ],
            true,
        ),
    ] {
        let extents: Vec<_> = (0..16)
            .map(|i| Extent::new((i * BLOCK) as u64, BLOCK as u64, kinds[i % kinds.len()]).unwrap())
            .collect();
        let mut expected = vec![0; SIZE];
        let mut data_bytes = 0;
        for extent in &extents {
            if extent.kind() == ExtentKind::Data {
                expected[extent.offset() as usize..extent.end() as usize].fill(0x5a);
                data_bytes += extent.length();
            }
        }
        let source = Source {
            disk: RawDisk::new(MemoryBlockDevice::new(SIZE).unwrap()),
            extents,
        };
        source.disk.write_all_at(0, &expected).unwrap();
        let capabilities = Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH;
        let destination = RawDisk::new(
            MemoryBlockDevice::with_capabilities(
                SIZE,
                if accelerated {
                    capabilities | Capabilities::WRITE_ZERO | Capabilities::DISCARD
                } else {
                    capabilities
                },
            )
            .unwrap(),
        );
        let reset = vec![0xa5; SIZE];
        let mut actual = vec![0; SIZE];
        let mut group = criterion.benchmark_group(format!("semantic_policy/{profile}"));
        group.sampling_mode(SamplingMode::Flat);
        group.throughput(Throughput::Bytes(SIZE as u64));
        for workers in [1, 4] {
            let mover =
                DataMover::new(CopyOptions::with_concurrency(BLOCK, 4096, workers).unwrap());
            let plan = mover.plan_with_destination(&source, &destination).unwrap();
            group.bench_function(format!("workers{workers}"), |b| {
                b.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        destination.write_all_at(0, &reset).unwrap();
                        let start = Instant::now();
                        let report = if workers == 1 {
                            mover.execute_plan_with_observer(
                                &plan,
                                &source,
                                &destination,
                                &NoopProgressObserver,
                            )
                        } else {
                            mover.execute_plan(&plan, &source, &destination)
                        }
                        .unwrap();
                        elapsed += start.elapsed();
                        let stats = report.stats();
                        assert_eq!(stats.bytes_read(), data_bytes);
                        assert_eq!(
                            stats.bytes_written() + stats.bytes_zeroed() + stats.bytes_discarded(),
                            SIZE as u64
                        );
                        assert_eq!(stats.extents_processed(), 16);
                        destination.read_exact_at(0, &mut actual).unwrap();
                        assert_eq!(actual, expected);
                    }
                    elapsed
                })
            });
        }
        group.finish();
    }
}
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = semantic_policy
}
criterion_main!(benches);
