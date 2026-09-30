#[path = "../tests/support/sparse_metadata.rs"]
mod f;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_core::{MemoryBlockDevice, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, Verifier};
use rvvdk_vmdk::{SparseDescriptor, SparseDisk, SparseDiskLimits};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
fn logical(c: &mut Criterion) {
    let text = f::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"");
    let parsed = SparseDescriptor::parse(text.as_bytes()).unwrap();
    let bytes = f::bytes(Some(&text));
    let r = f::Resolver(f::device(&bytes));
    let disk = SparseDisk::load(&parsed, &r, SparseDiskLimits::default()).unwrap();
    let mut fragment = bytes.clone();
    for t in [22, 27] {
        f::put32(&mut fragment, t * 512, 256);
        f::put32(&mut fragment, t * 512 + 4, 128);
    }
    let fragmented = SparseDisk::load(
        &parsed,
        &f::Resolver(f::device(&fragment)),
        SparseDiskLimits::default(),
    )
    .unwrap();
    let split = f::descriptor(
        "twoGbMaxExtentSparse",
        "RW 2048 SPARSE \"disk.vmdk\"\nRW 2048 SPARSE \"disk.vmdk\"",
    );
    let sd = SparseDescriptor::parse(split.as_bytes()).unwrap();
    let sr = f::Resolver(f::device(&f::bytes(None)));
    let split = SparseDisk::load(&sd, &sr, SparseDiskLimits::default()).unwrap();
    let mut group = c.benchmark_group("sparse_disk");
    group.sampling_mode(SamplingMode::Flat);
    for (name, source, offset, length) in [
        ("read_contiguous_128k", &disk, 0, 131072),
        ("read_fragmented_128k", &fragmented, 0, 131072),
        ("read_cross_zero_64k", &disk, 98304, 65536),
        ("read_split_64k", &split, 1048576 - 32768, 65536),
    ] {
        let mut buffer = vec![0; length];
        source.read_exact_at(offset, &mut buffer).unwrap();
        for (i, byte) in buffer.iter().enumerate() {
            let local = (offset + i as u64) % 1048576;
            let expected = if local >= 131072 {
                0
            } else if (local < 65536) != (name == "read_fragmented_128k") {
                0x5a
            } else {
                0xa5
            };
            assert_eq!(*byte, expected);
        }
        group.bench_function(name, |b| {
            b.iter(|| {
                source
                    .read_exact_at(black_box(offset), black_box(&mut buffer))
                    .unwrap();
                black_box(&buffer);
            })
        });
    }
    assert_eq!(disk.extents(0, disk.size()).unwrap().len(), 2);
    group.bench_function("extents_1m", |b| {
        b.iter(|| black_box(disk.extents(black_box(0), black_box(disk.size())).unwrap()))
    });
    group.bench_function("load_split_2m", |b| {
        b.iter(|| {
            black_box(
                SparseDisk::load(black_box(&sd), black_box(&sr), SparseDiskLimits::default())
                    .unwrap(),
            )
        })
    });
    let target = RawDisk::new(MemoryBlockDevice::new(2 * 1048576).unwrap());
    let mover = DataMover::new(CopyOptions::with_concurrency(65536, 4096, 4).unwrap());
    let mut verifier = Verifier::new(split.size(), 65536, 131072).unwrap();
    let dirty = vec![0xcc; 2 * 1048576];
    let mut actual = vec![0; dirty.len()];
    group.bench_function("copy_verify_2m", |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iters {
                target.write_all_at(0, &dirty).unwrap();
                let start = Instant::now();
                black_box(mover.copy(&split, &target).unwrap());
                black_box(verifier.verify(&split, &target).unwrap());
                elapsed += start.elapsed();
                target.read_exact_at(0, &mut actual).unwrap();
                for (i, byte) in actual.iter().enumerate() {
                    assert_eq!(
                        *byte,
                        if i % 1048576 < 65536 {
                            0x5a
                        } else if i % 1048576 < 131072 {
                            0xa5
                        } else {
                            0
                        }
                    );
                }
            }
            elapsed
        })
    });
    group.finish();
}
criterion_group!(benches, logical);
criterion_main!(benches);
