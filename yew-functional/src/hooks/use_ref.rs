use crate::use_hook;
use std::{cell::RefCell, rc::Rc};

/// A hook for maintaing a RefCell value between renders
/// This is an efficient hook for storing data that should not cause re-renders
pub fn use_ref<T: 'static>(initial_value: impl FnOnce() -> T + 'static) -> Rc<RefCell<T>> {
    use_hook(
        || Rc::new(RefCell::new(initial_value())),
        |state, _| state.clone(),
        |_| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::use_state;
    use crate::util::*;
    use crate::{FunctionComponent, FunctionProvider};
    use std::ops::DerefMut;
    use wasm_bindgen_test::*;
    use yew::prelude::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn use_ref_works() {
        struct UseRefFunction {}
        impl FunctionProvider for UseRefFunction {
            type TProps = ();

            fn run(_: &Self::TProps) -> Html {
                let ref_example = use_ref(|| 0);
                *ref_example.borrow_mut().deref_mut() += 1;
                let (counter, set_counter) = use_state(|| 0);
                if *counter < 5 {
                    set_counter(*counter + 1)
                }
                return html! {
                    <div>
                        {"The test output is: "}
                        <div id="result">{*ref_example.borrow_mut().deref_mut() > 4}</div>
                        {"\n"}
                    </div>
                };
            }
        }
        type UseRefComponent = FunctionComponent<UseRefFunction>;
        let app: App<UseRefComponent> = yew::App::new();
        app.mount(yew::utils::document().get_element_by_id("output").unwrap());

        let result = obtain_result();
        assert_eq!(result.as_str(), "true");
    }
}
