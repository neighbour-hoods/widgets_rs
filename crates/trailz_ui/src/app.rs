use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Geolocation;
use weblog::{console_error, console_log};
use yew::prelude::*;

// use social_sensemaker_core::SENSEMAKER_ZOME_NAME;

use holochain_client_wrapper::{
    AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket, AppWsCmd, AppWsCmdResponse,
    CellId, DeserializeFromJsObj,
};

use crate::bindings::{GeolocationCoordinates, GeolocationPosition};

// TODO get rid of this once we're using proper sensemaker app name
const TEST_APP_NAME: &str = "test-app";

pub enum Msg {
    AdminWs(WsMsg<AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWsCmd, AppWsCmdResponse>),
    Log(String),
    Error(String),
}

pub enum WsMsg<WSCMD, WSCMDRESP> {
    Cmd(WSCMD),
    CmdResponse(Result<WSCMDRESP, JsValue>),
}

pub struct Model {
    admin_ws: AdminWebsocket,
    app_ws: AppWebsocket,
    cell_id: CellId,
    opt_geolocation: Option<Geolocation>,
}

#[derive(Properties, PartialEq)]
pub struct ModelProps {
    pub admin_ws_js: JsValue,
    pub app_ws_js: JsValue,
    pub cell_id_js: JsValue,
}

fn geo_success_fn(gp: GeolocationPosition) {
    console_log!("gp {:?}", gp);
}

impl Component for Model {
    type Message = Msg;
    type Properties = ModelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let cell_id = CellId::deserialize_from_js_obj(props.cell_id_js.clone());
        let app_ws: AppWebsocket = props.app_ws_js.clone().into();
        let admin_ws: AdminWebsocket = props.admin_ws_js.clone().into();

        // geolocation
        let geo_res = move || -> Result<Geolocation, JsValue> {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let geolocation = navigator.geolocation()?;
            console_log!("geolocation {:?}", geolocation.clone());
            let geo_success_closure = Closure::new(geo_success_fn);
            match geo_success_closure.as_ref().dyn_ref() {
                Some(geo_success_closure) => {
                    geolocation
                        .clone()
                        .watch_position_with_error_callback(geo_success_closure, None)?;
                    console_log!("success");
                }
                None => {
                    console_log!("failure");
                }
            }
            Ok(geolocation)
        };
        let opt_geolocation = match geo_res() {
            Ok(geolocation) => Some(geolocation),
            Err(err) => {
                console_error!("geolocation error: ", err);
                None
            }
        };

        Self {
            admin_ws,
            app_ws,
            cell_id,
            opt_geolocation,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AdminWs(ws_msg) => match ws_msg {
                WsMsg::Cmd(cmd) => {
                    let ws = self.admin_ws.clone();
                    ctx.link().send_future(async move {
                        Msg::AdminWs(WsMsg::CmdResponse(ws.call(cmd).await))
                    });
                    false
                }

                WsMsg::CmdResponse(resp) => {
                    match resp {
                        Ok(val) => {
                            console_log!(format!("WsMsg::CmdResponse: {:?}", val));
                        }
                        Err(err) => {
                            console_error!(format!("WsMsg::CmdResponse: error: {:?}", err));
                        }
                    };
                    false
                }
            },

            Msg::AppWs(ws_msg) => match ws_msg {
                WsMsg::Cmd(cmd) => {
                    let ws = self.app_ws.clone();
                    ctx.link().send_future(async move {
                        Msg::AppWs(WsMsg::CmdResponse(ws.call(cmd).await))
                    });
                    false
                }

                WsMsg::CmdResponse(resp) => {
                    match resp {
                        Ok(val) => {
                            console_log!(format!("WsMsg::CmdResponse: {:?}", val));
                        }
                        Err(err) => {
                            console_error!(format!("WsMsg::CmdResponse: error: {:?}", err));
                        }
                    };
                    false
                }
            },

            Msg::Error(err) => {
                console_error!("Error: {}", err);
                false
            }

            Msg::Log(err) => {
                console_log!("Log: {}", err);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <p>{"hello, trailz 👋"}</p>
            </div>
        }
    }
}
