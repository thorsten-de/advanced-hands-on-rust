use criterion::{Criterion, criterion_group, criterion_main};
use my_library::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random", |b| {
        // Create RNG once outside the timings
        let mut rng = RandomNumberGenerator::new();

        // Define benchmark functions that are iterated
        // for getting benchmark timings
        b.iter(|| {
            rng.range(1.0_f32..10_000_000_f32);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
