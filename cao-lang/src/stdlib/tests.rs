use tracing_test::traced_test;

use crate::{
    compiler::{compile, BinaryExpression},
    value::Value,
    vm::Vm,
};

use super::*;

#[tracing_test::traced_test]
#[test]
fn filter_test() {
    let program = Module {
        imports: vec!["std.filter".to_string()],
        lanes: vec![
            (
                "main".to_string(),
                Function::default().with_cards(vec![
                    Card::set_var("t", Card::CreateTable),
                    Card::set_property(
                        Card::scalar_int(1),
                        Card::read_var("t"),
                        Card::string_card("winnie"),
                    ),
                    Card::set_property(
                        Card::scalar_int(2),
                        Card::read_var("t"),
                        Card::string_card("pooh"),
                    ),
                    // call filter
                    Card::set_global_var(
                        "g_result",
                        Card::call_function(
                            "filter",
                            vec![Card::Function("cb".to_string()), Card::read_var("t")],
                        ),
                    ),
                ]),
            ),
            (
                "cb".to_string(),
                Function::default()
                    .with_arg("k")
                    .with_cards(vec![Card::return_card(Card::Equals(Box::new([
                        Card::read_var("k"),
                        Card::string_card("winnie"),
                    ])))]),
            ),
        ],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    match result {
        Value::Object(o) => unsafe {
            let o = o.as_ref();
            let o = o.as_table().unwrap();
            assert!(o.contains("winnie"));
            assert_eq!(o.len(), 1);
        },
        a @ _ => panic!("Unexpected result: {a:?}"),
    }
}

#[test]
fn can_import_stdlib_test() {
    let program = Module {
        imports: vec!["std.filter".to_string()],
        lanes: vec![("main".to_string(), Default::default())],
        ..Default::default()
    };

    let _ = compile(program, None).unwrap();
}

#[test]
fn std_named_module_is_error_test() {
    let program = Module {
        imports: vec![],
        lanes: vec![("main".to_string(), Default::default())],
        submodules: vec![("std".to_string(), Default::default())],
    };

    let res = compile(program, None);
    assert!(res.is_err());
}

#[test]
fn stdlib_can_be_imported_in_submodule_test() {
    let submodule = Module {
        imports: vec!["std.filter".to_string()],
        ..Default::default()
    };
    let program = Module {
        lanes: vec![("main".to_string(), Default::default())],
        submodules: vec![("foo".to_string(), submodule)],
        ..Default::default()
    };

    let _ = compile(program, None).unwrap();
}

#[test]
fn map_test() {
    let program = Module {
        imports: vec!["std.map".to_string()],
        lanes: vec![
            (
                "main".to_string(),
                Function::default().with_cards(vec![
                    Card::set_var("t", Card::CreateTable),
                    Card::set_property(
                        Card::scalar_int(1),
                        Card::read_var("t"),
                        Card::string_card("winnie"),
                    ),
                    Card::set_property(
                        Card::scalar_int(2),
                        Card::read_var("t"),
                        Card::string_card("pooh"),
                    ),
                    // call filter
                    Card::set_global_var(
                        "g_result",
                        Card::call_function(
                            "map",
                            vec![Card::Function("cb".to_string()), Card::read_var("t")],
                        ),
                    ),
                ]),
            ),
            (
                "cb".to_string(),
                Function::default()
                    .with_arg("k")
                    .with_card(Card::return_card(Card::Equals(Box::new([
                        Card::read_var("k"),
                        Card::string_card("winnie"),
                    ])))),
            ),
        ],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    match result {
        Value::Object(mut o) => unsafe {
            let o = o.as_mut();
            let o = o.as_table_mut().unwrap();
            assert!(o.contains("winnie"));
            for (k, v) in o.iter() {
                let k = k.as_str().unwrap();
                match k {
                    "winnie" => assert_eq!(v, &Value::Integer(1)),
                    _ => assert_eq!(v, &Value::Integer(0)),
                }
            }
        },
        a @ _ => panic!("Unexpected result: {a:?}"),
    }
}

#[traced_test]
#[test]
fn min_test() {
    let program = Module {
        imports: vec!["std.min".to_string()],
        lanes: vec![(
            "main".to_string(),
            Function::default().with_cards(vec![
                Card::set_global_var("t", Card::CreateTable),
                Card::set_var("t.winnie", Card::scalar_int(10)),
                Card::set_var("t.pooh", Card::scalar_int(20)),
                Card::set_var("t.tiggers", Card::scalar_int(30)),
                // call min
                Card::set_global_var(
                    "g_result",
                    Card::call_function("min", vec![Card::read_var("t")]),
                ),
            ]),
        )],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    unsafe {
        let t = vm
            .read_var_by_name("t", &compiled.variables)
            .unwrap()
            .as_table()
            .expect("table");

        dbg!(t);
        dbg!(t.get(&t.nth_key(0)));
    }

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    unsafe {
        let t = result.as_table().expect("table");
        dbg!(t);
        match t.get("value").expect("value") {
            Value::Integer(i) => {
                assert_eq!(*i, 10);
            }
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }
}

#[traced_test]
#[test]
fn max_test() {
    let program = Module {
        imports: vec!["std.max".to_string()],
        lanes: vec![(
            "main".to_string(),
            Function::default().with_cards(vec![
                Card::set_var("t", Card::CreateTable),
                Card::AppendTable(BinaryExpression::new([
                    Card::scalar_int(1),
                    Card::read_var("t"),
                ])),
                Card::AppendTable(BinaryExpression::new([
                    Card::ScalarFloat(3.42),
                    Card::read_var("t"),
                ])),
                // call max
                Card::set_global_var(
                    "g_result",
                    Card::call_function("max", vec![Card::read_var("t")]),
                ),
            ]),
        )],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    let row = unsafe { result.as_table().unwrap() };
    match row.get("value").unwrap() {
        Value::Real(i) => {
            assert_eq!(*i, 3.42);
        }
        a @ _ => panic!("Unexpected result: {a:?}"),
    }
}

#[traced_test]
#[test]
fn max_empty_list_returns_nil_test() {
    let program = Module {
        imports: vec!["std.max".to_string()],
        lanes: vec![(
            "main".to_string(),
            Function::default().with_cards(vec![Card::set_global_var(
                "g_result",
                Card::call_function("max", vec![Card::CreateTable]),
            )]),
        )],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    match result {
        Value::Nil => {}
        a @ _ => panic!("Unexpected result: {a:?}"),
    }
}

#[traced_test]
#[test]
fn min_by_key_test() {
    let program = Module {
        imports: vec!["std.min_by_key".to_string()],
        lanes: vec![(
            "main".to_string(),
            Function::default().with_cards(vec![
                Card::set_var(
                    "t",
                    Card::Array(vec![
                        Card::ScalarInt(2),
                        Card::ScalarInt(3),
                        Card::ScalarInt(1),
                        Card::ScalarInt(4),
                    ]),
                ),
                // call min
                Card::set_global_var(
                    "g_result",
                    Card::call_function(
                        "min_by_key",
                        vec![
                            Card::Closure(Box::new(
                                Function::default()
                                    .with_arg("key")
                                    .with_arg("val")
                                    .with_card(Card::return_card(Card::Div(
                                        BinaryExpression::new([
                                            Card::scalar_int(10),
                                            Card::read_var("val"),
                                        ]),
                                    ))),
                            )),
                            Card::read_var("t"),
                        ],
                    ),
                ),
            ]),
        )],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    compiled.print_disassembly();
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    unsafe {
        dbg!(result);
        let t = result.as_table().expect("table");
        match t.get("value").unwrap() {
            Value::Integer(i) => {
                assert_eq!(*i, 4);
            }
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }
}

#[traced_test]
#[test]
fn sort_by_key_test() {
    let program = Module {
        imports: vec!["std.sorted_by_key".to_string()],
        lanes: vec![
            (
                "main".to_string(),
                Function::default().with_cards(vec![
                    Card::set_var(
                        "t",
                        Card::Array(vec![
                            Card::ScalarInt(2),
                            Card::ScalarInt(3),
                            Card::ScalarInt(1),
                            Card::ScalarInt(4),
                        ]),
                    ),
                    // call min
                    Card::set_global_var(
                        "g_result",
                        Card::call_function(
                            "sorted_by_key",
                            vec![Card::Function("keyfn".to_string()), Card::read_var("t")],
                        ),
                    ),
                ]),
            ),
            (
                "keyfn".to_string(),
                Function::default()
                    .with_arg("_key")
                    .with_arg("val")
                    .with_card(Card::return_card(Card::read_var("val"))),
            ),
        ],
        ..Default::default()
    };

    let compiled = compile(program, None).expect("Failed to compile");
    compiled.print_disassembly();
    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&compiled).expect("run");

    let result = vm
        .read_var_by_name("g_result", &compiled.variables)
        .unwrap();
    unsafe {
        let t = result.as_table().expect("table");
        let values = t.iter().map(|(_, v)| *v).collect::<Vec<_>>();
        assert_eq!(
            values,
            [
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ]
        );
    }
}
