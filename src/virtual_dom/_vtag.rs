use super::internal;
use crate::html::{ScopeHolder, Component};
use std::marker::PhantomData;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use super::VNode;

pub struct VTag<COMP: Component> {
    pub(crate) _vtag: internal::vtag::VTag,
    _type: PhantomData<COMP>,
}

impl<COMP: Component> VTag<COMP> {
    /// Creates a new `VTag` instance with `tag` name (cannot be changed later in DOM).
    pub fn new_with_scope<S: Into<Cow<'static, str>>>(tag: S, scope_holder: ScopeHolder<COMP>) -> Self {
        VTag {
            _vtag: internal::vtag::VTag::new_with_scope(tag, scope_holder),
            _type: PhantomData,
        }
    }

    /// Creates a new `VTag` instance with `tag` name (cannot be changed later in DOM).
    pub fn new<S: Into<Cow<'static, str>>>(tag: S) -> Self {
        VTag {
            _vtag: internal::vtag::VTag::new(tag),
            _type: PhantomData,
        }
    }

    /// Add `VNode` child.
    pub fn add_child(&mut self, child: VNode<COMP>) {
        self.children.add_child(child.into());
    }

    /// Add multiple `VNode` children.
    pub fn add_children(&mut self, children: Vec<VNode<COMP>>) {
        for child in children {
            self.add_child(child);
        }
    }
}

impl<COMP: Component> Deref for VTag<COMP> {
    type Target = internal::vtag::VTag;

    fn deref(&self) -> &Self::Target {
        &self._vtag
    }
}

impl<COMP: Component> DerefMut for VTag<COMP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vtag
    }
}
