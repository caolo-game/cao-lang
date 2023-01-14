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
            Card::CreateTable,
            Card::set_var("res"),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![
                        Card::read_var("i"),
                        Card::read_var("v"),
                        Card::read_var("k"),
                        Card::read_var("callback"),
                        Card::DynamicJump,
                        Card::IfTrue(Box::new(Card::composite_card(
                            "_",
                            vec![
                                Card::read_var("v"),
                                Card::read_var("res"),
                                Card::read_var("k"),
                                Card::SetProperty,
                            ],
                        ))),
                    ],
                )),
            })),
            Card::read_var("res"),
            Card::Return,
        ])
}

/// Returns the key of the first row that returns True from the callback
pub fn any() -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("callback")
        .with_cards(vec![
            Card::CreateTable,
            Card::set_var("res"),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![
                        Card::read_var("i"),
                        Card::read_var("v"),
                        Card::read_var("k"),
                        Card::read_var("callback"),
                        Card::DynamicJump,
                        Card::IfTrue(Box::new(Card::composite_card(
                            "_",
                            vec![Card::read_var("k"), Card::Return],
                        ))),
                    ],
                )),
            })),
            Card::ScalarNil,
            Card::Return,
        ])
}

/// Iterate on a table calling the provided callback for each row.
/// Build a new table from the callback return values, using the same keys
pub fn map() -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("callback")
        .with_cards(vec![
            Card::CreateTable,
            Card::set_var("res"),
            Card::ForEach(Box::new(ForEach {
                i: Some("i".to_string()),
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![
                        Card::read_var("i"),
                        Card::read_var("v"),
                        Card::read_var("k"),
                        Card::read_var("callback"),
                        Card::DynamicJump,
                        Card::read_var("res"),
                        Card::read_var("k"),
                        Card::SetProperty,
                    ],
                )),
            })),
            Card::read_var("res"),
            Card::Return,
        ])
}

