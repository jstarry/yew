use super::Hovered;
use crate::{header::ListHeader, header::Props as HeaderProps};
use crate::{item::ListItem, item::Props as ItemProps};
use yew::html::{_Component, ChildrenRenderer};
use yew::prelude::*;
use yew::virtual_dom::{VChild, VComp, VNode};
use std::rc::Rc;

#[derive(Clone)]
pub enum Variants {
    Item(Rc<<ListItem as _Component>::Properties>),
    Header(Rc<<ListHeader as _Component>::Properties>),
}

impl From<Rc<ItemProps>> for Variants {
    fn from(props: Rc<ItemProps>) -> Self {
        Variants::Item(props)
    }
}

impl From<Rc<HeaderProps>> for Variants {
    fn from(props: Rc<HeaderProps>) -> Self {
        Variants::Header(props)
    }
}

#[derive(Clone)]
pub struct ListVariant {
    props: Variants,
}

// component!(
//     #[derive(Default)]
//     pub struct List;
//
//     #[derive(Clone, Properties)]
//     pub struct Props {
//         #[props(required)]
//         pub children: ChildrenRenderer<ListVariant>,
//         #[props(required)]
//         pub on_hover: Callback<Hovered>,
//     }
// )

pub enum Msg {
    Hover(Hovered),
}

impl Component for List {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover(hovered) => {
                self.props().on_hover.emit(hovered);
            }
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div
                class="list-container"
                onmouseout=self.callback(|_| Msg::Hover(Hovered::None))
                onmouseover=self.callback(|_| Msg::Hover(Hovered::List))
            >
                <div class="list">
                    {self.view_header()}
                    <div class="items">
                        {self.view_items()}
                    </div>
                </div>
            </div>
        }
    }
}

impl List {
    fn view_header(&self) -> Html {
        html! {{
            for self.props().children.iter().filter(|c| match c.props {
                Variants::Header(_) => true,
                _ => false
            })
        }}
    }

    fn view_items(&self) -> Html {
        html! {{
            for self.props().children.iter().filter(|c| match &c.props {
                Variants::Item(props) => !props.hide,
                _ => false,
            })
        }}
    }
}

impl<CHILD> From<VChild<CHILD>> for ListVariant
where
    CHILD: Component,
    Rc<CHILD::Properties>: Into<Variants>,
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

// Here is the code that will be generated from the new `component!` macro.
// ========================================================================
pub struct List {
    id: ::yew::html::ComponentId,
    props: ::std::rc::Rc<Props>,
    state: __yew_List,
}

impl ::yew::html::_Component for List {
    type Properties = Props;
    type State = __yew_List;
    fn create(props: ::std::rc::Rc<Props>) -> Self {
        List {
            id: ::yew::html::ComponentId::next(),
            state: <__yew_List as ::yew::html::FromProps<Props>>::from_props(&props),
            props,
        }
    }
    fn update_props(&mut self, props: ::std::rc::Rc<Self::Properties>) {
        self.props = props;
    }
    fn props(&self) -> &Self::Properties {
        &self.props
    }
}

impl ::yew::html::Identifiable for List {
    fn get_id(&self) -> ::yew::html::ComponentId {
        self.id
    }
}

impl ::std::ops::Deref for List {
    type Target = __yew_List;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl ::std::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Clone, Properties)]
pub struct Props {
    #[props(required)]
    pub children: ChildrenRenderer<ListVariant>,
    #[props(required)]
    pub on_hover: Callback<Hovered>,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct __yew_List;
// ========================================================================
// End of derived content.
