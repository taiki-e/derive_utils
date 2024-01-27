// SPDX-License-Identifier: Apache-2.0 OR MIT

/*!
A procedural macro helper for easily writing [derives macros][proc-macro-derive] for enums.

## Examples

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

## Related Projects

- [auto_enums]: A library for to allow multiple return types by automatically generated enum.
- [io-enum]: \#\[derive(Read, Write, Seek, BufRead)\] for enums.
- [iter-enum]: \#\[derive(Iterator, DoubleEndedIterator, ExactSizeIterator, FusedIterator, Extend)\] for enums.

[`quick_derive!`]: https://docs.rs/derive_utils/0.12/derive_utils/macro.quick_derive.html
[auto_enums]: https://github.com/taiki-e/auto_enums
[io-enum]: https://github.com/taiki-e/io-enum
[iter-enum]: https://github.com/taiki-e/iter-enum
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
#![warn(
    // Lints that may help when writing public library.
    // missing_docs,
    clippy::alloc_instead_of_core,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::impl_trait_in_params,
    // clippy::missing_inline_in_public_items,
    // clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
)]
#![allow(missing_debug_implementations, clippy::must_use_candidate)]

#[macro_use]
mod error;

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
    ($input:expr, $trait_path:expr, <$super:ident>, $($trait_def:tt)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                &$crate::__private::parse_quote!($trait_path),
                $crate::__private::Some(
                    $crate::__private::format_ident!($crate::__private::stringify!($super))
                ),
                $crate::__private::parse_quote!($($trait_def)*),
            )
        })
        .into()
    };
    ($input:expr, $trait_path:expr, <$($super:ident),+ $(,)?>, $($trait_def:tt)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                &$crate::__private::parse_quote!($trait_path),
                $crate::__private::vec![
                    $( $crate::__private::format_ident!($crate::__private::stringify!($super)) ),+
                ],
                $crate::__private::parse_quote!($($trait_def)*),
            )
        })
        .into()
    };
    ($input:expr, $trait_path:expr, $($trait_def:tt)*) => {
        $crate::__private::parse_input($input, |data| {
            $crate::derive_trait(
                &data,
                &$crate::__private::parse_quote!($trait_path),
                $crate::__private::None,
                $crate::__private::parse_quote!($($trait_def)*),
            )
        })
        .into()
    };
}

// Not public API.
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use core::{
        option::Option::{None, Some},
        stringify,
    };
    #[doc(hidden)]
    pub use std::vec;

    use proc_macro2::TokenStream;
    #[doc(hidden)]
    pub use quote::{format_ident, quote};
    use syn::Error;
    #[doc(hidden)]
    pub use syn::{parse2, parse_quote, ItemTrait, Path};

    use crate::EnumData;

    #[doc(hidden)]
    pub fn parse_input<T: Into<TokenStream>, F: Fn(EnumData) -> TokenStream>(
        input: T,
        f: F,
    ) -> TokenStream {
        parse2::<EnumData>(input.into()).map_or_else(Error::into_compile_error, f)
    }
}
