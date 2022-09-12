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
    GpUpdate(GeolocationPosition),
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
    opt_gp: Option<GeolocationPosition>,
    gp_update_count: u64,
}

#[derive(Properties, PartialEq)]
pub struct ModelProps {
    pub admin_ws_js: JsValue,
    pub app_ws_js: JsValue,
    pub cell_id_js: JsValue,
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
            let link = ctx.link().clone();
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let geolocation = navigator.geolocation()?;
            console_log!("geolocation {:?}", geolocation.clone());
            let geo_success_closure: Closure<dyn FnMut(GeolocationPosition)> =
                Closure::new(move |gp: GeolocationPosition| {
                    link.send_future(async move { Msg::GpUpdate(gp) });
                });
            geolocation.clone().watch_position_with_error_callback(
                geo_success_closure.as_ref().unchecked_ref(),
                None,
            )?;
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
            opt_gp: None,
            gp_update_count: 0,
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

            Msg::GpUpdate(gp) => {
                self.opt_gp = Some(gp);
                self.gp_update_count += 1;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let gp_rendered: String = match &self.opt_gp {
            None => "no gp info".into(),
            Some(gp) => format!(
                "coords: ({},{})\
                                \ntimestamp: {}",
                gp.coords().latitude(),
                gp.coords().longitude(),
                gp.timestamp()
            ),
        };
        let gp_update_count_rendered: String = self.gp_update_count.to_string();
        html! {
            <div>
                <p>{"hello, trailz 👋"}</p>
                <p>{gp_rendered}</p>
                <p>{gp_update_count_rendered}</p>
            </div>
        }
    }
}
