//! This module contains the implementation of a virtual component `VComp`.

use super::{VDiff, VNode};
use crate::callback::Callback;
use crate::html::{Component, ComponentUpdate, NodeCell, Renderable, Scope};
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::unstable::TryInto;
use stdweb::web::{document, Element, INode, Node};

struct Hidden;

type HiddenScope = *mut Hidden;

/// The method generates an instance of a (child) component.
type Generator<COMP> = dyn FnOnce(GeneratorType, Scope<COMP>) -> Mounted;

/// Components can be generated by mounting or by overwriting an old component.
enum GeneratorType {
    Mount(Element, Node),
    Overwrite(TypeId, HiddenScope, NodeCell),
}

/// A reference to unknown scope which will be attached later with a generator function.
pub type ScopeHolder<COMP> = Rc<RefCell<Option<Scope<COMP>>>>;

/// A reference to unknown scope which will be attached later with a generator function.
pub type PropsHolder<COMP> = Rc<RefCell<<COMP as Component>::Properties>>;

// TODO think about making this a trait... then I could call getProps() instead.. and have an
// associated type for PARENT so that devs don't have to htink about hte parent type

/// A virtual child component .
pub struct VChild<SELF: Component, PARENT: Component> {
    /// The component properties
    pub props: SELF::Properties,
    /// The component scope
    pub scope: ScopeHolder<PARENT>,
}

impl<SELF, PARENT> VChild<SELF, PARENT>
where
    SELF: Component,
    PARENT: Component,
{
    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn new(props: SELF::Properties, scope: ScopeHolder<PARENT>) -> Self {
        Self { props, scope }
    }
}

impl<COMP, CHILD> From<&VChild<CHILD, COMP>> for VComp<COMP>
where
    COMP: Component,
    CHILD: Component + Renderable<CHILD>,
    CHILD::Properties: Clone,
{
    fn from(vchild: &VChild<CHILD, COMP>) -> Self {
        VComp::new::<CHILD>(vchild.props.clone(), vchild.scope.clone())
    }
}

impl<COMP, CHILD> From<VChild<CHILD, COMP>> for VComp<COMP>
where
    COMP: Component,
    CHILD: Component + Renderable<CHILD>,
    CHILD::Properties: Clone,
{
    fn from(vchild: VChild<CHILD, COMP>) -> Self {
        VComp::new::<CHILD>(vchild.props, vchild.scope)
    }
}

/// A virtual component.
pub struct VComp<COMP: Component> {
    type_id: TypeId,
    state: Rc<RefCell<MountState<COMP>>>,
}

enum MountState<COMP: Component> {
    Unmounted(Unmounted<COMP>),
    Mounted(Mounted),
    Mounting,
    Detached,
    Overwritten,
}

struct Unmounted<COMP: Component> {
    generator: Box<Generator<COMP>>,
}

struct Mounted {
    occupied: NodeCell,
    scope: HiddenScope,
    destroyer: Box<dyn FnOnce()>,
}

impl<COMP: Component> VComp<COMP> {
    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn new<CHILD>(props: CHILD::Properties, scope_holder: ScopeHolder<COMP>) -> Self
    where
        CHILD: Component + Renderable<CHILD>,
    {
        let generator = move |generator_type: GeneratorType, parent: Scope<COMP>| -> Mounted {
            *scope_holder.borrow_mut() = Some(parent);
            match generator_type {
                GeneratorType::Mount(element, ancestor) => {
                    let occupied: NodeCell = Rc::new(RefCell::new(None));
                    let scope: Scope<CHILD> = Scope::new();

                    // TODO Consider to send ComponentUpdate::Create after `mount_in_place` call
                    let mut scope = scope.mount_in_place(
                        element,
                        Some(VNode::VRef(ancestor)),
                        Some(occupied.clone()),
                        props,
                    );

                    Mounted {
                        occupied,
                        scope: Box::into_raw(Box::new(scope.clone())) as *mut Hidden,
                        destroyer: Box::new(move || scope.destroy()),
                    }
                }
                GeneratorType::Overwrite(type_id, scope, occupied) => {
                    if type_id != TypeId::of::<CHILD>() {
                        panic!("tried to overwrite a different type of component");
                    }

                    let mut scope = unsafe {
                        let raw: *mut Scope<CHILD> = ::std::mem::transmute(scope);
                        *Box::from_raw(raw)
                    };

                    scope.update(ComponentUpdate::Properties(props));

                    Mounted {
                        occupied,
                        scope: Box::into_raw(Box::new(scope.clone())) as *mut Hidden,
                        destroyer: Box::new(move || scope.destroy()),
                    }
                }
            }
        };

        VComp {
            type_id: TypeId::of::<CHILD>(),
            state: Rc::new(RefCell::new(MountState::Unmounted(Unmounted {
                generator: Box::new(generator),
            }))),
        }
    }
}

/// Converts property and attach empty scope holder which will be activated later.
pub trait Transformer<COMP: Component, FROM, TO> {
    /// Transforms one type to another.
    fn transform(scope_holder: ScopeHolder<COMP>, from: FROM) -> TO;
}

