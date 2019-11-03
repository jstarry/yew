use super::*;
use crate::scheduler::{scheduler, Runnable, Shared};
use crate::virtual_dom::{VDiff, VNode};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use stdweb::web::{Element, Node};

/// Holder for the element.
pub type NodeCell = Rc<RefCell<Option<Node>>>;

/// Updates for a `Components` instance. Used by scope sender.
pub(crate) enum ComponentUpdate<COMP: Component> {
    /// Wraps messages for a component.
    Message(COMP::Message),
    /// Wraps batch of messages for a component.
    MessageBatch(Vec<COMP::Message>),
    /// Wraps properties for a component.
    Properties(COMP::Properties),
}

/// A context which contains a bridge to send a messages to a loop.
/// Mostly services uses it.
pub struct Scope<COMP: Component> {
    shared_state: Shared<ComponentState<COMP>>,
}

impl<COMP: Component> fmt::Debug for Scope<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Scope<_>")
    }
}

impl<COMP: Component> Clone for Scope<COMP> {
    fn clone(&self) -> Self {
        Scope {
            shared_state: self.shared_state.clone(),
        }
    }
}

impl<COMP: Component> Default for Scope<COMP> {
    fn default() -> Self {
        Scope::new()
    }
}

pub(crate) struct MountOptions<COMP>
where
    COMP: Component,
{
    // Mount synchronously (used when mounting child components to avoid re-rendering)
    pub sync: bool,
    // The node that will be replaced after mounting
    pub ancestor: Option<VNode<COMP>>,
    // The holder for the DOM element that this component will be mounted to
    pub occupied: Option<NodeCell>,
}

impl<COMP: Component> Default for MountOptions<COMP> {
    fn default() -> Self {
        MountOptions {
            sync: false,
            ancestor: None,
            occupied: None,
        }
    }
}

impl<COMP: Component> Scope<COMP> {
    /// visible for testing
    pub fn new() -> Self {
        let shared_state = Rc::new(RefCell::new(ComponentState::Empty));
        Scope { shared_state }
    }

    // TODO Consider to use &Node instead of Element as parent
    /// Mounts a component with `props` to the specified `element` in the DOM.
    ///
    /// If `MountOptions.ancestor` is specified, overwrite it.
    /// If `MountOptions.sync` is false, mount asynchonously.
    pub(crate) fn mount_in_place(
        self,
        element: Element,
        props: COMP::Properties,
        opts: MountOptions<COMP>,
    ) -> Scope<COMP> {
        let mut scope = self.clone();
        let MountOptions {
            ancestor,
            occupied,
            sync,
        } = opts;
        let link = ComponentLink::connect(&scope);
        let ready_state = ReadyState {
            env: self.clone(),
            element,
            occupied,
            link,
            props,
            ancestor,
        };
        *scope.shared_state.borrow_mut() = ComponentState::Ready(ready_state);
        scope.create(sync);
        scope
    }

    // Creates and mounts a component.
    //
    // If `sync` is false, create asynchonously.
    pub(crate) fn create(&mut self, sync: bool) {
        let shared_state = self.shared_state.clone();
        let create = Box::new(CreateComponent { shared_state });
        if sync {
            create.run();
        } else {
            scheduler().put_and_try_run(create);
        }
    }

    // Updates a component with a message or with new properties.
    //
    // If `sync` is false, update asynchonously.
    pub(crate) fn update(&mut self, update: ComponentUpdate<COMP>, sync: bool) {
        let update = Box::new(UpdateComponent {
            shared_state: self.shared_state.clone(),
            update,
        });
        if sync {
            update.run();
        } else {
            scheduler().put_and_try_run(update);
        }
    }

    pub(crate) fn destroy(&mut self) {
        let shared_state = self.shared_state.clone();
        let destroy = DestroyComponent { shared_state };
        scheduler().put_and_try_run(Box::new(destroy));
    }

    /// Send a message to the component
    pub fn send_message(&mut self, msg: COMP::Message) {
        self.update(ComponentUpdate::Message(msg), false);
    }

    /// send batch of messages to the component
    pub fn send_message_batch(&mut self, messages: Vec<COMP::Message>) {
        self.update(ComponentUpdate::MessageBatch(messages), false);
    }
}

