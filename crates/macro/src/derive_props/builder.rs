use super::generics::{with_param, with_param_bounds};
use super::{DerivePropsInput, PropField};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::iter;
use syn::Generics;

pub struct PropsBuilder<'a> {
    builder_name: &'a Ident,
    step_trait: &'a Ident,
    step_names: Vec<Ident>,
    props: &'a DerivePropsInput,
    wrapper_name: &'a Ident,
}

impl ToTokens for PropsBuilder<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            builder_name,
            step_trait,
            step_names,
            props,
            wrapper_name,
        } = self;

        let DerivePropsInput {
            vis,
            generics,
            props_name,
            ..
        } = props;
        let (_, ty_generics, _) = generics.split_for_impl();
        let turbofish_generics = ty_generics.as_turbofish();

        let build_step = self.build_step();
        let step_trait_repeat = iter::repeat(step_trait);
        let vis_repeat = iter::repeat(&vis);

        let impl_steps = self.impl_steps();
        let set_fields = self.set_fields();

        let step_generic_param = Ident::new("YEW_PROPS_BUILDER_STEP", Span::call_site());
        let build_step_generics =
            with_param_bounds(&generics, step_generic_param, build_step.clone());
        let (impl_build_step_generics, build_step_ty_generics, build_step_where_clause) =
            build_step_generics.split_for_impl();

        let builder = quote! {
            #(
                #[doc(hidden)]
                #vis_repeat struct #step_names;
            )*

            #[doc(hidden)]
            #vis trait #step_trait {}

            #(impl #step_trait_repeat for #step_names {})*

            #[doc(hidden)]
            #vis struct #builder_name#generics {
                wrapped: ::std::boxed::Box<#wrapper_name#ty_generics>,
                _marker: ::std::marker::PhantomData<step_generic_param>,
            }

            #(#impl_steps)*

            impl#impl_build_step_generics #builder_name#build_step_ty_generics #build_step_where_clause {
                #[doc(hidden)]
                #vis fn build(self) -> #props_name#ty_generics {
                    #props_name#turbofish_generics {
                        #(#set_fields)*
                    }
                }
            }
        };

        tokens.extend(builder);
    }
}

impl<'a> PropsBuilder<'_> {
    pub fn new(
        name: &'a Ident,
        step_trait: &'a Ident,
        props: &'a DerivePropsInput,
        wrapper_name: &'a Ident,
    ) -> PropsBuilder<'a> {
        PropsBuilder {
            builder_name: name,
            step_trait,
            step_names: Self::step_names(&props.props_name, &props.prop_fields),
            props,
            wrapper_name,
        }
    }
}

impl PropsBuilder<'_> {
    /// Returns the generics type for the first build step
    pub fn to_ty_generics(&self) -> Generics {
        with_param(&self.props.generics, self.first_step().clone())
    }

    fn first_step(&self) -> &Ident {
        &self.step_names[0]
    }

    fn build_step(&self) -> &Ident {
        &self.step_names[self.step_names.len() - 1]
    }

    fn step_names(prefix: &Ident, prop_fields: &[PropField]) -> Vec<Ident> {
        let mut step_names: Vec<Ident> = prop_fields
            .iter()
            .filter(|pf| pf.is_required())
            .map(|pf| pf.to_step_name(prefix))
            .collect();

        step_names.push(Ident::new(
            &format!("{}BuildStep", prefix),
            Span::call_site(),
        ));

        step_names
    }

    fn set_fields(&self) -> impl Iterator<Item = impl ToTokens + '_> {
        self.props.prop_fields.iter().map(|pf| pf.to_field_setter())
    }

    fn impl_steps(&self) -> proc_macro2::TokenStream {
        let Self {
            builder_name,
            props,
            step_names,
            ..
        } = self;
        let DerivePropsInput {
            vis,
            generics,
            prop_fields,
            ..
        } = props;

        let mut fields_index = 0;
        let mut token_stream = proc_macro2::TokenStream::new();

        for (step, step_name) in step_names.iter().enumerate() {
            let mut optional_fields = Vec::new();
            let mut required_field = None;

            if fields_index >= prop_fields.len() {
                break;
            }

            while let Some(pf) = prop_fields.get(fields_index) {
                fields_index += 1;
                if pf.is_required() {
                    required_field = Some(pf);
                    break;
                } else {
                    optional_fields.push(pf);
                }
            }

            let current_step_generics = with_param(generics, step_name.clone());
            let (impl_step_generics, step_ty_generics, step_where_clause) =
                current_step_generics.split_for_impl();
            let optional_prop_fn = optional_fields
                .iter()
                .map(|pf| pf.to_fn(builder_name, &step_ty_generics, vis));
            let required_prop_fn = required_field.iter().map(|pf| {
                let next_step_name = &step_names[step + 1];
                let next_step_generics = with_param(generics, next_step_name.clone());
                let (_, next_step_ty_generics, _) = next_step_generics.split_for_impl();
                pf.to_fn(builder_name, &next_step_ty_generics, vis)
            });

            token_stream.extend(quote! {
                impl#impl_step_generics #builder_name#step_ty_generics #step_where_clause {
                    #(#optional_prop_fn)*
                    #(#required_prop_fn)*
                }
            });
        }
        token_stream
    }
}
