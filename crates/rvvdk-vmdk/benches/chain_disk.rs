#[path = "../tests/support/chain_disk.rs"]
#[allow(dead_code)]
mod f;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use f::{Fixture, Spec};
use rvvdk_core::VirtualDisk;
use std::hint::black_box;
fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_disk");
    group.sampling_mode(SamplingMode::Flat);
    const SIZE: u64 = 1 << 20;
    for depth in [1, 4, 16] {
        let mut specs = vec![Spec::new(vec![SIZE], 8192, 0, 0, 1); depth];
        specs[depth - 1] = Spec::new(vec![SIZE], 65536, 1, 0, 71);
        let f = Fixture::new(specs, false);
        let disk = f.disk();
        let mut buffer = vec![0; SIZE as usize];
        disk.read_exact_at(0, &mut buffer).unwrap();
        assert_eq!(buffer, f.oracle(0, buffer.len()));
        group.bench_function(format!("read_base_1m_depth{depth}"), |b| {
            b.iter(|| {
                disk.read_exact_at(black_box(0), black_box(&mut buffer))
                    .unwrap();
                black_box(&buffer);
            })
        });
        group.bench_function(format!("query_base_1m_depth{depth}"), |b| {
            b.iter(|| black_box(disk.extents(black_box(0), black_box(SIZE)).unwrap()))
        });
    }
    for (name, step, reverse) in [
        ("leaf", 1, false),
        ("alternating", 2, false),
        ("fragmented", 2, true),
        ("zero", 0, false),
    ] {
        let mut child = Spec::new(vec![SIZE], 8192, step, 0, 11);
        child.reverse = reverse;
        let parent = Spec::new(vec![SIZE], 65536, if step == 0 { 0 } else { 1 }, 0, 99);
        let f = Fixture::new(vec![child, parent], false);
        let disk = f.disk();
        let mut buffer = vec![0; SIZE as usize];
        disk.read_exact_at(0, &mut buffer).unwrap();
        assert_eq!(buffer, f.oracle(0, buffer.len()));
        group.bench_function(format!("read_{name}_1m"), |b| {
            b.iter(|| {
                disk.read_exact_at(black_box(0), black_box(&mut buffer))
                    .unwrap();
                black_box(&buffer);
            })
        });
    }
    group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
