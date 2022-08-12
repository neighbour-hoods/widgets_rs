use web_sys::HtmlInputElement as InputElement;
use weblog::{console_error, console_log};
use yew::{html::Scope, prelude::*};

use crate::{
    holochain_client_wrapper::{connect, AdminWebsocket, AdminWsCmd},
    myclass::MyClass,
};

pub enum Msg {
    AddOne,
    SubOne,
    SetNumber(u32),
    AdminWsConnected(AdminWebsocket),
    AdminWsError(String),
    AdminWsCmd(AdminWsCmd),
    AdminWsCmdResponse(AdminWsCmdResponse),
}

pub enum AdminWsCmdResponse {
    Success,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum AdminWsState {
    Present(AdminWebsocket),
    Absent(String),
}

pub struct Model {
    value: i64,
    myclass: MyClass,
    admin_ws: AdminWsState,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match connect("ws://localhost:9000".into(), None).await {
                Ok(ws) => Msg::AdminWsConnected(ws),
                Err(err) => Msg::AdminWsError(err),
            }
        });
        Self {
            value: 0,
            myclass: MyClass::new(),
            admin_ws: AdminWsState::Absent("".into()),
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
            Msg::SetNumber(n) => {
                self.myclass.set_number(n);
                true
            }
            Msg::AdminWsConnected(ws) => {
                self.admin_ws = AdminWsState::Present(ws.clone());
                console_log!("Holochain admin ws connected: {:?}", ws);
                true
            }
            Msg::AdminWsError(err) => {
                self.admin_ws = AdminWsState::Absent(err.clone());
                console_error!(err);
                true
            }
            Msg::AdminWsCmd(cmd) => {
                let ws_clone = self.admin_ws.clone();
                match ws_clone {
                    AdminWsState::Absent(_err) => console_log!("AdminWsCmd but no admin ws"),
                    AdminWsState::Present(ws) => {
                        console_log!("AdminWsCmd w/ admin ws");
                        ctx.link().send_future(async move {
                            match ws.call(cmd).await {
                                Ok(_) => Msg::AdminWsCmdResponse(AdminWsCmdResponse::Success),
                                Err(err) => Msg::AdminWsCmdResponse(AdminWsCmdResponse::Error(
                                    format!("{:?}", err),
                                )),
                            }
                        });
                    }
                };
                false
            }
            Msg::AdminWsCmdResponse(resp) => {
                match resp {
                    AdminWsCmdResponse::Success => {}
                    AdminWsCmdResponse::Error(err) => {
                        console_error!("AdminWsCmdResponse: error:", err);
                    }
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ws_debug: String = match &self.admin_ws {
            AdminWsState::Absent(_err) => "admin_ws absent".into(),
            AdminWsState::Present(ws) => format!("typeof: {:?}", ws.js_ws.js_typeof(),),
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
        html! {
            <div>
                <button onclick={ctx.link().callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <button onclick={ctx.link().callback(|_| Msg::SubOne)}>{ "-1" }</button>
                <p>{ self.value }</p>
                <br/>
                <button onclick={ctx.link().callback(|_| Msg::SetNumber(0))}>{ "set number" }</button>
                <p>{self.myclass.render()}</p>

                <p>{format!("{:?}", self.admin_ws)}</p>
                <p>{format!("{:?}", ws_debug)}</p>

                { self.view_string_input(ctx.link(), enable_app_handler, "enable_app".into(), "enable which app?".into()) }
                <br/>
                { self.view_string_input(ctx.link(), disable_app_handler, "disable_app".into(), "disable which app?".into()) }
                <br/>
                { self.view_string_input(ctx.link(), uninstall_app_handler, "uninstall_app".into(), "uninstall which app?".into()) }
                <br/>
                { self.view_string_input(ctx.link(), attach_app_interface_handler, "attach_app_interface".into(), "desired app port?".into()) }
                <br/>
                { mk_nullary_button(AdminWsCmd::GenerateAgentPubKey) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListDnas) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListCellIds) }
                <br/>
                { mk_nullary_button(AdminWsCmd::ListActiveApps) }
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
