use super::vnode::VNode;
use crate::html::{Component, Scope};
use stdweb::web::{Element, Node};

/// Patch for DOM node modification.
pub(crate) enum Patch<ID, T> {
    Add(ID, T),
    Replace(ID, T),
    Remove(ID),
}

/// Reform of a node.
pub(crate) enum Reform {
    /// Don't create a NEW reference (js Node).
    ///
    /// The reference _may still be mutated_.
    Keep,

    /// Create a new reference (js Node).
    ///
    /// The optional `Node` is used to insert the
    /// new node in the correct slot of the parent.
    ///
    /// If it does not exist, a `previous_sibling` must be
    /// specified (see `VDiff::apply()`).
    Before(Option<Node>),
}

// TODO What about to implement `VDiff` for `Element`?
// In makes possible to include ANY element into the tree.
// `Ace` editor embedding for example?

/// This trait provides features to update a tree by calculating a difference against another tree.
pub trait VDiff {
    /// Remove itself from parent and return the next sibling.
    fn detach(&mut self, parent: &Element) -> Option<Node>;

    /// Scoped diff apply to other tree.
    ///
    /// Virtual rendering for the node. It uses parent node and existing children (virtual and DOM)
    /// to check the difference and apply patches to the actual DOM representation.
    ///
    /// Parameters:
    /// - `parent`: the parent node in the DOM.
    /// - `previous_sibling`: the "previous node" in a list of nodes, used to efficiently
    ///   find where to put the node.
    /// - `ancestor`: the node that this node will be replacing in the DOM.
    ///   This method will _always_ remove the `ancestor` from the `parent`.
    /// - `parent_scope`: the parent `Scope` used for passing messages to the parent `Component`.
    ///
    /// ### Internal Behavior Notice:
    ///
    /// Note that these modify the DOM by modifying the reference that _already_ exists
    /// on the `ancestor`. If `self.reference` exists (which it _shouldn't_) this method
    /// will panic.
    ///
    /// The exception to this is obviously `VRef` which simply uses the inner `Node` directly
    /// (always removes the `Node` that exists).
    fn apply<COMP>(
        &mut self,
        parent: &Element,
        previous_sibling: Option<&Node>,
        ancestor: Option<VNode>,
        parent_scope: Scope<COMP>,
    ) -> Option<Node>
    where
        COMP: Component;
}
