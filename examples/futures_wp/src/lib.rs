use wasm_bindgen::prelude::*;
use yew::prelude::*;

use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, window, MediaDevices};

pub struct App {
    link: ComponentLink<Self>,
    value: i64,
    _media_devices: MediaDevices,
}

pub enum Msg {
    AddOne,
    SubtractOne,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let window = window().unwrap();
        let navigator = window.navigator();
        let media_devices = navigator.media_devices().unwrap();
        let devices_promise = MediaDevices::enumerate_devices(&media_devices).unwrap();

        let handler = async move {
            let devices = JsFuture::from(devices_promise).await.unwrap();
            console::log_2(&"We have devices".into(), &devices);
        };

        spawn_local(handler);

        Self {
            link,
            value: 0,
            _media_devices: media_devices,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1,
            Msg::SubtractOne => self.value -= 1,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>
                    { "+1" }
                </button>

                <button onclick=self.link.callback(|_| Msg::SubtractOne)>
                    { "-1" }
                </button>

                <p>{ self.value }</p>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::start_app::<App>();
}
