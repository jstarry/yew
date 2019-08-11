use super::PropField;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::Generics;

pub struct WrappedProps<'a> {
    wrapped_props_name: &'a Ident,
    generics: &'a Generics,
    prop_fields: &'a [PropField],
}

impl ToTokens for WrappedProps<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            generics,
            wrapped_props_name,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let turbofish_generics = ty_generics.as_turbofish();

        let wrapped_field_defs = self.wrapped_field_defs();
        let wrapped_default_setters = self.wrapped_default_setters();

        let expanded = quote! {
            struct #wrapped_props_name<#ty_generics> {
                #(#wrapped_field_defs)*
            }

            impl#impl_generics ::std::default::Default for #wrapped_props_name<#ty_generics> #where_clause {
                fn default() -> Self {
                    #wrapped_props_name#turbofish_generics {
                        #(#wrapped_default_setters)*
                    }
                }
            }
        };
        tokens.extend(proc_macro2::TokenStream::from(expanded));
    }
}

impl<'a> WrappedProps<'_> {
    pub fn new(
        name: &'a Ident,
        generics: &'a Generics,
        prop_fields: &'a [PropField],
    ) -> WrappedProps<'a> {
        WrappedProps {
            wrapped_props_name: name,
            generics,
            prop_fields,
        }
    }
}

impl WrappedProps<'_> {
    fn wrapped_field_defs(&self) -> impl Iterator<Item = impl ToTokens + '_> {
        self.prop_fields.iter().map(|pf| pf.to_field_def())
    }

    fn wrapped_default_setters(&self) -> impl Iterator<Item = impl ToTokens + '_> {
        self.prop_fields.iter().map(|pf| pf.to_default_setter())
    }
}
