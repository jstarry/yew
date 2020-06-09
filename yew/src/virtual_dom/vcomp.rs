//! This module contains the implementation of a virtual component (`VComp`).

use super::{Transformer, VDiff, VNode};
use crate::html::{AnyScope, Component, ComponentUpdate, NodeRef, Scope, Scoped};
use cfg_if::cfg_if;
use std::any::TypeId;
use std::fmt;
use std::mem::replace;
cfg_if! {
    if #[cfg(feature = "std_web")] {
        use stdweb::web::{Element, Node};
    } else if #[cfg(feature = "web_sys")] {
        use web_sys::{Element, Node};
    }
}

/// A virtual component.
#[derive(Clone)]
pub struct VComp {
    type_id: TypeId,
    state: MountState,
    pub(crate) key: Option<String>,
}

impl VComp {
    pub(crate) fn first_node(&self) -> Node {
        match &self.state {
            MountState::Mounted(scope) => scope
                .node_ref()
                .expect("VComp should always wrap a node")
                .get()
                .expect("VComp should always wrap a node"),
            _ => {
                panic!("VComp has no first node when not mounted");
            }
        }
    }
}

enum MountState {
    Unmounted(Box<dyn Mountable>),
    Mounted(Box<dyn Scoped>),
    Mounting,
    Detached,
    Overwritten,
}

impl Clone for MountState {
    fn clone(&self) -> Self {
        match &self {
            Self::Unmounted(mountable) => Self::Unmounted(mountable.copy()),
            Self::Mounted(_) => panic!("Mounted components are not allowed to be cloned!"),
            Self::Mounting => Self::Mounting,
            Self::Detached => Self::Detached,
            Self::Overwritten => Self::Overwritten,
        }
    }
}

impl VComp {
    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn new<COMP>(props: COMP::Properties, node_ref: NodeRef, key: Option<String>) -> Self
    where
        COMP: Component,
    {
        VComp {
            type_id: TypeId::of::<COMP>(),
            state: MountState::Unmounted(Box::new(Unmounted::<COMP>::new(props, node_ref))),
            key,
        }
    }
}

trait Mountable {
    fn copy(&self) -> Box<dyn Mountable>;
    fn mount(
        self: Box<Self>,
        parent_scope: &AnyScope,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn Scoped>;
    fn overwrite(self: Box<Self>, scope: &Box<dyn Scoped>, next_sibling: NodeRef);
}

struct Unmounted<COMP: Component> {
    props: COMP::Properties,
    node_ref: NodeRef,
}

impl<COMP: Component> Unmounted<COMP> {
    pub fn new(props: COMP::Properties, node_ref: NodeRef) -> Self {
        Self { props, node_ref }
    }
}

impl<COMP: Component> Mountable for Unmounted<COMP> {
    fn copy(&self) -> Box<dyn Mountable> {
        let mountable: Box<Unmounted<COMP>> = Box::new(Unmounted {
            props: self.props.clone(),
            node_ref: self.node_ref.clone(),
        });
        mountable
    }

    fn mount(
        self: Box<Self>,
        parent_scope: &AnyScope,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn Scoped> {
        Box::new(Scope::mount_in_place(
            parent,
            Some(parent_scope.clone()),
            next_sibling,
            self.node_ref,
            self.props,
        ) as Scope<COMP>)
    }

    fn overwrite(self: Box<Self>, scope: &Box<dyn Scoped>, next_sibling: NodeRef) {
        let scope: Scope<COMP> = scope.to_any().downcast();
        scope.update(
            ComponentUpdate::Properties(self.props, self.node_ref, next_sibling),
            false,
        );
    }
}

impl VDiff for VComp {
    fn detach(&mut self, _parent: &Element) {
        if let MountState::Mounted(mut this) = replace(&mut self.state, MountState::Detached) {
            this.destroy();
        }
    }

