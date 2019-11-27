#![recursion_limit = "512"]

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
        let on_hover = &self.link.send_back(Msg::Hover);
        let onmouseenter = &self.link.send_back(|_| Msg::Hover(Hovered::None));
        html! {
            <div class="main" onmouseenter=onmouseenter>
                <h1>{ "Nested List Demo" }</h1>
                <List on_hover=on_hover>
                    <ListHeader text="Calling all Rusties!" on_hover=on_hover />
                    <ListItem name="Rustin" on_hover=on_hover />
                    <ListItem hide={true} name="Rustaroo" on_hover=on_hover />
                    <ListItem name="Rustifer" on_hover=on_hover>
                        <div class="sublist">{"Sublist!"}</div>
                        {
                            html! {
                                <List on_hover=on_hover>
                                    <ListHeader text="Sub Rusties!" on_hover=on_hover />
                                    <ListItem name="Sub Rustin" on_hover=on_hover />
                                    <ListItem hide={true} name="Sub Rustaroo" on_hover=on_hover />
                                    <ListItem name="Sub Rustifer" on_hover=on_hover />
                                </List>
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
    fn view_last_hovered(&self) -> Html {
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
