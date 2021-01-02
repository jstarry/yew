//! Component trait and related types

#![allow(missing_docs)]

mod children;
pub(crate) mod context;
mod properties;

pub use children::*;
pub use context::{AnyContext, Context};
pub use properties::Properties;

use crate::html::Html;

/// This type indicates that component should be rendered again.
pub type ShouldRender = bool;

/// Yew component
pub trait Component: Sized + 'static {
    type Message: 'static;
    type Properties: Properties;

    fn create(ctx: &Context<Self>) -> Self;
    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> ShouldRender {
        false
    }
    fn changed(&mut self, _ctx: &Context<Self>, _new_props: &Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html;
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
    fn destroy(&mut self, _ctx: &Context<Self>) {}
}