    fn apply(
        &mut self,
        parent_scope: &AnyScope,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef {
        if let MountState::Unmounted(this) = replace(&mut self.state, MountState::Mounting) {
            if let Some(mut ancestor) = ancestor {
                if let VNode::VComp(ref mut vcomp) = &mut ancestor {
                    // If the ancestor is a Component of the same type, don't recreate, keep the
                    // old Component and update the properties.
                    if self.type_id == vcomp.type_id {
                        if let MountState::Mounted(scope) =
                            replace(&mut vcomp.state, MountState::Overwritten)
                        {
                            let node = scope.node_ref();
                            // Send properties update when the component is already rendered.
                            this.overwrite(&scope, next_sibling);
                            self.state = MountState::Mounted(scope);
                            return node.unwrap();
                        }
                    }
                }

                ancestor.detach(parent);
            }

            let scope = this.mount(parent_scope, parent.to_owned(), next_sibling);
            let node_ref = scope.node_ref().unwrap();
            self.state = MountState::Mounted(scope);
            node_ref
        } else {
            unreachable!("Only unmounted components can be mounted");
        }
    }
}

/// A virtual child component.
pub struct VChild<COMP: Component> {
    /// The component properties
    pub props: COMP::Properties,
    /// Reference to the mounted node
    node_ref: NodeRef,
    key: Option<String>,
}

impl<COMP: Component> Clone for VChild<COMP> {
    fn clone(&self) -> Self {
        VChild {
            props: self.props.clone(),
            node_ref: self.node_ref.clone(),
            key: self.key.clone(),
        }
    }
}

impl<COMP: Component> PartialEq for VChild<COMP>
where
    COMP::Properties: PartialEq,
{
    fn eq(&self, other: &VChild<COMP>) -> bool {
        self.props == other.props
    }
}

impl<COMP> VChild<COMP>
where
    COMP: Component,
{
    /// Creates a child component that can be accessed and modified by its parent.
    pub fn new(props: COMP::Properties, node_ref: NodeRef, key: Option<String>) -> Self {
        Self {
            props,
            node_ref,
            key,
        }
    }
}

impl<COMP> From<VChild<COMP>> for VComp
where
    COMP: Component,
{
    fn from(vchild: VChild<COMP>) -> Self {
        VComp::new::<COMP>(vchild.props, vchild.node_ref, vchild.key)
    }
}

impl<T> Transformer<T, T> for VComp {
    fn transform(from: T) -> T {
        from
    }
}

impl<'a, T> Transformer<&'a T, T> for VComp
where
    T: Clone,
{
    fn transform(from: &'a T) -> T {
        from.clone()
    }
}

impl<'a> Transformer<&'a str, String> for VComp {
    fn transform(from: &'a str) -> String {
        from.to_owned()
    }
}

impl<T> Transformer<T, Option<T>> for VComp {
    fn transform(from: T) -> Option<T> {
        Some(from)
    }
}

impl<'a, T> Transformer<&'a T, Option<T>> for VComp
where
    T: Clone,
{
    fn transform(from: &T) -> Option<T> {
        Some(from.clone())
    }
}

impl<'a> Transformer<&'a str, Option<String>> for VComp {
    fn transform(from: &'a str) -> Option<String> {
        Some(from.to_owned())
    }
}

impl<'a> Transformer<Option<&'a str>, Option<String>> for VComp {
    fn transform(from: Option<&'a str>) -> Option<String> {
        from.map(|s| s.to_owned())
    }
}

impl PartialEq for VComp {
    fn eq(&self, other: &VComp) -> bool {
        self.type_id == other.type_id
    }
}

impl fmt::Debug for VComp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VComp")
    }
}

impl<COMP: Component> fmt::Debug for VChild<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VChild<_>")
    }
}

#[cfg(test)]
mod tests {
    use super::VChild;
    use crate::macros::Properties;
    use crate::{html, Children, Component, ComponentLink, Html, NodeRef, ShouldRender};
    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    struct Comp;

    #[derive(Clone, PartialEq, Properties)]
    struct Props {
        #[prop_or_default]
        field_1: u32,
        #[prop_or_default]
        field_2: u32,
    }

    impl Component for Comp {
        type Message = ();
        type Properties = Props;

        fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
            Comp
        }

