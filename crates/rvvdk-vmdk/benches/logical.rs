use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_core::{BlockDevice, MemoryBlockDevice, VirtualDisk};
use rvvdk_vmdk::{
    BackingError, BackingResolver, Descriptor, ResolutionLimits, ResolvedDescriptor, VmdkDisk,
};
use std::{hint::black_box, sync::Arc};
struct Source(Arc<dyn BlockDevice>);
impl BackingResolver for Source {
    fn resolve(&self, _: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        Ok(self.0.clone())
    }
}
fn logical(c: &mut Criterion) {
    let source = MemoryBlockDevice::new(8 * 1024 * 1024).unwrap();
    source.write_all_at(0, &vec![7; 8 * 1024 * 1024]).unwrap();
    let source = Source(Arc::new(source));
    let header = "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\n";
    let mut group = c.benchmark_group("logical");
    group.sampling_mode(SamplingMode::Flat);
    for (name, records, offset, length) in [
        ("flat_64k", "RW 16384 FLAT \"x\" 0\n".to_string(), 0, 65536),
        (
            "cross_64k",
            "RW 8 FLAT \"x\" 1\nRW 8 ZERO\n".repeat(512),
            257,
            65536,
        ),
        (
            "last_4k_1024",
            "RW 8 FLAT \"x\" 0\n".repeat(1024),
            4194304 - 4096,
            4096,
        ),
    ] {
        let text = format!("{header}{records}");
        let parsed = Descriptor::parse(text.as_bytes()).unwrap();
        let disk = VmdkDisk::new(
            ResolvedDescriptor::resolve(&parsed, &source, ResolutionLimits::default()).unwrap(),
        )
        .unwrap();
        let mut buffer = vec![0; length];
        disk.read_exact_at(offset, &mut buffer).unwrap();
        for (i, byte) in buffer.iter().enumerate() {
            assert_eq!(
                *byte,
                if name == "cross_64k" && (offset + i as u64) / 4096 % 2 == 1 {
                    0
                } else {
                    7
                }
            );
        }
        group.bench_function(name, |b| {
            b.iter(|| {
                disk.read_exact_at(black_box(offset), black_box(&mut buffer))
                    .unwrap();
                black_box(&buffer);
            })
        });
    }
    group.finish();
}

#[cfg(target_os = "linux")]
fn local_copy(c: &mut Criterion) {
    use rvvdk_core::RawDisk;
    use rvvdk_datamover::{CopyOptions, DataMover};
    use rvvdk_local::LocalFileBlockDevice;
    use rvvdk_vmdk::{Limits, LocalResolver};
    use std::{
        fs::{self, File},
        path::PathBuf,
        time::{Duration, Instant},
    };
    let root = std::env::var_os("RVVDK_BENCH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = root.join(format!("rvvdk-logical-copy-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data"), vec![7; 1024 * 1024]).unwrap();
    let header = "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\n";
    let mut group = c.benchmark_group("logical_copy");
    group.sampling_mode(SamplingMode::Flat);
    for (name, records) in [
        ("flat_1m", "RW 2048 FLAT \"data\" 0\n".to_string()),
        ("mixed_1m", "RW 8 FLAT \"data\" 0\nRW 8 ZERO\n".repeat(128)),
    ] {
        fs::write(path.join("disk.vmdk"), format!("{header}{records}")).unwrap();
        let (text, resolver) =
            LocalResolver::open_descriptor(path.join("disk.vmdk"), Limits::default()).unwrap();
        let source = VmdkDisk::new(
            ResolvedDescriptor::resolve(
                &text.parse().unwrap(),
                &resolver,
                ResolutionLimits::default(),
            )
            .unwrap(),
        )
        .unwrap();
        File::create(path.join("output"))
            .unwrap()
            .set_len(1024 * 1024)
            .unwrap();
        let target =
            RawDisk::new(LocalFileBlockDevice::open_read_write(path.join("output")).unwrap());
        let mover = DataMover::new(CopyOptions::with_concurrency(65536, 4096, 4).unwrap());
        let dirty = vec![0xa5; 1024 * 1024];
        let mut actual = vec![0; 1024 * 1024];
        group.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iters {
                    target.write_all_at(0, &dirty).unwrap();
                    target.flush().unwrap();
                    let start = Instant::now();
                    black_box(mover.copy(&source, &target).unwrap());
                    elapsed += start.elapsed();
                    target.read_exact_at(0, &mut actual).unwrap();
                    for (i, byte) in actual.iter().enumerate() {
                        assert_eq!(
                            *byte,
                            if name == "mixed_1m" && i / 4096 % 2 == 1 {
                                0
                            } else {
                                7
                            }
                        );
                    }
                }
                elapsed
            })
        });
    }
    group.finish();
    fs::remove_dir_all(path).unwrap();
}
#[cfg(not(target_os = "linux"))]
fn local_copy(_: &mut Criterion) {}

criterion_group!(benches, logical, local_copy);
criterion_main!(benches);
