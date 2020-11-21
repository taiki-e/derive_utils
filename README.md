# derive_utils

[![crates.io](https://img.shields.io/crates/v/derive_utils.svg?style=flat-square&logo=rust)](https://crates.io/crates/derive_utils)
[![docs.rs](https://img.shields.io/badge/docs.rs-derive__utils-blue?style=flat-square)][docs]
[![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg?style=flat-square)](#license)
[![rustc](https://img.shields.io/badge/rustc-1.31+-blue.svg?style=flat-square)](https://www.rust-lang.org)
[![build status](https://img.shields.io/github/workflow/status/taiki-e/derive_utils/CI/master?style=flat-square)](https://github.com/taiki-e/derive_utils/actions?query=workflow%3ACI+branch%3Amaster)

A procedural macro helper for easily writing [derives macros][proc-macro-derive] for enums.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive_utils = "0.11"
```

The current derive_utils requires Rust 1.31 or later.

## Examples

[`quick_derive!`] macro make easy to write [`proc_macro_derive`][proc-macro-derive] like deriving trait to enum so long as all variants are implemented that trait.

```rust
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

#[proc_macro_derive(Future)]
pub fn derive_future(input: TokenStream) -> TokenStream {
    quick_derive! {
        input,
        // trait path
        std::future::Future,
        // trait definition
        trait Future {
            type Output;
            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output>;
        }
    }
}
```

### Generated code

When deriving for enum like the following:

```rust
#[derive(Iterator, ExactSizeIterator, Future)]
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

## Related Projects

* [auto_enums]: A library for to allow multiple return types by automatically generated enum.
* [futures-enum]: \#\[derive(Future, Stream, Sink, AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead)\] for enums.
* [io-enum]: \#\[derive(Read, Write, Seek, BufRead)\] for enums.
* [iter-enum]: \#\[derive(Iterator, DoubleEndedIterator, ExactSizeIterator, Extend)\] for enums.

[`quick_derive!`]: https://docs.rs/derive_utils/0.11/derive_utils/macro.quick_derive.html
[auto_enums]: https://github.com/taiki-e/auto_enums
[docs]: https://docs.rs/derive_utils
[futures-enum]: https://github.com/taiki-e/futures-enum
[io-enum]: https://github.com/taiki-e/io-enum
[iter-enum]: https://github.com/taiki-e/iter-enum
[proc-macro-derive]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
