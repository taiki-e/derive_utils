# derive_utils

[![Build Status](http://img.shields.io/travis/taiki-e/derive_utils.svg)](https://travis-ci.org/taiki-e/derive_utils)
[![version](https://img.shields.io/crates/v/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![documentation](https://docs.rs/derive_utils/badge.svg)](https://docs.rs/derive_utils/)
[![license](https://img.shields.io/crates/l/derive_utils.svg)](https://crates.io/crates/derive_utils/)
[![Rustc Version](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://blog.rust-lang.org/2018/10/25/Rust-1.30.0.html)

[API Documentation](https://docs.rs/derive_utils/)

A procedural macro helper for trait implemention for enums.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive_utils = "0.1"
```

and this to your crate root:

```rust
extern crate derive_utils;
```

## Examples

```rust
extern crate derive_utils;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use derive_utils::EnumData;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Iterator)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let data = EnumData::from_derive(&ast).unwrap();

    let path = syn::parse_str("Iterator").unwrap();
    let trait_ = syn::parse2(quote! {
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
        }
    }).unwrap();

    data.make_impl_trait(path, None, trait_)
        .unwrap()
        .build()
        .into()
}
```

#### Generated code

When deriving for enum like the following:

```rust
#[derive(Iterator)]
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
```

## Crate Features

* `std`
  * Enabled by default.
  * Generate code for `std` library.
  * Disable to generate code for `no_std`.

## Rust Version

The current minimum required Rust version is 1.27.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
