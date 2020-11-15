//! Component link module

use super::lifecycle::{
    ComponentLifecycleEvent, ComponentRunnable, ComponentState, CreateEvent, UpdateEvent,
};
use super::Component;
use crate::scheduler::{scheduler, Shared};
use crate::utils::document;
use crate::virtual_dom::{insert_node, VNode};
use crate::{Callback, NodeRef};
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use web_sys::{Element, Node};

/// Untyped link used for accessing parent link
#[derive(Debug, Clone)]
pub struct AnyLink {
    pub(crate) type_id: TypeId,
    pub(crate) parent: Option<Rc<AnyLink>>,
    pub(crate) state: Rc<dyn Any>,
}

impl<COMP: Component> From<ComponentLink<COMP>> for AnyLink {
    fn from(link: ComponentLink<COMP>) -> Self {
        AnyLink {
            type_id: TypeId::of::<COMP>(),
            parent: link.parent,
            state: Rc::new(link.state),
        }
    }
}

impl AnyLink {
    /// Returns the parent link
    pub fn get_parent(&self) -> Option<&AnyLink> {
        self.parent.as_deref()
    }

    /// Returns the type of the linked component
    pub fn get_type_id(&self) -> &TypeId {
        &self.type_id
    }

    /// Attempts to downcast into a typed link
    pub fn downcast<COMP: Component>(self) -> ComponentLink<COMP> {
        ComponentLink {
            parent: self.parent,
            state: self
                .state
                .downcast_ref::<Shared<Option<ComponentState<COMP>>>>()
                .expect("unexpected component type")
                .clone(),
        }
    }
}

pub(crate) trait LinkHandle {
    fn to_any(&self) -> AnyLink;
    fn root_vnode(&self) -> Option<Ref<'_, VNode>>;
    fn destroy(&mut self);
}

impl<COMP: Component> LinkHandle for ComponentLink<COMP> {
    fn to_any(&self) -> AnyLink {
        self.clone().into()
    }

    fn root_vnode(&self) -> Option<Ref<'_, VNode>> {
        let state_ref = self.state.borrow();

        // check that component hasn't been destroyed
        state_ref.as_ref()?;

        Some(Ref::map(state_ref, |state_ref| {
            &state_ref.as_ref().unwrap().root_node
        }))
    }

    /// Process an event to destroy a component
    fn destroy(&mut self) {
        self.process(ComponentLifecycleEvent::Destroy);
    }
}

/// A link which allows sending messages to a component.
pub struct ComponentLink<COMP: Component> {
    pub(crate) parent: Option<Rc<AnyLink>>,
    state: Rc<RefCell<Option<ComponentState<COMP>>>>,
}

impl<COMP: Component> fmt::Debug for ComponentLink<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Link<_>")
    }
}

impl<COMP: Component> Clone for ComponentLink<COMP> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            state: self.state.clone(),
        }
    }
}

impl<COMP: Component> ComponentLink<COMP> {
    /// Returns the parent link
    pub fn get_parent(&self) -> Option<&AnyLink> {
        self.parent.as_deref()
    }

    // TODO: consider combining this with get_component

