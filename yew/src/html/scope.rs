use super::{Callback, Component, NodeRef};
use crate::scheduler::{scheduler, ComponentRunnableType, Runnable, Shared};
use crate::virtual_dom::{VDiff, VNode};
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

/// Updates for a `Component` instance. Used by scope sender.
pub(crate) enum ComponentUpdate<COMP: Component> {
    /// First update
    First,
    /// Wraps messages for a component.
    Message(COMP::Message),
    /// Wraps batch of messages for a component.
    MessageBatch(Vec<COMP::Message>),
    /// Wraps properties and next sibling for a component.
    Properties(COMP::Properties),
}

/// Untyped scope used for accessing parent scope
#[derive(Debug, Clone)]
pub struct AnyScope {
    pub(crate) type_id: TypeId,
    pub(crate) parent: Option<Rc<AnyScope>>,
    pub(crate) state: Rc<dyn Any>,
}

impl<COMP: Component> From<Scope<COMP>> for AnyScope {
    fn from(scope: Scope<COMP>) -> Self {
        AnyScope {
            type_id: TypeId::of::<COMP>(),
            parent: scope.parent,
            state: Rc::new(scope.state),
        }
    }
}

impl AnyScope {
    /// Returns the parent scope
    pub fn get_parent(&self) -> Option<&AnyScope> {
        self.parent.as_deref()
    }

    /// Returns the type of the linked component
    pub fn get_type_id(&self) -> &TypeId {
        &self.type_id
    }

    /// Attempts to downcast into a typed scope
    pub fn downcast<COMP: Component>(self) -> Scope<COMP> {
        Scope {
            parent: self.parent,
            state: self
                .state
                .downcast_ref::<Shared<Option<ComponentState<COMP>>>>()
                .expect("unexpected component type")
                .clone(),
        }
    }
}

pub(crate) trait Scoped {
    fn to_any(&self) -> AnyScope;
    fn render_sync(&self, parent: Element, next_sibling: NodeRef);
    fn root_vnode(&self) -> Option<Ref<'_, VNode>>;
    fn destroy(&mut self);
}

impl<COMP: Component> Scoped for Scope<COMP> {
    fn to_any(&self) -> AnyScope {
        self.clone().into()
    }

    fn render_sync(&self, parent: Element, next_sibling: NodeRef) {
        if let Some(mut state) = self.state.borrow_mut().as_mut() {
            state.position = Some(Position {
                parent, next_sibling
            });
        }

        let first_render = self.state.borrow().as_ref().map(|state| state.last_root.is_none()).unwrap_or_default();
        Box::new(RenderComponent {
            state: self.state.clone(),
            first_render,
        }).run();
    }

    fn root_vnode(&self) -> Option<Ref<'_, VNode>> {
        let state_ref = self.state.borrow();
        state_ref.as_ref().and_then(|state| {
            state
                .last_root
                .as_ref()
        })?;

