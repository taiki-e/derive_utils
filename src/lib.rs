//! A procedural macro helper for easily writing [derives macros][proc-macro-derive] for enums.
//!
//! # Examples
//!
//! [`quick_derive!`] macro make easy to write [`proc_macro_derive`][proc-macro-derive]
//! like deriving trait to enum so long as all variants are implemented that trait.
//!
//! ```rust
//! # extern crate proc_macro;
//! #
//! use derive_utils::quick_derive;
//! use proc_macro::TokenStream;
//!
//! # #[cfg(any())]
//! #[proc_macro_derive(Iterator)]
//! # pub fn _derive_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
//! pub fn derive_iterator(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // trait path
//!         std::iter::Iterator,
//!         // trait definition
//!         trait Iterator {
//!             type Item;
//!             fn next(&mut self) -> Option<Self::Item>;
//!             fn size_hint(&self) -> (usize, Option<usize>);
//!         }
//!     }
//! }
//!
//! # #[cfg(any())]
//! #[proc_macro_derive(ExactSizeIterator)]
//! # pub fn _derive_exact_size_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
//! pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // trait path
//!         std::iter::ExactSizeIterator,
//!         // super trait's associated types
//!         <Item>,
//!         // trait definition
//!         trait ExactSizeIterator: Iterator {
//!             fn len(&self) -> usize;
//!         }
//!     }
//! }
//!
//! # #[cfg(any())]
//! #[proc_macro_derive(Future)]
//! # pub fn _derive_future(_: TokenStream) -> TokenStream { unimplemented!() }
//! pub fn derive_future(input: TokenStream) -> TokenStream {
//!     quick_derive! {
//!         input,
//!         // trait path
//!         std::future::Future,
//!         // trait definition
//!         trait Future {
//!             type Output;
//!             fn poll(
//!                 self: std::pin::Pin<&mut Self>,
//!                 cx: &mut std::task::Context<'_>,
//!             ) -> std::task::Poll<Self::Output>;
//!         }
//!     }
//! }
//! ```
//!
//! ### Generated code
//!
//! When deriving for enum like the following:
//!
//! ```rust
//! # #[cfg(any())]
//! #[derive(Iterator, ExactSizeIterator, Future)]
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
//! impl<A, B> std::iter::Iterator for Enum<A, B>
//! where
//!     A: std::iter::Iterator,
//!     B: std::iter::Iterator<Item = <A as std::iter::Iterator>::Item>,
//! {
//!     type Item = <A as std::iter::Iterator>::Item;
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
//! impl<A, B> std::iter::ExactSizeIterator for Enum<A, B>
//! where
//!     A: std::iter::ExactSizeIterator,
//!     B: std::iter::ExactSizeIterator<Item = <A as Iterator>::Item>,
//! {
//!     fn len(&self) -> usize {
//!         match self {
//!             Enum::A(x) => x.len(),
//!             Enum::B(x) => x.len(),
//!         }
//!     }
//! }
//!
//! impl<A, B> std::future::Future for Enum<A, B>
//! where
//!     A: std::future::Future,
//!     B: std::future::Future<Output = <A as std::future::Future>::Output>,
//! {
//!     type Output = <A as std::future::Future>::Output;
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
//! [proc-macro-derive]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![forbid(unsafe_code)]
#![warn(future_incompatible, rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(clippy::all, clippy::default_trait_access)]
// mem::take, #[non_exhaustive], and Option::{as_deref, as_deref_mut} require Rust 1.40,
// matches! requires Rust 1.42, str::{strip_prefix, strip_suffix} requires Rust 1.45
#![allow(
    clippy::mem_replace_with_default,
    clippy::manual_non_exhaustive,
    clippy::option_as_ref_deref,
    clippy::match_like_matches_macro,
    clippy::manual_strip
)]

#[macro_use]
mod utils;

mod ast;
mod parse;

pub use crate::{
    ast::EnumData,
    parse::{derive_trait, EnumImpl},
};

/// A macro for making easy to write `proc_macro_derive` like deriving trait to
/// enum so long as all variants are implemented that trait.
///
/// See crate level documentation for details.
#[macro_export]
macro_rules! quick_derive {
    ($input:expr, $trait_path:expr, <$super:ident>, $trait_def:item $(,)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                $crate::__private::parse_quote!($trait_path),
                $crate::__private::Some($crate::__private::format_ident!(stringify!($super))),
                $crate::__private::parse_quote!($trait_def),
            )
        })
        .into()
    };
    // TODO: $(,)? requires Rust 1.32.
    ($input:expr, $trait_path:expr, <$($super:ident),+ $(,)*>, $trait_def:item $(,)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                $crate::__private::parse_quote!($trait_path),
                vec![$( $crate::__private::format_ident!(stringify!($super)) ),+],
                $crate::__private::parse_quote!($trait_def),
            )
        })
        .into()
    };
    ($input:expr, $trait_path:expr, $trait_def:item $(,)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                $crate::__private::parse_quote!($trait_path),
                $crate::__private::None,
                $crate::__private::parse_quote!($trait_def),
            )
        })
        .into()
    };
}

// Not public API.
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use quote::{format_ident, quote};
    #[doc(hidden)]
    pub use std::option::Option::{None, Some};
    #[doc(hidden)]
    pub use syn::{parse2, parse_quote, ItemTrait, Path};

    use proc_macro2::TokenStream;

    use crate::EnumData;

    #[doc(hidden)]
    pub fn parse_input(
        input: impl Into<TokenStream>,
        f: fn(EnumData) -> TokenStream,
    ) -> TokenStream {
        parse2::<EnumData>(input.into()).map(f).unwrap_or_else(|e| e.to_compile_error())
    }
}
