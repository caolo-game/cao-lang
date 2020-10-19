use std::mem::take;

use cao_lang::{compiler as caoc, vm::VM};
use wasm_bindgen::prelude::*;

/// Init the error handling of the library
#[wasm_bindgen(start)]
pub fn _start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

/// Returns the compiled program on success or throws an error if compilation fails.
///
/// __Compilation unit schema:__
///
/// ```json
/// {
///     "lanes": [{
///         "name": "Foo",
///         "cards": [
///             {
///                 "ScalarInt": 1
///             }
///         ]
///     }]
/// }
/// ```
///
#[wasm_bindgen]
pub fn compile(compilation_unit: JsValue) -> Result<JsValue, JsValue> {
    let cu = compilation_unit
        .into_serde::<caoc::CompilationUnit>()
        .map_err(err_to_js)?;
    caoc::compile(None, cu)
        .map_err(err_to_js)
        .map(|res| JsValue::from_serde(&res).expect("Failed to serialize program"))
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct RunResult {
    #[wasm_bindgen(js_name = "returnCode")]
    pub return_code: i32,

    #[wasm_bindgen(skip)]
    pub history: Vec<cao_lang::vm::HistoryEntry>,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn history(&self) -> Box<[JsValue]> {
        self.history
            .iter()
            .map(JsValue::from_serde)
            .filter_map(|x| x.ok())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

/// Runs the given compiled Cao-Lang program (output of `compile`).
///
/// Will run in a 'plain' VM, no custom methods will be available!
#[wasm_bindgen(js_name = "runProgram")]
pub fn run_program(program: JsValue) -> Result<RunResult, JsValue> {
    let mut vm = VM::new(None, ());
    let program: cao_lang::prelude::CompiledProgram = program.into_serde().map_err(err_to_js)?;
    vm.run(&program).map_err(err_to_js).map(|res| {
        let history = take(&mut vm.history);
        RunResult {
            return_code: res,
            history,
        }
    })
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_serde(&format!("{:?}", e)).unwrap()
}
