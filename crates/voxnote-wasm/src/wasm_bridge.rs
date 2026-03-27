use wasm_bindgen::prelude::*;

/// JS → Rust 브릿지 — 노트 조회/검색 (로컬 AI 없음, cloud-providers만)

#[wasm_bindgen]
pub struct WasmBridge {
    // WASM 환경에서는 IndexedDB + OPFS를 사용
}

#[wasm_bindgen]
impl WasmBridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    /// 서버에서 노트 목록 조회 (동기화된 데이터)
    pub fn list_notes(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&Vec::<String>::new()).unwrap_or(JsValue::NULL)
    }

    /// 텍스트 검색
    pub fn search(&self, _query: &str) -> JsValue {
        serde_wasm_bindgen::to_value(&Vec::<String>::new()).unwrap_or(JsValue::NULL)
    }
}
