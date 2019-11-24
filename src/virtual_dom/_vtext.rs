use super::internal;
use crate::html::Component;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A virtual component.
pub struct VText<COMP: Component> {
    _vtext: internal::vtext::VText,
    _type: PhantomData<COMP>,
}

impl<COMP: Component> Deref for VText<COMP> {
    type Target = internal::vtext::VText;

    fn deref(&self) -> &Self::Target {
        &self._vtext
    }
}

impl<COMP: Component> DerefMut for VText<COMP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vtext
    }
}
