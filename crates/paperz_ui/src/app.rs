use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    connect_admin_ws, connect_app_ws, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket,
    AppWsCmd, AppWsCmdResponse, CellId, HashRoleProof,
};
use paperz::Paper;
use widget_helpers::{handle_update, WsMsg, WsState};

const PAPERZ_ZOME_NAME: &str = "paperz_main_zome";

pub enum Msg {
    AdminWs(WsMsg<AdminWebsocket, AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWebsocket, AppWsCmd, AppWsCmdResponse>),
    PaperzCellId(CellId),
    Log(String),
    Error(String),
    SensemakerEnabled(bool),
}

pub struct Model {
    admin_ws: WsState<AdminWebsocket>,
    app_ws: WsState<AppWebsocket>,
    paperz_cell_id: Option<CellId>,
    paperz: Vec<Paper>,
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
            admin_ws: WsState::Absent("".into()),
            app_ws: WsState::Absent("".into()),
            paperz_cell_id: None,
            paperz: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AdminWs(ws_msg) => {
                let (render_status, _ws_set_to_present, opt_cmd) =
                    handle_update(&mut self.admin_ws, ws_msg);
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
                let (render_status, ws_set_to_present, opt_cmd) =
                    handle_update(&mut self.app_ws, ws_msg);
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

                if ws_set_to_present {
                    match self.app_ws.clone() {
                        WsState::Absent(err) => {
                            console_error!(format!("WsState::Absent: {}", err));
                        }
                        WsState::Present(ws) => {
                            ctx.link().send_future(async move {
                                match ws
                                    .call(AppWsCmd::AppInfo {
                                        installed_app_id: "test-app".into(),
                                    })
                                    .await
                                {
                                    Ok(AppWsCmdResponse::AppInfo(app_info)) => {
                                        Msg::PaperzCellId(app_info.cell_data[0].cell_id.clone())
                                    }
                                    Ok(resp) => Msg::Error(format!(
                                        "impossible: invalid response: {:?}",
                                        resp
                                    )),
                                    Err(err) => Msg::Error(format!("err: {:?}", err)),
                                }
                            });
                        }
                    }
                }

                render_status_
            }

            Msg::PaperzCellId(cell_id) => {
                self.paperz_cell_id = Some(cell_id.clone());
                console_log!(format!("got cell_id: {:?}", cell_id));

                match self.admin_ws.clone() {
                    WsState::Absent(err) => {
                        console_error!(format!("WsState::Absent: {}", err));
                    }
                    WsState::Present(ws) => {
                        ctx.link().send_future(async move {
                            let ret = async {
                                let cell_ids = match ws.call(AdminWsCmd::ListCellIds).await {
                                    Ok(AdminWsCmdResponse::ListCellIds(x)) => Ok(x),
                                    Ok(resp) => Err(format!("impossible: invalid response: {:?}", resp)),
                                    Err(err) => Err(format!("err: {:?}", err)),
                                }?;

                                if cell_ids.len() == 1 {
                                    let cmd = AdminWsCmd::RegisterDna {
                                        path: "../social_sensemaker/happs/social_sensemaker/social_sensemaker.dna".into(),
                                        uid: None,
                                        properties: None,
                                    };
                                    let dna_hash = match ws.call(cmd).await {
                                        Ok(AdminWsCmdResponse::RegisterDna(x)) => Ok(x),
                                        Ok(resp) => Err(format!("impossible: invalid response: {:?}", resp)),
                                        Err(err) => Err(format!("err: {:?}", err)),
                                    }?;
                                    let installed_app_id: String = "sensemaker".into();
                                    let cmd = AdminWsCmd::InstallApp {
                                        installed_app_id: installed_app_id.clone(),
                                        agent_key: cell_id.1,
                                        dnas: vec![
                                            HashRoleProof {
                                                hash: dna_hash,
                                                role_id: "thedna".into(),
                                                membrane_proof: None,
                                            }
                                        ],
                                    };
                                    let install_app = match ws.call(cmd).await {
                                        Ok(AdminWsCmdResponse::InstallApp(x)) => Ok(x),
                                        Ok(resp) => Err(format!("impossible: invalid response: {:?}", resp)),
                                        Err(err) => Err(format!("err: {:?}", err)),
                                    }?;
                                    console_log!(format!("install_app: {:?}", install_app));
                                    let cmd = AdminWsCmd::EnableApp {
                                        installed_app_id,
                                    };
                                    let enable_app = match ws.call(cmd).await {
                                        Ok(AdminWsCmdResponse::EnableApp(x)) => Ok(x),
                                        Ok(resp) => Err(format!("impossible: invalid response: {:?}", resp)),
                                        Err(err) => Err(format!("err: {:?}", err)),
                                    }?;
                                    console_log!(format!("enable_app: {:?}", enable_app));
                                    Ok(Msg::SensemakerEnabled(true))
                                } else {
                                    Ok(Msg::SensemakerEnabled(false))
                                }
                            };
                            match ret.await {
                                Err(err) => Msg::Error(err),
                                Ok(msg) => msg
                            }
                        });
                    }
                }

                false
            }

            Msg::Error(err) => {
                console_error!("Error: {}", err);
                false
            }

            Msg::Log(err) => {
                console_log!("Log: {}", err);
                false
            }

            Msg::SensemakerEnabled(just_enabled) => {
                console_log!(format!(
                    "sensemaker enabled. just_enabled: {}",
                    just_enabled
                ));

                match self.app_ws.clone() {
                    WsState::Absent(err) => {
                        console_error!(format!("WsState::Absent: {}", err));
                    }
                    WsState::Present(ws) => match self.paperz_cell_id.clone() {
                        None => {
                            console_error!("paperz_cell_id is None");
                        }
                        Some(cell_id) => ctx.link().send_future(async move {
                            let cmd = AppWsCmd::CallZome {
                                cell_id: cell_id.clone(),
                                zome_name: PAPERZ_ZOME_NAME.into(),
                                fn_name: "get_all_paperz".into(),
                                payload: JsValue::NULL,
                                provenance: cell_id.1.clone(),
                                cap: "".into(),
                            };
                            let resp = ws.call(cmd).await;
                            Msg::Log(format!("zome call resp: {:?}", resp))
                        }),
                    },
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
