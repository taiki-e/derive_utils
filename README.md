# derive_utils

[![Build Status](https://travis-ci.org/taiki-e/derive_utils.svg?branch=master)](https://travis-ci.org/taiki-e/derive_utils)
[![version](https://img.shields.io/crates/v/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![documentation](https://docs.rs/derive_utils/badge.svg)](https://docs.rs/derive_utils/)
[![license](https://img.shields.io/crates/l/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![Rustc Version](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html)

A procedural macro helper for easily writing [custom derives] for enums.

[custom derives]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive_utils = "0.7"
```

The current derive_utils requires Rust 1.31 or later.

## Examples

`quick_derive!` macro make easy to write `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.

```rust
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
```

### Generated code

When deriving for enum like the following:

```rust
#[derive(Iterator, ExactSizeIterator, FusedIterator, Future)]
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

impl<A, B> Iterator for Enum<A, B>
where
    A: Iterator,
    B: Iterator<Item = <A as Iterator>::Item>,
{
    type Item = <A as Iterator>::Item;
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

impl<A, B> ExactSizeIterator for Enum<A, B>
where
    A: ExactSizeIterator,
    B: ExactSizeIterator<Item = <A as Iterator>::Item>,
{
    fn len(&self) -> usize {
        match self {
            Enum::A(x) => x.len(),
            Enum::B(x) => x.len(),
        }
    }
}

impl<A, B> std::iter::FusedIterator for Enum<A, B>
where
    A: std::iter::FusedIterator,
    B: std::iter::FusedIterator<Item = <A as Iterator>::Item>,
{
}

impl<A, B> std::future::Future for Enum<A, B>
where
    A: std::future::Future,
    B: std::future::Future<Output = <A as std::future::Future>::Output>,
{
    type Output = <A as std::future::Future>::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            match self.get_unchecked_mut() {
                Enum::A(x) => std::pin::Pin::new_unchecked(x).poll(cx),
                Enum::B(x) => std::pin::Pin::new_unchecked(x).poll(cx),
            }
        }
    }
}
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
