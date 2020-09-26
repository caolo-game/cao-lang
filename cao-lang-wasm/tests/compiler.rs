//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use cao_lang_wasm::{ast_node::AstNode, compilation_unit::CompilationUnit, compile};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn can_compile_simple_program() {
    let start_node = serde_json::json! {{
        "instruction": { "Start": null },
        "child": 1
    }};
    let start_node = JsValue::from_serde(&start_node).unwrap();
    let start_node: AstNode = start_node.into_serde().unwrap();

    let scalar_node = serde_json::json! {{
        "instruction": { "ScalarInt": 69 }
    }};
    let scalar_node = JsValue::from_serde(&scalar_node).unwrap();
    let scalar_node: AstNode = scalar_node.into_serde().unwrap();

    let cu = CompilationUnit::new()
        .with_node(0, start_node)
        .with_node(1, scalar_node);

    let promise = compile(&cu);
    let future = JsFuture::from(promise);

    future.await.expect("Failed to compile");
}

#[wasm_bindgen_test]
fn check_error_returns_null_for_valid_node() {
    let node = serde_json::json! {{
        "instruction": { "ScalarInt": 69 }
        , "child": 420
        , "foo": "bar"
    }};
    let node = JsValue::from_serde(&node).unwrap();

    let res = AstNode::check_error(&node);

    assert!(res.is_none(), "Valid nodes should return null");
}

#[wasm_bindgen_test]
fn check_error_returns_error_for_invalid_node() {
    let node = serde_json::json! {{
        "instruction": { "Asd": {"a": "bar"} }
        , "children": [420, 123]
    }};
    let node = JsValue::from_serde(&node).unwrap();

    let res = AstNode::check_error(&node);

    assert!(res.is_some(), "Invalid nodes should not return null");
}
