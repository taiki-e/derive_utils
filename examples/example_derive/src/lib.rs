#![crate_type = "proc-macro"]

extern crate derive_utils;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use derive_utils::{derive_trait, EnumData};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::DeriveInput;

#[proc_macro_derive(Iterator)]
pub fn derive_iterator(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let data = EnumData::from_derive(&ast).unwrap();

    derive_trait!(
        data,
        // path
        (Iterator),
        // trait
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
        }
    )
    .unwrap()
    .into()
}

#[proc_macro_derive(ExactSizeIterator)]
pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let data = EnumData::from_derive(&ast).unwrap();

    derive_trait!(
        data,
        // super trait's associated types
        Some(Ident::new("Item", Span::call_site())),
        // path
        (ExactSizeIterator),
        // trait
        trait ExactSizeIterator: Iterator {
            fn len(&self) -> usize;
        }
    )
    .unwrap()
    .into()
}
