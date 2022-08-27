use base64::encode;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

// use social_sensemaker_core::SENSEMAKER_ZOME_NAME;
// TODO figure out how to depend on `common` without getting:
// ```
// ✘ [ERROR] Could not resolve "env"
// ```
// use common::vec_u8_b64_encode;

use holochain_client_wrapper::{
    agent_pk_to_vec_u8, AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket, AppWsCmd,
    AppWsCmdResponse, CellId, DeserializeFromJsObj, EntryHashRaw, EntryHeaderHashPairRaw,
    SerializeToJsObj,
};
use paperz_core::{types::Paper, AGENT_PATH, PAPERZ_ZOME_NAME};
use widget_helpers::file_upload::{FileBytes, FileUploadApp};

use crate::js_ser_de::*;

// TODO get rid of this once we're using proper sensemaker app name
const TEST_APP_NAME: &str = "test-app";

pub enum Msg {
    AdminWs(WsMsg<AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWsCmd, AppWsCmdResponse>),
    Log(String),
    Error(String),
    ZomeCallResponse(ZomeCallResponse),
    BrowserUploadedPaper(Paper),
    SensemakerPresent(bool),
    SmInitSubmit(String),
    SmCompSubmit(String),
    SmDataInit,
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
    /// None means we don't know yet (no response). for `Some(b)`, `b == True` indicates presence.
    sensemaker_present: Option<bool>,
    /// (sm_init_expr_string, sm_comp_expr_string)
    paper_sm: (String, String),
}

