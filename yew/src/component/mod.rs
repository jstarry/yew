//! Components wrapped with context including properties, state, and link

#![allow(missing_docs)]

pub(crate) mod context;
mod handler;
mod properties;

pub use context::{AnyContext, Context};
pub use handler::MessageHandler;
pub use properties::Properties;

use crate::html::Html;

/// This type indicates that component should be rendered again.
pub type ShouldRender = bool;

/// Yew component
pub trait Component: Sized + 'static {
    type Properties: Properties;

    fn create(ctx: &Context<Self>) -> Self;
    fn changed(&mut self, _ctx: &Context<Self>, _new_props: &Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html;
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
    fn destroy(&mut self, _ctx: &Context<Self>) {}
}
