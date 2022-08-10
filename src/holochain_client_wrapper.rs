use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
pub struct AdminWebsocket {
    js_ws: JsValue
}

impl From<AdminWebsocket> for JsValue {
    fn from(ws: AdminWebsocket) -> Self {
        ws.js_ws
    }
}

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket)]
    async fn connect(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

pub async fn connect_wrapper(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    match connect(url, timeout).await {
        Ok(js_ws) => Ok(AdminWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}
