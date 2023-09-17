use cao_lang::{compiler::CompileOptions, prelude::*};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const FIB_PROG: &str = include_str!("fibonacci_program.yaml");
const FIB_RECURSE_PROG: &str = include_str!("fibonacci_program_recursive.yaml");

#[allow(unused)]
fn fib(n: i64) -> i64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    b
}

fn run_fib_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci numbers recursive");
    for iterations in 1..5 {
        let iterations = 1 << iterations;

        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            move |b, &iterations| {
                let cu = serde_yaml::from_str(FIB_RECURSE_PROG).unwrap();
                let program = compile(cu, CompileOptions::new()).unwrap();

                let mut vm = Vm::new(()).unwrap().with_max_iter(1 << 30);
                vm.runtime_data.set_memory_limit(1024 * 1024 * 1024);
                b.iter(|| {
                    vm.clear();
                    vm.stack_push(iterations).expect("Initial push");
                    let res = vm.run(&program).expect("run failed");
                    #[cfg(debug_assertions)]
                    {
                        let b = vm
                            .read_var_by_name("result", &program.variables)
                            .expect("Failed to read `b` variable");
                        assert!(b.is_integer());
                        let b: i64 = b.try_into().unwrap();
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
    for iterations in 1..=6 {
        let iterations = 1 << iterations;

        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            move |b, &iterations| {
                let cu = serde_yaml::from_str(FIB_PROG).unwrap();
                let program = compile(cu, CompileOptions::new()).unwrap();

                let mut vm = Vm::new(()).unwrap().with_max_iter(250 * iterations);
                let iterations = iterations as i64;
                b.iter(|| {
                    vm.clear();
                    vm.stack_push(iterations).expect("Initial push");
                    let res = vm.run(&program).expect("run failed");
                    #[cfg(debug_assertions)]
                    {
                        let b = vm
                            .read_var_by_name("b", &program.variables)
                            .expect("Failed to read `b` variable");
                        assert!(b.is_integer());
                        let b: i64 = b.try_into().unwrap();
                        assert_eq!(b, fib(iterations));
                    }
                    res
                })
            },
        );
    }
    group.finish();
}

fn run_empty_function_call(c: &mut Criterion) {
    c.bench_function("empty function call", |b| {
        let cu = CaoProgram {
            imports: Default::default(),
            submodules: Default::default(),
            functions: [
                (
                    // create a closure that captures a local variable
                    // and sets a global variable
                    "foo".into(),
                    Function::default(),
                ),
                (
                    "main".into(),
                    Function::default().with_card(Card::call_function("foo", vec![])),
                ),
            ]
            .into(),
        };
        let program = compile(cu, CompileOptions::new()).unwrap();

        let mut vm = Vm::new(()).unwrap();
        b.iter(|| {
            vm.clear();
            vm.run(&program).unwrap();
        });
    });
}

criterion_group!(
    benches,
    run_fib_iter,
    run_fib_recursive,
    run_empty_function_call
);

criterion_main!(benches);
