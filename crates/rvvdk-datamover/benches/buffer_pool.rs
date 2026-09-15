use criterion::{Criterion, criterion_group, criterion_main};

use rvvdk_core::BufferPool;

fn benchmark_buffer_pool(criterion: &mut Criterion) {
    let pool = BufferPool::new(32, 1024 * 1024, 4096).unwrap();

    criterion.bench_function("buffer_pool_acquire_release", |bencher| {
        bencher.iter(|| {
            let buffer = pool.acquire();

            std::hint::black_box(buffer.as_slice());
        });
    });
}

criterion_group!(benches, benchmark_buffer_pool);

criterion_main!(benches);
