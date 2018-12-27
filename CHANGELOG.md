# Unreleased

# 0.5.3 - 2018-12-27

* Improve macro implementations<br>
  The trailing comma is supported.

* Improve error messages

# 0.5.2 - 2018-12-27

* Hide some undocumented items from the public API

# 0.5.1 - 2018-12-26

* Fix Pin's implementation<br>
  `Pin::get_mut_unchecked` renamed to `Pin::get_unchecked_mut` in [stabilization](https://github.com/rust-lang/rust/pull/56939).

# 0.5.0 - 2018-12-23

* Add quick_derive macro

* Remove derive_trait_with_capacity macro

# 0.4.0 - 2018-12-22

* Support `self: Pin<&Self>` and `self: Pin<&mut Self>`

* Allow using the trait name as trait path

# 0.3.0 - 2018-12-22

* Add `derive_trait` and `derive_trait_with_capacity` macros<br>
  With this change, the minimum version will go up to 1.30.

# 0.2.0 - 2018-12-20

* Support Rust 1.27

# 0.1.1 - 2018-12-15

* Add `std::error::Error` impls for `derive_utils::Error`

# 0.1.0 - 2018-12-15

Initial release
