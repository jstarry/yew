#![recursion_limit = "128"]

use yew::prelude::*;

mod t1 {
    use super::*;

    #[derive(PartialEq, Properties)]
<<<<<<< HEAD:packages/yew-macro/tests/derive_props/pass.rs
    pub struct Props<T: Clone + Default> {
=======
    pub struct Props<T: PartialEq + Default> {
>>>>>>> consistent-agent-comp-api:yew-macro/tests/derive_props/pass.rs
        #[prop_or_default]
        value: T,
    }

    fn optional_prop_generics_should_work() {
        Props::<bool>::builder().build();
        Props::<bool>::builder().value(true).build();
    }
}

mod t2 {
    use super::*;

    #[derive(PartialEq)]
    struct Value;
    #[derive(PartialEq, Properties)]
<<<<<<< HEAD:packages/yew-macro/tests/derive_props/pass.rs
    pub struct Props<T: Clone> {
=======
    pub struct Props<T: PartialEq> {
>>>>>>> consistent-agent-comp-api:yew-macro/tests/derive_props/pass.rs
        value: T,
    }

    fn required_prop_generics_should_work() {
        Props::<Value>::builder().value(Value).build();
    }
}

mod t3 {
    use super::*;

    #[derive(PartialEq, Properties)]
    pub struct Props {
        b: i32,
        #[prop_or_default]
        a: i32,
    }

    fn order_is_alphabetized() {
        Props::builder().b(1).build();
        Props::builder().a(1).b(2).build();
    }
}

mod t4 {
    use super::*;

    #[derive(PartialEq, Properties)]
    pub struct Props<T>
    where
        T: PartialEq + Default,
    {
        #[prop_or_default]
        value: T,
    }

    fn optional_prop_generics_should_work() {
        Props::<bool>::builder().build();
        Props::<bool>::builder().value(true).build();
    }
}

mod t5 {
    use super::*;

    #[derive(PartialEq, Properties)]
<<<<<<< HEAD:packages/yew-macro/tests/derive_props/pass.rs
    pub struct Props<'a, T: Clone + Default + 'a> {
=======
    pub struct Props<'a, T: PartialEq + Default + 'a> {
>>>>>>> consistent-agent-comp-api:yew-macro/tests/derive_props/pass.rs
        #[prop_or_default]
        static_value: &'static str,
        value: &'a T,
    }

    fn optional_prop_generics_with_lifetime_should_work() {
        Props::<String>::builder().value(&String::from("")).build();
        Props::<String>::builder()
            .static_value("")
            .value(&String::from(""))
            .build();
    }
}

mod t6 {
    use super::*;
    use std::str::FromStr;

    #[derive(Properties, PartialEq)]
    pub struct Props<T: FromStr + PartialEq>
    where
        <T as FromStr>::Err: PartialEq,
    {
        value: Result<T, <T as FromStr>::Err>,
    }

    fn required_prop_generics_with_where_clause_should_work() {
        Props::<String>::builder()
            .value(Ok(String::from("")))
            .build();
    }
}

mod t7 {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum Foo {
        One,
        Two,
    }

    #[derive(PartialEq, Properties)]
    pub struct Props {
        #[prop_or(Foo::One)]
        value: Foo,
    }

    fn prop_or_value_should_work() {
        let props = Props::builder().build();
        assert_eq!(props.value, Foo::One);
        Props::builder().value(Foo::Two).build();
    }
}

mod t8 {
    use super::*;

    #[derive(PartialEq, Properties)]
    pub struct Props {
        #[prop_or_else(|| 123)]
        value: i32,
    }

    fn prop_or_else_closure_should_work() {
        let props = Props::builder().build();
        assert_eq!(props.value, 123);
        Props::builder().value(123).build();
    }
}

mod t9 {
    use super::*;
    use std::str::FromStr;

    #[derive(PartialEq, Properties)]
<<<<<<< HEAD:packages/yew-macro/tests/derive_props/pass.rs
    pub struct Props<T: FromStr + Clone>
=======
    pub struct Props<T: FromStr + PartialEq>
>>>>>>> consistent-agent-comp-api:yew-macro/tests/derive_props/pass.rs
    where
        <T as FromStr>::Err: PartialEq,
    {
        #[prop_or_else(default_value)]
        value: Result<T, <T as FromStr>::Err>,
    }

    fn default_value<T: FromStr + PartialEq>() -> Result<T, <T as FromStr>::Err>
    where
        <T as FromStr>::Err: PartialEq,
    {
        "123".parse()
    }

    fn prop_or_else_function_with_generics_should_work() {
        let props = Props::<i32>::builder().build();
        assert_eq!(props.value, Ok(123));
        Props::<i32>::builder().value(Ok(456)).build();
    }
}

mod t10 {
    use super::*;

    // this test makes sure that Yew handles generic params with default values properly.

    #[derive(PartialEq, Properties)]
    pub struct Foo<S, M = S>
    where
        S: PartialEq,
        M: PartialEq,
    {
        bar: S,
        baz: M,
    }
}

fn main() {}
