# Unreleased

* Added `quote::ToTokens` bound to `MaybeEnum`.

* Improved error messages.

# 0.7.2 - 2019-05-30

* Improved error messages.

# 0.7.1 - 2019-05-21

* Improved support for arbitrary self type.

* Improved error messages.

# 0.7.0 - 2019-03-13

* Transition to Rust 2018. With this change, the minimum required version will go up to Rust 1.31.

* Improved error messages.

* Removed `Error` and `Result` types. The current `derive_utils` uses [`syn::Error`](https://docs.rs/syn/0.15/syn/struct.Error.html) and [`syn::Result`](https://docs.rs/syn/0.15/syn/parse/type.Result.html) types.

# 0.6.3 - 2019-02-05

* Added `EnumData::new` method.

* Updated minimum `syn` version to 0.15.22.

* Removed dependency on `smallvec`.

# 0.6.2 - 2019-01-30

* Added `Error::to_compile_error` method.

* Hided some undocumented items from the public API.

# 0.6.1 - 2019-01-26

* Improved support for `self: Pin<&Self>` and `self: Pin<&mut Self>`.

* Updated minimum `smallvec` version to 0.6.8.

# 0.6.0 - 2019-01-09

* Removed `"std"` feature and `std_root` function. `derive_utils` can generate accurate code without `"std"` feature.

* Removed deprecated `push_method_pin*` methods. Use `push_method` instead.

* Improved documentation.

# 0.5.4 - 2019-01-03

* Improved documentation.

# 0.5.3 - 2018-12-27

* Improved macro implementations. The trailing comma is supported.

* Improved error messages.

# 0.5.2 - 2018-12-27

* Hided some undocumented items from the public API.

# 0.5.1 - 2018-12-26

* Updated to stable Pin API. `Pin::get_mut_unchecked` renamed to `Pin::get_unchecked_mut` in [stabilization](https://github.com/rust-lang/rust/pull/56939).

# 0.5.0 - 2018-12-23

* Added `quick_derive` macro.

* Removed `derive_trait_with_capacity` macro.

# 0.4.0 - 2018-12-22

* Added support for `self: Pin<&Self>` and `self: Pin<&mut Self>`.

* Allow using the trait name as trait path.

# 0.3.0 - 2018-12-22

* Add `derive_trait` and `derive_trait_with_capacity` macros. With this change, the minimum required version will go up to Rust 1.30.

# 0.2.0 - 2018-12-20

* Support Rust 1.27.

# 0.1.1 - 2018-12-15

* Add `std::error::Error` impls for `derive_utils::Error`.

# 0.1.0 - 2018-12-15

Initial release
