use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{Descriptor, ErrorKind, Limits};
use std::hint::black_box;

fn descriptor(c: &mut Criterion) {
    let small = include_bytes!("../tests/fixtures/monolithic.vmdk");
    let large = format!(
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\n{}",
        "RW 8 FLAT \"extent.vmdk\" 0\n".repeat(1024)
    );
    let late = format!("{large}RW 1 ZERO\n");
    let oversize = vec![b'#'; Limits::default().descriptor_bytes + 1];
    assert_eq!(Descriptor::parse(small).unwrap().size_bytes(), 8192);
    assert_eq!(
        Descriptor::parse(large.as_bytes()).unwrap().extents().len(),
        1024
    );
    assert_eq!(
        Descriptor::parse(late.as_bytes()).unwrap_err().kind,
        ErrorKind::Limit("extents")
    );
    assert_eq!(
        Descriptor::parse(&oversize).unwrap_err().kind,
        ErrorKind::Limit("descriptor bytes")
    );
    let mut group = c.benchmark_group("descriptor");
    group.sampling_mode(SamplingMode::Flat);
    for (name, input) in [
        ("small", small.as_slice()),
        ("1024_extents", large.as_bytes()),
        ("late_extent_limit", late.as_bytes()),
        ("oversize", oversize.as_slice()),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(Descriptor::parse(black_box(input))))
        });
    }
    group.finish();
}
criterion_group!(benches, descriptor);
criterion_main!(benches);
