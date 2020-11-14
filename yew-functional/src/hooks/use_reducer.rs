use crate::{use_hook, Hook, HookUpdater};
use std::rc::Rc;

pub fn use_reducer<Action: 'static, Reducer, State: 'static>(
    reducer: Reducer,
    initial_state: State,
) -> (Rc<State>, Rc<dyn Fn(Action)>)
where
    Reducer: Fn(Rc<State>, Action) -> State + 'static,
{
    use_reducer_with_init(reducer, initial_state, |a| a)
}

pub fn use_reducer_with_init<
    Action: 'static,
    Reducer,
    State: 'static,
    InitialState: 'static,
    InitFn: 'static,
>(
    reducer: Reducer,
    initial_state: InitialState,
    init: InitFn,
) -> (Rc<State>, Rc<dyn Fn(Action)>)
where
    Reducer: Fn(Rc<State>, Action) -> State + 'static,
    InitFn: Fn(InitialState) -> State,
{
    let init = Box::new(init);
    let reducer = Rc::new(reducer);
    use_hook::<UseReducer<State, Action, Reducer>, _>(reducer, move || UseReducer {
        current_state: Rc::new(init(initial_state)),
        m: std::marker::PhantomData::default(),
    })
}

struct UseReducer<State, Action, Reducer> {
    current_state: Rc<State>,
    m: std::marker::PhantomData<(Action, Reducer)>,
}

impl<State: 'static, Action: 'static, Reducer: 'static> Hook for UseReducer<State, Action, Reducer>
where
    Reducer: Fn(Rc<State>, Action) -> State + 'static,
{
    type Output = (Rc<State>, Rc<dyn Fn(Action)>);
    type Args = Rc<Reducer>;

    fn runner(&mut self, reducer: Self::Args, updater: HookUpdater) -> Self::Output {
        let setter = move |action: Action| {
            let reducer = reducer.clone();
            // We call the callback, consumer the updater
            // Required to put the type annotations on Self so the method knows how to downcast
            updater.callback(move |state: &mut Self| {
                let new_state = reducer(state.current_state.clone(), action);
                state.current_state = Rc::new(new_state);
                true
            });
        };

        let current = self.current_state.clone();
        (current, Rc::new(setter))
    }
}
