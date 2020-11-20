use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use wasm_bindgen_test::*;
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
use yew::{html, App, Children, Html, Properties};
use yew_functional::{
    use_context, use_effect, use_effect_with_deps, use_reducer_with_init, use_ref, use_state,
    ContextProvider, FunctionComponent, FunctionProvider,
};

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

#[wasm_bindgen_test]
fn use_effect_destroys_on_component_drop() {
    struct UseEffectFunction {}
    struct UseEffectWrapper {}
    #[derive(Properties, Clone)]
    struct WrapperProps {
        destroy_called: Rc<dyn Fn()>,
    }
    impl PartialEq for WrapperProps {
        fn eq(&self, _other: &Self) -> bool {
            false
        }
    }
    #[derive(Properties, Clone)]
    struct FunctionProps {
        effect_called: Rc<dyn Fn()>,
        destroy_called: Rc<dyn Fn()>,
    }
    impl PartialEq for FunctionProps {
        fn eq(&self, _other: &Self) -> bool {
            false
        }
    }
    type UseEffectComponent = FunctionComponent<UseEffectFunction>;
    type UseEffectWrapperComponent = FunctionComponent<UseEffectWrapper>;
    impl FunctionProvider for UseEffectFunction {
        type TProps = FunctionProps;

        fn run(props: &Self::TProps) -> Html {
            let effect_called = props.effect_called.clone();
            let destroy_called = props.destroy_called.clone();
            use_effect_with_deps(
                move |_| {
                    effect_called();
                    move || destroy_called()
                },
                (),
            );
            return html! {};
        }
    }
    impl FunctionProvider for UseEffectWrapper {
        type TProps = WrapperProps;

        fn run(props: &Self::TProps) -> Html {
            let (show, set_show) = use_state(|| true);
            if *show {
                let effect_called: Rc<dyn Fn()> = Rc::new(move || set_show(false));
                return html! {
                    <UseEffectComponent destroy_called=props.destroy_called.clone() effect_called=effect_called />
                };
            } else {
                return html! {
                    <div>{"EMPTY"}</div>
                };
            }
        }
    }
    let app: App<UseEffectWrapperComponent> = yew::App::new();
    let destroy_counter = Rc::new(std::cell::RefCell::new(0));
    let destroy_counter_c = destroy_counter.clone();
    app.mount_with_props(
        yew::utils::document().get_element_by_id("output").unwrap(),
        WrapperProps {
            destroy_called: Rc::new(move || *destroy_counter_c.borrow_mut().deref_mut() += 1),
        },
    );
    assert_eq!(1, *destroy_counter.borrow().deref());
}

#[wasm_bindgen_test]
fn use_effect_works_many_times() {
    struct UseEffectFunction {}
    impl FunctionProvider for UseEffectFunction {
        type TProps = ();

        fn run(_: &Self::TProps) -> Html {
            let (counter, set_counter) = use_state(|| 0);
            let counter_clone = counter.clone();

            use_effect_with_deps(
                move |_| {
                    if *counter_clone < 4 {
                        set_counter(*counter_clone + 1);
                    }
                    || {}
                },
                *counter,
            );

            return html! {
                <div>
                    {"The test result is"}
                    <div id="result">{counter}</div>
                    {"\n"}
                </div>
            };
        }
    }

    type UseEffectComponent = FunctionComponent<UseEffectFunction>;
    let app: App<UseEffectComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());
    let result = obtain_result();
    assert_eq!(result.as_str(), "4");
}

#[wasm_bindgen_test]
fn use_effect_works_once() {
    struct UseEffectFunction {}
    impl FunctionProvider for UseEffectFunction {
        type TProps = ();

        fn run(_: &Self::TProps) -> Html {
            let (counter, set_counter) = use_state(|| 0);
            let counter_clone = counter.clone();

            use_effect_with_deps(
                move |_| {
                    set_counter(*counter_clone + 1);
                    || panic!("Destructor should not have been called")
                },
                (),
            );

            return html! {
                <div>
                    {"The test result is"}
                    <div id="result">{counter}</div>
                    {"\n"}
                </div>
            };
        }
    }
    type UseEffectComponent = FunctionComponent<UseEffectFunction>;
    let app: App<UseEffectComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());
    let result = obtain_result();
    assert_eq!(result.as_str(), "1");
}

