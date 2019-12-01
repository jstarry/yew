use super::header::ListHeader;
use super::item::ListItem;
use super::list::List;
use super::Hovered;
use yew::prelude::*;

// component!(
//     #[derive(Default)]
//     pub struct App {
//         hovered: Hovered,
//     }
// )

pub enum Msg {
    Hover(Hovered),
}

impl Component for App {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hover(hovered) => self.hovered = hovered,
        }
        true
    }

    fn view(&self) -> Html {
        let on_hover = &self.callback(Msg::Hover);
        let onmouseenter = self.callback(|_| Msg::Hover(Hovered::None));
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

impl App {
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

// Here is the code that will be generated from the new `component!` macro.
// ========================================================================
pub struct App {
    id: ::yew::html::ComponentId,
    props: ::std::rc::Rc<()>,
    state: __yew_App,
}

impl ::yew::html::_Component for App {
    type Properties = ();
    type State = __yew_App;
    fn create(props: ::std::rc::Rc<()>) -> Self {
        App {
            id: ::yew::html::ComponentId::next(),
            state: <__yew_App as ::yew::html::FromProps<()>>::from_props(&props),
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

impl ::yew::html::Identifiable for App {
    fn get_id(&self) -> ::yew::html::ComponentId {
        self.id
    }
}

impl ::std::ops::Deref for App {
    type Target = __yew_App;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl ::std::ops::DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct __yew_App {
    hovered: Hovered,
}
// ========================================================================
// End of derived content.
