use crate::component::Component;
use crate::scheduler::{scheduler, ComponentRunnableType, Runnable, Shared};
use crate::virtual_dom::{VDiff, VNode};
use crate::{Callback, NodeRef};
use cfg_if::cfg_if;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
cfg_if! {
    if #[cfg(feature = "std_web")] {
        use stdweb::web::Element;
    } else if #[cfg(feature = "web_sys")] {
        use web_sys::Element;
    }
}

/// Updates for a `Component` instance. Used by link sender.
pub(crate) enum ComponentUpdate<COMP: Component> {
    /// First update
    First,
    /// Wraps messages for a component.
    Message(COMP::Message),
    /// Wraps batch of messages for a component.
    MessageBatch(Vec<COMP::Message>),
    /// Wraps properties, node ref, and next sibling for a component.
    Properties(Rc<COMP::Properties>, NodeRef, NodeRef),
}

/// Untyped link used for accessing parent link
#[derive(Debug, Clone)]
pub struct AnyLink {
    pub(crate) type_id: TypeId,
    pub(crate) parent: Option<Rc<AnyLink>>,
    pub(crate) state: Rc<dyn Any>,
}

impl<COMP: Component> From<Link<COMP>> for AnyLink {
    fn from(link: Link<COMP>) -> Self {
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
    pub fn downcast<COMP: Component>(self) -> Link<COMP> {
        Link {
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

impl<COMP: Component> LinkHandle for Link<COMP> {
    fn to_any(&self) -> AnyLink {
        self.clone().into()
    }

    fn root_vnode(&self) -> Option<Ref<'_, VNode>> {
        let state_ref = self.state.borrow();
        state_ref.as_ref().and_then(|state| {
            state
                .last_root
                .as_ref()
                .or_else(|| state.placeholder.as_ref())
        })?;

        Some(Ref::map(state_ref, |state_ref| {
            let state = state_ref.as_ref().unwrap();
            state
                .last_root
                .as_ref()
                .or_else(|| state.placeholder.as_ref())
                .unwrap()
        }))
    }

    /// Schedules a task to destroy a component
    fn destroy(&mut self) {
        let state = self.state.clone();
        let destroy = DestroyComponent { state };
        scheduler().push_comp(ComponentRunnableType::Destroy, Box::new(destroy));
    }
}

/// A link which allows sending messages to a component.
pub struct Link<COMP: Component> {
    pub(crate) parent: Option<Rc<AnyLink>>,
    state: Rc<RefCell<Option<ComponentState<COMP>>>>,
}

impl<COMP: Component> fmt::Debug for Link<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Link<_>")
    }
}

impl<COMP: Component> Clone for Link<COMP> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            state: self.state.clone(),
        }
    }
}

impl<COMP: Component> Link<COMP> {
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
        Link {
            parent,
            state,
        }
    }

    /// Mounts a component with `props` to the specified `element` in the DOM.
    pub(crate) fn mount_in_place(
        self,
        parent: Element,
        next_sibling: NodeRef,
        placeholder: Option<VNode>,
        node_ref: NodeRef,
        props: Rc<COMP::Properties>,
    ) -> Link<COMP> {
        let scheduler = scheduler();
        // Hold scheduler lock so that `create` doesn't run until `update` is scheduled
        let lock = scheduler.lock();
        scheduler.push_comp(
            ComponentRunnableType::Create,
            Box::new(CreateComponent {
                parent,
                next_sibling,
                placeholder,
                node_ref,
                props,
                link: self.clone(),
            }),
        );
        self.update(ComponentUpdate::First);
        drop(lock);
        scheduler.start();
        self
    }

    /// Schedules a task to send an update to a component
    pub(crate) fn update(&self, update: ComponentUpdate<COMP>) {
        let update = UpdateComponent {
            state: self.state.clone(),
            update,
        };
        scheduler().push_comp(ComponentRunnableType::Update, Box::new(update));
    }

    /// Send a message to the component.
    ///
    /// Please be aware that currently this method synchronously
    /// schedules a call to the [Component](Component) interface.
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<COMP::Message>,
    {
        self.update(ComponentUpdate::Message(msg.into()));
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

        self.update(ComponentUpdate::MessageBatch(messages));
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
    fn send(self, link: &Link<COMP>);
}

impl<COMP> SendAsMessage<COMP> for Option<COMP::Message>
where
    COMP: Component,
{
    fn send(self, link: &Link<COMP>) {
        if let Some(msg) = self {
            link.send_message(msg);
        }
    }
}

