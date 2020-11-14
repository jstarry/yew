use crate::HookUpdater;
use crate::{use_hook, Hook};
use std::rc::Rc;

/// A hook for maintaing and updating state between renders
/// Any setting of values will cause the component to update
pub fn use_state<T: 'static, F: FnOnce() -> T + 'static>(
    initial_state_fn: F,
) -> (Rc<T>, Rc<dyn Fn(T)>) {
    use_hook::<UseState<T>, _>((), move || UseState {
        current: Rc::new(initial_state_fn()),
    })
}

struct UseState<T2> {
    current: Rc<T2>,
}

impl<'a, T: 'static> Hook for UseState<T> {
    type Output = (Rc<T>, Rc<dyn Fn(T)>);
    type Args = ();

    fn runner(&mut self, _: Self::Args, updater: HookUpdater) -> Self::Output {
        let setter = move |new_val: T| {
            // We call the callback, consumer the updater
            // Required to put the type annotations on Self so the method knows how to downcast
            updater.callback(move |state: &mut Self| {
                state.current = Rc::new(new_val);
                true
            });
        };

        let current = self.current.clone();
        (current, Rc::new(setter))
    }
}
