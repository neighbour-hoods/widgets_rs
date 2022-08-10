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

impl AdminWebsocket {
    pub async fn activate_app(&self, installed_app_id: String) -> Result<(), String> {
        match activate_app_js(&self.js_ws, installed_app_id).await {
            Ok(_null) => Ok(()),
            Err(js_err) => Err(format!("{:?}", js_err)),
        }
    }
}

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="connect")]
    async fn connect_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="activateApp")]
    async fn activate_app_js(this: &JsValue, installed_app_id: String) -> Result<JsValue, JsValue>;
}

pub async fn connect(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    match connect_js(url, timeout).await {
        Ok(js_ws) => Ok(AdminWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}
