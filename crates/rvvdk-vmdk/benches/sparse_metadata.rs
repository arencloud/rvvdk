#[path = "../tests/support/sparse_metadata.rs"]
mod fixture;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{SparseDescriptor, SparseMetadata, SparseMetadataLimits};
use std::hint::black_box;
fn metadata(c: &mut Criterion) {
    let text = fixture::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"");
    let d = SparseDescriptor::parse(text.as_bytes()).unwrap();
    let bytes = fixture::bytes(Some(&text));
    let r = fixture::Resolver(fixture::device(&bytes));
    let mut bad = bytes.clone();
    fixture::put32(&mut bad, 22 * 512, 256);
    let bad = fixture::Resolver(fixture::device(&bad));
    let split = fixture::descriptor("twoGbMaxExtentSparse", "RW 2048 SPARSE \"disk.vmdk\"");
    let sd = SparseDescriptor::parse(split.as_bytes()).unwrap();
    let empty = fixture::Resolver(fixture::device(&fixture::bytes(None)));
    assert!(SparseMetadata::load(&d, 0, &r, SparseMetadataLimits::default()).is_ok());
    assert!(SparseMetadata::load(&d, 0, &bad, SparseMetadataLimits::default()).is_err());
    let mut group = c.benchmark_group("sparse_metadata");
    group.sampling_mode(SamplingMode::Flat);
    for (name, desc, resolver) in [
        ("monolithic", &d, &r),
        ("split_empty_region", &sd, &empty),
        ("redundant_rejection", &d, &bad),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(SparseMetadata::load(
                    black_box(desc),
                    0,
                    black_box(resolver),
                    SparseMetadataLimits::default(),
                ))
            })
        });
    }
    group.bench_function("descriptor", |b| {
        b.iter(|| black_box(SparseDescriptor::parse(black_box(text.as_bytes()))))
    });
    group.finish();
}
criterion_group!(benches, metadata);
criterion_main!(benches);
