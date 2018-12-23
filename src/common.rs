use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use smallvec::SmallVec;
use syn::{punctuated::Punctuated, *};

pub(crate) type Stack<T> = SmallVec<[T; 8]>;

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

pub(crate) fn param_ident(ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::with_capacity(0),
        ident,
        colon_token: None,
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    })
}

/// Returns standard library's root.
///
/// In default returns `::std`.
/// if disabled default crate feature, returned `::core`.
pub fn std_root() -> TokenStream {
    #[cfg(feature = "std")]
    let root = quote!(::std);
    #[cfg(not(feature = "std"))]
    let root = quote!(::core);
    root
}
