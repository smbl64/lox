use criterion::{criterion_group, criterion_main, Criterion};
use lox::Lox;

fn run_code(src: &str) {
    let mut lox = Lox::new();
    lox.run(src).unwrap();
}

fn fibonacci() {
    let src = r#"
        fun fib(n) {
        if (n < 2) return n;
            return fib(n - 2) + fib(n - 1);
        }

        fib(25);
    "#;

    run_code(src);
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

    run_code(src)
}

fn simple_call() {
    let src = r#"
        fun calc(a, b) {
            return a + b;
        }
        var a = 1;
        var b = 2;
        var c = calc(a, b);
    "#;

    run_code(src);
}

fn fib_benchmark(c: &mut Criterion) {
    c.bench_function("fib", |b| b.iter(fibonacci));
}

fn zoo_benchmark(c: &mut Criterion) {
    c.bench_function("zoo", |b| b.iter(zoo));
}

fn simple_call_benchmark(c: &mut Criterion) {
    c.bench_function("simple-call", |b| b.iter(simple_call));
}

criterion_group!(benches, fib_benchmark, zoo_benchmark, simple_call_benchmark);
criterion_main!(benches);
