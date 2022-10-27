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
    lox.run(src).unwrap();
}

fn zoo() {
    let src = r#"
        class Zoo {
        init() {
            this.aarvark  = 1;
            this.baboon   = 1;
            this.cat      = 1;
            this.donkey   = 1;
            this.elephant = 1;
            this.fox      = 1;
        }
            ant()    { return this.aarvark; }
            banana() { return this.baboon; }
            tuna()   { return this.cat; }
            hay()    { return this.donkey; }
            grass()  { return this.elephant; }
            mouse()  { return this.fox; }
        }

        var zoo = Zoo();
        var sum = 0;
        var start = clock();
        while (sum < 100000) {
        sum = sum + zoo.ant()
                + zoo.banana()
                + zoo.tuna()
                + zoo.hay()
                + zoo.grass()
                + zoo.mouse();
        }
    "#;

    let mut lox = Lox::new();
    lox.run(src).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("my-benchmark");
    group.sample_size(20);
    group.bench_function("fib 25", |b| b.iter(fibonacci));
    group.bench_function("zoo", |b| b.iter(zoo));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
