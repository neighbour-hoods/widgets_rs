use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

use macros::generate_call;

////////////////////////////////////////////////////////////////////////////////
// wasm_bindgen key bindings
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="connect")]
    async fn connect_admin_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = AppWebsocket, js_name="connect")]
    async fn connect_app_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

////////////////////////////////////////////////////////////////////////////////
// SerializeToJsObj trait
////////////////////////////////////////////////////////////////////////////////

trait SerializeToJsObj {
    fn serialize_to_js_obj(self) -> JsValue;
}

impl SerializeToJsObj for u16 {
    fn serialize_to_js_obj(self) -> JsValue {
        self.into()
    }
}

impl SerializeToJsObj for String {
    fn serialize_to_js_obj(self) -> JsValue {
        self.into()
    }
}

impl<T: SerializeToJsObj> SerializeToJsObj for Option<T> {
    fn serialize_to_js_obj(self) -> JsValue {
        match self {
            None => JsValue::NULL,
            Some(v) => v.serialize_to_js_obj(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DnaHash(JsValue);

impl SerializeToJsObj for DnaHash {
    fn serialize_to_js_obj(self) -> JsValue {
        let DnaHash(val) = self;
        val
    }
}

// TODO figure out why this doesn't work - unsatisfied trait bounds for String
// impl<T> SerializeToJsObj for T
// where
//     T: JsCast,
// {
//     fn serialize_to_js_obj(self) -> JsValue {
//         self.into()
//     }
// }

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

/// each constructor of this enum corresponds to a method on the AdminWebsocket:
/// <https://github.com/holochain/holochain-client-js/blob/develop/docs/API_adminwebsocket.md>
///
/// n.b. the order of the constructors is non-alphabetical & corresponds to the documentation
/// order.
#[generate_call(
    AdminWebsocket,
    AdminWsCmd,
    AdminWsCmdResponse,
    parse_admin_ws_cmd_response
)]
#[derive(Clone, Debug)]
pub enum AdminWsCmd {
    AttachAppInterface {
        port: u16,
    },
    DisableApp {
        installed_app_id: String,
    },
    // DumpState({ cell_id }),
    EnableApp {
        installed_app_id: String,
    },
    GenerateAgentPubKey,
    RegisterDna {
        path: String,
        uid: Option<String>,
        properties: Option<String>,
    },
    // InstallAppBundle({ installed_app_id, source as path | bundle | hash, uid?, properties? }),
    // InstallApp({ installed_app_id, agent_key, dnas }),
    UninstallApp {
        installed_app_id: String,
    },
    ListDnas,
    ListCellIds,
    ListActiveApps,
    // RequestAgentInfo({ cell_id }),
    // AddAgentInfo({ agent_infos }),
}

// TODO consider statically checking that AdminWsCmd/AdminWsCmdResponse have the right # of
// constructors and all their names match up. can also apply to AppWsCmd/AppWsCmdResponse.

#[derive(Clone, Debug)]
pub enum AdminWsCmdResponse {
    AttachAppInterface(JsValue),
    DisableApp(JsValue),
    // DumpState(JsValue),
    EnableApp(JsValue),
    GenerateAgentPubKey(JsValue),
    RegisterDna(DnaHash),
    // InstallAppBundle(JsValue),
    // InstallApp(JsValue),
    UninstallApp(JsValue),
    ListDnas(JsValue),
    ListCellIds(JsValue),
    ListActiveApps(JsValue),
    // RequestAgentInfo(JsValue),
    // AddAgentInfo(JsValue),
}

fn parse_admin_ws_cmd_response(val: JsValue, tag: String) -> AdminWsCmdResponse {
    match tag.as_str() {
        "AttachAppInterface" => AdminWsCmdResponse::AttachAppInterface(val),
        "DisableApp" => AdminWsCmdResponse::DisableApp(val),
        // "DumpState" => AdminWsCmdResponse::DumpState(val),
        "EnableApp" => AdminWsCmdResponse::EnableApp(val),
        "GenerateAgentPubKey" => AdminWsCmdResponse::GenerateAgentPubKey(val),
        "RegisterDna" => AdminWsCmdResponse::RegisterDna(DnaHash(val)),
        // "InstallAppBundle" => AdminWsCmdResponse::InstallAppBundle(val),
        // "InstallApp" => AdminWsCmdResponse::InstallApp(val),
        "UninstallApp" => AdminWsCmdResponse::UninstallApp(val),
        "ListDnas" => AdminWsCmdResponse::ListDnas(val),
        "ListCellIds" => AdminWsCmdResponse::ListCellIds(val),
        "ListActiveApps" => AdminWsCmdResponse::ListActiveApps(val),
        // "RequestAgentInfo" => AdminWsCmdResponse::RequestAgentInfo(val),
        // "AddAgentInfo" => AdminWsCmdResponse::AddAgentInfo(val),
        other => panic!(
            "parse_admin_ws_cmd_response: impossible: received unknown tag: {}",
            other
        ),
    }
}

////////////////////////////////////////
// payloads
////////////////////////////////////////

// this might be a good idea, but we'll leave it for later, maybe.
// pub struct RegisterDnaPayload {
//     bundle_src: BundleSource,
//     uid: Option<String>,
//     properties: Option<String>,
// }

// pub enum BundleSource {
//     Path(String),
//     // Hash(String),
//     // Bundle { ... },
// }

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

/// n.b. the order of the constructors is non-alphabetical & corresponds to the order documented in
/// <https://github.com/holochain/holochain-client-js/blob/develop/docs/API_appwebsocket.md>
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
