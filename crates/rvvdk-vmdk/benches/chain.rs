#[path = "../tests/support/chain.rs"]
#[allow(dead_code)]
mod f;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vmdk::{SparseChain, SparseChainLimits, SparseLayerDescriptor};
use std::hint::black_box;
fn chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse_chain");
    group.sampling_mode(SamplingMode::Flat);
    let text = f::text(1, Some((2, "parent.vmdk")), false);
    group.bench_function("parse_child", |b| {
        b.iter(|| black_box(SparseLayerDescriptor::parse(black_box(text.as_bytes())).unwrap()))
    });
    for depth in [1, 4, 16] {
        let graph = f::Graph::new(depth, true, false);
        let admitted =
            SparseChain::load(graph.root(), &graph, SparseChainLimits::default()).unwrap();
        assert_eq!(admitted.layers().len(), depth);
        group.bench_function(format!("open_embedded_{depth}"), |b| {
            b.iter(|| {
                black_box(
                    SparseChain::load(
                        graph.root(),
                        black_box(&graph),
                        SparseChainLimits::default(),
                    )
                    .unwrap(),
                )
            })
        });
        if depth == 16 {
            group.bench_function("revalidate_16", |b| {
                b.iter(|| black_box(&admitted).revalidate().unwrap())
            });
        }
    }
    let graph = f::Graph::new(17, false, false);
    group.bench_function("reject_depth_17", |b| {
        b.iter(|| {
            black_box(
                SparseChain::load(graph.root(), &graph, SparseChainLimits::default())
                    .err()
                    .unwrap(),
            )
        })
    });
    group.finish();
}
criterion_group!(benches, chain);
criterion_main!(benches);
