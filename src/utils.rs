use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Attribute, Block, GenericParam, Stmt, TypeParam};

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

pub(crate) fn block(stmts: Vec<Stmt>) -> Block {
    Block { brace_token: default(), stmts }
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

macro_rules! error {
    ($msg:expr) => {
        return Err(syn::Error::new_spanned(span!($msg), $msg))
    };
    ($span:expr, $msg:expr) => {
        return Err(syn::Error::new_spanned(span!($span), $msg))
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}
