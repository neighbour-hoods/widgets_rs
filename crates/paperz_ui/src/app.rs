use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    connect_admin_ws, connect_app_ws, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket,
    AppWsCmd, AppWsCmdResponse, CellId, DeserializeFromJsObj, EntryHashRaw, HashRoleProof,
};
use paperz_core::types::{Paper, PaperEhVec};
use widget_helpers::{handle_update, WsMsg, WsState};

use crate::file_upload::FileUploadApp;

const PAPERZ_ZOME_NAME: &str = "paperz_main_zome";

pub enum Msg {
    AdminWs(WsMsg<AdminWebsocket, AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWebsocket, AppWsCmd, AppWsCmdResponse>),
    PaperzCellId(CellId),
    Log(String),
    Error(String),
    SensemakerEnabled(bool),
    ZomeCallResponse(ZomeCallResponse),
    UploadedPaper(Paper),
}

pub enum ZomeCallResponse {
    Papers(Vec<(EntryHashRaw, Paper)>),
}

pub struct Model {
    admin_ws: WsState<AdminWebsocket>,
    app_ws: WsState<AppWebsocket>,
    paperz_cell_id: Option<CellId>,
    paperz: Vec<(EntryHashRaw, Paper)>,
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
                            match resp {
                                Ok(AppWsCmdResponse::CallZome(val)) => {
                                    Msg::ZomeCallResponse(ZomeCallResponse::Papers(
                                        PaperEhVec::deserialize_from_js_obj(val),
                                    ))
                                }
                                Ok(resp) => {
                                    Msg::Error(format!("impossible: invalid response: {:?}", resp))
                                }
                                Err(err) => Msg::Error(format!("err: {:?}", err)),
                            }
                        }),
                    },
                };

                false
            }

            Msg::ZomeCallResponse(ZomeCallResponse::Papers(paper_vec)) => {
                self.paperz = paper_vec;
                console_log!("got paper_vec");
                true
            }

            Msg::UploadedPaper(paper) => {
                console_log!(format!("paper: {:?}", paper));
                // self.paperz.push(paper) // types are wrong - we need an EntryHash
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_paper_upload: Callback<Paper> = {
            let link = ctx.link().clone();
            Callback::from(move |paper: Paper| {
                link.send_future(async { Msg::UploadedPaper(paper) })
            })
        };

        html! {
            <div>
                <p>{"hello, paperz 👋"}</p>
                <br/>
                <FileUploadApp {on_paper_upload} />
                <br/>
                <h3 class="subtitle">{"paperz"}</h3>
                { for self.paperz.iter().map(|paper| html!{ <iframe src={paper.1.blob_str.clone()} width="100%" height="500px" /> }) }
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