impl<COMP> SendAsMessage<COMP> for Vec<COMP::Message>
where
    COMP: Component,
{
    fn send(self, link: &Link<COMP>) {
        link.send_message_batch(self);
    }
}

struct ComponentState<COMP: Component> {
    parent: Element,
    next_sibling: NodeRef,
    node_ref: NodeRef,

    link: Link<COMP>,
    component: Box<COMP>,
    props: Rc<COMP::Properties>,

    placeholder: Option<VNode>,
    last_root: Option<VNode>,
    new_root: Option<VNode>,
    has_rendered: bool,
    pending_updates: Vec<Box<UpdateComponent<COMP>>>,
}

use super::Context;
impl<COMP: Component> ComponentState<COMP> {
    /// Creates a new `ComponentState`, also invokes the `create()`
    /// method on component to create it.
    fn new(
        parent: Element,
        next_sibling: NodeRef,
        placeholder: Option<VNode>,
        node_ref: NodeRef,
        link: Link<COMP>,
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

/// A `Runnable` task which creates the `ComponentState` (if there is
/// none) and invokes the `create()` method on a `Component` to create
/// it.
struct CreateComponent<COMP>
where
    COMP: Component,
{
    parent: Element,
    next_sibling: NodeRef,
    placeholder: Option<VNode>,
    node_ref: NodeRef,
    props: Rc<COMP::Properties>,
    link: Link<COMP>,
}

impl<COMP> Runnable for CreateComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let mut current_state = self.link.state.borrow_mut();
        if current_state.is_none() {
            *current_state = Some(ComponentState::new(
                self.parent,
                self.next_sibling,
                self.placeholder,
                self.node_ref,
                self.link.clone(),
                self.props,
            ));
        }
    }
}

/// A `Runnable` task which calls the `update()` method on a `Component`.
struct UpdateComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
    update: ComponentUpdate<COMP>,
}

impl<COMP> Runnable for UpdateComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let state_clone = self.state.clone();
        if let Some(mut state) = state_clone.borrow_mut().as_mut() {
            if state.new_root.is_some() {
                state.pending_updates.push(self);
                return;
            }

            let first_update = matches!(self.update, ComponentUpdate::First);

            let should_update = match self.update {
                ComponentUpdate::First => true,
                ComponentUpdate::Message(message) => {
                    let context = Context::new(&state.link, state.props.as_ref());
                    state.component.update(context, message)
                }
                ComponentUpdate::MessageBatch(messages) => {
                    let component = &mut state.component;
                    let context = Context::new(&state.link, state.props.as_ref());
                    messages.into_iter().fold(false, |acc, msg| {
                        component.update(context, msg) || acc
                    })
                }
                ComponentUpdate::Properties(props, node_ref, next_sibling) => {
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
                scheduler().push_comp(
                    ComponentRunnableType::Render,
                    Box::new(RenderComponent {
                        state: self.state,
                        first_render: first_update,
                    }),
                );
            };
        };
    }
}

/// A `Runnable` task which renders a `Component`.
struct RenderComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
    first_render: bool,
}

impl<COMP> Runnable for RenderComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let state_clone = self.state.clone();
        if let Some(mut state) = self.state.borrow_mut().as_mut() {
            // Skip render if we haven't seen the "first render" yet
            if !self.first_render && state.last_root.is_none() {
                return;
            }

            if let Some(mut new_root) = state.new_root.take() {
                let last_root = state.last_root.take().or_else(|| state.placeholder.take());
                let parent_link = state.link.clone().into();
                let next_sibling = state.next_sibling.clone();
                let node = new_root.apply(&parent_link, &state.parent, next_sibling, last_root);
                state.node_ref.link(node);
                state.last_root = Some(new_root);
                scheduler().push_comp(
                    ComponentRunnableType::Rendered,
                    Box::new(RenderedComponent {
                        state: state_clone,
                        first_render: self.first_render,
                    }),
                );
            }
        }
    }
}

/// A `Runnable` task which calls the `rendered()` method on a `Component`.
struct RenderedComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
    first_render: bool,
}

impl<COMP> Runnable for RenderedComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        if let Some(mut state) = self.state.borrow_mut().as_mut() {
            // Don't call rendered if we haven't seen the "first render" yet
            if !self.first_render && !state.has_rendered {
                return;
            }

