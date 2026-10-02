#[path = "../tests/support/stream_disk.rs"]
mod fixture;
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use rvvdk_core::{BlockDevice, MemoryBlockDevice, VirtualDisk};
use rvvdk_vmdk::{StreamDisk, StreamDiskLimits};
use std::{hint::black_box, sync::Arc};
fn disk(payload: &[u8], populated: bool) -> StreamDisk {
    let records = if populated {
        (0..64).map(|i| (i, payload.to_vec())).collect()
    } else {
        vec![]
    };
    let bytes = fixture::image(true, 64, &records);
    let source = Arc::new(MemoryBlockDevice::new(bytes.len()).unwrap());
    source.write_all_at(0, &bytes).unwrap();
    StreamDisk::load(source, StreamDiskLimits::default()).unwrap()
}
fn reads(c: &mut Criterion) {
    let mut g = c.benchmark_group("stream_reads");
    g.sampling_mode(SamplingMode::Flat);
    for (label, payload) in [
        (
            "compressed",
            include_bytes!("../tests/fixtures/stream/pattern.zlib").as_slice(),
        ),
        ("stored", fixture::stored(&fixture::bytes(1)).as_slice()),
    ] {
        let d = disk(payload, true);
        let mut out = vec![0; 1 << 20];
        d.read_exact_at(0, &mut out).unwrap();
        g.throughput(Throughput::Bytes(out.len() as u64));
        g.bench_function(format!("{label}_sequential_1m"), |b| {
            b.iter(|| {
                d.read_exact_at(0, black_box(&mut out)).unwrap();
                black_box(&out);
            })
        });
        let mut small = [0; 4096];
        let mut seed = 0x511b_u64;
        g.throughput(Throughput::Bytes(4096));
        g.bench_function(format!("{label}_random_4k"), |b| {
            b.iter(|| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let at = ((seed >> 32) % 64) * 65536 + (seed % 61441);
                d.read_exact_at(at, black_box(&mut small)).unwrap();
                black_box(&small);
            })
        });
    }
    let d = disk(
        include_bytes!("../tests/fixtures/stream/pattern.zlib"),
        true,
    );
    let mut out = [0; 4096];
    d.read_exact_at(0, &mut out).unwrap();
    g.throughput(Throughput::Bytes(4096));
    g.bench_function("cached_4k", |b| {
        b.iter(|| {
            d.read_exact_at(0, black_box(&mut out)).unwrap();
            black_box(&out);
        })
    });
    g.bench_function("cross_grain_4k", |b| {
        b.iter(|| {
            d.read_exact_at(65536 - 2048, black_box(&mut out)).unwrap();
            black_box(&out);
        })
    });
    let d = disk(&[], false);
    let mut out = vec![0; 1 << 20];
    g.throughput(Throughput::Bytes(out.len() as u64));
    g.bench_function("sparse_zero_1m", |b| {
        b.iter(|| {
            d.read_exact_at(0, black_box(&mut out)).unwrap();
            black_box(&out);
        })
    });
    g.finish();
}
criterion_group!(benches, reads);
criterion_main!(benches);
