//! A procedural macro helper for easily writing [derive macros] for enums.
//!
//! [derive macros]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
//!
//! ## Examples
//!
//! `quick_derive!` macro make easy to write `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
//!
//! ```rust
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
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(Future)]
//! # pub fn derive_future(input: TokenStream) -> TokenStream { input }
//! pub fn derive_future(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // path
//!         (std::future::Future),
//!         // trait
//!         trait Future {
//!             type Output;
//!             fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>)
//!                 -> std::task::Poll<Self::Output>;
//!         }
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! #### Generated code
//!
//! When deriving for enum like the following:
//!
//! ```rust
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[derive(Iterator, ExactSizeIterator, FusedIterator, Future)]
//! # struct _Enum<A>(A);
//! enum Enum<A, B> {
//!     A(A),
//!     B(B),
//! }
//! ```
//!
//! Code like this will be generated:
//!
//! ```rust
//! enum Enum<A, B> {
//!     A(A),
//!     B(B),
//! }
//!
//! impl<A, B> Iterator for Enum<A, B>
//! where
//!     A: Iterator,
//!     B: Iterator<Item = <A as Iterator>::Item>,
//! {
//!     type Item = <A as Iterator>::Item;
//!     fn next(&mut self) -> Option<Self::Item> {
//!         match self {
//!             Enum::A(x) => x.next(),
//!             Enum::B(x) => x.next(),
//!         }
//!     }
//!     fn size_hint(&self) -> (usize, Option<usize>) {
//!         match self {
//!             Enum::A(x) => x.size_hint(),
//!             Enum::B(x) => x.size_hint(),
//!         }
//!     }
//! }
//!
//! impl<A, B> ExactSizeIterator for Enum<A, B>
//! where
//!     A: ExactSizeIterator,
//!     B: ExactSizeIterator<Item = <A as Iterator>::Item>,
//! {
//!     fn len(&self) -> usize {
//!         match self {
//!             Enum::A(x) => x.len(),
//!             Enum::B(x) => x.len(),
//!         }
//!     }
//! }
//!
//! impl<A, B> std::iter::FusedIterator for Enum<A, B>
//! where
//!     A: std::iter::FusedIterator,
//!     B: std::iter::FusedIterator<Item = <A as Iterator>::Item>,
//! {
//! }
//!
//! impl<A, B> std::future::Future for Enum<A, B>
//! where
//!     A: std::future::Future,
//!     B: std::future::Future<Output = <A as std::future::Future>::Output>,
//! {
//!     type Output = <A as std::future::Future>::Output;
//!
//!     fn poll(
//!         self: std::pin::Pin<&mut Self>,
//!         cx: &mut std::task::Context<'_>,
//!     ) -> std::task::Poll<Self::Output> {
//!         unsafe {
//!             match self.get_unchecked_mut() {
//!                 Enum::A(x) => std::pin::Pin::new_unchecked(x).poll(cx),
//!                 Enum::B(x) => std::pin::Pin::new_unchecked(x).poll(cx),
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! See [auto_enums] crate for more examples.
//!
//! [auto_enums]: (https://github.com/taiki-e/auto_enums)
//!

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/derive_utils/0.9.0")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because these lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::use_self)]

#[macro_use]
mod macros;

mod parse;

pub use crate::parse::*;

// Not public API.
#[doc(hidden)]
pub mod __rt {
    #[doc(hidden)]
    pub use crate::{derive_trait, derive_trait_internal, parse::build_item};
    #[doc(hidden)]
    pub use quote::{format_ident, quote, ToTokens};
    #[doc(hidden)]
    pub use syn::*;
}
