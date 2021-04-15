use std::{convert::TryInto, str::FromStr};
use test_env_log::test;

use cao_lang::{
    compiler::{CallNode, IntegerNode, VarNode},
    prelude::*,
};

#[test]
fn test_init_table() {
    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CreateTable)
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("g_foo")))],
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
    let foo = vm
        .read_var_by_name("g_foo", &program.variables)
        .expect("Failed to read foo variable");

    let p: *mut FieldTable = foo.try_into().expect("Expected an Object");
    assert!(!p.is_null());
}

#[test]
fn test_get_set() {
    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CreateTable)
            .with_card(Card::SetVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::ScalarInt(IntegerNode(42)))
            .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::SetProperty(VarNode::from_str_unchecked("bar"))) // foo.bar
            .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::GetProperty(VarNode::from_str_unchecked("bar")))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("scoobie")))],
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let scoobie = vm
        .read_var_by_name("scoobie", &program.variables)
        .expect("Failed to read scoobie variable");

    assert!(matches!(scoobie, Value::Integer(42)));
}

#[test]
fn test_native_w_table_input() {
    struct State {
        param: i64,
    }

    let myboi = move |vm: &mut Vm<State>, table: &FieldTable| {
        let key = Key::from_str("boi").unwrap();
        let res = table.get(key).copied().unwrap_or_default();
        if let Value::Integer(i) = res {
            vm.get_aux_mut().param = i;
        } else {
            panic!("bad value");
        }
        Ok(())
    };

    let mut vm = Vm::new(State { param: 0 }).unwrap();
    vm.register_function("boii", into_f1(myboi));

    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CreateTable)
            .with_card(Card::SetVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::ScalarInt(IntegerNode(42)))
            .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::SetProperty(VarNode::from_str_unchecked("boi"))) // foo.bar
            .with_card(Card::CallNative(Box::new(CallNode(
                InputString::from("boii").unwrap(),
            ))))],
    };

    let program = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&program).expect("run failed");
    assert_eq!(vm.unwrap_aux().param, 42);
}

#[test]
fn test_native_w_table_output() {
    let myboi = move |vm: &mut Vm<()>| {
        let table = vm.init_table()?.as_ptr();
        unsafe {
            (*table)
                .insert(Key::from_str("bar").unwrap(), Value::Integer(42))
                .map_err(|_| ExecutionError::OutOfMemory)?;
        }
        vm.stack_push(Value::Object(table))?;
        Ok(())
    };

    let mut vm = Vm::new(()).unwrap();
    vm.register_function("boii", myboi);

    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CallNative(Box::new(CallNode(
                InputString::from("boii").unwrap(),
            ))))
            .with_card(Card::GetProperty(VarNode::from_str_unchecked("bar")))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("scoobie")))],
    };

    let program = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&program).expect("run failed");
    let scoobie = vm
        .read_var_by_name("scoobie", &program.variables)
        .expect("Failed to read scoobie variable");

    assert!(matches!(scoobie, Value::Integer(42)));
}
