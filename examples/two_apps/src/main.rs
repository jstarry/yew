use yew::{html, App, Component, Context, Html, ShouldRender};

pub enum Msg {
    SetOpposite(Context<Model>),
    SendToOpposite(String),
    SetTitle(String),
}

pub struct Model {
    opposite: Option<Context<Model>>,
    selector: &'static str,
    title: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Model {
            opposite: None,
            selector: "",
            title: "Nothing".to_owned(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetOpposite(opposite) => {
                self.opposite = Some(opposite);
                false
            }
            Msg::SendToOpposite(title) => {
                self.opposite
                    .as_mut()
                    .unwrap()
                    .send_message(Msg::SetTitle(title));
                false
            }
            Msg::SetTitle(title) => {
                let send_msg = match title.as_ref() {
                    "Ping" => Some(Msg::SetTitle("Pong".into())),
                    "Pong" => Some(Msg::SetTitle("Pong Done".into())),
                    "Pong Done" => Some(Msg::SetTitle("Ping Done".into())),
                    _ => None,
                };

                if let Some(send_msg) = send_msg {
                    self.opposite.as_mut().unwrap().send_message(send_msg);
                }

                self.title = title;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <h3>{ format!("{} received <{}>", self.selector, self.title) }</h3>
                <button onclick=ctx.callback(|_| Msg::SendToOpposite("One".into()))>{ "One" }</button>
                <button onclick=ctx.callback(|_| Msg::SendToOpposite("Two".into()))>{ "Two" }</button>
                <button onclick=ctx.callback(|_| Msg::SendToOpposite("Three".into()))>{ "Three" }</button>
                <button onclick=ctx.callback(|_| Msg::SendToOpposite("Ping".into()))>{ "Ping" }</button>
            </div>
        }
    }
}

fn mount_app(selector: &'static str, app: App<Model>) -> Context<Model> {
    let document = yew::utils::document();
    let element = document.query_selector(selector).unwrap().unwrap();
    app.mount(element)
}

fn main() {
    let first_app = App::new();
    let second_app = App::new();
    let to_first = mount_app(".first-app", first_app);
    let to_second = mount_app(".second-app", second_app);
    to_first.send_message(Msg::SetOpposite(to_second.clone()));
    to_second.send_message(Msg::SetOpposite(to_first));
}
