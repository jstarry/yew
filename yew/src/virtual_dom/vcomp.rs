//! This module contains the implementation of a virtual component (`VComp`).

use super::{Key, Transformer, VDiff, VNode};
use crate::component::{
    context::{ComponentUpdate, ContextHandle},
    AnyContext, Component, Context,
};
use crate::html::NodeRef;
use crate::utils::document;
use cfg_if::cfg_if;
use std::{any::TypeId, borrow::Borrow, cell::RefCell, fmt, ops::Deref, rc::Rc};
cfg_if! {
    if #[cfg(feature = "std_web")] {
        use stdweb::web::{Element, Node};
    } else if #[cfg(feature = "web_sys")] {
        use web_sys::{Element, Node};
    }
}

/// A virtual component.
pub struct VComp {
    type_id: TypeId,
    context: Option<Box<dyn ContextHandle>>,
    props: Option<Box<dyn Mountable>>,
    pub(crate) node_ref: NodeRef,
    pub(crate) key: Option<Key>,
}

impl Clone for VComp {
    fn clone(&self) -> Self {
        if self.context.is_some() {
            panic!("Mounted components are not allowed to be cloned!");
        }

        Self {
            type_id: self.type_id,
            context: None,
            props: self.props.as_ref().map(|m| m.copy()),
            node_ref: self.node_ref.clone(),
            key: self.key.clone(),
        }
    }
}

/// A virtual child component.
pub struct VChild<COMP: Component> {
    /// The component properties
    pub props: RefCell<COMP::Properties>,
    /// Reference to the mounted node
    node_ref: NodeRef,
    key: Option<Key>,
}

impl<COMP: Component> Clone for VChild<COMP>
where
    COMP::Properties: Clone,
{
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
    pub fn new(props: COMP::Properties, node_ref: NodeRef, key: Option<Key>) -> Self {
        Self {
            props: RefCell::new(props),
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
        VComp::new::<COMP>(
            Rc::new(vchild.props.into_inner()),
            vchild.node_ref,
            vchild.key,
        )
    }
}

impl VComp {
    /// Creates a new `VComp` instance.
    pub fn new<COMP>(props: Rc<COMP::Properties>, node_ref: NodeRef, key: Option<Key>) -> Self
    where
        COMP: Component,
    {
        VComp {
            type_id: TypeId::of::<COMP>(),
            node_ref,
            props: Some(Box::new(PropsWrapper::<COMP>::new(props))),
            context: None,
            key,
        }
    }

    #[allow(unused)]
    pub(crate) fn root_vnode(&self) -> Option<impl Deref<Target = VNode> + '_> {
        self.context
            .as_ref()
            .and_then(|context| context.root_vnode())
    }
}

trait Mountable {
    fn copy(&self) -> Box<dyn Mountable>;
    fn mount(
        self: Box<Self>,
        node_ref: NodeRef,
        parent_context: &AnyContext,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn ContextHandle>;
    fn reuse(
        self: Box<Self>,
        node_ref: NodeRef,
        context: &dyn ContextHandle,
        next_sibling: NodeRef,
    );
}

struct PropsWrapper<COMP: Component> {
    props: Rc<COMP::Properties>,
}

impl<COMP: Component> PropsWrapper<COMP> {
    pub fn new(props: Rc<COMP::Properties>) -> Self {
        Self { props }
    }
}

impl<COMP: Component> Mountable for PropsWrapper<COMP> {
    fn copy(&self) -> Box<dyn Mountable> {
        let wrapper: PropsWrapper<COMP> = PropsWrapper {
            props: self.props.clone(),
        };
        Box::new(wrapper)
    }

    fn mount(
        self: Box<Self>,
        node_ref: NodeRef,
        parent_context: &AnyContext,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn ContextHandle> {
        let context: Context<COMP> =
            Context::new(Some(Rc::new(parent_context.clone())), self.props);
        let context = context.mount_in_place(
            parent,
            next_sibling,
            Some(VNode::VRef(node_ref.get().unwrap())),
            node_ref,
        );

        Box::new(context)
    }

    fn reuse(
        self: Box<Self>,
        node_ref: NodeRef,
        context: &dyn ContextHandle,
        next_sibling: NodeRef,
    ) {
        let context: Context<COMP> = context.to_any().downcast();
        context.update(ComponentUpdate::Properties(
            self.props,
            node_ref,
            next_sibling,
        ));
    }
}

impl VDiff for VComp {
    fn detach(&mut self, _parent: &Element) {
        self.context.take().expect("VComp is not mounted").destroy();
    }

