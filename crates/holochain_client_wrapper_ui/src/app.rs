use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    connect_admin_ws, connect_app_ws, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket,
    AppWsCmd, AppWsCmdResponse,
};
use widget_helpers::{handle_update, WsMsg, WsState};

pub enum Msg {
    AddOne,
    SubOne,
    AdminWs(WsMsg<AdminWebsocket, AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWebsocket, AppWsCmd, AppWsCmdResponse>),
}

pub struct Model {
    value: i64,
    admin_ws: WsState<AdminWebsocket>,
    app_ws: WsState<AppWebsocket>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match connect_admin_ws("ws://localhost:9000".into(), None).await {
                Ok(ws) => Msg::AdminWs(WsMsg::Connected(ws)),
                Err(err) => Msg::AdminWs(WsMsg::Error(err)),
            }
        });
        ctx.link().send_future(async {
            match connect_app_ws("ws://localhost:9999".into(), None).await {
                Ok(ws) => Msg::AppWs(WsMsg::Connected(ws)),
                Err(err) => Msg::AppWs(WsMsg::Error(err)),
            }
        });
        Self {
            value: 0,
            admin_ws: WsState::Absent("".into()),
            app_ws: WsState::Absent("".into()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::SubOne => {
                self.value -= 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }

            Msg::AdminWs(ws_msg) => {
                let (render_status, opt_cmd) = handle_update(&mut self.admin_ws, ws_msg);
                let render_status_ = match opt_cmd {
                    None => render_status,
                    Some(cmd) => {
                        let ws_ref_clone = self.admin_ws.clone();
                        match ws_ref_clone {
                            WsState::Absent(err) => {
                                console_error!(format!("AdminWsCmd - WsState::Absent: {}", err));
                            }
                            WsState::Present(ws) => {
                                console_log!("AdminWsCmd w/ admin ws");
                                ctx.link().send_future(async move {
                                    Msg::AdminWs(WsMsg::CmdResponse(ws.call(cmd).await))
                                })
                            }
                        };
                        false
                    }
                };
                render_status_
            }

            Msg::AppWs(ws_msg) => {
                let (render_status, opt_cmd) = handle_update(&mut self.app_ws, ws_msg);
                let render_status_ = match opt_cmd {
                    None => render_status,
                    Some(cmd) => {
                        let ws_ref_clone = self.app_ws.clone();
                        match ws_ref_clone {
                            WsState::Absent(err) => {
                                console_error!(format!("AppWsCmd - WsState::Absent: {}", err));
                            }
                            WsState::Present(ws) => {
                                console_log!("AppWsCmd w/ app ws");
                                ctx.link().send_future(async move {
                                    Msg::AppWs(WsMsg::CmdResponse(ws.call(cmd).await))
                                })
                            }
                        };
                        false
                    }
                };
                render_status_
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ws_debug: String = match &self.admin_ws {
            WsState::Absent(_err) => "admin_ws absent".into(),
            WsState::Present(ws) => format!("typeof: {:?}", ws.js_ws.js_typeof(),),
        };
        let enable_app_handler = |input: String| {
            Ok(Msg::AdminWs(WsMsg::Cmd(AdminWsCmd::EnableApp {
                installed_app_id: input,
            })))
        };
        let disable_app_handler = |input: String| {
            Ok(Msg::AdminWs(WsMsg::Cmd(AdminWsCmd::DisableApp {
                installed_app_id: input,
            })))
        };
        let uninstall_app_handler = |input: String| {
            Ok(Msg::AdminWs(WsMsg::Cmd(AdminWsCmd::UninstallApp {
                installed_app_id: input,
            })))
        };
        let register_dna_handler = |input: String| {
            Ok(Msg::AdminWs(WsMsg::Cmd(AdminWsCmd::RegisterDna {
                path: input,
                uid: None,
                properties: None,
            })))
        };
        let attach_app_interface_handler = |input: String| {
            input
                .parse()
                .map(|port| Msg::AdminWs(WsMsg::Cmd(AdminWsCmd::AttachAppInterface { port })))
                .map_err(|err| format!("attach_app_interface_handler: {}", err))
        };
        let mk_nullary_button = |msg: AdminWsCmd| {
            let msg_ = msg.clone();
            html! {
                <button onclick={ctx.link().callback(move |_| Msg::AdminWs(WsMsg::Cmd(msg.clone())))}>{ format!("{:?}", msg_) }</button>
            }
        };
        let app_info_handler = |input: String| {
            Ok(Msg::AppWs(WsMsg::Cmd(AppWsCmd::AppInfo {
                installed_app_id: input,
            })))
        };
        html! {
            <div>
                <button onclick={ctx.link().callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <button onclick={ctx.link().callback(|_| Msg::SubOne)}>{ "-1" }</button>
                <p>{ self.value }</p>
                <br/>

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