enum ComponentState<COMP: Component> {
    Empty,
    Ready(ReadyState<COMP>),
    Created(CreatedState<COMP>),
    Processing,
    Destroyed,
}

impl<COMP: Component> fmt::Display for ComponentState<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ComponentState::Empty => "empty",
            ComponentState::Ready(_) => "ready",
            ComponentState::Created(_) => "created",
            ComponentState::Processing => "processing",
            ComponentState::Destroyed => "destroyed",
        };
        write!(f, "{}", name)
    }
}

struct ReadyState<COMP: Component> {
    env: Scope<COMP>,
    element: Element,
    occupied: Option<NodeCell>,
    props: COMP::Properties,
    link: ComponentLink<COMP>,
    ancestor: Option<VNode<COMP>>,
}

impl<COMP: Component> ReadyState<COMP> {
    fn create(self) -> CreatedState<COMP> {
        CreatedState {
            component: COMP::create(self.props, self.link),
            env: self.env,
            element: self.element,
            last_frame: self.ancestor,
            occupied: self.occupied,
        }
    }
}

struct CreatedState<COMP: Component> {
    env: Scope<COMP>,
    element: Element,
    component: COMP,
    last_frame: Option<VNode<COMP>>,
    occupied: Option<NodeCell>,
}

impl<COMP: Component> CreatedState<COMP> {
    /// Called once immediately after the component is created.
    fn mounted(mut self) -> Self {
        if self.component.mounted() {
            self.update()
        } else {
            self
        }
    }

    fn update(mut self) -> Self {
        let mut next_frame = self.component.render();
        let node = next_frame.apply(&self.element, None, self.last_frame, &self.env);
        if let Some(ref mut cell) = self.occupied {
            *cell.borrow_mut() = node;
        }

        self.last_frame = Some(next_frame);
        self
    }
}

struct CreateComponent<COMP>
where
    COMP: Component,
{
    shared_state: Shared<ComponentState<COMP>>,
}

impl<COMP> Runnable for CreateComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let current_state = self.shared_state.replace(ComponentState::Processing);
        self.shared_state.replace(match current_state {
            ComponentState::Ready(state) => {
                ComponentState::Created(state.create().update().mounted())
            }
            ComponentState::Created(_) | ComponentState::Destroyed => current_state,
            ComponentState::Empty | ComponentState::Processing => {
                panic!("unexpected component state: {}", current_state);
            }
        });
    }
}

struct DestroyComponent<COMP>
where
    COMP: Component,
{
    shared_state: Shared<ComponentState<COMP>>,
}

impl<COMP> Runnable for DestroyComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        match self.shared_state.replace(ComponentState::Destroyed) {
            ComponentState::Created(mut this) => {
                this.component.destroy();
                if let Some(last_frame) = &mut this.last_frame {
                    last_frame.detach(&this.element);
                }
            }
            ComponentState::Ready(mut this) => {
                if let Some(ancestor) = &mut this.ancestor {
                    ancestor.detach(&this.element);
                }
            }
            ComponentState::Empty | ComponentState::Destroyed => {}
            s @ ComponentState::Processing => panic!("unexpected component state: {}", s),
        };
    }
}

struct UpdateComponent<COMP>
where
    COMP: Component,
{
    shared_state: Shared<ComponentState<COMP>>,
    update: ComponentUpdate<COMP>,
}

impl<COMP> Runnable for UpdateComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let current_state = self.shared_state.replace(ComponentState::Processing);
        self.shared_state.replace(match current_state {
            ComponentState::Created(mut this) => {
                let should_update = match self.update {
                    ComponentUpdate::Message(message) => this.component.update(message),
                    ComponentUpdate::MessageBatch(messages) => messages
                        .into_iter()
                        .fold(false, |acc, msg| this.component.update(msg) || acc),
                    ComponentUpdate::Properties(props) => this.component.change(props),
                };
                let next_state = if should_update { this.update() } else { this };
                ComponentState::Created(next_state)
            }
            ComponentState::Destroyed => current_state,
            ComponentState::Processing | ComponentState::Ready(_) | ComponentState::Empty => {
                panic!("unexpected component state: {}", current_state);
            }
        });
    }
}
