use std::{convert::TryInto, str::FromStr};
use test_log::test;

use cao_lang::{
    compiler::{CallNode, IntegerNode, StringNode, VarNode},
    prelude::*,
};

#[test]
fn test_init_table() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::CreateTable),
            (
                2.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_foo")),
            ),
        ]
        .into(),
        lanes: [("main".into(), Lane::default().with_card(1).with_card(2))].into(),
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
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::CreateTable),
            (2.into(), Card::SetVar(VarNode::from_str_unchecked("foo"))),
            (3.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
            (4.into(), Card::StringLiteral(StringNode("bar".to_string()))),
            (5.into(), Card::ScalarInt(IntegerNode(42))),
            (6.into(), Card::SetProperty),
            (7.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
            (8.into(), Card::StringLiteral(StringNode("bar".to_string()))),
            (9.into(), Card::GetProperty),
            (
                10.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("scoobie")),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(1)
                .with_card(2)
                .with_card(3)
                .with_card(4)
                .with_card(5)
                .with_card(6)
                .with_card(7)
                .with_card(8)
                .with_card(9)
                .with_card(10),
        )]
        .into(),
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
        let key = Handle::from_str("boi").unwrap();
        let res = table.get_value(key).unwrap_or_default();
        if let Value::Integer(i) = res {
            vm.get_aux_mut().param = i;
        } else {
            panic!("bad value");
        }
        Ok(())
    };

    let mut vm = Vm::new(State { param: 0 }).unwrap();
    vm.register_function("boii", into_f1(myboi));

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::CreateTable),
            (2.into(), Card::SetVar(VarNode::from_str_unchecked("foo"))),
            (3.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
            (4.into(), Card::StringLiteral(StringNode("boi".to_string()))),
            (5.into(), Card::ScalarInt(IntegerNode(42))),
            (6.into(), Card::SetProperty),
            (
                7.into(),
                Card::CallNative(Box::new(CallNode(InputString::from("boii").unwrap()))),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(1)
                .with_card(2)
                .with_card(3)
                .with_card(4)
                .with_card(5)
                .with_card(6)
                .with_card(7),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&program).expect("run failed");
    assert_eq!(vm.unwrap_aux().param, 42);
}
