//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;

use cao_lang_wasm::{compile, run_program, CompileResult};
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let cu = json!({
        "submodules": [],
        "imports": [],
        "lanes": [["main", {
            "name": "main",
            "arguments": [],
            "cards": [ {"ScalarInt": 69 } ]
        }]]
    });
    let result = compile(serde_json::to_string(&cu).unwrap(), None);

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}

/// The comile method will return an object w/ a compilationError if the compilation fails, rather
/// than throwing an exception
#[wasm_bindgen_test]
fn compiler_returns_error_not_exception() {
    let cu = json!({
        "submodules": [],
        "imports": [],
        "lanes": [["main", {
            "name": "main",
            "arguments": [],
            "cards": [ {"Call": {"lane_name":"42", "args":[]} } ]
        }]]
    });
    let output =
        compile(serde_json::to_string(&cu).unwrap(), None).expect("Compile returned error");
    let output: CompileResult = output
        .into_serde()
        .expect("Failed to deserialize compiler output");

    match output {
        CompileResult::Program(_) => panic!("Expected a compile error"),
        CompileResult::CompileError(_) => {}
    }
}

#[wasm_bindgen_test]
fn can_run_simple_program() {
    let cu = json!({
        "submodules": [],
        "imports": [],
        "lanes": [[ "main", {
            "name": "main",
            "arguments": [],
            "cards": [
            
            { "SetGlobalVar": {
                "name": "g_pogman",
                "value": {  "StringLiteral": "Poggers" }
                }
            }
            ]
        }]]
    });
    let output = compile(serde_json::to_string(&cu).unwrap(), None).expect("failed to run compile");

    let output: CompileResult = output
        .into_serde()
        .expect("Failed to deserialize compiler output");

    let program = match output {
        CompileResult::Program(p) => p,
        CompileResult::CompileError(err) => panic!("Failed to compile {:?}", err),
    };

    let prog_js = JsValue::from_serde(&program).expect("serialize");

    run_program(prog_js).expect("Failed to run");
}
