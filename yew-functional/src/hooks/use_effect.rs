use crate::{use_hook, Hook, HookUpdater};

pub fn use_effect<F, Destructor>(callback: F)
where
    F: FnOnce() -> Destructor + 'static,
    Destructor: FnOnce() + 'static,
{
    use_hook::<UseEffect<Destructor, F>, _>(Box::new(callback), move || {
        // Load the callback
        UseEffect {
            destructor: None,
            callback: None,
        }
    })
}

struct UseEffect<Destructor, Callback> {
    destructor: Option<Box<Destructor>>,
    callback: Option<Box<Callback>>,
}

impl<Destructor, Callback> Hook for UseEffect<Destructor, Callback>
where
    Destructor: FnOnce() + 'static,
    Callback: FnOnce() -> Destructor + 'static,
{
    type Output = ();
    type Args = Box<Callback>;

    fn tear_down(&mut self) {
        if let Some(destructor) = self.destructor.take() {
            destructor()
        }
    }

    fn runner(&mut self, callback: Self::Args, updater: HookUpdater) -> Self::Output {
        if self.callback.is_none() {
            self.callback = Some(callback);
        }

        // Call the post-render method that updates the callback state after rendering is complete
        updater.post_render(|state: &mut Self| {
            if let Some(destructor) = state.destructor.take() {
                destructor();
            }

            if let Some(cb) = state.callback.take() {
                let new_destructor = cb();
                state.destructor.replace(Box::new(new_destructor));
            }

            // Don't re-render the component, otherwise we'll get into a loop
            false
        });
    }
}
