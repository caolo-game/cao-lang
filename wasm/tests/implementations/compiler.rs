//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;

use cao_lang::prelude::*;
use cao_lang_wasm::{compile, run_program};
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let cu = json!({
        "lanes": [{
        "name": "PogChamp",
        "cards": [ { "ScalarInt": 69 } ]
        }]
    });
    let result = compile(JsValue::from_serde(&cu).unwrap());

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}

#[wasm_bindgen_test]
fn can_run_simple_program() {
    let cu = json!({
        "lanes": [ {
            "name": "Foo",
            "cards": [ { "StringLiteral": "Poggers" } ]
        }]
    });
    let program = compile(JsValue::from_serde(&cu).unwrap()).unwrap();

    let result = run_program(program).expect("Failed to run");

    assert_eq!(
        &result.history[..1],
        &[cao_lang::vm::HistoryEntry {
            id: NodeId { lane: 0, pos: 0 },
            instr: Instruction::StringLiteral
        }]
    )
}
