use criterion::{criterion_group, criterion_main, Criterion};
use lox::Lox;

fn fibonacci() {
    let src = r#"
        fun fib(n) {
        if (n < 2) return n;
            return fib(n - 2) + fib(n - 1);
        }

        fib(25);
    "#;

    let mut lox = Lox::new();
    lox.run(&src).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("my-benchmark");
    group.sample_size(20);
    group.bench_function("fib 25", |b| b.iter(|| fibonacci()));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
