use super::{internal, VComp, VList, VTag, VText};
use crate::html::{Component, Renderable, Scope};
use crate::virtual_dom::VNode as TypedNode;
use stdweb::web::{Element, Node};
use std::iter::FromIterator;

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
    VRef(internal::vnode::VNode),
}

impl<COMP: Component> internal::vdiff::VDiff for VNode<COMP> {
    /// Remove VNode from parent.
    fn detach(&mut self, parent: &Element) -> Option<Node> {
        match *self {
            VNode::VTag(ref mut vtag) => vtag.detach(parent),
            VNode::VText(ref mut vtext) => vtext.detach(parent),
            VNode::VComp(ref mut vcomp) => vcomp.detach(parent),
            VNode::VList(ref mut vlist) => vlist.detach(parent),
            VNode::VRef(ref mut vref) => vref.detach(parent),
        }
    }

    fn apply<PARENT: Component>(
        &mut self,
        parent: &Element,
        previous_sibling: Option<&Node>,
        ancestor: Option<TypedNode<PARENT>>,
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
            VNode::VRef(ref mut vref) => {
                vref.apply(parent, previous_sibling, ancestor, parent_scope)
            }
        }
    }
}

impl<COMP: Component> From<VText<COMP>> for VNode<COMP> {
    fn from(vtext: VText<COMP>) -> Self {
        VNode::VText(vtext)
    }
}

impl<COMP: Component> From<VList<COMP>> for VNode<COMP> {
    fn from(vlist: VList<COMP>) -> Self {
        VNode::VList(vlist)
    }
}

impl<COMP: Component> From<VTag<COMP>> for VNode<COMP> {
    fn from(vtag: VTag<COMP>) -> Self {
        VNode::VTag(Box::new(vtag))
    }
}

impl<COMP: Component> From<VComp<COMP>> for VNode<COMP> {
    fn from(vcomp: VComp<COMP>) -> Self {
        VNode::VComp(vcomp)
    }
}

impl<COMP: Component, T: ToString> From<T> for VNode<COMP> {
    fn from(value: T) -> Self {
        VNode::VText(VText::new(value.to_string()))
    }
}

impl<'a, PARENT: Component> From<&'a dyn Renderable<PARENT>> for VNode<PARENT> {
    fn from(value: &'a dyn Renderable<PARENT>) -> Self {
        value.render().into()
    }
}

impl<COMP: Component, A: Into<VNode<COMP>>> FromIterator<A> for VNode<COMP> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let vlist = iter.into_iter().fold(VList::default(), |mut acc, x| {
            acc.add_child(x.into().into());
            acc
        });
        VNode::VList(vlist)
    }
}