        Some(Ref::map(state_ref, |state_ref| {
            let state = state_ref.as_ref().unwrap();
            state
                .last_root
                .as_ref()
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

/// A context which allows sending messages to a component.
pub struct Scope<COMP: Component> {
    parent: Option<Rc<AnyScope>>,
    state: Shared<Option<ComponentState<COMP>>>,
}

impl<COMP: Component> fmt::Debug for Scope<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Scope<_>")
    }
}

impl<COMP: Component> Clone for Scope<COMP> {
    fn clone(&self) -> Self {
        Scope {
            parent: self.parent.clone(),
            state: self.state.clone(),
        }
    }
}

impl<COMP: Component> Scope<COMP> {
    /// Returns the parent scope
    pub fn get_parent(&self) -> Option<&AnyScope> {
        self.parent.as_deref()
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

    pub(crate) fn new(parent: Option<AnyScope>) -> Self {
        let parent = parent.map(Rc::new);
        let state = Rc::new(RefCell::new(None));
        Scope { parent, state }
    }

    pub(crate) fn render(&self, parent: Element, next_sibling: NodeRef) {
        if let Some(mut state) = self.state.borrow_mut().as_mut() {
            state.position = Some(Position {
                parent, next_sibling
            });
        }

        let first_render = self.state.borrow().as_ref().map(|state| state.last_root.is_none()).unwrap_or_default();
        scheduler().push_comp(
            ComponentRunnableType::Render,
            Box::new(RenderComponent {
                state: self.state.clone(),
                first_render,
            })
        );
    }

    /// Mounts a component with `props` to the specified `element` in the DOM.
    pub(crate) fn create(
        self,
        node_ref: NodeRef,
        props: COMP::Properties,
    ) -> Scope<COMP> {
        let scheduler = scheduler();
        // Hold scheduler lock so that `create` doesn't run until `update` is scheduled
        let lock = scheduler.lock();
        scheduler.push_comp(
            ComponentRunnableType::Create,
            Box::new(CreateComponent {
                state: self.state.clone(),
                node_ref,
                scope: self.clone(),
                props,
            }),
        );
        self.update(ComponentUpdate::First);
        drop(lock);
        scheduler.start();
        self
    }

    /// Mounts a component with `props` to the specified `element` in the DOM.
    pub(crate) fn create_sync(
        self,
        node_ref: NodeRef,
        props: COMP::Properties,
    ) -> Scope<COMP> {
        Box::new(CreateComponent {
            state: self.state.clone(),
            node_ref,
            scope: self.clone(),
            props,
        }).run();
        Box::new(UpdateComponent {
            state: self.state.clone(),
            update: ComponentUpdate::First,
        }).run();
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
        let scope = self.clone();
        let closure = move |input| {
            let output = function(input);
            scope.send_message(output);
        };
        closure.into()
    }

    /// Creates a `Callback` from a FnOnce which will send a message
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
        let scope = self.clone();
        let closure = move |input| {
            let output = function(input);
            scope.send_message(output);
        };
        Callback::once(closure)
    }

    /// Creates a `Callback` which will send a batch of messages back
    /// to the linked component's update method when invoked.
    ///
    /// Please be aware that currently the results of these callbacks
    /// will synchronously schedule calls to the
    /// [Component](Component) interface.
    pub fn batch_callback<F, IN>(&self, function: F) -> Callback<IN>
    where
        F: Fn(IN) -> Vec<COMP::Message> + 'static,
    {
        let scope = self.clone();
        let closure = move |input| {
            let messages = function(input);
            scope.send_message_batch(messages);
        };
        closure.into()
    }
}

struct Position {
    parent: Element,
    next_sibling: NodeRef,
}

struct ComponentState<COMP: Component> {
    position: Option<Position>,
    node_ref: NodeRef,
    scope: Scope<COMP>,
    component: Box<COMP>,
    last_root: Option<VNode>,
    new_root: Option<VNode>,
    has_rendered: bool,
    pending_updates: Vec<Box<UpdateComponent<COMP>>>,
}

impl<COMP: Component> ComponentState<COMP> {
    /// Creates a new `ComponentState`, also invokes the `create()`
    /// method on component to create it.
    fn new(
        node_ref: NodeRef,
        scope: Scope<COMP>,
        props: COMP::Properties,
    ) -> Self {
        let component = Box::new(COMP::create(props, scope.clone()));
        Self {
            position: None,
            node_ref,
            scope,
            component,
            last_root: None,
            new_root: None,
            has_rendered: false,
            pending_updates: Vec::new(),
        }
    }
}

/// A `Runnable` task which creates the `ComponentState` (if there is
/// none) and invokes the `create()` method on a `Component` to create
/// it.
struct CreateComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
    node_ref: NodeRef,
    scope: Scope<COMP>,
    props: COMP::Properties,
}

impl<COMP> Runnable for CreateComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        let mut current_state = self.state.borrow_mut();
        if current_state.is_none() {
            *current_state = Some(ComponentState::new(
                self.node_ref,
                self.scope,
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
        let mut first_update = false;
        let mut update_sync = false;
        let mut should_update = false;
        if let Some(mut state) = state_clone.borrow_mut().as_mut() {
            if state.new_root.is_some() {
                state.pending_updates.push(self);
                return;
            }

            first_update = match self.update {
                ComponentUpdate::First => true,
                _ => false,
            };

            update_sync = match &self.update {
                ComponentUpdate::First => true,
                ComponentUpdate::Properties(_) => true,
                _ => false,
            };

            should_update = match self.update {
                ComponentUpdate::First => true,
                ComponentUpdate::Message(message) => state.component.update(message),
                ComponentUpdate::MessageBatch(messages) => messages
                    .into_iter()
                    .fold(false, |acc, msg| state.component.update(msg) || acc),
                ComponentUpdate::Properties(props) => {
                    // When components are updated, their siblings were likely also updated
                    // state.next_sibling = next_sibling;
                    state.component.change(props)
                }
            };

            if should_update {
                state.new_root = Some(state.component.view());
            }
        }

        if should_update {
            if update_sync {
                Box::new(ExpandComponent {
                    state: self.state,
                }).run();
            } else {
                scheduler().push_comp(
                    ComponentRunnableType::Expand,
                    Box::new(ExpandComponent {
                        state: self.state.clone(),
                    }),
                );
                scheduler().push_comp(
                    ComponentRunnableType::Render,
                    Box::new(RenderComponent {
                        state: self.state,
                        first_render: first_update,
                    }),
                );
            }
        }
    }
}

/// A `Runnable` task which expands a virtual DOM `Component` to
/// prepare it for rendering to the screen.
struct ExpandComponent<COMP>
where
    COMP: Component,
{
    state: Shared<Option<ComponentState<COMP>>>,
}

impl<COMP> Runnable for ExpandComponent<COMP>
where
    COMP: Component,
{
    fn run(self: Box<Self>) {
        if let Some(state) = self.state.borrow_mut().as_mut() {
            if let Some(new_root) = state.new_root.as_mut() {
                let last_root = state.last_root.as_mut();
                let parent_scope = state.scope.clone().into();
                new_root.expand(&parent_scope, last_root);
            } else {
                panic!("why you ain't got a new root");
            }
        }
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
                let last_root = state.last_root.take();
                if let Some(position) = &state.position {
                    let next_sibling = position.next_sibling.clone();
                    let node = new_root.apply(&position.parent, next_sibling, last_root);
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
            state.component.rendered(self.first_render);
            for update in state.pending_updates.drain(..) {
                scheduler().push_comp(ComponentRunnableType::Update, update);
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
            state.component.destroy();
            if let Some(last_frame) = &mut state.last_root {
                if let Some(position) = state.position {
                    last_frame.detach(&position.parent);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::html::*;
    use crate::Properties;
    use std::ops::Deref;
    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone, Properties, Default)]
    struct ChildProps {
        lifecycle: Rc<RefCell<Vec<String>>>,
    }

    struct Child {
        props: ChildProps,
    }

    impl Component for Child {
        type Message = ();
        type Properties = ChildProps;

        fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
            Child { props }
        }

        fn rendered(&mut self, _first_render: bool) {
            self.props
                .lifecycle
                .borrow_mut()
                .push("child rendered".into());
        }

        fn update(&mut self, _: Self::Message) -> ShouldRender {
            false
        }

        fn change(&mut self, _: Self::Properties) -> ShouldRender {
            false
        }

        fn view(&self) -> Html {
            html! {}
        }
    }

    #[derive(Clone, Properties, Default)]
    struct Props {
        lifecycle: Rc<RefCell<Vec<String>>>,
        create_message: Option<bool>,
        update_message: RefCell<Option<bool>>,
        view_message: RefCell<Option<bool>>,
        rendered_message: RefCell<Option<bool>>,
    }

    struct Comp {
        props: Props,
        link: ComponentLink<Self>,
    }

    impl Component for Comp {
        type Message = bool;
        type Properties = Props;

        fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
            props.lifecycle.borrow_mut().push("create".into());
            if let Some(msg) = props.create_message {
                link.send_message(msg);
            }
            Comp { props, link }
        }

        fn rendered(&mut self, first_render: bool) {
            if let Some(msg) = self.props.rendered_message.borrow_mut().take() {
                self.link.send_message(msg);
            }
            self.props
                .lifecycle
                .borrow_mut()
                .push(format!("rendered({})", first_render));
        }

        fn update(&mut self, msg: Self::Message) -> ShouldRender {
            if let Some(msg) = self.props.update_message.borrow_mut().take() {
                self.link.send_message(msg);
            }
            self.props
                .lifecycle
                .borrow_mut()
                .push(format!("update({})", msg));
            msg
        }

        fn change(&mut self, _: Self::Properties) -> ShouldRender {
            self.props.lifecycle.borrow_mut().push("change".into());
            false
        }

        fn view(&self) -> Html {
            if let Some(msg) = self.props.view_message.borrow_mut().take() {
                self.link.send_message(msg);
            }
            self.props.lifecycle.borrow_mut().push("view".into());
            html! { <Child lifecycle=self.props.lifecycle.clone() /> }
        }
    }

    impl Drop for Comp {
        fn drop(&mut self) {
            self.props.lifecycle.borrow_mut().push("drop".into());
        }
    }

    fn test_lifecycle(props: Props, expected: &[String]) {
        let document = crate::utils::document();
        let scope = Scope::<Comp>::new(None);
        let el = document.create_element("div").unwrap();
        let lifecycle = props.lifecycle.clone();

        lifecycle.borrow_mut().clear();
        scope.create(el, NodeRef::default(), props);

        assert_eq!(&lifecycle.borrow_mut().deref()[..], expected);
    }

    #[test]
    fn lifecyle_tests() {
        let lifecycle: Rc<RefCell<Vec<String>>> = Rc::default();

        test_lifecycle(
            Props {
                lifecycle: lifecycle.clone(),
                ..Props::default()
            },
            &vec![
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
            &vec![
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
            &vec![
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
            &vec![
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
            &vec![
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
            &vec![
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
                create_message: Some(true),
                update_message: RefCell::new(Some(true)),
                ..Props::default()
            },
            &vec![
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
