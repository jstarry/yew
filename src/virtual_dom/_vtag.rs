use super::internal;
use crate::html::Component;
use std::marker::PhantomData;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

pub struct VTag<COMP: Component> {
    pub(crate) _vtag: internal::vtag::VTag,
    _type: PhantomData<COMP>,
}

impl<COMP: Component> VTag<COMP> {
    /// Creates a new `VTag` instance with `tag` name (cannot be changed later in DOM).
    pub fn new<S: Into<Cow<'static, str>>>(tag: S) -> Self {
        VTag {
            _vtag: internal::vtag::VTag::new(tag),
            _type: PhantomData,
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
