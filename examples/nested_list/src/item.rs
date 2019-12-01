use super::Hovered;
use yew::html::Children;
use yew::prelude::*;

// component!(
//     #[derive(Default)]
//     pub struct ListItem;
//
//     #[derive(Clone, Properties)]
//     pub struct Props {
//         pub hide: bool,
//         #[props(required)]
//         pub on_hover: Callback<Hovered>,
//         #[props(required)]
//         pub name: String,
//         pub children: Children,
//     }
// )

pub enum Msg {
    Hover,
}

impl Component for ListItem {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover => {
                self.props
                    .on_hover
                    .emit(Hovered::Item(self.props.name.clone()));
            }
        }
        false
    }

    fn view(&self) -> Html {
        let onmouseover = self.callback(|_| Msg::Hover);
        html! {
            <div class="list-item" onmouseover=onmouseover>
                { &self.props.name }
                { self.view_details() }
            </div>
        }
    }
}

impl ListItem {
    fn view_details(&self) -> Html {
        if self.props.children.is_empty() {
            return html! {};
        }

        html! {
            <div class="list-item-details">
                { self.props.children.render() }
            </div>
        }
    }
}

// Here is the code that will be generated from the new `component!` macro.
// ========================================================================
pub struct ListItem {
    id: ::yew::html::ComponentId,
    props: ::std::rc::Rc<Props>,
    state: __yew_ListItem,
}

impl ::yew::html::_Component for ListItem {
    type Properties = Props;
    type State = __yew_ListItem;
    fn create(props: ::std::rc::Rc<Props>) -> Self {
        ListItem {
            id: ::yew::html::ComponentId::next(),
            state: <__yew_ListItem as ::yew::html::FromProps<Props>>::from_props(&props),
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

impl ::yew::html::Identifiable for ListItem {
    fn get_id(&self) -> ::yew::html::ComponentId {
        self.id
    }
}

impl ::std::ops::Deref for ListItem {
    type Target = __yew_ListItem;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl ::std::ops::DerefMut for ListItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Clone, Properties)]
pub struct Props {
    pub hide: bool,
    #[props(required)]
    pub on_hover: Callback<Hovered>,
    #[props(required)]
    pub name: String,
    pub children: Children,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct __yew_ListItem;
// ========================================================================
// End of derived content.
