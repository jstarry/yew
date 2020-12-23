use crate::use_hook;
use std::rc::Rc;

struct UseReducer<State> {
    current_state: Rc<State>,
}

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
    Reducer,
    Action: 'static,
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
    use_hook(
        move || UseReducer {
            current_state: Rc::new(init(initial_state)),
        },
        |s, updater| {
            let setter: Rc<dyn Fn(Action)> = Rc::new(move |action: Action| {
                let reducer = reducer.clone();
                // We call the callback, consumer the updater
                // Required to put the type annotations on Self so the method knows how to downcast
                updater.callback(move |state: &mut UseReducer<State>| {
                    let new_state = reducer(state.current_state.clone(), action);
                    state.current_state = Rc::new(new_state);
                    true
                });
            });

            let current = s.current_state.clone();
            (current, setter)
        },
        |_| {},
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hooks::use_effect_with_deps;
    use crate::util::*;
    use crate::{FunctionComponent, FunctionProvider};
    use wasm_bindgen_test::*;
    use yew::prelude::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn use_reducer_works() {
        struct UseReducerFunction {}
        impl FunctionProvider for UseReducerFunction {
            type TProps = ();
            fn run(_: &Self::TProps) -> Html {
                struct CounterState {
                    counter: i32,
                }
                let (counter, dispatch) = use_reducer_with_init(
                    |prev: std::rc::Rc<CounterState>, action: i32| CounterState {
                        counter: prev.counter + action,
                    },
                    0,
                    |initial: i32| CounterState {
                        counter: initial + 10,
                    },
                );

                use_effect_with_deps(
                    move |_| {
                        dispatch(1);
                        || {}
                    },
                    (),
                );
                return html! {
                    <div>
                        {"The test result is"}
                        <div id="result">{counter.counter}</div>
                        {"\n"}
                    </div>
                };
            }
        }
        type UseReducerComponent = FunctionComponent<UseReducerFunction>;
        let app: App<UseReducerComponent> = yew::App::new();
        app.mount(yew::utils::document().get_element_by_id("output").unwrap());
        let result = obtain_result();

        assert_eq!(result.as_str(), "11");
    }
}
