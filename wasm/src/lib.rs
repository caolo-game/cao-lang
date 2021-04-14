use cao_lang::{compiler as caoc, prelude::*, vm::Vm};
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
///                 "ty": "ScalarInt" , "val": 1
///             },
///             {
///                 "ty": "Pass"
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

    let res = match caoc::compile(cu, ops) {
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
    pub program: Option<CaoProgram>,
    #[serde(rename = "compileError")]
    pub compile_error: Option<String>,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CompileOptions {}

impl Into<caoc::CompileOptions> for CompileOptions {
    fn into(self) -> caoc::CompileOptions {
        caoc::CompileOptions::new()
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct RunResult {}

#[wasm_bindgen]
impl RunResult {}

/// Runs the given compiled Cao-Lang program (output of `compile`).
///
/// Will run in a 'plain' Vm, no custom methods will be available!
#[wasm_bindgen(js_name = "runProgram")]
pub fn run_program(program: JsValue) -> Result<RunResult, JsValue> {
    let mut vm = Vm::new(()).expect("Failed to initialize VM");
    let program: CaoProgram = program.into_serde().map_err(err_to_js)?;
    vm.run(&program).map_err(err_to_js).map(|()| RunResult {})
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_serde(&format!("{:?}", e)).unwrap()
}
