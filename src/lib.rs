//! A procedural macro helper for easily writing [custom derives] for enums.
//!
//! [custom derives]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
//!
//! ## Examples
//!
//! `quick_derive!` macro make easy to write `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
//!
//! ```rust
//! extern crate derive_utils;
//! extern crate proc_macro;
//!
//! use derive_utils::quick_derive;
//! use proc_macro::TokenStream;
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(Iterator)]
//! # pub fn _derive_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_iterator(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // trait
//!         trait Iterator {
//!             type Item;
//!             fn next(&mut self) -> Option<Self::Item>;
//!             fn size_hint(&self) -> (usize, Option<usize>);
//!         }
//!     }
//! }
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(ExactSizeIterator)]
//! # pub fn _derive_exact_size_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // super trait's associated types
//!         Item,
//!         // trait
//!         trait ExactSizeIterator: Iterator {
//!             fn len(&self) -> usize;
//!         }
//!     }
//! }
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(FusedIterator)]
//! # pub fn _derive_fused_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_fused_iterator(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // super trait's associated types
//!         Item,
//!         // path
//!         (std::iter::FusedIterator),
//!         // trait
//!         trait FusedIterator: Iterator {},
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! #### Generated code
//!
//! When deriving for enum like the following:
//!
//! ```rust
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[derive(Iterator, ExactSizeIterator, FusedIterator)]
//! # struct _Iter<A>(A);
//! enum Iter<A, B> {
//!     A(A),
//!     B(B),
//! }
//! ```
//!
//! Code like this will be generated:
//!
//! ```rust
//! enum Iter<A, B> {
//!     A(A),
//!     B(B),
//! }
//!
//! impl<A, B> Iterator for Iter<A, B>
//! where
//!     A: Iterator,
//!     B: Iterator<Item = <A as Iterator>::Item>,
//! {
//!     type Item = <A as Iterator>::Item;
//!     fn next(&mut self) -> Option<Self::Item> {
//!         match self {
//!             Iter::A(x) => x.next(),
//!             Iter::B(x) => x.next(),
//!         }
//!     }
//!     fn size_hint(&self) -> (usize, Option<usize>) {
//!         match self {
//!             Iter::A(x) => x.size_hint(),
//!             Iter::B(x) => x.size_hint(),
//!         }
//!     }
//! }
//!
//! impl<A, B> ExactSizeIterator for Iter<A, B>
//! where
//!     A: ExactSizeIterator,
//!     B: ExactSizeIterator<Item = <A as Iterator>::Item>,
//! {
//!     fn len(&self) -> usize {
//!         match self {
//!             Iter::A(x) => x.len(),
//!             Iter::B(x) => x.len(),
//!         }
//!     }
//! }
//!
//! impl<A, B> std::iter::FusedIterator for Iter<A, B>
//! where
//!     A: std::iter::FusedIterator,
//!     B: std::iter::FusedIterator<Item = <A as Iterator>::Item>,
//! {
//! }
//! ```
//!
//! See [auto_enums crate](https://github.com/taiki-e/auto_enums/tree/master/derive/src/derive) for more examples.
//!
//! ## Crate Features
//!
//! * `std`
//!   * Enabled by default.
//!   * Generate code for `std` library.
//!   * Disable this feature to generate code for `no_std`.
//!
//! ## Rust Version
//!
//! The current minimum required Rust version is 1.30.
//!

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/derive_utils/0.5.2")]

extern crate proc_macro2;
extern crate quote;
extern crate smallvec;
extern crate syn;

#[macro_use]
mod macros;

mod common;
mod error;
mod parse;

pub use self::common::std_root;
pub use self::error::{Error, Result, *};
pub use self::parse::*;

// Not public API.
#[doc(hidden)]
pub mod __rt {
    #[doc(hidden)]
    pub use crate::{
        common::{ident_call_site, path},
        derive_trait,
    };
    #[doc(hidden)]
    pub use quote::quote;
    #[doc(hidden)]
    pub use syn::*;
}
