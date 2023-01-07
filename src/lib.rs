/*!
A procedural macro helper for easily writing [derives macros][proc-macro-derive] for enums.

# Examples

[`quick_derive!`] macro make easy to write [`proc_macro_derive`][proc-macro-derive]
like deriving trait to enum so long as all variants are implemented that trait.

```rust
# extern crate proc_macro;
#
use derive_utils::quick_derive;
use proc_macro::TokenStream;

# #[cfg(any())]
#[proc_macro_derive(Iterator)]
# pub fn _derive_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
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

# #[cfg(any())]
#[proc_macro_derive(ExactSizeIterator)]
# pub fn _derive_exact_size_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
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
```

### Generated code

When deriving for enum like the following:

```rust
# #[cfg(any())]
#[derive(Iterator, ExactSizeIterator, Future)]
# struct _Enum<A>(A);
enum Enum<A, B> {
    A(A),
    B(B),
}
```

Code like this will be generated:

```rust
enum Enum<A, B> {
    A(A),
    B(B),
}

impl<A, B> std::iter::Iterator for Enum<A, B>
where
    A: std::iter::Iterator,
    B: std::iter::Iterator<Item = <A as std::iter::Iterator>::Item>,
{
    type Item = <A as std::iter::Iterator>::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Enum::A(x) => x.next(),
            Enum::B(x) => x.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Enum::A(x) => x.size_hint(),
            Enum::B(x) => x.size_hint(),
        }
    }
}

impl<A, B> std::iter::ExactSizeIterator for Enum<A, B>
where
    A: std::iter::ExactSizeIterator,
    B: std::iter::ExactSizeIterator<Item = <A as Iterator>::Item>,
{
    fn len(&self) -> usize {
        match self {
            Enum::A(x) => x.len(),
            Enum::B(x) => x.len(),
        }
    }
}
```

[proc-macro-derive]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
*/

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(
    clippy::pedantic,
    // lints for public library
    clippy::alloc_instead_of_core,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    // clippy::std_instead_of_alloc, // alloc requires Rust 1.36
    clippy::std_instead_of_core,
)]
#![allow(clippy::must_use_candidate)]

macro_rules! format_err {
    ($span:expr, $msg:expr $(,)*) => {
        syn::Error::new_spanned(&$span as &dyn quote::ToTokens, &$msg as &dyn core::fmt::Display)
    };
    ($span:expr, $($tt:tt)*) => {
        format_err!($span, format!($($tt)*))
    };
}

macro_rules! bail {
    ($($tt:tt)*) => {
        return Err(format_err!($($tt)*))
    };
}

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
    pub use core::option::Option::{None, Some};

    use proc_macro2::TokenStream;
    #[doc(hidden)]
    pub use quote::{format_ident, quote};
    use syn::Error;
    #[doc(hidden)]
    pub use syn::{parse2, parse_quote, ItemTrait, Path};

    use crate::EnumData;

    #[doc(hidden)]
    pub fn parse_input(
        input: impl Into<TokenStream>,
        f: fn(EnumData) -> TokenStream,
    ) -> TokenStream {
        parse2::<EnumData>(input.into()).map(f).unwrap_or_else(Error::into_compile_error)
    }
}
