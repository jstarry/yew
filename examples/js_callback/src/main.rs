#![allow(deprecated)]

use wasm_bindgen::prelude::*;
use yew::component::{Component, Context, ShouldRender};
use yew::prelude::*;

mod bindings;

pub enum Msg {
    Payload(String),
    AsyncPayload,
}

pub struct Model {
    payload: String,
    // Pointless field just to have something that's been manipulated
    debugged_payload: String,
}

impl LegacyComponent for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            payload: String::default(),
            debugged_payload: format!("{:?}", ""),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Payload(payload) => {
                if payload != self.payload {
                    self.debugged_payload = format!("{:?}", payload);
                    self.payload = payload;
                    true
                } else {
                    false
                }
            }
            Msg::AsyncPayload => {
                let callback = ctx.callback(Msg::Payload);
                bindings::get_payload_later(Closure::once_into_js(move |payload: String| {
                    callback.emit(payload)
                }));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <textarea
                    class="code-block"
                    oninput=ctx.callback(|input: InputData| Msg::Payload(input.value))
                    value=&self.payload
                />
                <button onclick=ctx.callback(|_| Msg::Payload(bindings::get_payload()))>
                    { "Get the payload!" }
                </button>
                <button onclick=ctx.callback(|_| Msg::AsyncPayload) >
                    { "Get the payload later!" }
                </button>
                <p class="code-block">
                    { &self.debugged_payload }
                </p>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Legacy<Model>>();
}