        fn update(&mut self, _: Self::Message) -> ShouldRender {
            unimplemented!();
        }

        fn change(&mut self, _: Self::Properties) -> ShouldRender {
            unimplemented!();
        }

        fn view(&self) -> Html {
            unimplemented!();
        }
    }

    #[test]
    fn set_properties_to_component() {
        html! {
            <Comp />
        };

        html! {
            <Comp field_1=1 />
        };

        html! {
            <Comp field_2=2 />
        };

        html! {
            <Comp field_1=1 field_2=2 />
        };

        let props = Props {
            field_1: 1,
            field_2: 1,
        };

        html! {
            <Comp with props />
        };
    }

    #[test]
    fn vchild_partialeq() {
        let vchild1: VChild<Comp> = VChild::new(
            Props {
                field_1: 1,
                field_2: 1,
            },
            NodeRef::default(),
            None,
        );

        let vchild2: VChild<Comp> = VChild::new(
            Props {
                field_1: 1,
                field_2: 1,
            },
            NodeRef::default(),
            None,
        );

        let vchild3: VChild<Comp> = VChild::new(
            Props {
                field_1: 2,
                field_2: 2,
            },
            NodeRef::default(),
            None,
        );

        assert_eq!(vchild1, vchild2);
        assert_ne!(vchild1, vchild3);
        assert_ne!(vchild2, vchild3);
    }

    #[derive(Clone, Properties)]
    pub struct ListProps {
        pub children: Children,
    }
    pub struct List(ListProps);
    impl Component for List {
        type Message = ();
        type Properties = ListProps;

        fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
            Self(props)
        }
        fn update(&mut self, _: Self::Message) -> ShouldRender {
            unimplemented!();
        }
        fn change(&mut self, _: Self::Properties) -> ShouldRender {
            unimplemented!();
        }
        fn view(&self) -> Html {
            let item_iter = self.0.children.iter().map(|item| html! {<li>{ item }</li>});
            html! {
                <ul>{ for item_iter }</ul>
            }
        }
    }

    #[cfg(feature = "web_sys")]
    use super::{AnyScope, Element};

    #[cfg(feature = "web_sys")]
    fn setup_parent() -> (AnyScope, Element) {
        use crate::utils::document;

        let scope = AnyScope {
            type_id: std::any::TypeId::of::<()>(),
            parent: None,
            state: std::rc::Rc::new(()),
        };
        let parent = document().create_element("div").unwrap();

        document().body().unwrap().append_child(&parent).unwrap();

        (scope, parent)
    }

    #[cfg(feature = "web_sys")]
    fn get_html(mut node: Html, scope: &AnyScope, parent: &Element) -> String {
        use super::VDiff;

        // clear parent
        parent.set_inner_html("");

        node.apply(&scope, &parent, NodeRef::default(), None);
        parent.inner_html()
    }

    #[test]
    #[cfg(feature = "web_sys")]
    fn all_ways_of_passing_children_work() {
        let (scope, parent) = setup_parent();

        let children: Vec<_> = vec!["a", "b", "c"]
            .drain(..)
            .map(|text| html! {<span>{ text }</span>})
            .collect();
        let children_renderer = Children::new(children.clone());
        let expected_html = "\
        <ul>\
            <li><span>a</span></li>\
            <li><span>b</span></li>\
            <li><span>c</span></li>\
        </ul>";

        let prop_method = html! {
            <List children=children_renderer.clone()/>
        };
        assert_eq!(get_html(prop_method, &scope, &parent), expected_html);

        let children_renderer_method = html! {
            <List>
                { children_renderer.clone() }
            </List>
        };
        assert_eq!(
            get_html(children_renderer_method, &scope, &parent),
            expected_html
        );

        let direct_method = html! {
            <List>
                { children.clone() }
            </List>
        };
        assert_eq!(get_html(direct_method, &scope, &parent), expected_html);

        let for_method = html! {
            <List>
                { for children }
            </List>
        };
        assert_eq!(get_html(for_method, &scope, &parent), expected_html);
    }
}

