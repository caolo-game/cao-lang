//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;

use cao_lang_wasm::{basic_schema, compile, run_program, CompileResult};
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let cu = json!({
        "submodules": {},
        "lanes": {"main": {
            "name": "main",
            "cards": [ {"ty": "ScalarInt", "val": 69 } ]
        }}
    });
    let result = compile(JsValue::from_serde(&cu).unwrap(), None);

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}

/// The comile method will return an object w/ a compilationError if the compilation fails, rather
/// than throwing an exception
#[wasm_bindgen_test]
fn compiler_returns_error_not_exception() {
    let cu = json!({
        "submodules": {},
        "lanes": {"main": {
            "name": "main",
            "cards": [ {"ty": "Jump", "val": "42" } ]
        }}
    });
    let output = compile(JsValue::from_serde(&cu).unwrap(), None).expect("Compile returned error");
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
        "submodules": {},
        "lanes": { "main": {
            "name": "main",
            "cards": [
            { "ty": "StringLiteral", "val": "Poggers" }
            , {"ty": "SetGlobalVar", "val": "g_pogman" }
            ]
        }}
    });
    let output = compile(JsValue::from_serde(&cu).unwrap(), None).expect("failed to run compile");

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

#[wasm_bindgen_test]
fn can_query_the_schema() {
    // smoke test
    let _res = basic_schema();
}

// TODO
// #[wasm_bindgen_test]
// fn test_mandlebrot() {
//     const PROG: &str = include_str!("mandelbrot.json");
//
//     let cu: serde_json::Value = serde_json::from_str(PROG).unwrap();
//     let output = compile(JsValue::from_serde(&cu).unwrap(), None).expect("failed to run compile");
//
//     let output: CompileResult = output.into_serde().unwrap();
//
//     assert!(output.compile_error.is_none());
//     assert!(output.program.is_some());
//
//     let mut vm = Vm::new(());
//     // push the input y,x
//     vm.runtime_data.stack.push(Scalar::Floating(42.0)).unwrap();
//     vm.runtime_data.stack.push(Scalar::Floating(69.0)).unwrap();
//     vm.run(output.program.as_ref().unwrap())
//         .expect("mandlebrot program failed");
//
//     let res = vm.runtime_data.stack.pop();
//     todo!("boi {:?}", res)
// }
