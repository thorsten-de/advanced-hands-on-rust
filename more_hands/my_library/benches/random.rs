use criterion::{Criterion, criterion_group, criterion_main};
use my_library::*;

pub fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
