use super::{Component, ComponentLink, Context};
use crate::scheduler::{scheduler, Runnable, Shared};
use crate::virtual_dom::{VDiff, VNode};
use crate::NodeRef;
use cfg_if::cfg_if;
use std::rc::Rc;
cfg_if! {
    if #[cfg(feature = "std_web")] {
        use stdweb::web::Element;
    } else if #[cfg(feature = "web_sys")] {
        use web_sys::Element;
    }
}

pub struct ComponentState<COMP: Component> {
    parent: Element,
    next_sibling: NodeRef,
    node_ref: NodeRef,

    link: ComponentLink<COMP>,
    pub(crate) component: Box<COMP>,
    pub(crate) props: Rc<COMP::Properties>,

    pub(crate) placeholder: Option<VNode>,
    pub(crate) last_root: Option<VNode>,
    new_root: Option<VNode>,
    has_rendered: bool,
    pending_updates: Vec<UpdateTask<COMP>>,
}

impl<COMP: Component> ComponentState<COMP> {
    /// Creates a new `ComponentState`, also invokes the `create()`
    /// method on component to create it.
    pub(crate) fn new(
        parent: Element,
        next_sibling: NodeRef,
        placeholder: Option<VNode>,
        node_ref: NodeRef,
        link: ComponentLink<COMP>,
        props: Rc<COMP::Properties>,
    ) -> Self {
        let context = Context::new(&link, props.as_ref());
        let component = Box::new(COMP::create(context));
        Self {
            parent,
            next_sibling,
            node_ref,
            link,
            component,
            props,
            placeholder,
            last_root: None,
            new_root: None,
            has_rendered: false,
            pending_updates: Vec::new(),
        }
    }

    fn as_context(&self) -> Context<'_, COMP> {
        Context::new(&self.link, self.props.as_ref())
    }
}

/// Internal Component runnable tasks
pub(crate) enum ComponentTask<COMP: Component> {
    Create(CreateTask<COMP>),
    Update(UpdateTask<COMP>),
    Render(bool),
    Rendered(bool),
    Destroy,
}

impl<COMP: Component> From<CreateTask<COMP>> for ComponentTask<COMP> {
    fn from(create: CreateTask<COMP>) -> Self {
        Self::Create(create)
    }
}

pub(crate) struct CreateTask<COMP: Component> {
    pub(crate) parent: Element,
    pub(crate) next_sibling: NodeRef,
    pub(crate) placeholder: Option<VNode>,
    pub(crate) node_ref: NodeRef,
    pub(crate) props: Rc<COMP::Properties>,
    pub(crate) link: ComponentLink<COMP>,
}

impl<COMP: Component> From<UpdateTask<COMP>> for ComponentTask<COMP> {
    fn from(update: UpdateTask<COMP>) -> Self {
        Self::Update(update)
    }
}

pub(crate) enum UpdateTask<COMP: Component> {
    /// First update
    First,
    /// Wraps messages for a component.
    Message(COMP::Message),
    /// Wraps batch of messages for a component.
    MessageBatch(Vec<COMP::Message>),
    /// Wraps properties, node ref, and next sibling for a component.
    Properties(Rc<COMP::Properties>, NodeRef, NodeRef),
}

pub(crate) struct ComponentRunnable<COMP: Component> {
    pub(crate) state: Shared<Option<ComponentState<COMP>>>,
    pub(crate) task: ComponentTask<COMP>,
}

impl<COMP: Component> Runnable for ComponentRunnable<COMP> {
    fn run(self: Box<Self>) {
        let mut current_state = self.state.borrow_mut();
        match self.task {
            ComponentTask::Create(this) => {
                if current_state.is_none() {
                    *current_state = Some(ComponentState::new(
                        this.parent,
                        this.next_sibling,
                        this.placeholder,
                        this.node_ref,
                        this.link.clone(),
                        this.props,
                    ));
                }
            }
            ComponentTask::Render(first_render) => {
                if let Some(mut state) = self.state.borrow_mut().as_mut() {
                    // Skip render if we haven't seen the "first render" yet
                    if !first_render && state.last_root.is_none() {
                        return;
                    }

                    if let Some(mut new_root) = state.new_root.take() {
                        let last_root = state.last_root.take().or_else(|| state.placeholder.take());
                        let parent_link = state.link.clone().into();
                        let next_sibling = state.next_sibling.clone();
                        let node =
                            new_root.apply(&parent_link, &state.parent, next_sibling, last_root);
                        state.node_ref.link(node);
                        state.last_root = Some(new_root);
                        state.link.run(ComponentTask::Rendered(first_render));
                    }
                }
            }
            ComponentTask::Rendered(first_render) => {
                if let Some(mut state) = self.state.borrow_mut().as_mut() {
                    // Don't call rendered if we haven't seen the "first render" yet
                    if !first_render && !state.has_rendered {
                        return;
                    }

                    state.has_rendered = true;
                    let context = Context::new(&state.link, state.props.as_ref());
                    state.component.rendered(context, first_render);
                    if !state.pending_updates.is_empty() {
                        scheduler().push_comp_update_batch(
                            state
                                .pending_updates
                                .drain(..)
                                .map(|update| Box::new(ComponentRunnable {
                                    state: self.state.clone(),
                                    task: update.into(),
                                }) as Box<dyn Runnable>),
                        );
                    }
                }
            }
            ComponentTask::Update(event) => {
                if let Some(mut state) = current_state.as_mut() {
                    if state.new_root.is_some() {
                        state.pending_updates.push(event);
                        return;
                    }

                    let first_update = matches!(event, UpdateTask::First);

                    let should_update = match event {
                        UpdateTask::First => true,
                        UpdateTask::Message(message) => {
                            let context = Context::new(&state.link, state.props.as_ref());
                            state.component.update(context, message)
                        }
                        UpdateTask::MessageBatch(messages) => {
                            let component = &mut state.component;
                            let context = Context::new(&state.link, state.props.as_ref());
                            messages
                                .into_iter()
                                .fold(false, |acc, msg| component.update(context, msg) || acc)
                        }
                        UpdateTask::Properties(props, node_ref, next_sibling) => {
                            // When components are updated, a new node ref could have been passed in
                            state.node_ref = node_ref;
                            // When components are updated, their siblings were likely also updated
                            state.next_sibling = next_sibling;
                            let should_render = if *state.props != *props {
                                let context = Context::new(&state.link, state.props.as_ref());
                                state.component.changed(context, &props)
                            } else {
                                false
                            };
                            state.props = props;
                            should_render
                        }
                    };

                    if should_update {
                        state.new_root = Some(state.component.view(state.as_context()));
                        state.link.run(ComponentTask::Render(first_update));
                    };
                }
            }
            ComponentTask::Destroy => {
                if let Some(mut state) = self.state.borrow_mut().take() {
                    let context = Context::new(&state.link, state.props.as_ref());
                    state.component.destroy(context);
                    if let Some(last_frame) = &mut state.last_root {
                        last_frame.detach(&state.parent);
                    }
                    state.node_ref.set(None);
                }
            }
        }
    }
}
