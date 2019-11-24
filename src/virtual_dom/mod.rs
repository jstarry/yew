//! This module contains the implementation of reactive virtual dom concept.

mod _vcomp;
mod _vlist;
mod _vnode;
mod _vtag;
mod _vtext;
pub(crate) mod internal;

pub use self::internal::vtag::{Classes, Listener, HTML_NAMESPACE, SVG_NAMESPACE};

pub use self::_vcomp::VComp;
pub use self::_vlist::VList;
pub use self::_vnode::VNode;
pub use self::_vtag::VTag;
pub use self::_vtext::VText;
