use super::internal;
use crate::html::Component;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// This struct represents a fragment of the Virtual DOM tree.
pub struct VList<COMP: Component> {
    _vlist: internal::vlist::VList,
    _type: PhantomData<COMP>,
}

impl<COMP: Component> Deref for VList<COMP> {
    type Target = internal::vlist::VList;

    fn deref(&self) -> &Self::Target {
        &self._vlist
    }
}

impl<COMP: Component> DerefMut for VList<COMP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vlist
    }
}
