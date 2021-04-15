#![cfg(test)]
#![cfg(target_arch = "wasm32")]

pub mod implementations;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);
