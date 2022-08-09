use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    pub type AdminWebsocket;

    #[wasm_bindgen(constructor)]
    pub fn connect(url: String, timeout: Option<u32>) -> AdminWebsocket;
}
