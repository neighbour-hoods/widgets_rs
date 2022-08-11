use js_sys::Object;
use weblog::{console_error, console_log};
use yew::prelude::*;

use crate::{
    holochain_client_wrapper::{connect, AdminWebsocket},
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

pub enum AdminWsCmd {
    ActivateApp,
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
            Msg::AdminWsCmd(AdminWsCmd::ActivateApp) => {
                let ws_clone = self.admin_ws.clone();
                match ws_clone {
                    AdminWsState::Absent(_err) => console_log!("activateApp but no admin ws"),
                    AdminWsState::Present(ws) => console_log!("activateApp w/ admin ws"),
                };
                false
            }
            Msg::AdminWsCmdResponse(resp) => {
                match resp {
                    AdminWsCmdResponse::Success => {}
                    AdminWsCmdResponse::Error(err) => {
                        console_error!("AdminWsCmdResponse: error: {}", err);
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

                <button onclick={ctx.link().callback(|_| Msg::AdminWsCmd(AdminWsCmd::ActivateApp))}>{ "activateApp" }</button>
            </div>
        }
    }
}
