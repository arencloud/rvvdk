#[path = "../tests/support/stream.rs"]
mod fixture;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{StreamEnvelope, StreamHeader, StreamLimits, StreamMarker};
use std::{hint::black_box, io::Cursor};
fn stream(c: &mut Criterion) {
    let limits = StreamLimits::default();
    let front = fixture::image(false);
    let footer = fixture::image(true);
    let mut large = fixture::header(true);
    fixture::put64(&mut large, 12, (1_u64 << 40) / 512);
    let h = StreamHeader::parse(&large, 1 << 30, limits).unwrap();
    let mut rejected = large;
    rejected[77] = 2;
    let grain = fixture::marker(128, 65536, 0);
    let mut group = c.benchmark_group("stream_admission");
    group.sampling_mode(SamplingMode::Flat);
    for (name, bytes, size) in [
        (
            "header_front",
            front[..512].try_into().unwrap(),
            front.len() as u64,
        ),
        ("header_footer", fixture::header(true), footer.len() as u64),
        ("header_1t", large, 1 << 30),
        ("reject_algorithm", rejected, 1 << 30),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(StreamHeader::parse(
                    black_box(&bytes),
                    black_box(size),
                    limits,
                ))
            })
        });
    }
    group.bench_function("grain_marker", |b| {
        b.iter(|| {
            black_box(StreamMarker::parse(
                black_box(&grain),
                65536,
                black_box(&h),
                limits,
            ))
        })
    });
    for (name, image) in [("envelope_front", front), ("envelope_footer", footer)] {
        StreamEnvelope::read_from(Cursor::new(&image), image.len() as u64, limits).unwrap();
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(StreamEnvelope::read_from(
                    Cursor::new(black_box(&image)),
                    image.len() as u64,
                    limits,
                ))
            })
        });
    }
    group.finish();
}
criterion_group!(benches, stream);
criterion_main!(benches);