#[wasm_bindgen_test]
fn use_effect_refires_on_dependency_change() {
    struct UseEffectFunction {}
    impl FunctionProvider for UseEffectFunction {
        type TProps = ();

        fn run(_: &Self::TProps) -> Html {
            let number_ref = use_ref(|| 0);
            let number_ref_c = number_ref.clone();
            let number_ref2 = use_ref(|| 0);
            let number_ref2_c = number_ref2.clone();
            let arg = *number_ref.borrow_mut().deref_mut();
            let (_, set_counter) = use_state(|| 0);
            use_effect_with_deps(
                move |dep| {
                    let mut ref_mut = number_ref_c.borrow_mut();
                    let inner_ref_mut = ref_mut.deref_mut();
                    if *inner_ref_mut < 1 {
                        *inner_ref_mut += 1;
                        assert_eq!(dep, &0);
                    } else {
                        assert_eq!(dep, &1);
                    }
                    set_counter(10); // we just need to make sure it does not panic
                    move || {
                        set_counter(11);
                        *number_ref2_c.borrow_mut().deref_mut() += 1;
                    }
                },
                arg,
            );
            return html! {
                <div>
                    {"The test result is"}
                    <div id="result">{*number_ref.borrow_mut().deref_mut()}{*number_ref2.borrow_mut().deref_mut()}</div>
                    {"\n"}
                </div>
            };
        }
    }
    type UseEffectComponent = FunctionComponent<UseEffectFunction>;
    let app: App<UseEffectComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());
    let result: String = obtain_result();

    assert_eq!(result.as_str(), "11");
}

#[wasm_bindgen_test]
fn use_context_scoping_works() {
    #[derive(Clone, Debug, PartialEq)]
    struct ExampleContext(String);
    struct UseContextFunctionOuter {}
    struct UseContextFunctionInner {}
    struct ExpectNoContextFunction {}
    type UseContextComponent = FunctionComponent<UseContextFunctionOuter>;
    type UseContextComponentInner = FunctionComponent<UseContextFunctionInner>;
    type ExpectNoContextComponent = FunctionComponent<ExpectNoContextFunction>;
    impl FunctionProvider for ExpectNoContextFunction {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            if use_context::<ExampleContext>().is_some() {
                yew::services::ConsoleService::log(&format!(
                    "Context should be None here, but was {:?}!",
                    use_context::<ExampleContext>().unwrap()
                ));
            };
            return html! {
                <div></div>
            };
        }
    }
    impl FunctionProvider for UseContextFunctionOuter {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            type ExampleContextProvider = ContextProvider<ExampleContext>;
            return html! {
                <div>
                    <ExampleContextProvider context=ExampleContext("wrong1".into())>
                        <div>{"ignored"}</div>
                    </ExampleContextProvider>
                    <ExampleContextProvider context=ExampleContext("wrong2".into())>
                        <ExampleContextProvider context=ExampleContext("correct".into())>
                            <ExampleContextProvider context=ExampleContext("wrong1".into())>
                                <div>{"ignored"}</div>
                            </ExampleContextProvider>
                            <UseContextComponentInner />
                        </ExampleContextProvider>
                    </ExampleContextProvider>
                    <ExampleContextProvider context=ExampleContext("wrong3".into())>
                        <div>{"ignored"}</div>
                    </ExampleContextProvider>
                    <ExpectNoContextComponent />
                </div>
            };
        }
    }
    impl FunctionProvider for UseContextFunctionInner {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            let context = use_context::<ExampleContext>();
            return html! {
                <div id="result">{ &context.unwrap().0 }</div>
            };
        }
    }

    let app: App<UseContextComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());
    let result: String = obtain_result_by_id("result");
    assert_eq!("correct", result);
}

#[wasm_bindgen_test]
fn use_context_works_with_multiple_types() {
    #[derive(Clone, Debug, PartialEq)]
    struct ContextA(u32);
    #[derive(Clone, Debug, PartialEq)]
    struct ContextB(u32);

    struct Test1Function;
    impl FunctionProvider for Test1Function {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            assert_eq!(use_context::<ContextA>(), Some(Rc::new(ContextA(2))));
            assert_eq!(use_context::<ContextB>(), Some(Rc::new(ContextB(1))));

            return html! {};
        }
    }
    type Test1 = FunctionComponent<Test1Function>;

    struct Test2Function;
    impl FunctionProvider for Test2Function {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            assert_eq!(use_context::<ContextA>(), Some(Rc::new(ContextA(0))));
            assert_eq!(use_context::<ContextB>(), Some(Rc::new(ContextB(1))));

            return html! {};
        }
    }
    type Test2 = FunctionComponent<Test2Function>;

    struct Test3Function;
    impl FunctionProvider for Test3Function {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            assert_eq!(use_context::<ContextA>(), Some(Rc::new(ContextA(0))));
            assert_eq!(use_context::<ContextB>(), None);

            return html! {};
        }
    }
    type Test3 = FunctionComponent<Test3Function>;

    struct Test4Function;
    impl FunctionProvider for Test4Function {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            assert_eq!(use_context::<ContextA>(), None);
            assert_eq!(use_context::<ContextB>(), None);

            return html! {};
        }
    }
    type Test4 = FunctionComponent<Test4Function>;

    struct TestFunction;
    impl FunctionProvider for TestFunction {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            type ContextAProvider = ContextProvider<ContextA>;
            type ContextBProvider = ContextProvider<ContextB>;

            return html! {
                <div>
                    <ContextAProvider context=ContextA(0)>
                        <ContextBProvider context=ContextB(1)>
                            <ContextAProvider context=ContextA(2)>
                                <Test1/>
                            </ContextAProvider>
                            <Test2/>
                        </ContextBProvider>
                        <Test3/>
                    </ContextAProvider>
                    <Test4 />
                </div>
            };
        }
    }
    type TestComponent = FunctionComponent<TestFunction>;

    let app: App<TestComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());
}

