use cao_lang::{compiler as caoc, prelude::*, vm::Vm};
use wasm_bindgen::prelude::*;

/// Init the error handling of the library
#[wasm_bindgen(start)]
pub fn _start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

/// Return a list of types that can appear on Property descriptions
#[wasm_bindgen(js_name = "caoLangPropertyTypes")]
pub fn cao_lang_prop_types() -> Box<[JsValue]> {
    caoc::card_description::PropertyName::all_props()
        .iter()
        .map(|name| name.to_str())
        .map(|name|JsValue::from_str(name))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[wasm_bindgen(js_name = "versionInfo")]
pub fn version_info() -> VersionInfo {
    VersionInfo {}
}

#[wasm_bindgen]
pub struct VersionInfo {}

#[wasm_bindgen]
impl VersionInfo {
    #[wasm_bindgen(method, getter)]
    pub fn native(&self) -> String {
        cao_lang::version::VERSION_STR.to_string()
    }
}

/// Return the basic cards accepted by this Cao-Lang instance
#[wasm_bindgen(js_name = "basicSchema")]
pub fn basic_schema() -> Vec<JsValue> {
    cao_lang::compiler::card_description::get_instruction_descriptions()
        .into_iter()
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect()
}

/// ## Compilation errors:
///
/// The `compile` function will return an object with a `compilationError` if the compilation fails, rather
/// than throwing an exception. Exceptions are thrown on contract violation, e.g. passing in an
/// object that can not be deserialized into a compilable object.
///
/// ```json
/// {
///     "ty": "compileError",
///     "val": "someerror"
/// }
/// ```
///
/// `ty` is one of:
///
/// - `program`
/// - `compileError`
///
///  ## Compilation unit (input) example:
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
        .into_serde::<caoc::CaoIr>()
        .map_err(err_to_js)?;
    let ops: Option<caoc::CompileOptions> = compile_options.map(|ops| ops.into());

    let res = match caoc::compile(cu, ops) {
        Ok(res) => CompileResult::Program(res),
        Err(err) => CompileResult::CompileError(err.to_string()),
    };

    let res = JsValue::from_serde(&res).expect("failed to serialize result");
    Ok(res)
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "ty", content = "val")]
#[serde(rename_all = "camelCase")]
pub enum CompileResult {
    Program(CaoProgram),
    CompileError(String),
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CompileOptions {}

impl From<CompileOptions > for caoc::CompileOptions{
    fn from(_: CompileOptions) -> Self {
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
