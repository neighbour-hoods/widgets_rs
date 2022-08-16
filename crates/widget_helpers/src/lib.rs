use wasm_bindgen::prelude::*;

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
