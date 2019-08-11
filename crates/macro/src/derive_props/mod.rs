mod builder;
mod field;
mod generics;
mod wrapped;

use builder::PropsBuilder;
use field::PropField;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::convert::TryInto;
use syn::parse::{Parse, ParseStream, Result};
use syn::{DeriveInput, Generics, Visibility};
use wrapped::WrappedProps;

pub struct DerivePropsInput {
    vis: Visibility,
    generics: Generics,
    props_name: Ident,
    prop_fields: Vec<PropField>,
}

impl Parse for DerivePropsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let named_fields = match input.data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(fields) => fields.named,
                _ => unimplemented!("only structs are supported"),
            },
            _ => unimplemented!("only structs are supported"),
        };

        let mut prop_fields: Vec<PropField> = named_fields
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<PropField>>>()?;

        // Alphabetize
        prop_fields.sort();

        Ok(Self {
            vis: input.vis,
            props_name: input.ident,
            generics: input.generics,
            prop_fields,
        })
    }
}

impl ToTokens for DerivePropsInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            generics,
            props_name,
            prop_fields,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let wrapped_props_name = Ident::new(&format!("Wrapped{}", props_name), Span::call_site());
        let wrapped_props = WrappedProps::new(&wrapped_props_name, &generics, &prop_fields);

        let builder_name = Ident::new(&format!("{}Builder", props_name), Span::call_site());
        let builder_step = Ident::new(&format!("{}BuilderStep", props_name), Span::call_site());
        let builder = PropsBuilder::new(&builder_name, &builder_step, &self, &wrapped_props_name);
        let builder_ty_generics = builder.to_ty_generics();

        let impl_properties = quote! {
            impl#impl_generics ::yew::html::Properties for #props_name<#ty_generics> #where_clause {
                type Builder = #builder_name#builder_ty_generics;

                fn builder() -> Self::Builder {
                    #builder_name {
                        wrapped: ::std::boxed::Box::new(::std::default::Default::default()),
                        _marker: ::std::marker::PhantomData,
                    }
                }
            }
        };

        wrapped_props.to_tokens(tokens);
        builder.to_tokens(tokens);
        impl_properties.to_tokens(tokens);
    }
}

impl DerivePropsInput {}
