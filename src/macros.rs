#[macro_export]
macro_rules! derive_trait {
    ($data:expr, ($($path:tt)*), $($trait:tt)*) => {
        derive_trait!($data, None, ($($path)*), $($trait)*)
    };
    ($data:expr, $super:expr, ($($path:tt)*), $($trait:tt)*) => {
        $crate::syn::parse2($crate::quote!($($path)*))
            .map_err(|e| e.into())
            .and_then(|path| derive_trait!($data, $super, path, $crate::syn::parse2($crate::quote!($($trait)*))?))
    };
    ($data:expr, $path:expr, $trait:expr) => {
        derive_trait!($data, None, $path, $trait)
    };
    ($data:expr, $super:expr, $path:expr, $trait:expr) => {
        $data.make_impl_trait($path, $super, $trait).map($crate::build)
    };
}

#[macro_export]
macro_rules! derive_trait_with_capacity {
    ($data:expr, $capacity:expr, ($($path:tt)*), $($trait:tt)*) => {
        derive_trait_with_capacity!($data, $capacity, None, ($($path)*), $($trait)*)
    };
    ($data:expr, $capacity:expr, $super:expr, ($($path:tt)*), $($trait:tt)*) => {
        $crate::syn::parse2($crate::quote!($($path)*))
            .map_err(|e| e.into())
            .and_then(|path| derive_trait_with_capacity!($data, $capacity, $super, path, $crate::syn::parse2($crate::quote!($($trait)*))?))
    };
    ($data:expr, $capacity:expr, $path:expr, $trait:expr) => {
        derive_trait_with_capacity!($data, $capacity, None, $path, $trait)
    };
    ($data:expr, $capacity:expr, $super:expr, $path:expr, $trait:expr) => {
        $data.impl_trait_with_capacity($capacity, $path, $super, $trait).map($crate::build)
    };
}
