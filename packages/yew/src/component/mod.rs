//! Component trait and related types

mod children;
mod lifecycle;
mod link;
mod properties;

pub use children::*;
pub(crate) use link::LinkHandle;
pub use link::{AnyLink, ComponentLink, SendAsMessage};
pub use properties::*;

use crate::html::Html;
use std::fmt;

/// This type indicates that component should be rendered again.
pub type ShouldRender = bool;

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
    ///# #[derive(PartialEq, Properties)]
    ///# struct Props {
    ///#     prop: String,
    ///# }
    ///# impl Component for Model {
    ///#     type Message = ();type Properties = Props;
    ///#     fn create(_: Context<Self>) -> Self {unimplemented!()}
    ///#     fn view(&self, _: Context<Self>) -> Html {
    /// html! {
    ///     <Model prop="value" />
    /// }
    ///# }}
    /// ```
    type Properties: Properties;

    /// Components are created with a context which provides access to properties as well as a
    /// `ComponentLink` which can be used to send messages and create callbacks for triggering updates.
    fn create(_ctx: Context<'_, Self>) -> Self;

    /// Components handle messages in their `update` method and commonly use this method
    /// to update their state and (optionally) re-render themselves.
    fn update(&mut self, _ctx: Context<'_, Self>, _msg: Self::Message) -> ShouldRender {
        false
    }

    /// When the parent of a Component is re-rendered, it will either be re-created or
    /// receive new properties.  The default behavior is to re-render when new properties
    /// are not equal to the previous properties. Component's can override this behavior
    /// by implementing this lifecycle method.
    fn changed(&mut self, _ctx: Context<'_, Self>, _new_props: &Self::Properties) -> ShouldRender {
        true
    }

    /// Components define their visual layout using a JSX-style syntax through the use of the
    /// `html!` procedural macro. The full guide to using the macro can be found in [Yew's
    /// documentation](https://yew.rs/docs/concepts/html).
    fn view(&self, ctx: Context<'_, Self>) -> Html;

    /// The `rendered` method is called after each time a Component is rendered but
    /// before the browser updates the page.
    fn rendered(&mut self, _ctx: Context<'_, Self>, _first_render: bool) {}

    /// The `destroy` method is called right before a Component is unmounted.
    fn destroy(&mut self, _ctx: Context<'_, Self>) {}
}

/// Component lifecycle context
pub struct Context<'a, COMP: Component> {
    /// Link to component for sending messages and creating callbacks.
    pub link: &'a ComponentLink<COMP>,
    /// Reference to component properties
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
