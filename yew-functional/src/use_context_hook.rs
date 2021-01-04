// Naming this file use_context could be confusing. Not least to the IDE.
use super::{get_component_link, use_hook, Hook};
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::{iter, mem};
use yew::html;
use yew::component::{Context, AnyLink, ComponentLink, Component};
use yew::{Children, Html, Properties};

type ConsumerCallback<T> = Box<dyn Fn(Rc<T>)>;

/// Props for [`ContextProvider`]
#[derive(PartialEq, Properties)]
pub struct ContextProviderProps<T: PartialEq> {
    pub context: Rc<T>,
    pub children: Children,
}

/// The context provider component.
///
/// Every child (direct or indirect) of this component may access the context value.
/// Currently the only way to consume the context is using the [`use_context`] hook.
pub struct ContextProvider<T: PartialEq + 'static> {
    consumers: RefCell<Vec<Weak<ConsumerCallback<T>>>>,
}

impl<T: PartialEq> ContextProvider<T> {
    /// Add the callback to the subscriber list to be called whenever the context changes.
    /// The consumer is unsubscribed as soon as the callback is dropped.
    fn subscribe_consumer(&self, mut callback: Weak<ConsumerCallback<T>>) {
        let mut consumers = self.consumers.borrow_mut();
        // consumers re-subscribe on every render. Try to keep the subscriber list small by reusing dead slots.
        for cb in consumers.iter_mut() {
            if cb.strong_count() == 0 {
                mem::swap(cb, &mut callback);
                return;
            }
        }

        // no slot to reuse, this is a new consumer
        consumers.push(callback);
    }

    /// Notify all subscribed consumers and remove dropped consumers from the list.
    fn notify_consumers(&mut self, context: Rc<T>) {
        self.consumers.borrow_mut().retain(|cb| {
            if let Some(cb) = cb.upgrade() {
                cb(context.clone());
                true
            } else {
                false
            }
        });
    }
}

impl<T: PartialEq + 'static> Component for ContextProvider<T> {
    type Message = Weak<ConsumerCallback<T>>;
    type Properties = ContextProviderProps<T>;

    fn create(_ctx: Context<Self>) -> Self {
        Self {
            consumers: RefCell::new(Vec::new()),
        }
    }

    fn update(&mut self, _ctx: Context<Self>, msg: Self::Message) -> bool {
        self.subscribe_consumer(msg);
        false
    }

    fn changed(&mut self, ctx: Context<Self>, new_props: &Self::Properties) -> bool {
        if ctx.props.context != new_props.context {
            self.notify_consumers(new_props.context.clone());
        }

        true
    }

    fn view(&self, ctx: Context<Self>) -> Html {
        html! { <>{ ctx.props.children.clone() }</> }
    }
}

fn find_context_provider_link<T: PartialEq + 'static>(
    link: &AnyLink,
) -> Option<ComponentLink<ContextProvider<T>>> {
    let expected_type_id = TypeId::of::<ContextProvider<T>>();
    iter::successors(Some(link), |link| link.get_parent())
        .filter(|link| link.get_type_id() == &expected_type_id)
        .cloned()
        .map(AnyLink::downcast::<ContextProvider<T>>)
        .next()
}

/// Hook for consuming context values in function components.
/// The context of the type passed as `T` is returned. If there is no such context in scope, `None` is returned.
/// A component which calls `use_context` will re-render when the data of the context changes.
///
/// More information about contexts and how to define and consume them can be found on [Yew Docs](https://yew.rs).
///
/// # Example
/// ```rust
/// # use yew_functional::{function_component, use_context};
/// # use yew::prelude::*;
/// # use std::rc::Rc;
///
/// # #[derive(Clone, Debug, PartialEq)]
/// # struct ThemeContext {
/// #    foreground: String,
/// #    background: String,
/// # }
/// #[function_component(ThemedButton)]
/// pub fn themed_button() -> Html {
///     let theme = use_context::<ThemeContext>().expect("no ctx found");
///
///     html! {
///         <button style=format!("background: {}; color: {}", theme.background, theme.foreground)>
///             { "Click me" }
///         </button>
///     }
/// }
/// ```
pub fn use_context<T: PartialEq + 'static>() -> Option<Rc<T>> {
    struct UseContextState<T2: PartialEq + 'static> {
        provider_link: Option<ComponentLink<ContextProvider<T2>>>,
        current_context: Option<Rc<T2>>,
        callback: Option<Rc<ConsumerCallback<T2>>>,
    }
    impl<T: PartialEq + 'static> Hook for UseContextState<T> {
        fn tear_down(&mut self) {
            if let Some(cb) = self.callback.take() {
                drop(cb);
            }
        }
    }

    let link = get_component_link()
        .expect("No current component link. `use_context` can only be called inside function components");

    use_hook(
        |state: &mut UseContextState<T>, hook_callback| {
            state.callback = Some(Rc::new(Box::new(move |ctx: Rc<T>| {
                hook_callback(
                    |state: &mut UseContextState<T>| {
                        state.current_context = Some(ctx);
                        true
                    },
                    false, // run pre render
                );
            })));
            let weak_cb = Rc::downgrade(state.callback.as_ref().unwrap());
            if let Some(link) = state.provider_link.as_ref() {
                link.send_message(weak_cb)
            }
            state.current_context.clone()
        },
        move || {
            let provider_link = find_context_provider_link::<T>(&link);
            let current_context = provider_link
                .as_ref()
                .and_then(|scope| scope.get_props())
                .map(|props| {
                    Rc::clone(&props.context)
                });
            UseContextState {
                provider_link,
                current_context,
                callback: None,
            }
        },
    )
}