    fn apply(
        &mut self,
        parent_context: &AnyContext,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef {
        let mountable = self.props.take().expect("VComp has already been mounted");

        if let Some(mut ancestor) = ancestor {
            if let VNode::VComp(ref mut vcomp) = &mut ancestor {
                // If the ancestor is the same type, reuse it and update its properties
                if self.type_id == vcomp.type_id && self.key == vcomp.key {
                    self.node_ref.reuse(vcomp.node_ref.clone());
                    let context = vcomp.context.take().expect("VComp is not mounted");
                    mountable.reuse(self.node_ref.clone(), context.borrow(), next_sibling);
                    self.context = Some(context);
                    return vcomp.node_ref.clone();
                }
            }

            ancestor.detach(parent);
        }

        let placeholder: Node = document().create_text_node("").into();
        super::insert_node(&placeholder, parent, next_sibling.get());
        self.node_ref.set(Some(placeholder));
        let context = mountable.mount(
            self.node_ref.clone(),
            parent_context,
            parent.to_owned(),
            next_sibling,
        );
        self.context = Some(context);
        self.node_ref.clone()
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
    use super::*;
    use crate::component::{Component, Context, Properties};
    use crate::{html, Children, Html, NodeRef};
    use cfg_match::cfg_match;

    #[cfg(feature = "std_web")]
    use stdweb::web::INode;

    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    struct Comp;

    #[derive(Clone, Default, PartialEq, Properties)]
    struct Props {
        #[prop_or_default]
        field_1: u32,
        #[prop_or_default]
        field_2: u32,
    }

    impl Component for Comp {
        type Message = ();
        type Properties = Props;

        fn create(_ctx: &Context<Self>) -> Self {
            Comp
        }

        fn view(&self, _ctx: &Context<Self>) -> Html {
            html! { <div/> }
        }
    }

    #[test]
    fn update_loop() {
        let document = crate::utils::document();
        let parent_context: AnyContext =
            Context::<Comp>::new(None, Rc::new(Props::default())).into();
        let parent_element = document.create_element("div").unwrap();

        let mut ancestor = html! { <Comp></Comp> };
        ancestor.apply(&parent_context, &parent_element, NodeRef::default(), None);

        for _ in 0..10000 {
            let mut node = html! { <Comp></Comp> };
            node.apply(
                &parent_context,
                &parent_element,
                NodeRef::default(),
                Some(ancestor),
            );
            ancestor = node;
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
    fn set_component_key() {
        let test_key: Key = "test".to_string().into();
        let check_key = |vnode: VNode| {
            assert_eq!(vnode.key().as_ref(), Some(&test_key));
        };

        let props = Props {
            field_1: 1,
            field_2: 1,
        };
        let props_2 = props.clone();

        check_key(html! { <Comp key=test_key.clone() /> });
        check_key(html! { <Comp key=test_key.clone() field_1=1 /> });
        check_key(html! { <Comp field_1=1 key=test_key.clone() /> });
        check_key(html! { <Comp with props key=test_key.clone() /> });
        check_key(html! { <Comp key=test_key.clone() with props_2 /> });
    }

    #[test]
    fn set_component_node_ref() {
        let test_node: Node = document().create_text_node("test").into();
        let test_node_ref = NodeRef::new(test_node);
        let check_node_ref = |vnode: VNode| {
            assert_eq!(vnode.first_node(), test_node_ref.get().unwrap());
        };

        let props = Props {
            field_1: 1,
            field_2: 1,
        };
        let props_2 = props.clone();

        check_node_ref(html! { <Comp ref=test_node_ref.clone() /> });
        check_node_ref(html! { <Comp ref=test_node_ref.clone() field_1=1 /> });
        check_node_ref(html! { <Comp field_1=1 ref=test_node_ref.clone() /> });
        check_node_ref(html! { <Comp with props ref=test_node_ref.clone() /> });
        check_node_ref(html! { <Comp ref=test_node_ref.clone() with props_2 /> });
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

    #[derive(Clone, PartialEq, Properties)]
    pub struct ListProps {
        pub children: Children,
    }

    pub struct List;
    impl Component for List {
        type Message = ();
        type Properties = ListProps;

        fn create(_ctx: &Context<Self>) -> Self {
            Self
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            let item_iter = ctx
                .props
                .children
                .iter()
                .map(|item| html! {<li>{ item }</li>});
            html! {
                <ul>{ for item_iter }</ul>
            }
        }
    }

    #[cfg(feature = "web_sys")]
    use super::{AnyContext, Element};

    #[cfg(feature = "web_sys")]
    fn setup_parent() -> (AnyContext, Element) {
        let context = AnyContext {
            type_id: TypeId::of::<()>(),
            parent: None,
            props: Rc::new(()),
            state: Rc::new(()),
        };
        let parent = document().create_element("div").unwrap();

        document().body().unwrap().append_child(&parent).unwrap();

        (context, parent)
    }

    #[cfg(feature = "web_sys")]
    fn get_html(mut node: Html, context: &AnyContext, parent: &Element) -> String {
        // clear parent
        parent.set_inner_html("");

        node.apply(&context, &parent, NodeRef::default(), None);
        parent.inner_html()
    }

    #[test]
    #[cfg(feature = "web_sys")]
    fn all_ways_of_passing_children_work() {
        let (context, parent) = setup_parent();

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
        assert_eq!(get_html(prop_method, &context, &parent), expected_html);

        let children_renderer_method = html! {
            <List>
                { children_renderer }
            </List>
        };
        assert_eq!(
            get_html(children_renderer_method, &context, &parent),
            expected_html
        );

        let direct_method = html! {
            <List>
                { children.clone() }
            </List>
        };
        assert_eq!(get_html(direct_method, &context, &parent), expected_html);

        let for_method = html! {
            <List>
                { for children }
            </List>
        };
        assert_eq!(get_html(for_method, &context, &parent), expected_html);
    }

    #[test]
    fn reset_node_ref() {
        let context = AnyContext {
            type_id: TypeId::of::<()>(),
            parent: None,
            state: Rc::new(()),
            props: Rc::new(()),
        };
        let parent = document().create_element("div").unwrap();

        #[cfg(feature = "std_web")]
        document().body().unwrap().append_child(&parent);
        #[cfg(feature = "web_sys")]
        document().body().unwrap().append_child(&parent).unwrap();

        let node_ref = NodeRef::default();
        let mut elem: VNode = html! { <Comp ref=node_ref.clone()></Comp> };
        elem.apply(&context, &parent, NodeRef::default(), None);
        let parent_node = cfg_match! {
            feature = "std_web" => parent.as_node(),
            feature = "web_sys" => parent.deref(),
        };
        assert_eq!(node_ref.get(), parent_node.first_child());
        elem.detach(&parent);
        assert!(node_ref.get().is_none());
    }
}

#[cfg(all(test, feature = "web_sys"))]
mod layout_tests {
    extern crate self as yew;

    use crate::component::{Component, Context, Properties};
    use crate::html;
    use crate::virtual_dom::layout_tests::{diff_layouts, TestLayout};
    use crate::{Children, Html};
    use std::marker::PhantomData;

    #[cfg(feature = "wasm_test")]
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

    #[cfg(feature = "wasm_test")]
    wasm_bindgen_test_configure!(run_in_browser);

    struct Comp<T> {
        _marker: PhantomData<T>,
    }

    #[derive(Properties, PartialEq)]
    struct CompProps {
        #[prop_or_default]
        children: Children,
    }

    impl<T: 'static> Component for Comp<T> {
        type Message = ();
        type Properties = CompProps;

        fn create(_ctx: &Context<Self>) -> Self {
            Comp {
                _marker: PhantomData::default(),
            }
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            html! {
                <>{ ctx.props.children.clone() }</>
            }
        }
    }

    struct A;
    struct B;

    #[test]
    fn diff() {
        let layout1 = TestLayout {
            name: "1",
            node: html! {
                <Comp<A>>
                    <Comp<B>></Comp<B>>
                    {"C"}
                </Comp<A>>
            },
            expected: "C",
        };

        let layout2 = TestLayout {
            name: "2",
            node: html! {
                <Comp<A>>
                    {"A"}
                </Comp<A>>
            },
            expected: "A",
        };

        let layout3 = TestLayout {
            name: "3",
            node: html! {
                <Comp<B>>
                    <Comp<A>></Comp<A>>
                    {"B"}
                </Comp<B>>
            },
            expected: "B",
        };

        let layout4 = TestLayout {
            name: "4",
            node: html! {
                <Comp<B>>
                    <Comp<A>>{"A"}</Comp<A>>
                    {"B"}
                </Comp<B>>
            },
            expected: "AB",
        };

        let layout5 = TestLayout {
            name: "5",
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
            name: "6",
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
            name: "7",
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
            name: "8",
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
            name: "9",
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
            name: "10",
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
            name: "11",
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
            name: "12",
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
