use proc_macro2::{Ident, Span};
use syn::{punctuated::Punctuated, *};

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

pub(crate) fn ident<S: AsRef<str>>(s: S) -> Ident {
    Ident::new(s.as_ref(), Span::call_site())
}

pub(crate) fn path<I: IntoIterator<Item = PathSegment>>(segments: I) -> Path {
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

macro_rules! span {
    ($expr:expr) => {
        $expr.clone()
    };
}

macro_rules! err {
    ($msg:expr) => {{
        Err(syn::Error::new_spanned(span!($msg), $msg))
    }};
    ($span:expr, $msg:expr) => {
        Err(syn::Error::new_spanned(span!($span), $msg))
    };
    ($span:expr, $($tt:tt)*) => {
        err!($span, format!($($tt)*))
    };
}
