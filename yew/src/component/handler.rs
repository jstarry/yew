//! Message handling for components

use super::Component;
use super::Context;

/// Trait for handling messages for a component
pub trait MessageHandler<MSG>: Component {
    /// Handle component message and conditionally re-render
    fn handle(&mut self, ctx: &Context<Self>, msg: MSG) -> super::ShouldRender;
}
