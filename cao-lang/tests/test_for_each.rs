use cao_lang::prelude::*;
use test_log::test;

#[test]
fn test_foreach_1() {
    let cu: CaoIr = serde_yaml::from_str(include_str!("foreach/simple_foreach.yml")).unwrap();

    let program = compile(&cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let res = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(res, Value::Integer(3 + 5 + 7));
}

#[test]
fn test_foreach_nested() {
    let cu: CaoIr = serde_yaml::from_str(include_str!("foreach/nested_foreach.yml")).unwrap();

    let program = compile(&cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let res = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(res, Value::Integer((3 + 5 + 7) * 3));

    let res = vm
        .read_var_by_name("g_iters", &program.variables)
        .expect("Failed to read iter variable");

    assert_eq!(res, Value::Integer(9));
}
