use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated, token::Colon2, GenericArgument, GenericParam, Generics, Path,
    PathArguments, PathSegment, Token, TraitBound, TraitBoundModifier, Type, TypeParam,
    TypeParamBound, TypePath,
};

pub type GenericArguments = Punctuated<GenericArgument, Token![,]>;

pub fn to_arguments(generics: &Generics, type_ident: Ident) -> GenericArguments {
    let mut args: GenericArguments = Punctuated::new();
    args.extend(generics.params.iter().map(|param| match param {
        GenericParam::Type(type_param) => GenericArgument::Type(Type::Path(TypePath {
            path: Path::from(type_param.ident.clone()),
            qself: None,
        })),
        GenericParam::Lifetime(lifetime_param) => {
            GenericArgument::Lifetime(lifetime_param.lifetime.clone())
        }
        _ => unimplemented!("const params are not supported in the derive macro"),
    }));
    args.push(GenericArgument::Type(Type::Path(TypePath {
        path: Path::from(type_ident),
        qself: None,
    })));
    args
}

pub fn with_param_bounds(generics: &Generics, param_ident: Ident, param_bounds: Ident) -> Generics {
    let mut new_generics = generics.clone();
    new_generics
        .params
        .push(new_param_bounds(param_ident, param_bounds));
    new_generics
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
