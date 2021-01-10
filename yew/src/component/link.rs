use super::Component;
use super::lifecycle::{ComponentState, ComponentTask, ComponentRunnable, UpdateTask, CreateTask};
use crate::scheduler::{scheduler, Shared};
use crate::virtual_dom::VNode;
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
        self.schedule(ComponentTask::Destroy);
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
        placeholder: Option<VNode>,
        node_ref: NodeRef,
        props: Rc<COMP::Properties>,
    ) -> ComponentLink<COMP> {
        self.schedule(UpdateTask::First.into());
        self.run(ComponentTask::Create(CreateTask {
            parent,
            next_sibling,
            placeholder,
            node_ref,
            props,
            link: self.clone(),
        }));
        self
    }

    pub(crate) fn run(&self, task: ComponentTask<COMP>) {
        let scheduler = scheduler();
        scheduler.component.push(Box::new(ComponentRunnable {
            state: self.state.clone(),
            task
        }));
        scheduler.start();
    }

    fn schedule(&self, task: ComponentTask<COMP>) {
        let scheduler = &scheduler().component;
        scheduler.push(Box::new(ComponentRunnable {
            state: self.state.clone(),
            task
        }));
    }

    /// Send a message to the component.
    ///
    /// Please be aware that currently this method synchronously
    /// schedules a call to the [Component](Component) interface.
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<COMP::Message>,
    {
        self.run(UpdateTask::Message(msg.into()).into());
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

        self.run(UpdateTask::MessageBatch(messages).into());
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
