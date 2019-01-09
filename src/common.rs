use proc_macro2::{Ident, Span};
use syn::{punctuated::Punctuated, *};

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

#[doc(hidden)]
pub fn ident_call_site(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

#[doc(hidden)]
pub fn path<I: IntoIterator<Item = PathSegment>>(segments: I) -> Path {
    Path {
        leading_colon: None,
        segments: segments.into_iter().collect(),
    }
}

pub(crate) fn block(stmts: Vec<Stmt>) -> Block {
    Block {
        brace_token: default(),
        stmts,
    }
}

pub(crate) fn param_ident(attrs: Vec<Attribute>, ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs,
        ident,
        colon_token: None,
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    })
}
