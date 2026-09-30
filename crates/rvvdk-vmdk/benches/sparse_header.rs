#[path = "../tests/support/sparse.rs"]
mod fixture;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{SparseHeader, SparseLimits};
use std::hint::black_box;
fn sparse(c: &mut Criterion) {
    let valid = fixture::header();
    let mut rejected = valid;
    rejected[511] = 1;
    let mut large = valid;
    fixture::put64(&mut large, 12, (1_u64 << 40) / 512);
    fixture::put64(&mut large, 64, 262784);
    // Separate directory regions for the 32,768 entries required by a 1 TiB disk.
    fixture::put64(&mut large, 48, 21);
    fixture::put64(&mut large, 56, 277);
    assert!(SparseHeader::parse(&large, 262784 * 512).is_ok());
    assert!(SparseHeader::parse(&rejected, 65536).is_err());
    let mut group = c.benchmark_group("sparse_header");
    group.sampling_mode(SamplingMode::Flat);
    for (name, bytes, size) in [
        ("valid", &valid, 65536),
        ("large_capacity", &large, 262784 * 512),
        ("reserved_rejection", &rejected, 65536),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(SparseHeader::parse(black_box(bytes), black_box(size))))
        });
    }
    group.bench_function("read_sector", |b| {
        b.iter(|| {
            black_box(SparseHeader::read_from(
                black_box(valid.as_slice()),
                65536,
                SparseLimits::default(),
            ))
        })
    });
    group.finish();
}
criterion_group!(benches, sparse);
criterion_main!(benches);
