use cao_lang::compiler as caoc;
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
///     "nodes": {
///         "0": {
///             "node": {
///                 "Start": null
///             },
///             "child": 1
///         },
///         "1": {
///             "node": {
///                 "ScalarInt": 1
///             }
///         }
///     }
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
        .map(|res| JsValue::from_serde(&res).unwrap())
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct CompileResult {
    #[wasm_bindgen(js_name = "returnCode")]
    pub return_code: i32,

    history: Vec<cao_lang::vm::HistoryEntry>,
}

#[wasm_bindgen]
impl CompileResult {
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
#[wasm_bindgen(js_name="runProgram")]
pub fn run_program(program: JsValue) -> Result<CompileResult, JsValue> {
    let mut vm = cao_lang::vm::VM::new(None, ());
    let program: cao_lang::CompiledProgram = program.into_serde().map_err(err_to_js)?;
    vm.run(&program).map_err(err_to_js).map(|res| {
        let history = std::mem::take(&mut vm.history);
        CompileResult {
            return_code: res,
            history,
        }
    })
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_serde(&format!("{:?}", e)).unwrap()
}
