use base64::encode;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

// use social_sensemaker_core::SENSEMAKER_ZOME_NAME;

use holochain_client_wrapper::{
    AdminWebsocket, AdminWsCmd, AdminWsCmdResponse, AppWebsocket, AppWsCmd, AppWsCmdResponse,
    CellId, DeserializeFromJsObj, EntryHashRaw, EntryHeaderHashPairRaw, SerializeToJsObj,
};
use memez_core::{types::Meme, MEMEZ_PATH};
use widget_helpers::file_upload::{FileBytes, FileUploadApp};

use crate::js_ser_de::*;

const MEMEZ_ZOME_NAME: &str = "memez_main_zome";
// TODO get rid of this once we're using proper sensemaker app name
const TEST_APP_NAME: &str = "test-app";

pub enum Msg {
    AdminWs(WsMsg<AdminWsCmd, AdminWsCmdResponse>),
    AppWs(WsMsg<AppWsCmd, AppWsCmdResponse>),
    Log(String),
    Error(String),
    ZomeCallResponse(ZomeCallResponse),
    BrowserUploadedMeme(Meme),
    SensemakerPresent(bool),
    SmInitSubmit(String),
    SmCompSubmit(String),
}

pub enum WsMsg<WSCMD, WSCMDRESP> {
    Cmd(WSCMD),
    CmdResponse(Result<WSCMDRESP, JsValue>),
}

pub enum ZomeCallResponse {
    Memes(Vec<(EntryHashRaw, Meme, i64)>),
    UploadMeme(EntryHashRaw, Meme),
}

pub struct Model {
    admin_ws: AdminWebsocket,
    app_ws: AppWebsocket,
    memez_cell_id: CellId,
    memez: Vec<(EntryHashRaw, Meme, i64)>,
    /// None means we don't know yet (no response). for `Some(b)`, `b == True` indicates presence.
    sensemaker_present: Option<bool>,
    /// (sm_init_expr_string, sm_comp_expr_string)
    meme_sm: (String, String),
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
        let cell_id_ = cell_id.clone();
        let app_ws: AppWebsocket = props.app_ws_js.clone().into();
        let app_ws_: AppWebsocket = app_ws.clone();
        ctx.link().send_future(async move {
            let cmd = AppWsCmd::CallZome {
                cell_id: cell_id_.clone(),
                zome_name: MEMEZ_ZOME_NAME.into(),
                fn_name: "get_all_memez".into(),
                payload: JsValue::NULL,
                provenance: cell_id_.1.clone(),
                cap: "".into(),
            };
            let resp = app_ws_.call(cmd).await;
            match resp {
                Ok(AppWsCmdResponse::CallZome(val)) => {
                    Msg::ZomeCallResponse(ZomeCallResponse::Memes(
                        MemeEhScoreVec::deserialize_from_js_obj_(val)
                            .into_iter()
                            .map(|x| x.into())
                            .collect(),
                    ))
                }
                Ok(resp) => Msg::Error(format!("impossible: invalid response: {:?}", resp)),
                Err(err) => Msg::Error(format!("err: {:?}", err)),
            }
        });
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

        // state machine setup
        let meme_sm: (String, String) = (
            STARTER_SM_INIT_EXPR_STRING.into(),
            STARTER_SM_COMP_EXPR_STRING.into(),
        );

        Self {
            admin_ws,
            app_ws,
            memez_cell_id: cell_id.clone(),
            memez: Vec::new(),
            sensemaker_present: None,
            meme_sm,
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

            Msg::ZomeCallResponse(ZomeCallResponse::Memes(meme_vec)) => {
                self.memez = meme_vec;
                console_log!("got meme_vec");
                true
            }

            Msg::ZomeCallResponse(ZomeCallResponse::UploadMeme(meme_eh, meme)) => {
                self.memez.push((meme_eh, meme, 0));
                true
            }

            Msg::BrowserUploadedMeme(meme) => {
                let ws = self.app_ws.clone();
                let cell_id = self.memez_cell_id.clone();
                ctx.link().send_future(async move {
                    let cmd = AppWsCmd::CallZome {
                        cell_id: cell_id.clone(),
                        zome_name: MEMEZ_ZOME_NAME.into(),
                        fn_name: "upload_meme".into(),
                        payload: meme.clone().serialize_to_js_obj_(),
                        provenance: cell_id.1.clone(),
                        cap: "".into(),
                    };
                    let resp = ws.call(cmd).await;
                    match resp {
                        Ok(AppWsCmdResponse::CallZome(val)) => {
                            let (meme_eh, _meme_hh) =
                                EntryHeaderHashPairRaw::deserialize_from_js_obj_(val);
                            Msg::ZomeCallResponse(ZomeCallResponse::UploadMeme(meme_eh, meme))
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
                self.meme_sm.0 = expr_str;
                true
            }

            Msg::SmCompSubmit(expr_str) => {
                self.set_sm(ctx.link(), expr_str.clone(), "set_sm_comp".into());
                // TODO ideally we would wait for confirmation before setting this
                self.meme_sm.1 = expr_str;
                true
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
                  <p>{"please install it into your `we`, as `memez` requires it to function."}</p>
                </div>
            },
        };
        //
        let sm_init_handler = |input: String| Ok(Msg::SmInitSubmit(input));
        let sm_comp_handler = |input: String| Ok(Msg::SmCompSubmit(input));
        //
        let content_name = "meme";
        let on_file_upload: Callback<FileBytes> = {
            let link = ctx.link().clone();
            Callback::from(move |fb: FileBytes| {
                let meme = Meme {
                    filename: fb.filename,
                    blob_str: encode(fb.bytes),
                };
                link.send_future(async { Msg::BrowserUploadedMeme(meme) })
            })
        };
        let mk_meme_src =
            |meme: Meme| -> String { "data:img;base64,".to_string() + &meme.blob_str };

        html! {
            <div>
                <p>{"hello, memez 👋"}</p>
                <br/>
                {sensemaker_present_html}
                <br/>
                { self.view_string_input(ctx.link(), sm_init_handler, "sm_init".into(), "meme sm_init".into(), self.meme_sm.0.clone()) }
                <br/>
                { self.view_string_input(ctx.link(), sm_comp_handler, "sm_comp".into(), "meme sm_comp".into(), self.meme_sm.1.clone()) }
                <br/>
                <FileUploadApp {content_name} {on_file_upload} />
                <br/>
                <h3 class="subtitle">{"memez"}</h3>
                { for self.memez.iter().map(|triple| html!{
                    <div>
                        <img src={mk_meme_src(triple.1.clone())} width="95%" height="500px" />
                        <p>{ format!("score: {}", triple.2.clone()) }</p>
                    </div>
                }) }
            </div>
        }
    }
}

impl Model {
    // TODO dedup
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
        let cell_id_ = self.memez_cell_id.clone();
        link.send_future(async move {
            let cmd = AppWsCmd::CallZome {
                cell_id: cell_id_.clone(),
                zome_name: MEMEZ_ZOME_NAME.into(),
                fn_name: zome_fn.clone(),
                payload: (MEMEZ_PATH.to_string(), expr_str).serialize_to_js_obj(),
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
