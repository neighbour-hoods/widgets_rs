use wasm_bindgen::prelude::*;

pub struct AdminWebsocket;

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn connect(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

pub async fn connect_wrapper(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    todo!()
}
