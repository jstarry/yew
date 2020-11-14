use crate::{use_hook, Hook, HookUpdater};
use std::borrow::Borrow;
use std::rc::Rc;

pub fn use_effect_with_deps<Effect, Destructor, Dependents>(effect: Effect, deps: Dependents)
where
    Effect: FnOnce(&Dependents) -> Destructor + 'static,
    Destructor: FnOnce() + 'static,
    Dependents: PartialEq + 'static,
{
    let deps = Rc::new(deps);

    use_hook::<UseEffectDeps<Effect, Destructor, Dependents>, _>(
        // Pass the arguments through to the runner
        (effect, deps.clone()),
        // Initialize the hook if need be
        move || UseEffectDeps {
            destructor: None,
            deps,
            _effect: None,
        },
    );
}

struct UseEffectDeps<Effect, Destructor, Dependents> {
    destructor: Option<Box<Destructor>>,
    deps: Rc<Dependents>,
    _effect: Option<Box<Effect>>,
}

impl<Effect, Destructor, Dependents> Hook for UseEffectDeps<Effect, Destructor, Dependents>
where
    Effect: FnOnce(&Dependents) -> Destructor + 'static,
    Destructor: FnOnce() + 'static,
    Dependents: PartialEq + 'static,
{
    type Output = ();
    type Args = (Effect, Rc<Dependents>);

    fn tear_down(&mut self) {
        if let Some(destructor) = self.destructor.take() {
            destructor()
        }
    }

    fn runner(&mut self, (callback, deps): Self::Args, updater: HookUpdater) -> Self::Output {
        updater.post_render(move |state: &mut Self| {
            if state.deps != deps {
                if let Some(de) = state.destructor.take() {
                    de();
                }
                let new_destructor = callback(deps.borrow());
                state.deps = deps;
                state.destructor.replace(Box::new(new_destructor));
            } else if state.destructor.is_none() {
                state
                    .destructor
                    .replace(Box::new(callback(state.deps.borrow())));
            }
            false
        });
    }
}
