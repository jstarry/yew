use crate::use_hook;
use std::rc::Rc;

struct UseState<T2> {
    current: Rc<T2>,
}

/// A hook for maintaing and updating state between renders
/// Any setting of values will cause the component to update
pub fn use_state<T: 'static, F: FnOnce() -> T + 'static>(
    initial_state_fn: F,
) -> (Rc<T>, Rc<dyn Fn(T)>) {
    use_hook(
        move || UseState {
            current: Rc::new(initial_state_fn()),
        },
        move |hook, updater| {
            let setter: Rc<(dyn Fn(T))> = Rc::new(move |new_val: T| {
                updater.callback(move |st: &mut UseState<T>| {
                    st.current = Rc::new(new_val);
                    true
                })
            });

            let current = hook.current.clone();
            (current, setter)
        },
        |_| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::use_effect_with_deps;
    use crate::util::*;
    use crate::{FunctionComponent, FunctionProvider};
    use wasm_bindgen_test::*;
    use yew::prelude::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn use_state_works() {
        struct UseStateFunction {}
        impl FunctionProvider for UseStateFunction {
            type TProps = ();

            fn run(_: &Self::TProps) -> Html {
                let (counter, set_counter) = use_state(|| 0);
                if *counter < 5 {
                    set_counter(*counter + 1)
                }
                return html! {
                    <div>
                        {"Test Output: "}
                        <div id="result">{*counter}</div>
                        {"\n"}
                    </div>
                };
            }
        }
        type UseComponent = FunctionComponent<UseStateFunction>;
        let app: App<UseComponent> = yew::App::new();
        app.mount(yew::utils::document().get_element_by_id("output").unwrap());
        let result = obtain_result();
        assert_eq!(result.as_str(), "5");
    }

    #[wasm_bindgen_test]
    fn multiple_use_state_setters() {
        struct UseStateFunction {}
        impl FunctionProvider for UseStateFunction {
            type TProps = ();

            fn run(_: &Self::TProps) -> Html {
                let (counter, set_counter_in_use_effect) = use_state(|| 0);
                let counter = *counter;
                // clone without manually wrapping with Rc
                let set_counter_in_another_scope = set_counter_in_use_effect.clone();
                use_effect_with_deps(
                    move |_| {
                        // 1st location
                        set_counter_in_use_effect(counter + 1);
                        || {}
                    },
                    (),
                );
                let another_scope = move || {
                    if counter < 11 {
                        // 2nd location
                        set_counter_in_another_scope(counter + 10)
                    }
                };
                another_scope();
                return html! {
                    <div>
                        {"Test Output: "}
                        // expected output
                        <div id="result">{counter}</div>
                        {"\n"}
                    </div>
                };
            }
        }
        type UseComponent = FunctionComponent<UseStateFunction>;
        let app: App<UseComponent> = yew::App::new();
        app.mount(yew::utils::document().get_element_by_id("output").unwrap());
        let result = obtain_result();
        assert_eq!(result.as_str(), "11");
    }
}
