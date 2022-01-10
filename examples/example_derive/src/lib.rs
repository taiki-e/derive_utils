#![warn(rust_2018_idioms, single_use_lifetimes)]

use derive_utils::quick_derive;
use proc_macro::TokenStream;

#[proc_macro_derive(Iterator)]
pub fn derive_iterator(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // trait path
        std::iter::Iterator,
        // trait definition
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
        // trait path
        std::iter::ExactSizeIterator,
        // super trait's associated types
        <Item>,
        // trait definition
        trait ExactSizeIterator: Iterator {
            fn len(&self) -> usize;
        }
    }
}

#[proc_macro_derive(MyTrait1)]
pub fn derive_my_trait1(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // trait path
        MyTrait1,
        // trait definition
        trait MyTrait1 {
            type Assoc1;
            type Assoc2;
        }
    }
}

#[proc_macro_derive(MyTrait2)]
pub fn derive_my_trait2(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // trait path
        MyTrait2,
        // super trait's associated types
        <Assoc1, Assoc2>,
        // trait definition
        trait MyTrait2: MyTrait1 {}
    }
}
