use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use wasm_bindgen_test::*;
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
use yew::{html, App, Children, Html, Properties};
use yew_functional::{
    use_context, use_effect, use_effect_with_deps, use_reducer_with_init, use_ref, use_state,
    util::obtain_result, ContextProvider, FunctionComponent, FunctionProvider,
};

#[wasm_bindgen_test]
fn props_are_passed() {
    struct PropsPassedFunction {}
    #[derive(Properties, Clone, PartialEq)]
    struct PropsPassedFunctionProps {
        value: String,
    }
    impl FunctionProvider for PropsPassedFunction {
        type TProps = PropsPassedFunctionProps;

        fn run(props: &Self::TProps) -> Html {
            assert_eq!(&props.value, "props");
            return html! {
                <div id="result">
                    {"done"}
                </div>
            };
        }
    }
    type PropsComponent = FunctionComponent<PropsPassedFunction>;
    let app: App<PropsComponent> = yew::App::new();
    app.mount_with_props(
        yew::utils::document().get_element_by_id("output").unwrap(),
        PropsPassedFunctionProps {
            value: "props".to_string(),
        },
    );
    let result = obtain_result();
    assert_eq!(result.as_str(), "done");
}
