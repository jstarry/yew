use std::cell::RefCell;

use std::rc::Rc;
use yew::html::AnyScope;
use yew::{Component, ComponentLink, Html, Properties};

pub mod hooks;
pub use hooks::*;
/// This attribute creates a function component from a normal Rust function.
///
/// Functions with this attribute **must** return `Html` and can optionally take an argument for props.
/// Note that the function only receives a reference to the props.
///
/// When using this attribute you need to provide a name for the component:
/// `#[function_component(ComponentName)]`.
/// The attribute will then automatically create a [`FunctionComponent`] with the given identifier
/// which you can use like a normal component.
///
/// # Example
/// ```rust
/// # use yew_functional::function_component;
/// # use yew::prelude::*;
/// #
/// # #[derive(Properties, Clone, PartialEq)]
/// # pub struct Props {
/// #     text: String
/// # }
/// #
/// #[function_component(NameOfComponent)]
/// pub fn component(props: &Props) -> Html {
///     html! {
///         <p>{ &props.text }</p>
///     }
/// }
/// ```
pub use yew_functional_macro::function_component;

thread_local! {
    static CURRENT_HOOK: RefCell<Option<HookState>> = RefCell::new(None);
}

type Msg = Box<dyn FnOnce() -> bool>;
type ProcessMessage = Rc<dyn Fn(Msg, bool)>;

struct HookState {
    counter: usize,
    scope: AnyScope,
    process_message: ProcessMessage,
    hooks: Vec<Rc<RefCell<dyn std::any::Any>>>,
    destroy_listeners: Vec<Box<dyn FnOnce()>>,
}

pub trait FunctionProvider {
    type TProps: Properties + PartialEq;
    fn run(props: &Self::TProps) -> Html;
}

#[derive(Clone, Default)]
pub struct MsgQueue(Rc<RefCell<Vec<Msg>>>);

impl MsgQueue {
    fn push(&self, msg: Msg) {
        self.0.borrow_mut().push(msg);
    }

    fn drain(&self) -> Vec<Msg> {
        self.0.borrow_mut().drain(..).collect()
    }
}

pub struct FunctionComponent<T: FunctionProvider + 'static> {
    _never: std::marker::PhantomData<T>,
    props: T::TProps,
    hook_state: RefCell<Option<HookState>>,
    link: ComponentLink<Self>,
    message_queue: MsgQueue,
}

impl<T> FunctionComponent<T>
where
    T: FunctionProvider,
{
    fn swap_hook_state(&self) {
        CURRENT_HOOK.with(|previous_hook| {
            std::mem::swap(
                &mut *previous_hook
                    .try_borrow_mut()
                    .expect("Previous hook still borrowed"),
                &mut *self.hook_state.borrow_mut(),
            );
        });
    }
}

impl<T: 'static> Component for FunctionComponent<T>
where
    T: FunctionProvider,
{
    type Message = Box<dyn FnOnce() -> bool>;
    type Properties = T::TProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let scope = AnyScope::from(link.clone());
        let message_queue = MsgQueue::default();

        Self {
            _never: std::marker::PhantomData::default(),
            props,
            link: link.clone(),
            message_queue: message_queue.clone(),
            hook_state: RefCell::new(Some(HookState {
                counter: 0,
                scope,
                process_message: Rc::new(move |msg, post_render| {
                    if post_render {
                        message_queue.push(msg);
                    } else {
                        link.send_message(msg);
                    }
                }),
                hooks: vec![],
                destroy_listeners: vec![],
            })),
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        for msg in self.message_queue.drain() {
            self.link.send_message(msg);
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        msg()
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        let mut props = props;
        std::mem::swap(&mut self.props, &mut props);
        props != self.props
    }

    fn view(&self) -> Html {
        // Reset hook
        self.hook_state
            .try_borrow_mut()
            .expect("Unexpected concurrent/nested view call")
            .as_mut()
            .unwrap()
            .counter = 0;

        // Load hook
        self.swap_hook_state();

        let ret = T::run(&self.props);

        // Restore previous hook
        self.swap_hook_state();

        ret
    }

    fn destroy(&mut self) {
        if let Some(ref mut hook_state) = *self.hook_state.borrow_mut() {
            for hook in hook_state.destroy_listeners.drain(..) {
                hook()
            }
        }
    }
}

pub fn get_current_scope() -> Option<AnyScope> {
    CURRENT_HOOK.with(|cell| cell.borrow().as_ref().map(|state| state.scope.clone()))
}

use std::ops::DerefMut;

#[derive(Clone)]
pub struct HookUpdater {
    hook: Rc<RefCell<dyn std::any::Any>>,
    process_message: ProcessMessage,
}
impl HookUpdater {
    pub fn callback<T: 'static, F>(&self, cb: F)
    where
        F: FnOnce(&mut T) -> bool + 'static,
    {
        let internal_hook_state = self.hook.clone();
        let process_message = self.process_message.clone();

        // Update the component
        // We're calling "link.send_message", so we're not calling it post-render
        let post_render = false;
        process_message(
            Box::new(move || {
                let mut r = internal_hook_state.borrow_mut();
                let hook: &mut T = r.downcast_mut().expect("Wrong type");
                cb(hook)
            }),
            post_render,
        );
    }

    pub fn post_render<T: 'static, F>(&self, cb: F)
    where
        F: FnOnce(&mut T) -> bool + 'static,
    {
        let internal_hook_state = self.hook.clone();
        let process_message = self.process_message.clone();

        // Update the component
        // We're calling "messagequeue.push", so not calling it post-render
        let post_render = true;
        process_message(
            Box::new(move || {
                let mut hook = internal_hook_state.borrow_mut();
                let hook: &mut T = hook.downcast_mut().expect("Wrong type");
                cb(hook)
            }),
            post_render,
        );
    }
}

pub trait Hook {
    type Output;
    type Args;
    fn tear_down(&mut self) {}
    fn runner(&mut self, args: Self::Args, updater: HookUpdater) -> Self::Output;
}

pub fn use_hook<InternalHook: Hook + 'static, I: FnOnce() -> InternalHook>(
    args: InternalHook::Args,
    initializer: I,
) -> InternalHook::Output {
    // Extract current hook
    let updater = CURRENT_HOOK.with(|hook_state_holder| {
        let mut hook_state_holder = hook_state_holder
            .try_borrow_mut()
            .expect("Nested hooks not supported");

        let mut hook_state = hook_state_holder
            .as_mut()
            .expect("No current hook. Hooks can only be called inside function components");

        // Determine which hook position we're at and increment for the next hook
        let hook_pos = hook_state.counter;
        hook_state.counter += 1;

        // Initialize hook if this is the first call
        if hook_pos >= hook_state.hooks.len() {
            let initial_state = Rc::new(RefCell::new(initializer()));
            hook_state.hooks.push(initial_state.clone());
            hook_state.destroy_listeners.push(Box::new(move || {
                initial_state.borrow_mut().deref_mut().tear_down();
            }));
        }

        let hook = hook_state
            .hooks
            .get(hook_pos)
            .expect("Not the same number of hooks. Hooks must not be called conditionally")
            .clone();

        HookUpdater {
            hook,
            process_message: hook_state.process_message.clone(),
        }
    });

    // Execute the actual hook closure we were given. Let it mutate the hook state and let
    // it create a callback that takes the mutable hook state.
    let mut hook = updater.hook.borrow_mut();
    let hook: &mut InternalHook = hook
        .downcast_mut()
        .expect("Incompatible hook type. Hooks must always be called in the same order");

    hook.runner(args, updater.clone())
}
