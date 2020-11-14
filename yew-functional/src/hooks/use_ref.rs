use crate::HookUpdater;
use crate::{use_hook, Hook};
use std::{cell::RefCell, rc::Rc};

/// A hook for maintaing a RefCell value between renders
/// This is an efficient hook for storing data that should not cause re-renders
pub fn use_ref<T: 'static>(initial_value: impl FnOnce() -> T + 'static) -> UseRefOutput<T> {
    use_hook::<UseRef<T>, _>((), || UseRef(Rc::new(RefCell::new(initial_value()))))
}

type UseRefOutput<T> = Rc<RefCell<T>>;
struct UseRef<T>(Rc<RefCell<T>>);
impl<T> Hook for UseRef<T> {
    type Output = UseRefOutput<T>;
    type Args = ();

    fn runner(&mut self, _args: Self::Args, _: HookUpdater) -> Self::Output {
        self.0.clone()
    }
}