            state.has_rendered = true;
            let context = Context::new(&state.link, state.props.as_ref());
            state.component.rendered(context, self.first_render);
            if !state.pending_updates.is_empty() {
                scheduler().push_comp_update_batch(
                    state
                        .pending_updates
                        .drain(..)
                        .map(|u| u as Box<dyn Runnable>),
                );
            }
        }
    }
}

/// A `Runnable` task which calls the `destroy()` method on a `Component`.
struct DestroyComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
}

impl<COMP> Runnable for DestroyComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
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

#[cfg(test)]
mod tests {
    extern crate self as yew;

    use super::*;
    use crate::component::{Component, Properties, ShouldRender};
    use crate::html;
    use crate::html::Html;

    use std::ops::Deref;
    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone, PartialEq, Properties, Default)]
    struct ChildProps {
        lifecycle: Rc<RefCell<Vec<String>>>,
    }

    struct Child;
    impl Component for Child {
        type Message = ();
        type Properties = ChildProps;

        fn create(_: Context<Self>) -> Self {
            Child
        }

        fn rendered(&mut self, ctx: Context<Self>, _first_render: bool) {
            ctx.props
                .lifecycle
                .borrow_mut()
                .push("child rendered".into());
        }

        fn view(&self, _ctx: Context<Self>) -> Html {
            html! {}
        }
    }

    #[derive(Clone, PartialEq, Properties, Default)]
    struct Props {
        lifecycle: Rc<RefCell<Vec<String>>>,
        create_message: Option<bool>,
        update_message: RefCell<Option<bool>>,
        view_message: RefCell<Option<bool>>,
        rendered_message: RefCell<Option<bool>>,
    }

    struct Comp;
    impl Component for Comp {
        type Message = bool;
        type Properties = Props;

        fn create(ctx: &Context<Self>) -> Self {
            ctx.props.lifecycle.borrow_mut().push("create".into());
            if let Some(msg) = ctx.props.create_message {
                ctx.send_message(msg);
            }
            Comp
        }

        fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
            if let Some(msg) = ctx.props.rendered_message.borrow_mut().take() {
                ctx.send_message(msg);
            }
            ctx.props
                .lifecycle
                .borrow_mut()
                .push(format!("rendered({})", first_render));
        }

        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
            if let Some(msg) = ctx.props.update_message.borrow_mut().take() {
                ctx.send_message(msg);
            }
            ctx.props
                .lifecycle
                .borrow_mut()
                .push(format!("update({})", msg));
            msg
        }

        fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> ShouldRender {
            ctx.props.lifecycle.borrow_mut().push("change".into());
            false
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            if let Some(msg) = ctx.props.view_message.borrow_mut().take() {
                ctx.send_message(msg);
            }
            ctx.props.lifecycle.borrow_mut().push("view".into());
            html! { <Child lifecycle=ctx.props.lifecycle.clone() /> }
        }

        fn destroy(&mut self, ctx: &Context<Self>) {
            ctx.props.lifecycle.borrow_mut().push("destroy".into());
        }
    }

    fn test_lifecycle(props: Props, expected: &[String]) {
        let document = crate::utils::document();
        let lifecycle = props.lifecycle.clone();
        let link = Context::<Comp>::new(None, Rc::new(props));
        let el = document.create_element("div").unwrap();

        lifecycle.borrow_mut().clear();
        link.mount_in_place(el, NodeRef::default(), None, NodeRef::default());

        assert_eq!(&lifecycle.borrow_mut().deref()[..], expected);
    }

    #[test]
    fn lifecycle_tests() {
        let lifecycle: Rc<RefCell<Vec<String>>> = Rc::default();

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                create_message: Some(false),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(false)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                view_message: RefCell::new(Some(true)),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(true)".to_string(),
                "view".to_string(),
                "rendered(false)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                view_message: RefCell::new(Some(false)),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(false)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                rendered_message: RefCell::new(Some(false)),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(false)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                rendered_message: RefCell::new(Some(true)),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(true)".to_string(),
                "view".to_string(),
                "rendered(false)".to_string(),
            ],
        );

        test_lifecycle(
            Props {
                lifecycle,
                create_message: Some(true),
                update_message: RefCell::new(Some(true)),
                ..Props::default()
            },
            &[
                "create".to_string(),
                "view".to_string(),
                "child rendered".to_string(),
                "rendered(true)".to_string(),
                "update(true)".to_string(),
                "view".to_string(),
                "rendered(false)".to_string(),
                "update(true)".to_string(),
                "view".to_string(),
                "rendered(false)".to_string(),
            ],
        );
    }
}
