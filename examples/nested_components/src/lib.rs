use yew::prelude::*;

mod child;
mod parent;

use child::Child;
use parent::{Msg as ParentMsg, Parent};

pub struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <Parent>
                <Child hide={true} name="Rusty" on_click=|_| ParentMsg::ChildClick>
                    <span>{"Rusty says hi"}</span>
                </Child>
                <Child name="Rustifer" on_click=|_| ParentMsg::ChildClick>
                    <span>{"Rustifer says hello"}</span>
                </Child>
            </Parent>
        }
    }
}
