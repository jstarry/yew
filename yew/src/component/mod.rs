//! Component trait and related types

#![allow(missing_docs)]

mod children;
pub(crate) mod lifecycle;
pub(crate) mod link;
mod properties;

use crate::html::Html;
pub use children::*;
pub use properties::*;
pub use link::{AnyLink, ComponentLink};
use std::fmt;

/// This type indicates that component should be rendered again.
pub type ShouldRender = bool;

/// Component lifecycle context
pub struct Context<'a, COMP: Component> {
    pub link: &'a ComponentLink<COMP>,
    pub props: &'a COMP::Properties,
}

impl<COMP: Component> fmt::Debug for Context<'_, COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Context<_>")
    }
}

impl<COMP: Component> Clone for Context<'_, COMP> {
    fn clone(&self) -> Self {
        Self {
            link: Clone::clone(&self.link),
            props: self.props.clone(),
        }
    }
}
impl<COMP: Component> Copy for Context<'_, COMP> {}

impl<'a, COMP: Component> Context<'a, COMP> {
    pub(crate) fn new(link: &'a ComponentLink<COMP>, props: &'a COMP::Properties) -> Self {
        Self { link, props }
    }
}

/// Components are the basic building blocks of the UI in a Yew app. Each Component
/// chooses how to display itself using received props and self-managed state.
/// Components can be dynamic and interactive by declaring messages that are
/// triggered and handled asynchronously. This async update mechanism is inspired by
/// Elm and the actor model used in the Actix framework.
pub trait Component: Sized + 'static {
    /// Messages are used to make Components dynamic and interactive. Simple
    /// Component's can declare their Message type to be `()`. Complex Component's
    /// commonly use an enum to declare multiple Message types.
    type Message: 'static;

    /// Properties are the inputs to a Component and should not mutated within a
    /// Component. They are passed to a Component using a JSX-style syntax.
    /// ```
    ///# use yew::{Html, Component, Properties, ComponentLink, html};
    ///# struct Model;
    ///# #[derive(Clone, Properties)]
    ///# struct Props {
    ///#     prop: String,
    ///# }
    ///# impl Component for Model {
    ///#     type Message = ();type Properties = Props;
    ///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
    ///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {
    /// html! {
    ///     <Model prop="value" />
    /// }
    ///# }}
    /// ```
    type Properties: Properties;

    /// Components are created with their properties as well as a `ComponentLink` which
    /// can be used to send messages and create callbacks for triggering updates.
    fn create(_ctx: Context<'_, Self>) -> Self;

    /// Components handle messages in their `update` method and commonly use this method
    /// to update their state and (optionally) re-render themselves.
    fn update(&mut self, _ctx: Context<'_, Self>, _msg: Self::Message) -> ShouldRender {
        false
    }

    /// When the parent of a Component is re-rendered, it will either be re-created or
    /// receive new properties in the `change` lifecycle method. Component's can choose
    /// to re-render if the new properties are different than the previously
    /// received properties. Most Component's will use props with a `PartialEq`
    /// impl and will be implemented like this:
    /// ```
    ///# use yew::{Html, Component, ComponentLink, html, ShouldRender};
    ///# struct Model{props: ()};
    ///# impl Component for Model {
    ///#     type Message = ();type Properties = ();
    ///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {unimplemented!()}
    /// fn change(&mut self, props: Self::Properties) -> ShouldRender {
    ///     if self.props != props {
    ///         self.props = props;
    ///         true
    ///     } else {
    ///         false
    ///     }
    /// }
    ///# }
    /// ```
    /// Components which don't have properties should always return false.
    fn changed(&mut self, _ctx: Context<'_, Self>, _new_props: &Self::Properties) -> ShouldRender {
        true
    }

    /// Components define their visual layout using a JSX-style syntax through the use of the
    /// `html!` procedural macro. The full guide to using the macro can be found in [Yew's
    /// documentation](https://yew.rs/docs/concepts/html).
    fn view(&self, ctx: Context<'_, Self>) -> Html;

    /// The `rendered` method is called after each time a Component is rendered but
    /// before the browser updates the page.
    /// ## Examples
    /// ```rust
    ///# use yew::{Html, Component, ComponentLink, html, ShouldRender};
    ///# struct Model{props: ()};
    ///# impl Model { fn setup_element(&self) { } }
    ///# impl Component for Model {
    ///#     type Message = ();type Properties = ();
    ///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {unimplemented!()}
    ///#     fn change(&mut self, _props: Self::Properties) -> ShouldRender { unimplemented!() }
    /// fn rendered(&mut self, first_render: bool) {
    ///    if first_render {
    ///      self.setup_element(); // Similar to 'mounted' in other frameworks
    ///    }
    /// }
    ///# }
    /// ```
    fn rendered(&mut self, _ctx: Context<'_, Self>, _first_render: bool) {}

    /// The `destroy` method is called right before a Component is unmounted.
    fn destroy(&mut self, _ctx: Context<'_, Self>) {}
}
