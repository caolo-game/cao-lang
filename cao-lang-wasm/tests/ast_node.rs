#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use cao_lang_wasm::ast_node::AstNode;
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn can_update_value() {
    let node = serde_json::json! {{
        "instruction": { "ScalarInt": 42 },
        "child": 1
    }};
    let node = JsValue::from_serde(&node).unwrap();
    let mut node: AstNode = node.into_serde().unwrap();

    let new_value = serde_json::json! {42};
    let new_value = JsValue::from_serde(&new_value).unwrap();

    node.set_value(new_value).unwrap();
}

#[wasm_bindgen_test]
fn raises_error_on_invalid_value() {
    let node = serde_json::json! {{
        "instruction": { "ScalarInt": 42 },
        "child": 1
    }};
    let node = JsValue::from_serde(&node).unwrap();
    let mut node: AstNode = node.into_serde().unwrap();

    let new_value = serde_json::json! {"Foo"};
    let new_value = JsValue::from_serde(&new_value).unwrap();

    node.set_value(new_value).unwrap_err();
}
