#![crate_type = "proc-macro"]

extern crate derive_utils;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use derive_utils::EnumData;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Iterator)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let data = EnumData::from_derive(&ast).unwrap();

    let path = syn::parse_str("Iterator").unwrap();
    let trait_ = syn::parse2(quote! {
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
        }
    })
    .unwrap();

    data.make_impl_trait(path, None, trait_)
        .unwrap()
        .build()
        .into()
}
