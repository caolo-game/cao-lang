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

/// Returns an object.
/// ```json
/// {
///     "program": null,
///     "compileError": "someerror"
/// }
/// ```
/// Only 1 of the fields will be non-null, depending on the outcome.
///
/// __Compilation unit example:__
///
/// ```json
/// {
///     "lanes": [{
///         "name": "Foo",
///         "cards": [
///             {
///                 "ScalarInt": 1
///             },
///             {
///                 "Pass": null
///             }
///         ]
///     }]
/// }
/// ```
///
#[wasm_bindgen]
pub fn compile(
    compilation_unit: JsValue,
    compile_options: Option<CompileOptions>,
) -> Result<JsValue, JsValue> {
    let cu = compilation_unit
        .into_serde::<caoc::CompilationUnit>()
        .map_err(err_to_js)?;
    let ops: Option<caoc::CompileOptions> = compile_options.map(|ops| ops.into());

    let res = match caoc::compile(None, cu, ops) {
        Ok(res) => CompileResult {
            program: Some(res),
            compile_error: None,
        },
        Err(err) => CompileResult {
            program: None,
            compile_error: Some(err.to_string()),
        },
    };

    let res = JsValue::from_serde(&res).expect("failed to serialize result");
    Ok(res)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CompileResult {
    pub program: Option<cao_lang::program::CompiledProgram>,
    #[serde(rename = "compileError")]
    pub compile_error: Option<String>,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CompileOptions {
    /// Emit breadcrumbs / history when executing this program
    pub breadcrumbs: bool,
}

impl Into<caoc::CompileOptions> for CompileOptions {
    fn into(self) -> caoc::CompileOptions {
        caoc::CompileOptions::new().with_breadcrumbs(self.breadcrumbs)
    }
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
