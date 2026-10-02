#[path = "../tests/support/stream_map.rs"]
mod fixture;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{StreamMap, StreamMapLimits};
use std::{hint::black_box, io::Cursor};
fn stream_map(c: &mut Criterion) {
    let limits = StreamMapLimits::default();
    let mut group = c.benchmark_group("stream_map");
    group.sampling_mode(SamplingMode::Flat);
    let cases = [
        ("empty_front", false, 64, vec![]),
        ("sparse_front", false, 64, vec![1, 63]),
        ("dense_front", false, 64, (0..64).collect()),
        ("sparse_footer", true, 64, vec![1, 63]),
        ("dense_footer", true, 64, (0..64).collect()),
        ("two_tables_footer", true, 1024, vec![1, 1023]),
        ("empty_1t_footer", true, 1 << 24, vec![]),
    ];
    for (name, footer, n, indices) in cases {
        let bytes = fixture::image(footer, n, &indices);
        let map = StreamMap::read_from(Cursor::new(&bytes), bytes.len() as u64, limits).unwrap();
        assert_eq!(map.grains().len(), indices.len());
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(StreamMap::read_from(
                    Cursor::new(black_box(&bytes)),
                    bytes.len() as u64,
                    limits,
                ))
            })
        });
    }
    let bytes = fixture::image(true, 1024, &[1, 1023]);
    let limited = StreamMapLimits {
        table_entries: 0,
        ..limits
    };
    assert!(StreamMap::read_from(Cursor::new(&bytes), bytes.len() as u64, limited).is_err());
    group.bench_function("reject_table_budget", |b| {
        b.iter(|| {
            black_box(StreamMap::read_from(
                Cursor::new(black_box(&bytes)),
                bytes.len() as u64,
                limited,
            ))
        })
    });
    let map = StreamMap::read_from(Cursor::new(&bytes), bytes.len() as u64, limits).unwrap();
    group.bench_function("lookup_sparse", |b| {
        b.iter(|| black_box(map.grain(black_box(1023))))
    });
    group.finish();
}
criterion_group!(benches, stream_map);
criterion_main!(benches);
