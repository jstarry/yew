use super::internal;
use crate::html::Component;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A virtual component.
pub struct VComp<COMP: Component> {
    pub(crate) _vcomp: internal::vcomp::VComp,
    _type: PhantomData<COMP>,
}

impl<COMP: Component> Deref for VComp<COMP> {
    type Target = internal::vcomp::VComp;

    fn deref(&self) -> &Self::Target {
        &self._vcomp
    }
}

impl<COMP: Component> DerefMut for VComp<COMP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vcomp
    }
}
