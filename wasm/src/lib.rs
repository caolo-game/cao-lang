pub mod ast_node;
pub mod compilation_unit;

use compilation_unit::CompilationUnit;

use cao_lang::compiler as cc;
use wasm_bindgen::prelude::*;

/// Init the error handling of the library
#[wasm_bindgen(start)]
pub fn _start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Returns `null` on successful compilation or an error otherwise
#[wasm_bindgen]
pub fn compile(compilation_unit: &CompilationUnit) -> JsValue {
    let cu = compilation_unit.inner.clone();

    let err = if let Err(err) = cc::compile(None, cu)
        .map_err(|e| format!("{}", e))
        .map_err(|e| JsValue::from_serde(&e).unwrap())
    {
        Some(err)
    } else {
        None
    };
    err.into()
}

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_serde(&format!("{:?}", e)).unwrap()
}
