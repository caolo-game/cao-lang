use cao_lang::prelude::*;
use test_log::test;

const RECURSIVE_FIB: &str = include_str!("../benches/fibonacci_program_recursive.yaml");
const ITERATIVE_FIB: &str = include_str!("../benches/fibonacci_program.yaml");

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

#[test]
fn fibonacci_1() {
    let cu = serde_yaml::from_str(RECURSIVE_FIB).unwrap();
    let program = compile(&cu, CompileOptions::new()).unwrap();

    let mut vm = Vm::new(()).unwrap();
    vm.stack_push(Value::Integer(1)).unwrap();
    vm.run(&program).expect("run failed");

    let result = vm
        .read_var_by_name("result", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(result, Value::Integer(1));
}

#[test]
fn fibonacci_4() {
    let cu = serde_yaml::from_str(RECURSIVE_FIB).unwrap();
    let program = compile(&cu, CompileOptions::new()).unwrap();

    let mut vm = Vm::new(()).unwrap();
    vm.stack_push(Value::Integer(4)).unwrap();
    vm.run(&program).expect("run failed");

    let result = vm
        .read_var_by_name("result", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(result, Value::Integer(fib(4)));
}

#[test]
fn fibonacci_32() {
    let cu = serde_yaml::from_str(ITERATIVE_FIB).unwrap();
    let program = compile(&cu, CompileOptions::new()).unwrap();

    let mut vm = Vm::new(()).unwrap();
    vm.max_instr = 10_000_000;
    vm.stack_push(Value::Integer(32)).unwrap();
    vm.run(&program).expect("run failed");

    let result = vm
        .read_var_by_name("b", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(result, Value::Integer(fib(32)));
}