fn minmax(minimax: &str) -> Lane {
    Lane::default().with_arg("iterable").with_cards(vec![
        Card::function_value("row_to_value"),
        Card::read_var("iterable"),
        Card::jump(minimax),
        Card::Return,
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

fn minmax_by_key(on_less_card: impl FnOnce(&str, &str) -> Card) -> Lane {
    Lane::default()
        .with_arg("iterable")
        .with_arg("key_function")
        .with_cards(vec![
            Card::read_var("iterable"),
            Card::ScalarInt(0),
            Card::Get,
            Card::Pop,
            Card::set_var("result"),
            Card::ForEach(Box::new(ForEach {
                i: None,
                k: Some("k".to_string()),
                v: Some("v".to_string()),
                iterable: Box::new(Card::read_var("iterable")),
                body: Box::new(Card::composite_card(
                    "_",
                    vec![
                        Card::read_var("v"),
                        Card::read_var("k"),
                        Card::read_var("key_function"),
                        Card::DynamicJump,
                        Card::read_var("result"),
                        Card::Less,
                        on_less_card("v", "result"),
                    ],
                )),
            })),
            Card::read_var("result"),
            Card::Return,
        ])
}

/// Return the smallest value in the table, or nil if the table is empty
pub fn min_by_key() -> Lane {
    minmax_by_key(|var, res| {
        Card::IfTrue(Box::new(Card::composite_card(
            "_",
            vec![Card::read_var(var), Card::set_var(res)],
        )))
    })
}

pub fn max_by_key() -> Lane {
    minmax_by_key(|var, res| {
        Card::IfFalse(Box::new(Card::composite_card(
            "_",
            vec![Card::read_var(var), Card::set_var(res)],
        )))
    })
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
    use crate::{compiler::compile, value::Value, vm::Vm};

    use super::*;

    #[test]
    fn filter_test() {
        let program = Module {
            imports: vec!["std.filter".to_string()],
            lanes: vec![
                (
                    "main".to_string(),
                    Lane::default().with_cards(vec![
                        Card::CreateTable,
                        Card::set_var("t"),
                        Card::scalar_int(1),
                        Card::read_var("t"),
                        Card::string_card("winnie"),
                        Card::SetProperty,
                        Card::scalar_int(2),
                        Card::read_var("t"),
                        Card::string_card("pooh"),
                        Card::SetProperty,
                        // call filter
                        Card::Function("cb".to_string()),
                        Card::read_var("t"),
                        Card::jump("filter"),
                        Card::set_global_var("g_result"),
                    ]),
                ),
                (
                    "cb".to_string(),
                    Lane::default().with_arg("k").with_cards(vec![
                        Card::read_var("k"),
                        Card::string_card("winnie"),
                        Card::Equals,
                        Card::Return,
                    ]),
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
                let o = o.as_mut().unwrap();
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
                        Card::CreateTable,
                        Card::set_var("t"),
                        Card::scalar_int(1),
                        Card::read_var("t"),
                        Card::string_card("winnie"),
                        Card::SetProperty,
                        Card::scalar_int(2),
                        Card::read_var("t"),
                        Card::string_card("pooh"),
                        Card::SetProperty,
                        // call filter
                        Card::Function("cb".to_string()),
                        Card::read_var("t"),
                        Card::jump("map"),
                        Card::set_global_var("g_result"),
                    ]),
                ),
                (
                    "cb".to_string(),
                    Lane::default().with_arg("k").with_cards(vec![
                        Card::read_var("k"),
                        Card::string_card("winnie"),
                        Card::Equals,
                        Card::Return,
                    ]),
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
                let o = o.as_mut().unwrap();
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

    #[test]
    fn min_test() {
        let program = Module {
            imports: vec!["std.min".to_string()],
            lanes: vec![(
                "main".to_string(),
                Lane::default().with_cards(vec![
                    Card::CreateTable,
                    Card::set_var("t"),
                    Card::scalar_int(1),
                    Card::set_var("t.winnie"),
                    Card::scalar_int(2),
                    Card::set_var("t.pooh"),
                    Card::scalar_int(3),
                    Card::set_var("t.tiggers"),
                    // call min
                    Card::read_var("t"),
                    Card::jump("min"),
                    Card::set_global_var("g_result"),
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
        match result {
            Value::Integer(i) => {
                assert_eq!(i, 1);
            }
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }

    #[test]
    fn max_test() {
        let program = Module {
            imports: vec!["std.max".to_string()],
            lanes: vec![(
                "main".to_string(),
                Lane::default().with_cards(vec![
                    Card::CreateTable,
                    Card::set_var("t"),
                    Card::scalar_int(1),
                    Card::set_var("t.winnie"),
                    Card::ScalarFloat(2.4),
                    Card::set_var("t.pooh"),
                    Card::ScalarFloat(3.42),
                    Card::set_var("t.tiggers"),
                    // call max
                    Card::read_var("t"),
                    Card::jump("max"),
                    Card::set_global_var("g_result"),
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
        match result {
            Value::Real(i) => {
                assert_eq!(i, 3.42);
            }
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }

    #[test]
    fn max_empty_list_returns_nil_test() {
        let program = Module {
            imports: vec!["std.max".to_string()],
            lanes: vec![(
                "main".to_string(),
                Lane::default().with_cards(vec![
                    Card::CreateTable,
                    Card::jump("max"),
                    Card::set_global_var("g_result"),
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
        match result {
            Value::Nil => {}
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }

    #[test]
    fn min_by_key_test() {
        let program = Module {
            imports: vec!["std.min_by_key".to_string()],
            lanes: vec![
                (
                    "main".to_string(),
                    Lane::default().with_cards(vec![
                        Card::CreateTable,
                        Card::set_var("t"),
                        Card::scalar_int(1),
                        Card::set_var("t.winnie"),
                        Card::scalar_int(2),
                        Card::set_var("t.pooh"),
                        Card::scalar_int(3),
                        Card::set_var("t.tiggers"),
                        // call min
                        Card::Function("keyfn".to_string()),
                        Card::read_var("t"),
                        Card::jump("min_by_key"),
                        Card::set_global_var("g_result"),
                    ]),
                ),
                (
                    "keyfn".to_string(),
                    Lane::default()
                        .with_arg("key")
                        .with_arg("val")
                        .with_cards(vec![Card::read_var("val")]),
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
            Value::Integer(i) => {
                assert_eq!(i, 1);
            }
            a @ _ => panic!("Unexpected result: {a:?}"),
        }
    }
}
