use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub struct Model {
    link: ComponentLink<Self>,
}

pub enum Msg {
    Click,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link }
    }

    fn change(&mut self, _: &Self::Properties, _: &Self::Properties) -> bool {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {}
        }
        true
    }

    fn view(&self, _: &Self::Properties) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::Click)>{ "Click ( wasm-bindgen )" }</button>
            </div>
        }
    }
}
