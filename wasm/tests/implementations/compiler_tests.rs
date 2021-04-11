//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use wasm_bindgen_test::*;

use cao_lang_wasm::{compile, run_program, CompileResult};
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
fn can_compile_simple_program() {
    let cu = json!({
        "lanes": [{
            "name": "PogChamp",
            "cards": [ {"ty": "ScalarInt", "val": 69 } ]
        }]
    });
    let result = compile(JsValue::from_serde(&cu).unwrap(), None);

    assert!(result.is_ok(), "Failed to compile {:?}", result);
}

#[wasm_bindgen_test]
fn can_run_simple_program() {
    let cu = json!({
        "lanes": [ {
            "name": "Foo",
            "cards": [
            { "ty": "StringLiteral", "val": "Poggers" }
            , {"ty": "SetGlobalVar", "val": "pogman" }
            ]
        }]
    });
    let output = compile(JsValue::from_serde(&cu).unwrap(), None).expect("failed to run compile");

    let output: CompileResult = output.into_serde().unwrap();

    assert!(output.compile_error.is_none());
    assert!(output.program.is_some());

    run_program(JsValue::from_serde(&output.program).expect("serialize")).expect("Failed to run");
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
