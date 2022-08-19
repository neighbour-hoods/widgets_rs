use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use holochain_client_wrapper::{
    AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket, AppWsCmd, AppWsCmdResponse,
    CellId, DeserializeFromJsObj, EntryHashRaw, EntryHeaderHashPairRaw,
};
use paperz_core::types::Paper;

use crate::{file_upload::FileUploadApp, js_ser_de::*};

const PAPERZ_ZOME_NAME: &str = "paperz_main_zome";

pub enum Msg {
    AdminWs(WsMsg<AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWsCmd, AppWsCmdResponse>),
    Log(String),
    Error(String),
    ZomeCallResponse(ZomeCallResponse),
    BrowserUploadedPaper(Paper),
}

pub enum WsMsg<WSCMD, WSCMDRESP> {
    Cmd(WSCMD),
    CmdResponse(Result<WSCMDRESP, JsValue>),
}

pub enum ZomeCallResponse {
    Papers(Vec<(EntryHashRaw, Paper)>),
    UploadPaper(EntryHashRaw, Paper),
}

pub struct Model {
    admin_ws: AdminWebsocket,
    app_ws: AppWebsocket,
    paperz_cell_id: CellId,
    paperz: Vec<(EntryHashRaw, Paper)>,
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
        let cell_id_ = cell_id.clone();
        let app_ws: AppWebsocket = props.app_ws_js.clone().into();
        let app_ws_: AppWebsocket = app_ws.clone();
        ctx.link().send_future(async move {
            let cmd = AppWsCmd::CallZome {
                cell_id: cell_id_.clone(),
                zome_name: PAPERZ_ZOME_NAME.into(),
                fn_name: "get_all_paperz".into(),
                payload: JsValue::NULL,
                provenance: cell_id_.1.clone(),
                cap: "".into(),
            };
            let resp = app_ws_.call(cmd).await;
            match resp {
                Ok(AppWsCmdResponse::CallZome(val)) => {
                    Msg::ZomeCallResponse(ZomeCallResponse::Papers(
                        PaperEhVec::deserialize_from_js_obj_(val)
                            .into_iter()
                            .map(|pair| pair.into())
                            .collect(),
                    ))
                }
                Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                Err(err) => Msg::Error(format!("err: {:?}", err)),
            }
        });
        Self {
            admin_ws: props.admin_ws_js.clone().into(),
            app_ws,
            paperz_cell_id: cell_id.clone(),
            paperz: Vec::new(),
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
                let ws = self.app_ws.clone();
                let cell_id = self.paperz_cell_id.clone();
                ctx.link().send_future(async move {
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
                            Msg::ZomeCallResponse(ZomeCallResponse::UploadPaper(paper_eh, paper))
                        }
                        Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                        Err(err) => Msg::Error(format!("err: {:?}", err)),
                    }
                });
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
