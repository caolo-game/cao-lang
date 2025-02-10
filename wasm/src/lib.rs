use std::fmt::Write as _;

use cao_lang::{compiler as caoc, prelude::*, vm::Vm};
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
    /// Construct a CaoLangProgram from an ECMAScript object
    #[wasm_bindgen]
    pub fn from_js(compilation_unit: JsValue) -> Result<CaoLangProgram, JsValue> {
        let inner = serde_wasm_bindgen::from_value(compilation_unit).map_err(err_to_js)?;
        Ok(Self { inner })
    }

    /// Construct a CaoLangProgram from a JSON string
    #[wasm_bindgen]
    pub fn from_json(compilation_unit: String) -> Result<CaoLangProgram, JsValue> {
        let inner = serde_json::from_str(&compilation_unit).map_err(err_to_js)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen]
    pub fn get_card(&self, function: u32, card: &[u32]) -> JsValue {
        let idx = CardIndex::from_slice(function as usize, card);

        let card = self.inner.get_card(&idx).ok();

        serde_wasm_bindgen::to_value(&card).unwrap()
    }

    /// Return the card if it was removed or null if not found
    #[wasm_bindgen]
    pub fn remove_card(&mut self, function: u32, card: &[u32]) -> JsValue {
        let idx = CardIndex::from_slice(function as usize, card);

        let card = self.inner.remove_card(&idx).ok();

        serde_wasm_bindgen::to_value(&card).unwrap()
    }

    #[wasm_bindgen]
    pub fn set_card(&mut self, function: u32, card: &[u32], value: JsValue) -> Result<(), JsValue> {
        let idx = CardIndex::from_slice(function as usize, card);

        let result = self.inner.get_card_mut(&idx).map_err(err_to_js)?;

        *result = serde_wasm_bindgen::from_value(value).map_err(err_to_js)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn swap_cards(
        &mut self,
        lhs_function: u32,
        lhs_card: &[u32],
        rhs_function: u32,
        rhs_card: &[u32],
    ) -> Result<(), JsValue> {
        let lhs = CardIndex::from_slice(lhs_function as usize, lhs_card);
        let rhs = CardIndex::from_slice(rhs_function as usize, rhs_card);

        self.inner.swap_cards(&lhs, &rhs).map_err(err_to_js)?;

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

#[derive(Debug, Clone, Default)]
struct Context {
    logs: String,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct RunResult {
    logs: String,
    result: Result<(), JsValue>,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn logs(&self) -> String {
        self.logs.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> JsValue {
        match self.result.as_ref() {
            Ok(_) => JsValue::NULL,
            Err(err) => err.clone(),
        }
    }
}

fn cao_lang_log(vm: &mut Vm<Context>, val: Value) -> Result<Value, ExecutionErrorPayload> {
    match val {
        Value::Nil => writeln!(&mut vm.get_aux_mut().logs, "nil").unwrap(),
        Value::Object(o) => unsafe {
            let o = o.as_ref();
            match &o.body {
                cao_lang::vm::runtime::cao_lang_object::CaoLangObjectBody::String(s) => {
                    let pl = s.as_str();
                    writeln!(&mut vm.get_aux_mut().logs, "{pl}").unwrap();
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
        Value::Integer(pl) => writeln!(&mut vm.get_aux_mut().logs, "{pl}").unwrap(),
        Value::Real(pl) => writeln!(&mut vm.get_aux_mut().logs, "{pl}").unwrap(),
    }
    Ok(Value::Nil)
}

/// Runs the given compiled Cao-Lang program (output of `compile`).
///
/// Will run in a 'plain' Vm, no custom methods will be available!
#[wasm_bindgen(js_name = "runProgram")]
pub fn run_program(program: JsValue) -> Result<RunResult, JsValue> {
    let mut vm = Vm::new(Context::default()).expect("Failed to initialize VM");
    vm.register_native_stdlib().expect("Failed to init stdlib");
    vm.register_native_function("log", into_f1(cao_lang_log))
        .expect("Failed to register log function");
    let program: CaoCompiledProgram = serde_wasm_bindgen::from_value(program).map_err(err_to_js)?;
    let result = vm.run(&program).map_err(err_to_js).map(drop);

    Ok(RunResult {
        logs: vm.unwrap_aux().logs,
        result,
    })
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_str(&format!("{:?}", e))
}
