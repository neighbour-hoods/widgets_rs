use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

use macros::generate_call;

#[derive(Clone, Debug)]
pub struct AdminWebsocket {
    pub js_ws: JsValue,
}

impl From<AdminWebsocket> for JsValue {
    fn from(ws: AdminWebsocket) -> Self {
        ws.js_ws
    }
}

#[generate_call]
#[derive(Clone, Debug)]
pub enum AdminWsCmd {
    EnableApp { installed_app_id: String },
    DisableApp { installed_app_id: String },
    UninstallApp { installed_app_id: String },
    GenerateAgentPubKey,
    ListDnas,
    ListCellIds,
    ListActiveApps,
    AttachAppInterface { port: u16 },
}

#[derive(Clone, Debug)]
pub enum AdminWsCmdResponse {
    EnableApp(JsValue),
    DisableApp(JsValue),
    UninstallApp(JsValue),
    GenerateAgentPubKey(JsValue),
    ListDnas(JsValue),
    ListCellIds(JsValue),
    ListActiveApps(JsValue),
    AttachAppInterface(JsValue),
}

fn parse_admin_ws_cmd_response(val: JsValue, tag: String) -> AdminWsCmdResponse {
    match tag.as_str() {
        "EnableApp" => AdminWsCmdResponse::EnableApp(val),
        "DisableApp" => AdminWsCmdResponse::DisableApp(val),
        "UninstallApp" => AdminWsCmdResponse::UninstallApp(val),
        "GenerateAgentPubKey" => AdminWsCmdResponse::GenerateAgentPubKey(val),
        "ListDnas" => AdminWsCmdResponse::ListDnas(val),
        "ListCellIds" => AdminWsCmdResponse::ListCellIds(val),
        "ListActiveApps" => AdminWsCmdResponse::ListActiveApps(val),
        "AttachAppInterface" => AdminWsCmdResponse::AttachAppInterface(val),
        other => panic!(
            "parse_admin_ws_cmd_response: impossible: received unknown tag: {}",
            other
        ),
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
