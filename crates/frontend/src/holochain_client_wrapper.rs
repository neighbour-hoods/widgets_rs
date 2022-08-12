use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

use macros::generate_call;

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="connect")]
    async fn connect_admin_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = AppWebsocket, js_name="connect")]
    async fn connect_app_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

////////////////////////////////////////////////////////////////////////////////
// AdminWebsocket
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AdminWebsocket {
    pub js_ws: JsValue,
}

impl From<AdminWebsocket> for JsValue {
    fn from(ws: AdminWebsocket) -> Self {
        ws.js_ws
    }
}

pub async fn connect_admin_ws(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    match connect_admin_ws_js(url, timeout).await {
        Ok(js_ws) => Ok(AdminWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}

#[generate_call(
    AdminWebsocket,
    AdminWsCmd,
    AdminWsCmdResponse,
    parse_admin_ws_cmd_response
)]
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

////////////////////////////////////////////////////////////////////////////////
// AppWebsocket
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AppWebsocket {
    pub js_ws: JsValue,
}

impl From<AppWebsocket> for JsValue {
    fn from(ws: AppWebsocket) -> Self {
        ws.js_ws
    }
}

pub async fn connect_app_ws(url: String, timeout: Option<u32>) -> Result<AppWebsocket, String> {
    match connect_app_ws_js(url, timeout).await {
        Ok(js_ws) => Ok(AppWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}

#[generate_call(AppWebsocket, AppWsCmd, AppWsCmdResponse, parse_app_ws_cmd_response)]
#[derive(Clone, Debug)]
pub enum AppWsCmd {
    AppInfo { installed_app_id: String },
    // CallZome { cell_id, zome_name, fn_name, payload, provenance, cap }
}

#[derive(Clone, Debug)]
pub enum AppWsCmdResponse {
    AppInfo(JsValue),
    // CallZome(JsValue),
}

fn parse_app_ws_cmd_response(val: JsValue, tag: String) -> AppWsCmdResponse {
    match tag.as_str() {
        "AppInfo" => AppWsCmdResponse::AppInfo(val),
        other => panic!(
            "parse_app_ws_cmd_response: impossible: received unknown tag: {}",
            other
        ),
    }
}
