//! Cao-Lang standard library
//!
//! The standard library is injected into every `Module` at compilation time.
//! Standard functions can be imported via the `std` module

#[cfg(test)]
mod tests;

use crate::{
    compiler::{Card, ForEach, Function, Module},
    procedures::ExecutionErrorPayload,
    value::Value,
    vm::{runtime::cao_lang_object::CaoLangObjectBody, Vm},
};

/// Given a table and a callback that returns a bool create a new table whith the items that return
/// true
pub fn filter() -> Function {
    Function::default()
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
pub fn any() -> Function {
    Function::default()
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
pub fn map() -> Function {
    Function::default()
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

fn minmax(minimax: &str) -> Function {
    Function::default()
        .with_arg("iterable")
        .with_card(Card::return_card(Card::call_function(
            minimax,
            vec![
                Card::function_value("row_to_value"),
                Card::read_var("iterable"),
            ],
        )))
}

/// Return the smallest value in the table, or nil if the table is empty
pub fn min() -> Function {
    minmax("min_by_key")
}

/// Return the largest value in the table, or nil if the table is empty
pub fn max() -> Function {
    minmax("max_by_key")
}

pub fn sorted() -> Function {
    Function::default()
        .with_arg("iterable")
        .with_card(Card::return_card(Card::call_function(
            "sorted_by_key",
            vec![
                Card::function_value("row_to_value"),
                Card::read_var("iterable"),
            ],
        )))
}

pub fn native_minmax<T, const LESS: bool>(
    vm: &mut Vm<T>,
    iterable: Value,
    key_fn: Value,
) -> Result<Value, ExecutionErrorPayload> {
    match iterable {
        Value::Nil | Value::Integer(_) | Value::Real(_) => return Ok(iterable),
        Value::Object(o) => unsafe {
            match &o.as_ref().body {
                CaoLangObjectBody::Table(t) => {
                    let Some(first) = t.iter().next() else {
                        return Ok(Value::Nil);
                    };
                    vm.stack_push(*first.1)?;
                    vm.stack_push(*first.0)?;
                    let mut max_key = vm.run_function(key_fn)?;
                    let mut i = 0;

                    for (j, (k, value)) in t.iter().enumerate().skip(1) {
                        vm.stack_push(*value)?;
                        vm.stack_push(*k)?;
                        let key = vm.run_function(key_fn)?;
                        if if LESS { key < max_key } else { key > max_key } {
                            i = j;
                            max_key = key;
                        }
                    }
                    let k = t.nth_key(i);
                    let v = *t.get(&k).unwrap();
                    let mut result = vm.init_table()?;
                    let t = result.0.as_mut().as_table_mut().unwrap();
                    t.insert(vm.init_string("key")?, k)?;
                    t.insert(vm.init_string("value")?, v)?;

                    return Ok(Value::Object(result.0));
                }
                CaoLangObjectBody::String(_)
                | CaoLangObjectBody::Function(_)
                | CaoLangObjectBody::Closure(_)
                | CaoLangObjectBody::Upvalue(_)
                | CaoLangObjectBody::NativeFunction(_) => return Ok(iterable),
            }
        },
    }
}

pub fn native_sorted<T>(
    vm: &mut Vm<T>,
    iterable: Value,
    key_fn: Value,
) -> Result<Value, ExecutionErrorPayload> {
    match iterable {
        Value::Nil | Value::Integer(_) | Value::Real(_) => return Ok(iterable),
        Value::Object(o) => unsafe {
            match &o.as_ref().body {
                CaoLangObjectBody::Table(t) => {
                    // TODO:
                    // sort in place?
                    let mut result = Vec::with_capacity(t.len());
                    for (k, v) in t.iter() {
                        vm.stack_push(*v)?;
                        vm.stack_push(*k)?;
                        let key = vm.run_function(key_fn)?;
                        result.push((key, k, v));
                    }
                    result.sort_by(|(a, _, _), (b, _, _)| {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let mut out = vm.init_table()?;
                    let t = out.as_table_mut().unwrap();
                    for (_, k, v) in result {
                        t.insert(*k, *v)?;
                    }
                    Ok(Value::Object(out.0))
                }
                CaoLangObjectBody::String(_) // TODO: define sort for strings?
                | CaoLangObjectBody::Function(_)
                | CaoLangObjectBody::Closure(_)
                | CaoLangObjectBody::Upvalue(_)
                | CaoLangObjectBody::NativeFunction(_) => return Ok(iterable),
            }
        },
    }
}

pub fn native_to_array<T>(vm: &mut Vm<T>, iterable: Value) -> Result<Value, ExecutionErrorPayload> {
    match iterable {
        Value::Nil | Value::Integer(_) | Value::Real(_) => return Ok(iterable),
        Value::Object(o) => unsafe {
            match &o.as_ref().body {
                CaoLangObjectBody::Table(t) => {
                    let mut out = vm.init_table()?;
                    let table = out.as_table_mut().unwrap();
                    for (i, (_, val)) in t.iter().enumerate() {
                        table.insert(i as i64, *val)?;
                    }
                    Ok(Value::Object(out.0))
                }
                CaoLangObjectBody::String(_)
                | CaoLangObjectBody::Function(_)
                | CaoLangObjectBody::Closure(_)
                | CaoLangObjectBody::Upvalue(_)
                | CaoLangObjectBody::NativeFunction(_) => return Ok(iterable),
            }
        },
    }
}

/// Return the smallest value in the table, or nil if the table is empty
pub fn min_by_key() -> Function {
    Function::default()
        .with_arg("iterable")
        .with_arg("key_function")
        .with_card(Card::return_card(Card::call_native(
            "__min",
            vec![Card::read_var("iterable"), Card::read_var("key_function")],
        )))
}

pub fn sorted_by_key() -> Function {
    Function::default()
        .with_arg("iterable")
        .with_arg("key_function")
        .with_card(Card::return_card(Card::call_native(
            "__sort",
            vec![Card::read_var("iterable"), Card::read_var("key_function")],
        )))
}

pub fn max_by_key() -> Function {
    Function::default()
        .with_arg("iterable")
        .with_arg("key_function")
        .with_card(Card::return_card(Card::call_native(
            "__max",
            vec![Card::read_var("iterable"), Card::read_var("key_function")],
        )))
}

/// A (key, value) function that returns the value given
pub fn value_key_fn() -> Function {
    Function::default()
        .with_arg("_key")
        .with_arg("val")
        .with_card(Card::return_card(Card::read_var("val")))
}

pub fn to_array() -> Function {
    Function::default()
        .with_arg("iterable")
        .with_card(Card::return_card(Card::call_native(
            "__to_array",
            vec![Card::read_var("iterable")],
        )))
}

pub fn standard_library() -> Module {
    let mut module = Module::default();
    module.functions.push(("to_array".to_string(), to_array()));
    module.functions.push(("filter".to_string(), filter()));
    module.functions.push(("any".to_string(), any()));
    module.functions.push(("map".to_string(), map()));
    module.functions.push(("min".to_string(), min()));
    module.functions.push(("max".to_string(), max()));
    module
        .functions
        .push(("min_by_key".to_string(), min_by_key()));
    module
        .functions
        .push(("max_by_key".to_string(), max_by_key()));
    module
        .functions
        .push(("sorted_by_key".to_string(), sorted_by_key()));
    module.functions.push(("sorted".to_string(), sorted()));
    module
        .functions
        .push(("row_to_value".to_string(), value_key_fn()));
    module
}
