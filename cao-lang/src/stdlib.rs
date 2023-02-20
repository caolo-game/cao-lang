//! Cao-Lang standard library
//!
//! The standard library is injected into every `Module` at compilation time.
//! Standard functions can be imported via the `std` module

use crate::compiler::{Card, ForEach, Lane, Module};

/// Given a table and a callback that returns a bool create a new table whith the items that return
/// true
pub fn filter() -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("callback")
        .with_cards(vec![
            Card::set_var("res", Card::CreateTable),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![Card::IfTrue(Box::new([
                        Card::dynamic_call(
                            Card::read_var("callback"),
                            vec![
                                Card::read_var("i"),
                                Card::read_var("v"),
                                Card::read_var("k"),
                            ],
                        ),
                        Card::set_property(
                            Card::read_var("v"),
                            Card::read_var("res"),
                            Card::read_var("k"),
                        ),
                    ]))],
                )),
            })),
            Card::return_card(Card::read_var("res")),
        ])
}

/// Returns the key of the first row that returns True from the callback
pub fn any() -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("callback")
        .with_cards(vec![
            Card::set_var("res", Card::CreateTable),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![Card::IfTrue(Box::new([
                        Card::dynamic_call(
                            Card::read_var("callback"),
                            vec![
                                Card::read_var("i"),
                                Card::read_var("v"),
                                Card::read_var("k"),
                            ],
                        ),
                        Card::return_card(Card::read_var("k")),
                    ]))],
                )),
            })),
            Card::return_card(Card::ScalarNil),
        ])
}

/// Iterate on a table calling the provided callback for each row.
/// Build a new table from the callback return values, using the same keys
pub fn map() -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("callback")
        .with_cards(vec![
            Card::set_var("res", Card::CreateTable),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![Card::set_property(
                        Card::composite_card(
                            "",
                            vec![Card::dynamic_call(
                                Card::read_var("callback"),
                                vec![
                                    Card::read_var("i"),
                                    Card::read_var("v"),
                                    Card::read_var("k"),
                                ],
                            )],
                        ),
                        Card::read_var("res"),
                        Card::read_var("k"),
                    )],
                )),
            })),
            Card::return_card(Card::read_var("res")),
        ])
}

fn minmax(minimax: &str) -> Lane {
    Lane::default().with_arg("iterable").with_cards(vec![
        Card::function_value("row_to_value"),
        Card::read_var("iterable"),
        Card::return_card(Card::call_function(minimax, vec![])),
    ])
}

/// Return the smallest value in the table, or nil if the table is empty
pub fn min() -> Lane {
    minmax("min_by_key")
}

/// Return the largest value in the table, or nil if the table is empty
pub fn max() -> Lane {
    minmax("max_by_key")
}

fn minmax_by_key(on_less_card: impl FnOnce(Card, Card) -> Card) -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("key_function")
        .with_cards(vec![
            Card::set_var("i", Card::ScalarInt(0)),
            Card::set_var(
                "tmp",
                Card::Get(Box::new([Card::read_var("iterable"), Card::read_var("i")])),
            ),
            Card::set_var(
                "min_item",
                Card::dynamic_call(
                    Card::read_var("key_function"),
                    vec![Card::read_var("tmp.value"), Card::read_var("tmp.key")],
                ),
            ),
            Card::ForEach(Box::new(ForEach {
                i: Some("j".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(on_less_card(
                    Card::Less(Box::new([
                        Card::dynamic_call(
                            Card::read_var("key_function"),
                            vec![Card::read_var("v"), Card::read_var("k")],
                        ),
                        Card::read_var("min_item"),
                    ])),
                    Card::composite_card(
                        "",
                        vec![
                            Card::set_var("i", Card::read_var("j")),
                            Card::set_var("min_item", Card::read_var("v")),
                        ],
                    ),
                )),
            })),
            Card::return_card(Card::Get(Box::new([
                Card::read_var("iterable"),
                Card::read_var("i"),
            ]))),
        ])
}

/// Return the smallest value in the table, or nil if the table is empty
pub fn min_by_key() -> Lane {
    minmax_by_key(|cond, body| Card::IfTrue(Box::new([cond, body])))
}

pub fn max_by_key() -> Lane {
    minmax_by_key(|cond, body| Card::IfFalse(Box::new([cond, body])))
}

/// A (key, value) function that returns the value given
pub fn value_key_fn() -> Lane {
    Lane::default()
        .with_arg("_key")
        .with_arg("val")
        .with_cards(vec![Card::read_var("val")])
}

pub fn standard_library() -> Module {
    let mut module = Module::default();
    module.lanes.push(("filter".to_string(), filter()));
    module.lanes.push(("any".to_string(), any()));
    module.lanes.push(("map".to_string(), map()));
    module.lanes.push(("min".to_string(), min()));
    module.lanes.push(("max".to_string(), max()));
    module.lanes.push(("min_by_key".to_string(), min_by_key()));
    module.lanes.push(("max_by_key".to_string(), max_by_key()));
    module
        .lanes
        .push(("row_to_value".to_string(), value_key_fn()));
    module
}

#[cfg(test)]
mod tests {
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
                    Lane::default().with_cards(vec![
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
                    Lane::default()
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
                    Lane::default().with_cards(vec![
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
                    Lane::default()
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
                Lane::default().with_cards(vec![
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
                Lane::default().with_cards(vec![
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
                Lane::default().with_cards(vec![Card::set_global_var(
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
        unsafe {
            let row = result.as_table().unwrap();
            match row.get("value").unwrap() {
                Value::Nil => {}
                a @ _ => panic!("Unexpected result: {a:?}"),
            }
        }
    }

    #[traced_test]
    #[test]
    fn min_by_key_test() {
        let program = Module {
            imports: vec!["std.min_by_key".to_string()],
            lanes: vec![
                (
                    "main".to_string(),
                    Lane::default().with_cards(vec![
                        Card::set_var(
                            "t",
                            Card::Array(vec![
                                Card::ScalarInt(2),
                                Card::ScalarInt(3),
                                Card::ScalarInt(1),
                                Card::ScalarInt(4),
                                Card::ScalarInt(3),
                            ]),
                        ),
                        // call min
                        Card::set_global_var(
                            "g_result",
                            Card::call_function(
                                "min_by_key",
                                vec![Card::Function("keyfn".to_string()), Card::read_var("t")],
                            ),
                        ),
                    ]),
                ),
                (
                    "keyfn".to_string(),
                    Lane::default()
                        .with_arg("key")
                        .with_arg("val")
                        .with_cards(vec![Card::Div(BinaryExpression::new([
                            Card::read_var("val"),
                            Card::scalar_int(10),
                        ]))]),
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
        unsafe {
            let t = result.as_table().expect("table");
            match t.get("value").unwrap() {
                Value::Integer(i) => {
                    assert_eq!(*i, 1);
                }
                a @ _ => panic!("Unexpected result: {a:?}"),
            }
        }
    }
}