#[cfg(all(test, feature = "web_sys"))]
mod layout_tests {
    use crate::virtual_dom::layout_tests::{diff_layouts, TestLayout};
    use crate::{Children, Component, ComponentLink, Html, Properties, ShouldRender};
    use std::marker::PhantomData;

    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    struct Comp<T> {
        _marker: PhantomData<T>,
        props: CompProps,
    }

    #[derive(Properties, Clone)]
    struct CompProps {
        #[prop_or_default]
        children: Children,
    }

    impl<T: 'static> Component for Comp<T> {
        type Message = ();
        type Properties = CompProps;

        fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
            Comp {
                _marker: PhantomData::default(),
                props,
            }
        }

        fn update(&mut self, _: Self::Message) -> ShouldRender {
            unimplemented!();
        }

        fn change(&mut self, props: Self::Properties) -> ShouldRender {
            self.props = props;
            true
        }

        fn view(&self) -> Html {
            html! {
                <>{ self.props.children.clone() }</>
            }
        }
    }

    struct A;
    struct B;

    #[test]
    fn diff() {
        let layout1 = TestLayout {
            node: html! {
                <Comp<A>>
                    <Comp<B>></Comp<B>>
                    {"C"}
                </Comp<A>>
            },
            expected: "C",
        };

        let layout2 = TestLayout {
            node: html! {
                <Comp<A>>
                    {"A"}
                </Comp<A>>
            },
            expected: "A",
        };

        let layout3 = TestLayout {
            node: html! {
                <Comp<B>>
                    <Comp<A>></Comp<A>>
                    {"B"}
                </Comp<B>>
            },
            expected: "B",
        };

        let layout4 = TestLayout {
            node: html! {
                <Comp<B>>
                    <Comp<A>>{"A"}</Comp<A>>
                    {"B"}
                </Comp<B>>
            },
            expected: "AB",
        };

        let layout5 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>>
                            {"A"}
                        </Comp<A>>
                    </>
                    {"B"}
                </Comp<B>>
            },
            expected: "AB",
        };

        let layout6 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>>
                            {"A"}
                        </Comp<A>>
                        {"B"}
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout7 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>>
                            {"A"}
                        </Comp<A>>
                        <Comp<A>>
                            {"B"}
                        </Comp<A>>
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout8 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>>
                            {"A"}
                        </Comp<A>>
                        <Comp<A>>
                            <Comp<A>>
                                {"B"}
                            </Comp<A>>
                        </Comp<A>>
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout9 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <>
                            {"A"}
                        </>
                        <Comp<A>>
                            <Comp<A>>
                                {"B"}
                            </Comp<A>>
                        </Comp<A>>
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout10 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>>
                            <Comp<A>>
                                {"A"}
                            </Comp<A>>
                        </Comp<A>>
                        <>
                            {"B"}
                        </>
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout11 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <>
                            <Comp<A>>
                                <Comp<A>>
                                    {"A"}
                                </Comp<A>>
                                {"B"}
                            </Comp<A>>
                        </>
                    </>
                    {"C"}
                </Comp<B>>
            },
            expected: "ABC",
        };

        let layout12 = TestLayout {
            node: html! {
                <Comp<B>>
                    <>
                        <Comp<A>></Comp<A>>
                        <>
                            <Comp<A>>
                                <>
                                    <Comp<A>>
                                        {"A"}
                                    </Comp<A>>
                                    <></>
                                    <Comp<A>>
                                        <Comp<A>></Comp<A>>
                                        <></>
                                        {"B"}
                                        <></>
                                        <Comp<A>></Comp<A>>
                                    </Comp<A>>
                                </>
                            </Comp<A>>
                            <></>
                        </>
                        <Comp<A>></Comp<A>>
                    </>
                    {"C"}
                    <Comp<A>></Comp<A>>
                    <></>
                </Comp<B>>
            },
            expected: "ABC",
        };

        diff_layouts(vec![
            layout1, layout2, layout3, layout4, layout5, layout6, layout7, layout8, layout9,
            layout10, layout11, layout12,
        ]);
    }
}
