//! A procedural macro helper for trait implemention for enums.
//!
//! ## Examples
//!
//! ```rust
//! extern crate derive_utils;
//! extern crate proc_macro;
//! extern crate quote;
//! extern crate syn;
//!
//! use derive_utils::EnumData;
//! use proc_macro::TokenStream;
//! use quote::quote;
//! use syn::DeriveInput;
//!
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[proc_macro_derive(Iterator)]
//! # pub fn _derive(input: TokenStream) -> TokenStream { input }
//! pub fn derive(input: TokenStream) -> TokenStream {
//!     let ast: DeriveInput = syn::parse(input).unwrap();
//!     let data = EnumData::from_derive(&ast).unwrap();
//!
//!     let path = syn::parse_str("Iterator").unwrap();
//!     let trait_ = syn::parse2(quote! {
//!         trait Iterator {
//!             type Item;
//!             fn next(&mut self) -> Option<Self::Item>;
//!             fn size_hint(&self) -> (usize, Option<usize>);
//!         }
//!     }).unwrap();
//!
//!     data.make_impl_trait(path, None, trait_)
//!         .unwrap()
//!         .build()
//!         .into()
//! }
//! ```
//!
//! #### Generated code
//!
//! When deriving for enum like the following:
//!
//! ```rust
//! # #[cfg(all(feature = "std", not(feature = "std")))]
//! #[derive(Iterator)]
//! # struct A<A>(A);
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
//! ```
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
#![doc(html_root_url = "https://docs.rs/derive_utils/0.1.0")]

extern crate proc_macro2;
extern crate quote;
extern crate smallvec;
extern crate syn;

mod common;
mod error;
mod parse;

pub use self::common::*;
pub use self::error::{Error, Result, *};
pub use self::parse::*;
