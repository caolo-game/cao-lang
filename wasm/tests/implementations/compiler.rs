//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;

use cao_lang_wasm::compile;
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let start_node = json! {{
        "node": { "Start": null },
        "child": 1
    }};

    let scalar_node = json! {{
        "node": { "ScalarInt": 69 }
    }};

    let cu = json!({
        "nodes": json!({
            "0":start_node,
            "1":scalar_node,
        })
    });
    let result = compile(JsValue::from_serde(&cu).unwrap());

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}
