use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated, token::Colon2, GenericParam, Generics, Path, PathArguments,
    PathSegment, Token, TraitBound, TraitBoundModifier, TypeParam, TypeParamBound,
};

pub fn with_param_bounds(generics: &Generics, param_ident: Ident, param_bounds: Ident) -> Generics {
    let mut new_generics = generics.clone();
    new_generics
        .params
        .push(new_param_bounds(param_ident, param_bounds));
    new_generics
}

pub fn with_param(generics: &Generics, param_ident: Ident) -> Generics {
    let mut new_generics = generics.clone();
    new_generics.params.push(new_param(param_ident));
    new_generics
}

fn new_param(param_ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: param_ident,
        colon_token: Some(Token![:](Span::call_site())),
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    })
}

fn new_param_bounds(param_ident: Ident, param_bounds: Ident) -> GenericParam {
    let mut path_segments: Punctuated<PathSegment, Colon2> = Punctuated::new();
    path_segments.push(PathSegment {
        ident: param_bounds,
        arguments: PathArguments::None,
    });

    let mut param_bounds: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
    param_bounds.push(TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path {
            leading_colon: None,
            segments: path_segments,
        },
    }));

    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: param_ident,
        colon_token: Some(Token![:](Span::call_site())),
        bounds: param_bounds,
        eq_token: None,
        default: None,
    })
}
