# derive_utils

[![Build Status](http://img.shields.io/travis/taiki-e/derive_utils.svg)](https://travis-ci.org/taiki-e/derive_utils)
[![version](https://img.shields.io/crates/v/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![documentation](https://docs.rs/derive_utils/badge.svg)](https://docs.rs/derive_utils/)
[![license](https://img.shields.io/crates/l/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![Rustc Version](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://blog.rust-lang.org/2018/10/25/Rust-1.30.0.html)

A procedural macro helper for easily writing `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive_utils = "0.4"
```

and this to your crate root:

```rust
extern crate derive_utils;
```

## Examples

```rust
extern crate derive_utils;
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
        trait FusedIterator: Iterator {}
    }
}
```

#### Generated code

When deriving for enum like the following:

```rust
#[derive(Iterator, ExactSizeIterator, FusedIterator)]
enum Iter<A, B> {
    A(A),
    B(B),
}
```

Code like this will be generated:

```rust
enum Iter<A, B> {
    A(A),
    B(B),
}

impl<A, B> Iterator for Iter<A, B>
where
    A: Iterator,
    B: Iterator<Item = <A as Iterator>::Item>,
{
    type Item = <A as Iterator>::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::A(x) => x.next(),
            Iter::B(x) => x.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::A(x) => x.size_hint(),
            Iter::B(x) => x.size_hint(),
        }
    }
}

impl<A, B> ExactSizeIterator for Iter<A, B>
where
    A: ExactSizeIterator,
    B: ExactSizeIterator<Item = <A as Iterator>::Item>,
{
    fn len(&self) -> usize {
        match self {
            Iter::A(x) => x.len(),
            Iter::B(x) => x.len(),
        }
    }
}

impl<A, B> std::iter::FusedIterator for Iter<A, B>
where
    A: std::iter::FusedIterator,
    B: std::iter::FusedIterator<Item = <A as Iterator>::Item>,
{
}
```

See [auto_enums crate](https://github.com/taiki-e/auto_enums/tree/master/derive/src/derive) for more examples.

## Crate Features

* `std`
  * Enabled by default.
  * Generate code for `std` library.
  * Disable to generate code for `no_std`.

## Rust Version

The current minimum required Rust version is 1.30.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
