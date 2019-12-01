use super::Hovered;
use yew::prelude::*;

// component!(
//     #[derive(Default)]
//     pub struct ListHeader;
//
//     #[derive(Clone, Properties)]
//     pub struct Props {
//         #[props(required)]
//         pub on_hover: Callback<Hovered>,
//         #[props(required)]
//         pub text: String,
//     }
// )

pub enum Msg {
    Hover,
}

impl Component for ListHeader {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover => {
                self.props.on_hover.emit(Hovered::Header);
            }
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="list-header" onmouseover=self.callback(|_| Msg::Hover)>
                { &self.props.text }
            </div>
        }
    }
}

// Here is the code that will be generated from the new `component!` macro.
// ========================================================================
pub struct ListHeader {
    id: ::yew::html::ComponentId,
    props: ::std::rc::Rc<Props>,
    state: __yew_ListHeader,
}

impl ::yew::html::_Component for ListHeader {
    type Properties = Props;
    type State = __yew_ListHeader;
    fn create(props: ::std::rc::Rc<Props>) -> Self {
        ListHeader {
            id: ::yew::html::ComponentId::next(),
            state: <__yew_ListHeader as ::yew::html::FromProps<Props>>::from_props(&props),
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

impl ::yew::html::Identifiable for ListHeader {
    fn get_id(&self) -> ::yew::html::ComponentId {
        self.id
    }
}

impl ::std::ops::Deref for ListHeader {
    type Target = __yew_ListHeader;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl ::std::ops::DerefMut for ListHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Clone, Properties)]
pub struct Props {
    #[props(required)]
    pub on_hover: Callback<Hovered>,
    #[props(required)]
    pub text: String,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct __yew_ListHeader;
// ========================================================================
// End of derived content.
