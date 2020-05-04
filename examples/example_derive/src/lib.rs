#![warn(rust_2018_idioms, single_use_lifetimes)]

// older compilers require explicit `extern crate`.
#[allow(unused_extern_crates)]
extern crate proc_macro;

use derive_utils::quick_derive;
use proc_macro::TokenStream;

#[proc_macro_derive(Iterator)]
pub fn derive_iterator(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // trait
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
        }
    }
}

#[proc_macro_derive(ExactSizeIterator)]
pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // super trait's associated types
        Item,
        // trait
        trait ExactSizeIterator: Iterator {
            fn len(&self) -> usize;
        }
    }
}

#[proc_macro_derive(FusedIterator)]
pub fn derive_fused_iterator(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // super trait's associated types
        Item,
        // path
        (std::iter::FusedIterator),
        // trait
        trait FusedIterator: Iterator {},
    }
}

#[proc_macro_derive(Future)]
pub fn derive_future(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // path
        (std::future::Future),
        // trait
        trait Future {
            type Output;
            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>)
                -> std::task::Poll<Self::Output>;
        }
    }
}
