use wasm_bindgen::prelude::*;
use weblog::{console_error, console_log};

pub enum WsMsg<WS, WSCMD, WSCMDRESP> {
    Connected(WS),
    Error(String),
    Cmd(WSCMD),
    CmdResponse(Result<WSCMDRESP, JsValue>),
}

#[derive(Clone, Debug)]
pub enum WsState<WS> {
    Present(WS),
    Absent(String),
}

/// we cannot handle the `Cmd` case well b/c async traits are not well supported, and we would need
/// that in order to consistently make the appropriate `call` on both App/AdminWebsockets.
///
/// returns `(render_status, ws_set_to_present, opt_cmd)`
pub fn handle_update<WS, WSCMD, WSCMDRESP>(
    ws_ref: &mut WsState<WS>,
    msg: WsMsg<WS, WSCMD, WSCMDRESP>,
) -> (bool, bool, Option<WSCMD>)
where
    WS: Clone,
    wasm_bindgen::JsValue: From<WS>,
    WSCMDRESP: std::fmt::Debug,
{
    match msg {
        WsMsg::Connected(ws) => {
            *ws_ref = WsState::Present(ws.clone());
            console_log!("WsMsg::Connected: {:?}", ws);
            (true, true, None)
        }
        WsMsg::Error(err) => {
            *ws_ref = WsState::Absent(err.clone());
            console_error!(format!("WsMsg::Error: {}", err));
            (true, false, None)
        }
        WsMsg::Cmd(cmd) => (false, false, Some(cmd)),
        WsMsg::CmdResponse(resp) => {
            match resp {
                Ok(val) => {
                    console_log!(format!("WsMsg::CmdResponse: {:?}", val));
                }
                Err(err) => {
                    console_error!(format!("WsMsg::CmdResponse: error: {:?}", err));
                }
            };
            (false, false, None)
        }
    }
}
