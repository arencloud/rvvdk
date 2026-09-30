#[path = "../tests/support/sparse_scale.rs"]
#[allow(dead_code)]
mod f;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use f::{Device, Layout, Pattern, Resolver};
use rvvdk_core::VirtualDisk;
use rvvdk_vmdk::{SparseDescriptor, SparseDisk, SparseDiskLimits};
use std::{hint::black_box, sync::Arc};
fn scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse_scale");
    group.sampling_mode(SamplingMode::Flat);
    for (label, capacity) in [("1m", 1 << 20), ("1g", 1 << 30), ("64g", 64 << 30)] {
        let l = Layout::new(capacity, 65536);
        let source = Resolver(Arc::new(Device::new(l.clone(), Pattern::Zero, false)));
        let text = SparseDescriptor::parse(l.text.as_bytes()).unwrap();
        let disk = SparseDisk::load(&text, &source, SparseDiskLimits::default()).unwrap();
        assert_eq!(disk.extents(0, capacity).unwrap().len(), 1);
        group.bench_function(format!("open_zero_{label}"), |b| {
            b.iter(|| {
                black_box(
                    SparseDisk::load(
                        black_box(&text),
                        black_box(&source),
                        SparseDiskLimits::default(),
                    )
                    .unwrap(),
                )
            })
        });
        if capacity >= 1 << 30 {
            group.bench_function(format!("query_zero_{label}"), |b| {
                b.iter(|| black_box(disk.extents(black_box(0), black_box(capacity)).unwrap()))
            });
        }
        if capacity == 64 << 30 {
            let mut bytes = [0xa5; 4096];
            disk.read_exact_at(capacity - 4096, &mut bytes).unwrap();
            assert_eq!(bytes, [0; 4096]);
            group.bench_function("tail_zero_4k_64g", |b| {
                b.iter(|| {
                    disk.read_exact_at(black_box(capacity - 4096), black_box(&mut bytes))
                        .unwrap();
                    black_box(&bytes);
                })
            });
        }
    }
    for (label, grains) in [("256m", 32768), ("512m", 65536), ("limit", 65537)] {
        let l = Layout::new(grains * 8192, 8192);
        let source = Resolver(Arc::new(Device::new(
            l.clone(),
            Pattern::Alternating,
            false,
        )));
        let disk = SparseDisk::load(
            &SparseDescriptor::parse(l.text.as_bytes()).unwrap(),
            &source,
            SparseDiskLimits::default(),
        )
        .unwrap();
        assert_eq!(disk.extents(0, disk.size()).is_err(), grains > 65536);
        group.bench_function(format!("query_alternating_{label}"), |b| {
            b.iter(|| black_box(disk.extents(black_box(0), black_box(disk.size()))))
        });
    }
    for (label, pattern) in [
        ("contiguous", Pattern::Contiguous),
        ("reversed", Pattern::Reversed),
        ("permuted", Pattern::Permuted),
    ] {
        let l = Layout::new(1 << 30, 65536);
        let source = Resolver(Arc::new(Device::new(l.clone(), pattern, false)));
        let text = SparseDescriptor::parse(l.text.as_bytes()).unwrap();
        let disk = SparseDisk::load(&text, &source, SparseDiskLimits::default()).unwrap();
        let mut bytes = vec![0; 1 << 20];
        let offset = 537 * 65536;
        disk.read_exact_at(offset, &mut bytes).unwrap();
        assert_eq!(bytes, l.expected(offset, bytes.len(), pattern));
        group.bench_function(format!("read_1m_{label}"), |b| {
            b.iter(|| {
                disk.read_exact_at(black_box(offset), black_box(&mut bytes))
                    .unwrap();
                black_box(&bytes);
            })
        });
        if label == "permuted" {
            group.bench_function("open_permuted_1g", |b| {
                b.iter(|| {
                    black_box(
                        SparseDisk::load(
                            black_box(&text),
                            black_box(&source),
                            SparseDiskLimits::default(),
                        )
                        .unwrap(),
                    )
                })
            });
        }
    }
    group.finish();
}
criterion_group!(benches, scale);
criterion_main!(benches);
