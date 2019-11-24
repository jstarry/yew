//! This module contains fragments implementation.
use super::vdiff::VDiff;
use super::vnode::VNode;
use super::vtext::VText;
use crate::html::{Component, Scope};
use crate::virtual_dom::VNode as TypedNode;
use std::ops::{Deref, DerefMut};
use stdweb::web::{Element, Node};

/// This struct represents a fragment of the Virtual DOM tree.
#[derive(Debug, PartialEq, Default)]
pub struct VList {
    /// Whether the fragment has siblings or not.
    pub no_siblings: bool,
    /// The list of children nodes. Which also could have their own children.
    pub children: Vec<VNode>,
}

impl Deref for VList {
    type Target = Vec<VNode>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl DerefMut for VList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl VList {
    /// Creates a new empty `VList` instance.
    pub fn new(no_siblings: bool) -> Self {
        VList {
            no_siblings,
            children: Vec::new(),
        }
    }

    /// Add `VNode` child.
    pub fn add_child(&mut self, child: VNode) {
        self.children.push(child);
    }
}

impl VDiff for VList {
    fn detach(&mut self, parent: &Element) -> Option<Node> {
        let mut last_sibling = None;
        for mut child in self.children.drain(..) {
            last_sibling = child.detach(parent);
        }
        last_sibling
    }

    fn apply<PARENT>(
        &mut self,
        parent: &Element,
        previous_sibling: Option<&Node>,
        ancestor: Option<TypedNode<PARENT>>,
        parent_scope: Scope<PARENT>,
    ) -> Option<Node>
    where
        PARENT: Component,
    {
        // Reuse previous_sibling, because fragment reuse parent
        let mut previous_sibling = previous_sibling.cloned();
        let mut rights = match ancestor {
            // If element matched this type
            Some(TypedNode::VList(vlist)) => {
                // Previously rendered items
                vlist.children()
            }
            Some(vnode) => {
                // Use the current node as a single fragment list
                // and let the `apply` of `VNode` to handle it.
                vec![vnode.into()]
            }
            None => Vec::new(),
        };

        if self.children.is_empty() && !self.no_siblings {
            // Fixes: https://github.com/yewstack/yew/issues/294
            // Without a placeholder the next element becomes first
            // and corrupts the order of rendering
            // We use empty text element to stake out a place
            let placeholder = VText::new("".into());
            self.children.push(placeholder.into());
        }

        // Process children
        let mut lefts = self.children.iter_mut();
        let mut rights = rights.drain(..);
        loop {
            match (lefts.next(), rights.next()) {
                (Some(left), Some(right)) => {
                    previous_sibling = left.apply(
                        parent,
                        previous_sibling.as_ref(),
                        Some(TypedNode::VRef(right)),
                        parent_scope.clone().into(),
                    );
                }
                (Some(left), None) => {
                    previous_sibling = left.apply(
                        parent,
                        previous_sibling.as_ref(),
                        None,
                        parent_scope.clone().into(),
                    );
                }
                (None, Some(ref mut right)) => {
                    right.detach(parent);
                }
                (None, None) => break,
            }
        }
        previous_sibling
    }
}
