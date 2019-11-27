#![recursion_limit = "128"]

mod header;
mod item;
mod list;

use header::ListHeader;
use item::ListItem;
use list::{List, Msg as ListMsg};
use yew::prelude::*;

pub struct Model {
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let on_hover = self.link.send_back(ListMsg::Hover);
        html! {
            <div class="main">
                <h1>{ "Nested List Demo" }</h1>
                <List>
                    <ListHeader text="Calling all Rusties!" on_hover=on_hover.clone() />
                    <ListItem name="Rustin" on_hover=on_hover.clone() />
                    <ListItem hide={true} name="Rustaroo" on_hover=on_hover.clone() />
                    <ListItem name="Rustifer" on_hover=on_hover.clone()>
                        <span>{"Hello!"}</span>
                    </ListItem>
                </List>
            </div>
        }
    }
}
