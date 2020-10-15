use cao_lang::compiler as caoc;
use wasm_bindgen::prelude::*;

/// Init the error handling of the library
#[wasm_bindgen(start)]
pub fn _start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
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

fn err_to_js(e: impl std::error::Error) -> JsValue {
    JsValue::from_serde(&format!("{:?}", e)).unwrap()
}