#[wasm_bindgen_test]
fn use_context_update_works() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyContext(String);

    #[derive(Clone, Debug, PartialEq, Properties)]
    struct RenderCounterProps {
        id: String,
        children: Children,
    }

    struct RenderCounterFunction;
    impl FunctionProvider for RenderCounterFunction {
        type TProps = RenderCounterProps;

        fn run(props: &Self::TProps) -> Html {
            let counter = use_ref(|| 0);
            *counter.borrow_mut() += 1;
            return html! {
                <>
                    <div id=props.id.clone()>
                        { format!("total: {}", counter.borrow()) }
                    </div>
                    { props.children.clone() }
                </>
            };
        }
    }
    type RenderCounter = FunctionComponent<RenderCounterFunction>;

    #[derive(Clone, Debug, PartialEq, Properties)]
    struct ContextOutletProps {
        id: String,
        #[prop_or_default]
        magic: usize,
    }
    struct ContextOutletFunction;
    impl FunctionProvider for ContextOutletFunction {
        type TProps = ContextOutletProps;

        fn run(props: &Self::TProps) -> Html {
            let counter = use_ref(|| 0);
            *counter.borrow_mut() += 1;

            let ctx = use_context::<Rc<MyContext>>().expect("context not passed down");
            // log::info!("ctx is {:#?}", ctx);
            // log::info!("magic is {:#?}", props.magic);
            // log::info!("outlet counter is {:#?}", ctx);

            return html! {
                <>
                    <div>{ format!("magic: {}\n", props.magic) }</div>
                    <div id=props.id.clone()>
                        { format!("current: {}, total: {}", ctx.0, counter.borrow()) }
                    </div>
                </>
            };
        }
    }
    type ContextOutlet = FunctionComponent<ContextOutletFunction>;

    struct TestFunction;
    impl FunctionProvider for TestFunction {
        type TProps = ();

        fn run(_props: &Self::TProps) -> Html {
            type MyContextProvider = ContextProvider<Rc<MyContext>>;

            let (ctx, set_ctx) = use_state(|| MyContext("hello".into()));
            let rendered = use_ref(|| 0);

            // this is used to force an update specific to test-2
            let (magic_rc, set_magic) = use_state(|| 0);
            let magic: usize = *magic_rc;

            use_effect(move || {
                let count = *rendered.borrow();
                match count {
                    0 => {
                        set_ctx(MyContext("world".into()));
                        *rendered.borrow_mut() += 1;
                    }
                    1 => {
                        // force test-2 to re-render.
                        set_magic(1);
                        *rendered.borrow_mut() += 1;
                    }
                    2 => {
                        set_ctx(MyContext("hello world!".into()));
                        *rendered.borrow_mut() += 1;
                    }
                    _ => (),
                };
                || {}
            });

            return html! {
                <MyContextProvider context=ctx>
                    <RenderCounter id="test-0">
                        <ContextOutlet id="test-1"/>
                        <ContextOutlet id="test-2" magic=magic/>
                    </RenderCounter>
                </MyContextProvider>
            };
        }
    }
    type TestComponent = FunctionComponent<TestFunction>;

    let app: App<TestComponent> = yew::App::new();
    app.mount(yew::utils::document().get_element_by_id("output").unwrap());

    // 1 initial render + 3 update steps
    assert_eq!(obtain_result_by_id("test-0"), "total: 4");

    // 1 initial + 2 context update
    assert_eq!(
        obtain_result_by_id("test-1"),
        "current: hello world!, total: 3"
    );

    // 1 initial + 1 context update + 1 magic update + 1 context update
    assert_eq!(
        obtain_result_by_id("test-2"),
        "current: hello world!, total: 4"
    );
}

fn obtain_result() -> String {
    yew::utils::document()
        .get_element_by_id("result")
        .expect("No result found. Most likely, the application crashed and burned")
        .inner_html()
}

fn obtain_result_by_id(id: &str) -> String {
    yew::utils::document()
        .get_element_by_id(id)
        .expect("No result found. Most likely, the application crashed and burned")
        .inner_html()
}
