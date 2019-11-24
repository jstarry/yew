use super::internal;
use crate::html::Component;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub use internal::vtag::classes::Classes;
pub use internal::vtag::listener::Listener;

pub struct VTag<COMP: Component> {
    _vtag: internal::vtag::VTag,
    _type: PhantomData<COMP>,
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
