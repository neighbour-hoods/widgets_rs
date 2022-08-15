use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    connect_admin_ws, connect_app_ws, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket,
    AppWsCmd, AppWsCmdResponse,
};

pub enum Msg {
    // AdminWs
    AdminWsConnected(AdminWebsocket),
    AdminWsError(String),
    AdminWsCmd(AdminWsCmd),
    AdminWsCmdResponse(Result<AdminWsCmdResponse, JsValue>),
    // AppWs
    AppWsConnected(AppWebsocket),
    AppWsError(String),
    AppWsCmd(AppWsCmd),
    AppWsCmdResponse(Result<AppWsCmdResponse, JsValue>),
}

#[derive(Clone, Debug)]
pub enum WsState<WS> {
    Present(WS),
    Absent(String),
}

pub struct Model {
    admin_ws: WsState<AdminWebsocket>,
    app_ws: WsState<AppWebsocket>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match connect_admin_ws("ws://localhost:9000".into(), None).await {
                Ok(ws) => Msg::AdminWsConnected(ws),
                Err(err) => Msg::AdminWsError(err),
            }
        });
        ctx.link().send_future(async {
            match connect_app_ws("ws://localhost:9999".into(), None).await {
                Ok(ws) => Msg::AppWsConnected(ws),
                Err(err) => Msg::AppWsError(err),
            }
        });
        Self {
            admin_ws: WsState::Absent("".into()),
            app_ws: WsState::Absent("".into()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AdminWsConnected(ws) => {
                self.admin_ws = WsState::Present(ws.clone());
                console_log!("Holochain admin ws connected: {:?}", ws);
                true
            }
            Msg::AdminWsError(err) => {
                self.admin_ws = WsState::Absent(err.clone());
                console_error!(format!("AdminWsError: {}", err));
                true
            }
            Msg::AdminWsCmd(cmd) => {
                let ws_clone = self.admin_ws.clone();
                match ws_clone {
                    WsState::Absent(err) => {
                        console_error!(format!("AdminWsCmd - WsState::Absent: {}", err))
                    }
                    WsState::Present(ws) => {
                        console_log!("AdminWsCmd w/ admin ws");
                        ctx.link().send_future(async move {
                            Msg::AdminWsCmdResponse(ws.call(cmd).await)
                        });
                    }
                };
                false
            }
            Msg::AdminWsCmdResponse(resp) => {
                match resp {
                    Ok(val) => {
                        console_log!(format!("AdminWsCmdResponse: {:?}", val));
                    }
                    Err(err) => {
                        console_error!(format!("AdminWsCmdResponse: error: {:?}", err));
                    }
                };
                false
            }

            // AppWs
            // TODO consider consolidating / deduplicating admin and app bits
            Msg::AppWsConnected(ws) => {
                self.app_ws = WsState::Present(ws.clone());
                console_log!("Holochain app ws connected: {:?}", ws);
                true
            }
            Msg::AppWsError(err) => {
                self.app_ws = WsState::Absent(err.clone());
                console_error!(format!("AppWsError: {}", err));
                true
            }
            Msg::AppWsCmd(cmd) => {
                let ws_clone = self.app_ws.clone();
                match ws_clone {
                    WsState::Absent(err) => {
                        console_error!(format!("AppWsCmd - WsState::Absent: {}", err))
                    }
                    WsState::Present(ws) => {
                        console_log!("AppWsCmd w/ app ws");
                        ctx.link()
                            .send_future(async move { Msg::AppWsCmdResponse(ws.call(cmd).await) });
                    }
                };
                false
            }
            Msg::AppWsCmdResponse(resp) => {
                match resp {
                    Ok(val) => {
                        console_log!(format!("AppWsCmdResponse: {:?}", val));
                    }
                    Err(err) => {
                        console_error!(format!("AppWsCmdResponse: error: {:?}", err));
                    }
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ws_debug: String = match &self.admin_ws {
            WsState::Absent(_err) => "admin_ws absent".into(),
            WsState::Present(ws) => format!("typeof: {:?}", ws.js_ws.js_typeof(),),
        };
        let enable_app_handler = |input: String| {
            Ok(Msg::AdminWsCmd(AdminWsCmd::EnableApp {
                installed_app_id: input,
            }))
        };
        let disable_app_handler = |input: String| {
            Ok(Msg::AdminWsCmd(AdminWsCmd::DisableApp {
                installed_app_id: input,
            }))
        };
        let uninstall_app_handler = |input: String| {
            Ok(Msg::AdminWsCmd(AdminWsCmd::UninstallApp {
                installed_app_id: input,
            }))
        };
        let register_dna_handler = |input: String| {
            Ok(Msg::AdminWsCmd(AdminWsCmd::RegisterDna {
                path: input,
                uid: None,
                properties: None,
            }))
        };
        let attach_app_interface_handler = |input: String| {
            input
                .parse()
                .map(|port| Msg::AdminWsCmd(AdminWsCmd::AttachAppInterface { port }))
                .map_err(|err| format!("attach_app_interface_handler: {}", err))
        };
        let mk_nullary_button = |msg: AdminWsCmd| {
            let msg_ = msg.clone();
            html! {
                <button onclick={ctx.link().callback(move |_| Msg::AdminWsCmd(msg.clone()))}>{ format!("{:?}", msg_) }</button>
            }
        };
        let app_info_handler = |input: String| {
            Ok(Msg::AppWsCmd(AppWsCmd::AppInfo {
                installed_app_id: input,
            }))
        };
        html! {
            <div>
                <p>{format!("admin_ws: {:?}", self.admin_ws)}</p>
                <p>{format!("{:?}", ws_debug)}</p>

                { self.view_string_input(ctx.link(), attach_app_interface_handler, "attach_app_interface".into(), "desired app port?".into()) }
                <br/>
                { self.view_string_input(ctx.link(), disable_app_handler, "disable_app".into(), "disable which app?".into()) }
                <br/>
                // dump_state
                // <br/>
                { self.view_string_input(ctx.link(), enable_app_handler, "enable_app".into(), "enable which app?".into()) }
                <br/>
                { mk_nullary_button(AdminWsCmd::GenerateAgentPubKey) }
                <br/>
                { self.view_string_input(ctx.link(), register_dna_handler, "register_dna".into(), "register dna at what path?".into()) }
                <br/>
                // install_app_bundle
                // <br/>
                // install_app
                // <br/>
                { self.view_string_input(ctx.link(), uninstall_app_handler, "uninstall_app".into(), "uninstall which app?".into()) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListDnas) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListCellIds) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListActiveApps) }
                // request_agent_info
                // <br/>
                // add_agent_info
                // <br/>

                <br/>
                <br/>
                <p>{format!("app_ws: {:?}", self.app_ws)}</p>
                <br/>
                { self.view_string_input(ctx.link(), app_info_handler, "app_info".into(), "app info for which app?".into()) }
            </div>
        }
    }
}

impl Model {
    fn view_string_input<F>(
        &self,
        link: &Scope<Self>,
        f: F,
        class: String,
        placeholder: String,
    ) -> Html
    where
        F: Fn(String) -> Result<Msg, String> + 'static,
    {
        let onkeypress = link.batch_callback(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: InputElement = e.target_unchecked_into();
                match f(input.value()) {
                    Ok(msg) => {
                        input.set_value("");
                        Some(msg)
                    }
                    Err(err) => {
                        console_error!(format!("view_string_input: {}", err));
                        None
                    }
                }
            } else {
                None
            }
        });
        html! {
            <input
                {class}
                {placeholder}
                {onkeypress}
            />
        }
    }
}