const STARTER_SM_INIT_EXPR_STRING: &str = "0";
const STARTER_SM_COMP_EXPR_STRING: &str = "+";

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

        // get_all_paperz
        let app_ws_ = app_ws.clone();
        let cell_id_ = cell_id.clone();
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

        // state machine setup
        let paper_sm: (String, String) = (
            STARTER_SM_INIT_EXPR_STRING.into(),
            STARTER_SM_COMP_EXPR_STRING.into(),
        );

        let admin_ws: AdminWebsocket = props.admin_ws_js.clone().into();
        let admin_ws_ = admin_ws.clone();
        ctx.link().send_future(async move {
            let resp = admin_ws_.call(AdminWsCmd::ListActiveApps).await;
            match resp {
                Ok(AdminWsCmdResponse::ListActiveApps(active_apps)) => {
                    let sensemaker_present = active_apps.contains(&TEST_APP_NAME.into());
                    Msg::SensemakerPresent(sensemaker_present)
                }
                Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                Err(err) => Msg::Error(format!("err: {:?}", err)),
            }
        });
        Self {
            admin_ws,
            app_ws,
            paperz_cell_id: cell_id.clone(),
            paperz: Vec::new(),
            sensemaker_present: None,
            paper_sm,
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
                let ws = self.app_ws.clone();
                let cell_id = self.paperz_cell_id.clone();
                ctx.link().send_future(async move {
                    let typed_payload: (String, String, String) = (
                        AGENT_PATH.into(),
                        base64::encode(agent_pk_to_vec_u8(cell_id.1.clone())),
                        "1".into(),
                    );
                    let cmd = AppWsCmd::CallZome {
                        cell_id: cell_id.clone(),
                        zome_name: PAPERZ_ZOME_NAME.into(),
                        fn_name: "step_sm_path_remote".into(),
                        payload: typed_payload.serialize_to_js_obj(),
                        provenance: cell_id.1.clone(),
                        cap: "".into(),
                    };
                    let resp = ws.call(cmd).await;
                    match resp {
                        Ok(AppWsCmdResponse::CallZome(val)) => {
                            Msg::Log(format!("step_sm_path_remote: {:?}", val))
                        }
                        Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                        Err(err) => Msg::Error(format!("err: {:?}", err)),
                    }
                });
                true
            }

            Msg::SensemakerPresent(sensemaker_present) => {
                self.sensemaker_present = Some(sensemaker_present);
                true
            }

            Msg::SmInitSubmit(expr_str) => {
                self.set_sm(ctx.link(), expr_str.clone(), "set_sm_init".into());
                // TODO ideally we would wait for confirmation before setting this
                self.paper_sm.0 = expr_str;
                true
            }

            Msg::SmCompSubmit(expr_str) => {
                self.set_sm(ctx.link(), expr_str.clone(), "set_sm_comp".into());
                // TODO ideally we would wait for confirmation before setting this
                self.paper_sm.1 = expr_str;
                true
            }

            Msg::SmDataInit => {
                let app_ws_ = self.app_ws.clone();
                let cell_id_ = self.paperz_cell_id.clone();
                let payload: (String, String) = (
                    AGENT_PATH.into(),
                    base64::encode(agent_pk_to_vec_u8(self.paperz_cell_id.1.clone())),
                );
                ctx.link().send_future(async move {
                    let cmd = AppWsCmd::CallZome {
                        cell_id: cell_id_.clone(),
                        zome_name: PAPERZ_ZOME_NAME.into(),
                        fn_name: "init_agent_sm_data".into(),
                        payload: payload.serialize_to_js_obj(),
                        provenance: cell_id_.1.clone(),
                        cap: "".into(),
                    };
                    let resp = app_ws_.call(cmd).await;
                    match resp {
                        Ok(AppWsCmdResponse::CallZome(val)) => {
                            Msg::Log(format!("init_agent_sm_data: {:?}", val))
                        }
                        Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                        Err(err) => Msg::Error(format!("err: {:?}", err)),
                    }
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sensemaker_present_html = match self.sensemaker_present {
            None => html! {},
            Some(true) => html! {
                <p>{"sensemaker is present"}</p>
            },
            Some(false) => html! {
                <div class="alert">
                  <h3>{"social_sensemaker is absent!"}</h3>
                  <p>{"please install it into your `we`, as `paperz` requires it to function."}</p>
                </div>
            },
        };
        //
        let sm_init_handler = |input: String| Ok(Msg::SmInitSubmit(input));
        let sm_comp_handler = |input: String| Ok(Msg::SmCompSubmit(input));
        //
        let content_name = "paper";
        let on_file_upload: Callback<FileBytes> = {
            let link = ctx.link().clone();
            Callback::from(move |fb: FileBytes| {
                let paper = Paper {
                    filename: fb.filename,
                    blob_str: encode(fb.bytes),
                };
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
                {sensemaker_present_html}
                <br/>
                { self.view_string_input(ctx.link(), sm_init_handler, "sm_init".into(), "paperz/agent sm_init".into(), self.paper_sm.0.clone()) }
                <br/>
                { self.view_string_input(ctx.link(), sm_comp_handler, "sm_comp".into(), "paperz/agent sm_comp".into(), self.paper_sm.1.clone()) }
                <br/>
                <button onclick={ctx.link().callback(move |_| Msg::SmDataInit)}>{ "initialize_sm_data" }</button>
                <br/>
                <FileUploadApp {content_name} {on_file_upload} />
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
        label: String,
        value: String,
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
            <div>
                <label>{format!("{}: ", label)}</label>
                <input
                    {class}
                    {value}
                    {onkeypress}
                />
            </div>
        }
    }

    fn set_sm(&self, link: &Scope<Self>, expr_str: String, zome_fn: String) {
        let app_ws_ = self.app_ws.clone();
        let cell_id_ = self.paperz_cell_id.clone();
        link.send_future(async move {
            let cmd = AppWsCmd::CallZome {
                cell_id: cell_id_.clone(),
                zome_name: PAPERZ_ZOME_NAME.into(),
                fn_name: zome_fn.clone(),
                payload: (AGENT_PATH.to_string(), expr_str).serialize_to_js_obj(),
                provenance: cell_id_.1.clone(),
                cap: "".into(),
            };
            let resp = app_ws_.call(cmd).await;
            match resp {
                Ok(AppWsCmdResponse::CallZome(val)) => Msg::Log(format!("{}: {:?}", zome_fn, val)),
                Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                Err(err) => Msg::Error(format!("err: {:?}", err)),
            }
        });
    }
}
