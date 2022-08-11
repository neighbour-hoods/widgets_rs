use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

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
    pub async fn enable_app(&self, installed_app_id: String) -> Result<(), JsValue> {
        let tag: &str = "enableApp";
        let method: Function = Reflect::get(&self.js_ws, &JsValue::from_str(&tag))?.dyn_into()?;
        let payload: JsValue =
            mk_tagged_obj(tag, stringify!(installed_app_id), installed_app_id.into())?;
        let promise: Promise = method.call1(&self.js_ws, &payload)?.dyn_into()?;
        let future: JsFuture = promise.into();
        future.await?;
        Ok(())
    }
}

/// `tag` is more like "method name" here.
fn mk_tagged_obj(tag: &str, payload_key: &str, payload: JsValue) -> Result<JsValue, JsValue> {
    let target: JsValue = Object::new().dyn_into()?;
    assert!(Reflect::set(
        &target,
        &JsValue::from_str("tag"),
        &JsValue::from_str(tag)
    )?);
    assert!(Reflect::set(
        &target,
        &JsValue::from_str(payload_key),
        &payload
    )?);
    Ok(target)
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
