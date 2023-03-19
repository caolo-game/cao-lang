use cao_lang::prelude::*;

#[test]
#[cfg(feature = "serde")]
fn test_foreach_1() {
    let cu = serde_yaml::from_str(include_str!("foreach/simple_foreach.yml")).unwrap();

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let res = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read result variable");

    assert_eq!(res, Value::Integer(3 + 5 + 7));
}

#[test]
#[cfg(feature = "serde")]
fn test_foreach_nested() {
    let cu = serde_yaml::from_str(include_str!("foreach/nested_foreach.yml")).unwrap();

    let program = compile(cu, CompileOptions::new()).expect("compile");

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
