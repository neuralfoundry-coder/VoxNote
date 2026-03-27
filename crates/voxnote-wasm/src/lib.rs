use wasm_bindgen::prelude::*;

mod wasm_bridge;

#[wasm_bindgen]
pub fn init() {
    // WASM 초기화
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
