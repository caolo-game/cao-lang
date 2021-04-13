use cao_lang::{compiler::CompileOptions, prelude::*};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_PROG: &str = include_str!("fibonacci_program.yaml");
const FIB_RECURSE_PROG: &str = include_str!("fibonacci_program_recursive.yaml");
const JUMP_PROG: &str = include_str!("jump_program.yaml");

#[allow(unused)]
fn fib(n: i32) -> i32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    b
}

fn jump_lane(c: &mut Criterion) {
    c.bench_function("jump_lane", |b| {
        let cu = serde_yaml::from_str(JUMP_PROG).unwrap();
        let program = compile(cu, CompileOptions::new()).unwrap();

        let mut vm = Vm::new(()).with_max_iter(1 << 30);
        b.iter(|| {
            vm.clear();
            vm.run(&program).unwrap()
        })
    });
}

fn run_fib_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci numbers recursive");
    for iterations in 1..6 {
        let iterations = 1 << iterations;

        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            move |b, &iterations| {
                let cu = serde_yaml::from_str(FIB_RECURSE_PROG).unwrap();
                let program = compile(cu, CompileOptions::new()).unwrap();

                let mut vm = Vm::new(()).with_max_iter(1 << 30);
                b.iter(|| {
                    vm.clear();
                    vm.stack_push(iterations).expect("Initial push");
                    let res = vm.run(&program).expect("run failed");
                    #[cfg(debug_assertions)]
                    {
                        use std::convert::TryInto;

                        let b = vm
                            .read_var_by_name("result", &program.variables)
                            .expect("Failed to read `b` variable");
                        assert!(b.is_integer());
                        let b: i32 = b.try_into().unwrap();
                        assert_eq!(b, fib(iterations));
                    }
                    res
                })
            },
        );
    }
    group.finish();
}

fn run_fib_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci numbers iterative");
    for iterations in 1..6 {
        let iterations = 1 << iterations;

        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            move |b, &iterations| {
                let cu = serde_yaml::from_str(FIB_PROG).unwrap();
                let program = compile(cu, CompileOptions::new()).unwrap();

                let mut vm = Vm::new(()).with_max_iter(250 * iterations);
                b.iter(|| {
                    vm.clear();
                    vm.stack_push(iterations).expect("Initial push");
                    let res = vm.run(&program).expect("run failed");
                    #[cfg(debug_assertions)]
                    {
                        use std::convert::TryInto;

                        let b = vm
                            .read_var_by_name("b", &program.variables)
                            .expect("Failed to read `b` variable");
                        assert!(b.is_integer());
                        let b: i32 = b.try_into().unwrap();
                        assert_eq!(b, fib(iterations));
                    }
                    res
                })
            },
        );
    }
    group.finish();
}

criterion_group!(loop_benches, run_fib_iter, run_fib_recursive, jump_lane);

criterion_main!(loop_benches);