    /// Returns the linked component if available
    pub fn get_props(&self) -> Option<impl Deref<Target = COMP::Properties> + '_> {
        self.state.try_borrow().ok().and_then(|state_ref| {
            state_ref.as_ref()?;
            Some(Ref::map(state_ref, |state| {
                state.as_ref().unwrap().props.as_ref()
            }))
        })
    }

    /// Returns the linked component if available
    pub fn get_component(&self) -> Option<impl Deref<Target = COMP> + '_> {
        self.state.try_borrow().ok().and_then(|state_ref| {
            state_ref.as_ref()?;
            Some(Ref::map(state_ref, |state| {
                state.as_ref().unwrap().component.as_ref()
            }))
        })
    }

    pub(crate) fn new(parent: Option<Rc<AnyLink>>) -> Self {
        let state = Rc::new(RefCell::new(None));
        ComponentLink { parent, state }
    }

    /// Mounts a component with `props` to the specified `element` in the DOM.
    pub(crate) fn mount_in_place(
        self,
        parent: Element,
        next_sibling: NodeRef,
        node_ref: NodeRef,
        props: Rc<COMP::Properties>,
    ) -> ComponentLink<COMP> {
        let placeholder = {
            let placeholder: Node = document().create_text_node("").into();
            insert_node(&placeholder, &parent, next_sibling.get());
            node_ref.set(Some(placeholder.clone()));
            VNode::VRef(placeholder)
        };

        self.schedule(UpdateEvent::First.into());
        self.process(ComponentLifecycleEvent::Create(CreateEvent {
            parent,
            next_sibling,
            placeholder,
            node_ref,
            props,
            link: self.clone(),
        }));

        self
    }

    pub(crate) fn reuse(
        &self,
        props: Rc<COMP::Properties>,
        node_ref: NodeRef,
        next_sibling: NodeRef,
    ) {
        self.process(UpdateEvent::Properties(props, node_ref, next_sibling).into());
    }

    pub(crate) fn process(&self, event: ComponentLifecycleEvent<COMP>) {
        let scheduler = scheduler();
        scheduler.component.push(
            event.as_runnable_type(),
            Box::new(ComponentRunnable {
                state: self.state.clone(),
                event,
            }),
        );
        scheduler.start();
    }

    fn schedule(&self, event: ComponentLifecycleEvent<COMP>) {
        let scheduler = &scheduler().component;
        scheduler.push(
            event.as_runnable_type(),
            Box::new(ComponentRunnable {
                state: self.state.clone(),
                event,
            }),
        );
    }

    /// Send a message to the component.
    ///
    /// Please be aware that currently this method synchronously
    /// schedules a call to the [Component](Component) interface.
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<COMP::Message>,
    {
        self.process(UpdateEvent::Message(msg.into()).into());
    }

    /// Send a batch of messages to the component.
    ///
    /// This is useful for reducing re-renders of the components
    /// because the messages are handled together and the view
    /// function is called only once if needed.
    ///
    /// Please be aware that currently this method synchronously
    /// schedules calls to the [Component](Component) interface.
    pub fn send_message_batch(&self, messages: Vec<COMP::Message>) {
        // There is no reason to schedule empty batches.
        // This check is especially handy for the batch_callback method.
        if messages.is_empty() {
            return;
        }

        self.process(UpdateEvent::MessageBatch(messages).into());
    }

    /// Creates a `Callback` which will send a message to the linked
    /// component's update method when invoked.
    ///
    /// Please be aware that currently the result of this callback
    /// synchronously schedules a call to the [Component](Component)
    /// interface.
    pub fn callback<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        M: Into<COMP::Message>,
        F: Fn(IN) -> M + 'static,
    {
        let link = self.clone();
        let closure = move |input| {
            let output = function(input);
            link.send_message(output);
        };
        closure.into()
    }

    /// Creates a `Callback` from an `FnOnce` which will send a message
    /// to the linked component's update method when invoked.
    ///
    /// Please be aware that currently the result of this callback
    /// will synchronously schedule calls to the
    /// [Component](Component) interface.
    pub fn callback_once<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        M: Into<COMP::Message>,
        F: FnOnce(IN) -> M + 'static,
    {
        let link = self.clone();
        let closure = move |input| {
            let output = function(input);
            link.send_message(output);
        };
        Callback::once(closure)
    }

    /// Creates a `Callback` which will send a batch of messages back
    /// to the linked component's update method when invoked.
    ///
    /// The callback function's return type is generic to allow for dealing with both
    /// `Option` and `Vec` nicely. `Option` can be used when dealing with a callback that
    /// might not need to send an update.
    ///
    /// ```ignore
    /// link.batch_callback(|_| vec![Msg::A, Msg::B]);
    /// link.batch_callback(|_| Some(Msg::A));
    /// ```
    ///
    /// Please be aware that currently the results of these callbacks
    /// will synchronously schedule calls to the
    /// [Component](Component) interface.
    pub fn batch_callback<F, IN, OUT>(&self, function: F) -> Callback<IN>
    where
        F: Fn(IN) -> OUT + 'static,
        OUT: SendAsMessage<COMP>,
    {
        let link = self.clone();
        let closure = move |input| {
            let messages = function(input);
            messages.send(&link);
        };
        closure.into()
    }

    /// Creates a `Callback` from an `FnOnce` which will send a batch of messages back
    /// to the linked component's update method when invoked.
    ///
    /// The callback function's return type is generic to allow for dealing with both
    /// `Option` and `Vec` nicely. `Option` can be used when dealing with a callback that
    /// might not need to send an update.
    ///
    /// ```ignore
    /// link.batch_callback_once(|_| vec![Msg::A, Msg::B]);
    /// link.batch_callback_once(|_| Some(Msg::A));
    /// ```
    ///
    /// Please be aware that currently the results of these callbacks
    /// will synchronously schedule calls to the
    /// [Component](Component) interface.
    pub fn batch_callback_once<F, IN, OUT>(&self, function: F) -> Callback<IN>
    where
        F: FnOnce(IN) -> OUT + 'static,
        OUT: SendAsMessage<COMP>,
    {
        let link = self.clone();
        let closure = move |input| {
            let messages = function(input);
            messages.send(&link);
        };
        Callback::once(closure)
    }
}

/// Defines a message type that can be sent to a component.
/// Used for the return value of closure given to [Link::batch_callback](struct.Link.html#method.batch_callback).
pub trait SendAsMessage<COMP: Component> {
    /// Sends the message to the given component's link.
    /// See [Link::batch_callback](struct.Link.html#method.batch_callback).
    fn send(self, link: &ComponentLink<COMP>);
}

impl<COMP> SendAsMessage<COMP> for Option<COMP::Message>
where
    COMP: Component,
{
    fn send(self, link: &ComponentLink<COMP>) {
        if let Some(msg) = self {
            link.send_message(msg);
        }
    }
}

impl<COMP> SendAsMessage<COMP> for Vec<COMP::Message>
where
    COMP: Component,
{
    fn send(self, link: &ComponentLink<COMP>) {
        link.send_message_batch(self);
    }
}
