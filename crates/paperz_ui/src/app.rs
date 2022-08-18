use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    connect_admin_ws, connect_app_ws, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket,
    AppWsCmd, AppWsCmdResponse, CellId, EntryHashRaw, EntryHeaderHashPairRaw,
};
use paperz_core::types::Paper;
use widget_helpers::{handle_update, WsMsg, WsState};

use crate::{file_upload::FileUploadApp, js_ser_de::*};

const PAPERZ_ZOME_NAME: &str = "paperz_main_zome";

pub enum Msg {
    AdminWs(WsMsg<AdminWebsocket, AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWebsocket, AppWsCmd, AppWsCmdResponse>),
    PaperzCellId(CellId),
    Log(String),
    Error(String),
    ZomeCallResponse(ZomeCallResponse),
    BrowserUploadedPaper(Paper),
}

pub enum ZomeCallResponse {
    Papers(Vec<(EntryHashRaw, Paper)>),
    UploadPaper(EntryHashRaw, Paper),
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

                match self.app_ws.clone() {
                    WsState::Absent(err) => {
                        console_error!(format!("WsState::Absent: {}", err));
                    }
                    WsState::Present(ws) => ctx.link().send_future(async move {
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
                                    PaperEhVec::deserialize_from_js_obj_(val)
                                        .into_iter()
                                        .map(|pair| pair.into())
                                        .collect(),
                                ))
                            }
                            Ok(resp) => {
                                Msg::Error(format!("impossible: invalid response: {:?}", resp))
                            }
                            Err(err) => Msg::Error(format!("err: {:?}", err)),
                        }
                    }),
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

            Msg::ZomeCallResponse(ZomeCallResponse::Papers(paper_vec)) => {
                self.paperz = paper_vec;
                console_log!("got paper_vec");
                true
            }

            Msg::ZomeCallResponse(ZomeCallResponse::UploadPaper(paper_eh, paper)) => {
                self.paperz.push((paper_eh, paper));
                true
            }

            Msg::BrowserUploadedPaper(paper) => {
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
                                fn_name: "upload_paper".into(),
                                payload: paper.clone().serialize_to_js_obj(),
                                provenance: cell_id.1.clone(),
                                cap: "".into(),
                            };
                            let resp = ws.call(cmd).await;
                            match resp {
                                Ok(AppWsCmdResponse::CallZome(val)) => {
                                    let (paper_eh, _paper_hh) =
                                        EntryHeaderHashPairRaw::deserialize_from_js_obj_(val);
                                    Msg::ZomeCallResponse(ZomeCallResponse::UploadPaper(
                                        paper_eh, paper,
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
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_paper_upload: Callback<Paper> = {
            let link = ctx.link().clone();
            Callback::from(move |paper: Paper| {
                link.send_future(async { Msg::BrowserUploadedPaper(paper) })
            })
        };
        let mk_paper_src = |paper: Paper| -> String {
            "data:application/pdf;base64,".to_string() + &paper.blob_str
        };

        html! {
            <div>
                <p>{"hello, paperz 👋"}</p>
                <br/>
                <FileUploadApp {on_paper_upload} />
                <br/>
                <h3 class="subtitle">{"paperz"}</h3>
                { for self.paperz.iter().map(|pair| html!{ <iframe src={mk_paper_src(pair.1.clone())} width="100%" height="500px" /> }) }
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
