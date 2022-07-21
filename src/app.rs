use yew::prelude::*;
use crate::myclass::MyClass;

pub enum Msg {
    AddOne,
    SubOne,
    SetNumber(u32),
}

pub struct Model {
    value: i64,
    myclass: MyClass,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: 0,
            myclass: MyClass::new(),
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
