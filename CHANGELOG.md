# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

## [Unreleased]

## [0.10.0] - 2020-06-02

* `quick_derive!` macro now accepts both `proc_macro2::TokenStream` and `proc_macro::TokenStream` as input.

* `quick_derive!` macro no longer supports trait path elision.

* The parentheses that wrap the trait path passed to `quick_derive!` macro are no longer needed. For example:

  ```diff
    quick_derive! {
        input,
        // trait path
  -     (std::iter::Iterator),
  +     std::iter::Iterator,
        // trait definition
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
        }
    }
  ```

* The way of specifying super trait's associated types has been changed.

  ```diff
    quick_derive! {
        input,
  -     // super trait's associated types
  -     Item,
  -     // trait path
  -     (std::iter::ExactSizeIterator),
  +     // trait path
  +     std::iter::ExactSizeIterator,
  +     // super trait's associated types
  +     <Item>,
        // trait definition
        trait ExactSizeIterator: Iterator {
            fn len(&self) -> usize;
        }
    }
  ```

* Added `derive_trait` function.

* Added `EnumImpl::{new, from_trait}` functions.

* Added `EnumData::{field_types, variant_idents}` methods.

* Removed `derive_trait!` macro in favor of `derive_trait` function.

* Removed `EnumData::make_impl` and `EnumData::impl_with_capacity` in favor of `EnumImpl::new`.

* Removed `EnumData::make_impl_trait` and `EnumData::impl_trait_with_capacity` in favor of `EnumImpl::from_trait`.

* Removed `EnumImpl::push_generic_param_ident` in favor of `EnumImpl::push_generic_param(TypeParam::from(ident).into())`.

* Removed `MaybeEnum` and `EnumElements` in favor of `syn::parse::<EnumData>()`.

* Removed some hidden APIs.

* Implemented `Deref<Target = syn::ItemEnum>`, `syn::parse::Parse`, and `quote::ToTokens` for `EnumData`

* Implemented `From<EnumData>` for `syn::ItemEnum`

## [0.9.1] - 2019-09-15

* [Weakened requirements to a number of enum variants.][15]

[15]: https://github.com/taiki-e/derive_utils/pull/15

## [0.9.0] - 2019-08-14

* Updated `proc-macro2`, `syn`, and `quote` to 1.0.

* Banned manual implementation of `MaybeEnum` for forward compatibility.

* Added `vis: Visibility` field to `EnumElements`.

* Hided some undocumented items from the public API.

## [0.8.0] - 2019-06-26

* Added `quote::ToTokens` bound to `MaybeEnum`.

* Removed `EnumData::{from_item, from_derive}` in favor of `EnumData::new`

* Improved error messages.

## [0.7.2] - 2019-05-30

* Improved error messages.

## [0.7.1] - 2019-05-21

* Improved support for arbitrary self type.

* Improved error messages.

## [0.7.0] - 2019-03-13

* Transition to Rust 2018. With this change, the minimum required version will go up to Rust 1.31.

* Improved error messages.

* Removed `Error` and `Result` types. The current `derive_utils` uses [`syn::Error`](https://docs.rs/syn/0.15/syn/struct.Error.html) and [`syn::Result`](https://docs.rs/syn/0.15/syn/parse/type.Result.html) types.

## [0.6.3] - 2019-02-05

* Added `EnumData::new` method.

* Updated minimum `syn` version to 0.15.22.

* Removed dependency on `smallvec`.

## [0.6.2] - 2019-01-30

* Added `Error::to_compile_error` method.

* Hided some undocumented items from the public API.

## [0.6.1] - 2019-01-26

* Improved support for `self: Pin<&Self>` and `self: Pin<&mut Self>`.

* Updated minimum `smallvec` version to 0.6.8.

## [0.6.0] - 2019-01-09

* Removed `"std"` feature and `std_root` function. `derive_utils` can generate accurate code without `"std"` feature.

* Removed deprecated `push_method_pin*` methods. Use `push_method` instead.

* Improved documentation.

## [0.5.4] - 2019-01-03

* Improved documentation.

## [0.5.3] - 2018-12-27

* Improved macro implementations. The trailing comma is supported.

* Improved error messages.

## [0.5.2] - 2018-12-27

* Hided some undocumented items from the public API.

## [0.5.1] - 2018-12-26

* Updated to stable Pin API. `Pin::get_mut_unchecked` renamed to `Pin::get_unchecked_mut` in [stabilization](https://github.com/rust-lang/rust/pull/56939).

## [0.5.0] - 2018-12-23

* Added `quick_derive` macro.

* Removed `derive_trait_with_capacity` macro.

## [0.4.0] - 2018-12-22

* Added support for `self: Pin<&Self>` and `self: Pin<&mut Self>`.

* Allow using the trait name as trait path.

## [0.3.0] - 2018-12-22

* Add `derive_trait` and `derive_trait_with_capacity` macros. With this change, the minimum required version will go up to Rust 1.30.

## [0.2.0] - 2018-12-20

* Support Rust 1.27.

## [0.1.1] - 2018-12-15

* Add `std::error::Error` impls for `derive_utils::Error`.

## [0.1.0] - 2018-12-15 - YANKED

Initial release

[Unreleased]: https://github.com/taiki-e/derive_utils/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/taiki-e/derive_utils/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/taiki-e/derive_utils/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/taiki-e/derive_utils/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/taiki-e/derive_utils/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/taiki-e/derive_utils/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/taiki-e/derive_utils/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/taiki-e/derive_utils/compare/v0.6.3...v0.7.0
[0.6.3]: https://github.com/taiki-e/derive_utils/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/taiki-e/derive_utils/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/taiki-e/derive_utils/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/taiki-e/derive_utils/compare/v0.5.4...v0.6.0
[0.5.4]: https://github.com/taiki-e/derive_utils/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/taiki-e/derive_utils/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/taiki-e/derive_utils/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/taiki-e/derive_utils/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/taiki-e/derive_utils/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/taiki-e/derive_utils/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/taiki-e/derive_utils/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/taiki-e/derive_utils/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/taiki-e/derive_utils/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/derive_utils/releases/tag/v0.1.0
