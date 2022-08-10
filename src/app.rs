use yew::prelude::*;
use crate::{
    myclass::MyClass,
    holochain_client_wrapper::{AdminWebsocket, connect_wrapper},
};

pub enum Msg {
    AddOne,
    SubOne,
    SetNumber(u32),
    AdminWsConnected(AdminWebsocket),
    AdminWsError(String),
}

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
            match connect_wrapper("localhost:9999".into(), None).await {
                Ok(ws) => Msg::AdminWsConnected(ws),
                Err(err) => Msg::AdminWsError(format!("{:?}", err)),
            }
        });
        Self {
            value: 0,
            myclass: MyClass::new(),
            admin_ws: AdminWsState::Absent("".into()),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                self.admin_ws = AdminWsState::Present(ws);
                true
            }
            Msg::AdminWsError(err) => {
                self.admin_ws = AdminWsState::Absent(err);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <button onclick={ctx.link().callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <button onclick={ctx.link().callback(|_| Msg::SubOne)}>{ "-1" }</button>
                <p>{ self.value }</p>
                <br/>
                <button onclick={ctx.link().callback(|_| Msg::SetNumber(0))}>{ "set number" }</button>
                <p>{self.myclass.render()}</p>
            </div>
        }
    }
}
