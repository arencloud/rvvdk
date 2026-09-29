use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{DescriptorText, Limits};
use std::hint::black_box;
fn padding(c: &mut Criterion) {
    let text = b"version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 1 ZERO\n";
    let limit = Limits::default().descriptor_bytes;
    let mut small = text.to_vec();
    small.resize(512, 0);
    let mut maximum = text.to_vec();
    maximum.resize(limit, 0);
    let mut oversize = maximum.clone();
    oversize.push(0);
    let loaded = DescriptorText::read_from(maximum.as_slice(), Limits::default()).unwrap();
    assert_eq!(loaded.padding_bytes(), limit - text.len());
    assert!(DescriptorText::read_from(oversize.as_slice(), Limits::default()).is_err());
    let mut group = c.benchmark_group("padding");
    group.sampling_mode(SamplingMode::Flat);
    for (name, bytes) in [
        ("acquire_512", small.as_slice()),
        ("acquire_1m", maximum.as_slice()),
        ("reject_oversize", oversize.as_slice()),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(DescriptorText::read_from(
                    black_box(bytes),
                    Limits::default(),
                ))
            })
        });
    }
    group.bench_function("reparse_1m", |b| {
        b.iter(|| black_box(loaded.parse().unwrap()))
    });
    group.finish();
}
criterion_group!(benches, padding);
criterion_main!(benches);
