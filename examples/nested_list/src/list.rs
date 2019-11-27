use crate::{header::Props as HeaderProps, ListHeader};
use crate::{item::Props as ItemProps, ListItem};
use std::fmt;
use yew::html::ChildrenRenderer;
use yew::prelude::*;
use yew::virtual_dom::{VChild, VComp, VNode};

#[derive(Debug)]
pub enum Hovered {
    Header,
    Item(String),
    List,
    None,
}

pub enum Msg {
    Hover(Hovered),
}

pub enum Variants {
    Item(<ListItem as Component>::Properties),
    Header(<ListHeader as Component>::Properties),
}

impl From<ItemProps> for Variants {
    fn from(props: ItemProps) -> Self {
        Variants::Item(props)
    }
}

impl From<HeaderProps> for Variants {
    fn from(props: HeaderProps) -> Self {
        Variants::Header(props)
    }
}

pub struct ListVariant {
    props: Variants,
}

#[derive(Properties)]
pub struct Props {
    #[props(required)]
    pub children: ChildrenRenderer<ListVariant>,
}

pub struct List {
    link: ComponentLink<Self>,
    props: Props,
    hovered: Hovered,
}

impl Component for List {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        List {
            link,
            props,
            hovered: Hovered::None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover(hovered) => self.hovered = hovered,
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div
                class="list-container"
                onmouseout=self.link.send_back(|_| Msg::Hover(Hovered::None))
                onmouseover=self.link.send_back(|_| Msg::Hover(Hovered::List))
            >
                <div class="list">
                    {self.view_header()}
                    <div class="items">
                        {self.view_items()}
                    </div>
                </div>
                {self.view_last_hovered()}
            </div>
        }
    }
}

impl List {
    fn view_header(&self) -> Html {
        html! {{
            for self.props.children.iter().filter(|c| match c.props {
                Variants::Header(_) => true,
                _ => false
            })
        }}
    }

    fn view_items(&self) -> Html {
        html! {{
            for self.props.children.iter().filter(|c| match &c.props {
                Variants::Item(props) => !props.hide,
                _ => false,
            }).enumerate().map(|(i, mut c)| {
                if let Variants::Item(ref mut props) = c.props {
                    props.name = format!("#{} - {}", i + 1, props.name);
                }
                c
            })
        }}
    }

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

impl<CHILD> From<VChild<CHILD>> for ListVariant
where
    CHILD: Component,
    CHILD::Properties: Into<Variants>,
{
    fn from(vchild: VChild<CHILD>) -> Self {
        ListVariant {
            props: vchild.props.into(),
        }
    }
}

impl Into<VNode> for ListVariant {
    fn into(self) -> VNode {
        match self.props {
            Variants::Header(props) => VComp::new::<ListHeader>(props, NodeRef::default()).into(),
            Variants::Item(props) => VComp::new::<ListItem>(props, NodeRef::default()).into(),
        }
    }
}
