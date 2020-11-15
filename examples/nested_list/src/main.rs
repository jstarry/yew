mod app;
mod header;
mod item;
mod list;

use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use yew::component::{Component, Context};

pub struct WeakContextRef<COMP: Component>(Rc<RefCell<Option<Context<COMP>>>>);

impl<COMP: Component> Clone for WeakContextRef<COMP> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<COMP: Component> Default for WeakContextRef<COMP> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<COMP: Component> Deref for WeakContextRef<COMP> {
    type Target = Rc<RefCell<Option<Context<COMP>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<COMP: Component> PartialEq for WeakContextRef<COMP> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug)]
pub enum Hovered {
    Header,
    Item(String),
    List,
    None,
}

impl fmt::Display for Hovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Hovered::Header => "Header",
                Hovered::Item(name) => name,
                Hovered::List => "List container",
                Hovered::None => "Nothing",
            }
        )
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
}
