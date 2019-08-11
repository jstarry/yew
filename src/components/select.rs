//! This module contains implementation of `Select` component.
//! You can use it instead `<select>` tag, because the component
//! helps you to track selected value in an original type. Example:
//!
//! ```
//! enum Scene {
//!     First,
//!     Second,
//! }
//!
//! fn view() -> Html<Model> {
//!     let scenes = vec![Scene::First, Scene::Second];
//!     html! {
//!         <Select<Scenes> options=scenes />
//!     }
//! }

use crate::callback::Callback;
use crate::html::{ChangeData, Component, ComponentLink, Html, Renderable, ShouldRender};
use crate::macros::{html, Properties};

/// `Select` component.
pub struct Select<T: PartialEq + Clone + 'static> {
    props: Props<T>,
}

/// Internal message of the component.
pub enum Msg {
    /// This message indicates the option with id selected.
    Selected(Option<usize>),
}

#[derive(Properties)]
pub struct LifetimeProps<'a, T: Default + 'a> {
    value: &'a T,
    #[props(required)]
    required_value: &'a T,
}

/// Properties of `Select` component.
#[derive(PartialEq, Properties)]
pub struct Props<T: PartialEq + Clone + 'static> {
    /// Initially selected value.
    pub selected: Option<T>,
    /// Disabled the component's selector.
    pub disabled: bool,
    /// Options are available to choose.
    pub options: Vec<T>,
    /// Callback to handle changes.
    #[props(required)]
    pub onchange: Callback<T>,
}

impl<T> Component for Select<T>
where
    T: PartialEq + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Selected(value) => {
                if let Some(idx) = value {
                    let item = self.props.options.get(idx - 1).cloned();
                    if let Some(value) = item {
                        self.props.onchange.emit(value);
                    }
                }
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl<T> Renderable<Select<T>> for Select<T>
where
    T: ToString + PartialEq + Clone + 'static,
{
    fn view(&self) -> Html<Self> {
        let selected = self.props.selected.as_ref();
        let view_option = |value: &T| {
            let flag = selected == Some(value);
            html! {
                <option selected=flag>{ value.to_string() }</option>
            }
        };
        html! {
            <select disabled=self.props.disabled
                    onchange=|event| {
                        match event {
                            ChangeData::Select(elem) => {
                                let value = elem.selected_index().map(|x| x as usize);
                                Msg::Selected(value)
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }>
                <option disabled=true selected=selected.is_none()>
                    { "↪" }
                </option>
                { for self.props.options.iter().map(view_option) }
            </select>
        }
    }
}
