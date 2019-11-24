#![recursion_limit = "256"]

mod header;
mod item;
mod list;

use header::ListHeader;
use item::ListItem;
use std::fmt;
use list::List;
use yew::prelude::*;

pub struct Model {
    link: ComponentLink<Self>,
    hovered: Hovered,
}

#[derive(Debug)]
pub enum Hovered {
    Header,
    Item(String),
    List,
    None,
}

impl fmt::Display for Hovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Hovered::Header => "Header",
                Hovered::Item(name) => name,
                Hovered::List => "List container",
                Hovered::None => "Nothing",
            }
        )
    }
}

pub enum Msg {
    Hover(Hovered),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link, hovered: Hovered::None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover(hovered) => self.hovered = hovered,
        }
        true
    }

    fn view(&self) -> Html {
        let on_hover = self.link.send_back(ListMsg::Hover);
        html! {
            <div class="main" onmouseenter=|_| Msg::Hover(Hovered::None)>
                <h1>{ "Nested List Demo" }</h1>
                <List on_hover=on_hover.clone()>
                    <ListHeader text="Calling all Rusties!" on_hover=on_hover.clone() />
                    <ListItem name="Rustin" on_hover=on_hover.clone() />
                    <ListItem hide={true} name="Rustaroo" on_hover=on_hover.clone() />
                    <ListItem name="Rustifer" on_hover=on_hover.clone()>
                        <span>{"Hello!"}</span>
                        {
                            html! {
                                <ListHeader text="Sub Header" on_hover=on_hover.clone() />
                            }
                        }
                    </ListItem>
                </List>
                {self.view_last_hovered()}
            </div>
        }
    }
}

impl Model {
    fn view_last_hovered(&self) -> Html<Self> {
        html! {
            <div class="last-hovered">
                { "Last hovered:"}
                <span class="last-hovered-text">
                    { &self.hovered }
                </span>
            </div>
        }
    }
}
