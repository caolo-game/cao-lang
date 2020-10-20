use cao_lang::{compiler::CompileOptions, prelude::*};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_PROG: &str = include_str!("fibonacci_program.json");

#[allow(unused)]
fn fib(n: i32) -> i32 {
    let mut a = 1;
    let mut b = 1;
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    b
}

fn run_fib(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci numbers");
    for iterations in 1..6 {
        let iterations = 1 << iterations;

        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            move |b, &iterations| {
                let cu = serde_json::from_str(FIB_PROG).unwrap();
                let program =
                    compile(None, cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

                let mut vm = VM::new(None, ()).with_max_iter(250 * iterations);
                b.iter(|| {
                    vm.clear();
                    vm.stack_push(iterations).expect("Initial push");
                    let res = vm.run(&program).expect("run failed");
                    #[cfg(debug_assertions)]
                    {
                        use std::convert::TryInto;

                        let varid = program.variables.0["b"];
                        let val = *vm.read_var(varid).expect("failed to read b");
                        assert!(val.is_integer());
                        let val: i32 = val.try_into().unwrap();
                        assert_eq!(val, fib(iterations));
                    }
                    res
                })
            },
        );
    }
    group.finish();
}

fn clear_vm(c: &mut Criterion) {
    c.bench_function("clear_vm", move |b| {
        let mut vm = VM::new(None, ()).with_max_iter(250_000_000);
        b.iter(|| {
            vm.clear();
            criterion::black_box(&mut vm);
        });
    });
}

criterion_group!(loop_benches, run_fib, clear_vm);

criterion_main!(loop_benches);