impl<COMP, T> Transformer<COMP, T, T> for VComp<COMP>
where
    COMP: Component,
{
    fn transform(_: ScopeHolder<COMP>, from: T) -> T {
        from
    }
}

impl<'a, COMP, T> Transformer<COMP, &'a T, T> for VComp<COMP>
where
    COMP: Component,
    T: Clone,
{
    fn transform(_: ScopeHolder<COMP>, from: &'a T) -> T {
        from.clone()
    }
}

impl<'a, COMP> Transformer<COMP, &'a str, String> for VComp<COMP>
where
    COMP: Component,
{
    fn transform(_: ScopeHolder<COMP>, from: &'a str) -> String {
        from.to_owned()
    }
}

impl<'a, COMP, F, IN> Transformer<COMP, F, Callback<IN>> for VComp<COMP>
where
    COMP: Component + Renderable<COMP>,
    F: Fn(IN) -> COMP::Message + 'static,
{
    fn transform(scope: ScopeHolder<COMP>, from: F) -> Callback<IN> {
        let callback = move |arg| {
            let msg = from(arg);
            if let Some(ref mut sender) = *scope.borrow_mut() {
                sender.send_message(msg);
            } else {
                panic!("unactivated callback, parent component have to activate it");
            }
        };
        callback.into()
    }
}

impl<COMP: Component> Unmounted<COMP> {
    /// mount a virtual component with a generator.
    fn mount<T: INode>(
        self,
        parent: &T,
        ancestor: Node, // Any dummy expected
        env: Scope<COMP>,
    ) -> Mounted {
        let element: Element = parent
            .as_node()
            .as_ref()
            .to_owned()
            .try_into()
            .expect("element expected to mount VComp");
        (self.generator)(GeneratorType::Mount(element, ancestor), env)
    }

    /// Overwrite an existing virtual component with a generator.
    fn replace(self, type_id: TypeId, old: Mounted, env: Scope<COMP>) -> Mounted {
        (self.generator)(
            GeneratorType::Overwrite(type_id, old.scope, old.occupied),
            env,
        )
    }
}

enum Reform {
    Keep(TypeId, Mounted),
    Before(Option<Node>),
}

impl<COMP> VDiff for VComp<COMP>
where
    COMP: Component + 'static,
{
    type Component = COMP;

    /// Remove VComp from parent.
    fn detach(&mut self, parent: &Element) -> Option<Node> {
        match self.state.replace(MountState::Detached) {
            MountState::Mounted(this) => {
                (this.destroyer)();
                this.occupied.borrow_mut().take().and_then(|node| {
                    let sibling = node.next_sibling();
                    parent
                        .remove_child(&node)
                        .expect("can't remove the component");
                    sibling
                })
            }
            _ => None,
        }
    }

    /// Renders independent component over DOM `Element`.
    /// It compares this with an ancestor `VComp` and overwrites it if it is the same type.
    fn apply(
        &mut self,
        parent: &Element,
        precursor: Option<&Node>,
        ancestor: Option<VNode<Self::Component>>,
        env: &Scope<Self::Component>,
    ) -> Option<Node> {
        match self.state.replace(MountState::Mounting) {
            MountState::Unmounted(this) => {
                let reform = match ancestor {
                    Some(VNode::VComp(mut vcomp)) => {
                        if self.type_id == vcomp.type_id {
                            match vcomp.state.replace(MountState::Overwritten) {
                                MountState::Mounted(mounted) => {
                                    Reform::Keep(vcomp.type_id, mounted)
                                }
                                _ => Reform::Before(None),
                            }
                        } else {
                            let node = vcomp.detach(parent);
                            Reform::Before(node)
                        }
                    }
                    Some(mut vnode) => {
                        let node = vnode.detach(parent);
                        Reform::Before(node)
                    }
                    None => Reform::Before(None),
                };

                let mounted = match reform {
                    Reform::Keep(type_id, mounted) => {
                        // Send properties update when component still be rendered.
                        // But for the first initialization mount gets initial
                        // properties directly without this channel.
                        this.replace(type_id, mounted, env.clone())
                    }
                    Reform::Before(before) => {
                        // This is a workaround, because component should be mounted
                        // over ancestor element if it exists.
                        // There is created an empty text node to be replaced with mount call.
                        let element = document().create_text_node("");
                        if let Some(sibling) = before {
                            parent
                                .insert_before(&element, &sibling)
                                .expect("can't insert dummy element for a component");
                        } else {
                            let precursor = precursor.and_then(|before| before.next_sibling());
                            if let Some(precursor) = precursor {
                                parent
                                    .insert_before(&element, &precursor)
                                    .expect("can't insert dummy element before precursor");
                            } else {
                                parent.append_child(&element);
                            }
                        }
                        let node = element.as_node().to_owned();
                        this.mount(parent, node, env.clone())
                    }
                };

                let node = mounted
                    .occupied
                    .borrow()
                    .as_ref()
                    .map(|node| node.to_owned());
                self.state.replace(MountState::Mounted(mounted));
                node
            }
            state => {
                self.state.replace(state);
                None
            }
        }
    }
}

impl<COMP: Component> PartialEq for VComp<COMP> {
    fn eq(&self, other: &VComp<COMP>) -> bool {
        self.type_id == other.type_id
    }
}
