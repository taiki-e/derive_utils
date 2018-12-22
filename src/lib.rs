//! A procedural macro helper for easily writing `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
//!
//! ## Examples
//!
//! ```rust
//! extern crate derive_utils;
//! extern crate proc_macro;
//! extern crate proc_macro2;
//! extern crate syn;
//!
//! use derive_utils::{derive_trait, EnumData};
//! use proc_macro::TokenStream;
//! use proc_macro2::{Ident, Span};
//! use syn::DeriveInput;
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(Iterator)]
//! # pub fn _derive_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_iterator(input: TokenStream) -> TokenStream {
//!     let ast: DeriveInput = syn::parse(input).unwrap();
//!     let data = EnumData::from_derive(&ast).unwrap();
//!
//!     derive_trait!(
//!         data,
//!         _,
//!         // trait
//!         trait Iterator {
//!             type Item;
//!             fn next(&mut self) -> Option<Self::Item>;
//!             fn size_hint(&self) -> (usize, Option<usize>);
//!         }
//!     )
//!     .unwrap()
//!     .into()
//! }
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(ExactSizeIterator)]
//! # pub fn _derive_exact_size_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
//!     let ast: DeriveInput = syn::parse(input).unwrap();
//!     let data = EnumData::from_derive(&ast).unwrap();
//!
//!     derive_trait!(
//!         data,
//!         // super trait's associated types
//!         Some(Ident::new("Item", Span::call_site())),
//!         _,
//!         // trait
//!         trait ExactSizeIterator: Iterator {
//!             fn len(&self) -> usize;
//!         }
//!     )
//!     .unwrap()
//!     .into()
//! }
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(FusedIterator)]
//! # pub fn _derive_fused_iterator(input: TokenStream) -> TokenStream { input }
//! pub fn derive_fused_iterator(input: TokenStream) -> TokenStream {
//!     let ast: DeriveInput = syn::parse(input).unwrap();
//!     let data = EnumData::from_derive(&ast).unwrap();
//!
//!     derive_trait!(
//!         data,
//!         // super trait's associated types
//!         Some(Ident::new("Item", Span::call_site())),
//!         // path
//!         (std::iter::FusedIterator),
//!         // trait
//!         trait FusedIterator: Iterator {}
//!     )
//!     .unwrap()
//!     .into()
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
//!   * Disable to generate code for `no_std`.
//!
//! ## Rust Version
//!
//! The current minimum required Rust version is 1.30.
//!

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/derive_utils/0.4.0")]

extern crate proc_macro2;
extern crate quote;
extern crate smallvec;

#[doc(hidden)]
pub extern crate syn;

#[macro_use]
mod macros;

mod common;
mod error;
mod parse;

#[doc(hidden)]
pub use quote::quote;

pub use self::common::*;
pub use self::error::{Error, Result, *};
pub use self::parse::*;
