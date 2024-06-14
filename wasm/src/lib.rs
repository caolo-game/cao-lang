use cao_lang::{
    compiler::{self as caoc, FunctionCardIndex},
    prelude::*,
    vm::Vm,
};
use wasm_bindgen::prelude::*;

/// Init the error handling of the library
#[wasm_bindgen(start)]
pub fn _start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen(js_name = "versionInfo")]
pub fn version_info() -> VersionInfo {
    VersionInfo {}
}

#[wasm_bindgen]
pub struct VersionInfo {}

#[wasm_bindgen]
impl VersionInfo {
    #[wasm_bindgen(getter)]
    pub fn native(&self) -> String {
        cao_lang::version::VERSION_STR.to_string()
    }
}

#[wasm_bindgen]
pub struct CaoLangProgram {
    inner: caoc::CaoProgram,
}

#[wasm_bindgen]
impl CaoLangProgram {
    #[wasm_bindgen]
    pub fn from_js(compilation_unit: JsValue) -> Result<CaoLangProgram, JsValue> {
        let inner = serde_wasm_bindgen::from_value(compilation_unit).map_err(err_to_js)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen]
    pub fn get_card(&self, function: u32, card: &[u32]) -> JsValue {
        let mut card_index = FunctionCardIndex::default();
        for i in card {
            card_index.indices.push(*i);
        }
        let idx = CardIndex {
            function: function as usize,
            card_index,
        };

        let card = self.inner.get_card(&idx).ok();

        serde_wasm_bindgen::to_value(&card).unwrap()
    }

    #[wasm_bindgen]
    pub fn set_card(&mut self, function: u32, card: &[u32], value: JsValue) -> Result<(), JsValue> {
        let mut card_index = FunctionCardIndex::default();
        for i in card {
            card_index.indices.push(*i);
        }
        let idx = CardIndex {
            function: function as usize,
            card_index,
        };

        let result = self.inner.get_card_mut(&idx).map_err(err_to_js)?;

        *result = serde_wasm_bindgen::from_value(value).map_err(err_to_js)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn num_functions(&self) -> u32 {
        self.inner.functions.len() as u32
    }

    /// Return the number of cards in a given function or null if the function does not exist
    #[wasm_bindgen]
    pub fn num_cards(&self, function: u32) -> Option<u32> {
        self.inner
            .functions
            .get(function as usize)
            .map(|(_, f)| f.cards.len() as u32)
    }
}

/// ## Compilation errors:
///
/// The `compile` function will return an object with a `compilationError` if the compilation fails, rather
/// than throwing an exception. Exceptions are thrown on contract violation, e.g. passing in an
/// object that can not be deserialized into a compilable object.
///
#[wasm_bindgen]
pub fn compile(
    compilation_unit: &CaoLangProgram,
    compile_options: Option<CompileOptions>,
) -> Result<JsValue, JsValue> {
    let cu = compilation_unit.inner.clone();
    let ops: Option<caoc::CompileOptions> = compile_options.map(|ops| ops.into());

    let res = match caoc::compile(cu, ops) {
        Ok(res) => CompileResult::Program(res),
        Err(err) => CompileResult::CompileError(err.to_string()),
    };

    let res = serde_wasm_bindgen::to_value(&res).expect("failed to serialize result");
    Ok(res)
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "ty", content = "val")]
#[serde(rename_all = "camelCase")]
pub enum CompileResult {
    Program(CaoCompiledProgram),
    CompileError(String),
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CompileOptions {}

impl From<CompileOptions> for caoc::CompileOptions {
    fn from(_: CompileOptions) -> Self {
        caoc::CompileOptions::new()
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct RunResult {}

#[wasm_bindgen]
impl RunResult {}

fn cao_lang_log(_vm: &mut Vm<()>, val: Value) -> Result<Value, ExecutionErrorPayload> {
    match val {
        Value::Nil => log::info!("Console log: nil"),
        Value::Object(o) => unsafe {
            let o = o.as_ref();
            match &o.body {
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::String(s) => {
                    let pl = s.as_str();
                    log::info!("Console log: {pl}");
                }
                // TODO: log recursively
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::Table(_) => todo!(),
                // TODO: more information
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::Function(_) => todo!(),
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::NativeFunction(_) => {
                    todo!()
                }
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::Closure(_) => todo!(),
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::Upvalue(_) => todo!(),
            }
        },
        Value::Integer(pl) => log::info!("Console log: {pl}"),
        Value::Real(pl) => log::info!("Console log: {pl}"),
    }
    Ok(Value::Nil)
}

/// Runs the given compiled Cao-Lang program (output of `compile`).
///
/// Will run in a 'plain' Vm, no custom methods will be available!
#[wasm_bindgen(js_name = "runProgram")]
pub fn run_program(program: JsValue) -> Result<RunResult, JsValue> {
    let mut vm = Vm::new(()).expect("Failed to initialize VM");
    vm.register_native_stdlib().expect("Failed to init stdlib");
    vm.register_native_function("log", into_f1(cao_lang_log))
        .expect("Failed to register log function");
    let program: CaoCompiledProgram = serde_wasm_bindgen::from_value(program).map_err(err_to_js)?;
    vm.run(&program).map_err(err_to_js).map(|_| RunResult {})
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_str(&format!("{:?}", e))
}
