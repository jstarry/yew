//! Component trait and related types

#![allow(missing_docs)]

mod children;
pub(crate) mod lifecycle;
pub(crate) mod link;
mod properties;

pub use children::*;
pub use link::{AnyLink, ComponentLink};
pub use properties::Properties;

use crate::html::Html;
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

/// Yew component
pub trait Component: Sized + 'static {
    type Message: 'static;
    type Properties: Properties;

    fn create(_ctx: Context<'_, Self>) -> Self;
    fn update(&mut self, _ctx: Context<'_, Self>, _msg: Self::Message) -> ShouldRender {
        false
    }
    fn changed(&mut self, _ctx: Context<'_, Self>, _new_props: &Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self, ctx: Context<'_, Self>) -> Html;
    fn rendered(&mut self, _ctx: Context<'_, Self>, _first_render: bool) {}
    fn destroy(&mut self, _ctx: Context<'_, Self>) {}
}
