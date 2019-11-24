use super::{internal, VComp, VList, VTag, VText};
use crate::html::{Component, Scope};
use stdweb::web::{Element, Node};

/// Bind virtual element to a DOM reference.
pub enum VNode<COMP: Component> {
    /// A bind between `VTag` and `Element`.
    VTag(Box<VTag<COMP>>),
    /// A bind between `VText` and `TextNode`.
    VText(VText<COMP>),
    /// A bind between `VComp` and `Element`.
    VComp(VComp<COMP>),
    /// A holder for a list of other nodes.
    VList(VList<COMP>),
}

impl<COMP: Component> internal::vdiff::VDiff for VNode<COMP> {
    /// Remove VNode from parent.
    fn detach(&mut self, parent: &Element) -> Option<Node> {
        match *self {
            VNode::VTag(ref mut vtag) => vtag.detach(parent),
            VNode::VText(ref mut vtext) => vtext.detach(parent),
            VNode::VComp(ref mut vcomp) => vcomp.detach(parent),
            VNode::VList(ref mut vlist) => vlist.detach(parent),
        }
    }

    fn apply<PARENT: Component>(
        &mut self,
        parent: &Element,
        previous_sibling: Option<&Node>,
        ancestor: Option<internal::vnode::VNode>,
        parent_scope: Scope<PARENT>,
    ) -> Option<Node> {
        match *self {
            VNode::VTag(ref mut vtag) => {
                vtag.apply(parent, previous_sibling, ancestor, parent_scope)
            }
            VNode::VText(ref mut vtext) => {
                vtext.apply(parent, previous_sibling, ancestor, parent_scope)
            }
            VNode::VComp(ref mut vcomp) => {
                vcomp.apply(parent, previous_sibling, ancestor, parent_scope)
            }
            VNode::VList(ref mut vlist) => {
                vlist.apply(parent, previous_sibling, ancestor, parent_scope)
            }
        }
    }
}
