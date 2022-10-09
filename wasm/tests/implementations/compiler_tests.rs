//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;
use serde::Serialize;

use cao_lang_wasm::{basic_schema, compile, run_program, CompileResult};

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let cu = json!({
        "submodules": {},
        "imports": [],
        "cards": {
            "1": {"ScalarInt": 69 }
        },
        "lanes": {"main": {
            "name": "main",
            "arguments": [],
            "cards": [1]
        }}
    });
    let result = compile(serde_wasm_bindgen::to_value(&cu).unwrap(), None);

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}

/// The comile method will return an object w/ a compilationError if the compilation fails, rather
/// than throwing an exception
#[wasm_bindgen_test]
fn compiler_returns_error_not_exception() {
    let cu = json!({
        "submodules": {},
        "imports": [],
        "cards": {
            "1":{"Jump": "42" }
        },
        "lanes": {"main": {
            "name": "main",
            "arguments": [],
            "cards": [ 1 ]
        }}
    });
    let output =
        compile(serde_wasm_bindgen::to_value(&cu).unwrap(), None).expect("Compile returned error");
    let output: CompileResult =
        serde_wasm_bindgen::from_value(output).expect("Failed to deserialize compiler output");

    match output {
        CompileResult::Program(_) => panic!("Expected a compile error"),
        CompileResult::CompileError(_) => {}
    }
}

#[wasm_bindgen_test]
fn can_run_simple_program() {
    let cu = json!({
        "submodules": {},
        "imports": [],
        "cards": {
            "1": { "StringLiteral": "Poggers" },
            "2": { "SetGlobalVar": "g_pogman" }
        },
        "lanes": { "main": {
            "name": "main",
            "arguments": [],
            "cards": [ 1, 2 ]
        }}
    });
    let ser = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    let cu = cu.serialize(&ser).unwrap();
    let output = compile(cu, None).expect("failed to run compile");

    let output: CompileResult =
        serde_wasm_bindgen::from_value(output).expect("Failed to deserialize compiler output");

    let program = match output {
        CompileResult::Program(p) => p,
        CompileResult::CompileError(err) => panic!("Failed to compile {:?}", err),
    };

    let prog_js = serde_wasm_bindgen::to_value(&program).expect("serialize");

    run_program(prog_js).expect("Failed to run");
}

#[wasm_bindgen_test]
fn can_query_the_schema() {
    // smoke test
    let _res = basic_schema();
}
