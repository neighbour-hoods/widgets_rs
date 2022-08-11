use js_sys::{Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Clone, Debug)]
pub struct AdminWebsocket {
    pub js_ws: JsValue,
}

impl From<AdminWebsocket> for JsValue {
    fn from(ws: AdminWebsocket) -> Self {
        ws.js_ws
    }
}

impl AdminWebsocket {
    pub async fn activate_app(&self, installed_app_id: String) -> Result<(), String> {
        let method = Reflect::get(&self.js_ws, &JsValue::from_str("activateApp"))
            .expect("activateApp method should exist");
        todo!()
    }
}

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="connect")]
    async fn connect_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

pub async fn connect(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    match connect_js(url, timeout).await {
        Ok(js_ws) => Ok(AdminWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}
