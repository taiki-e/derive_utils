#[macro_export]
macro_rules! derive_trait {
    ($data:expr, _, $($trait:tt)*) => {
        $crate::__rt::derive_trait!($data, None, _, $($trait)*)
    };
    ($data:expr, ($($path:tt)*), $($trait:tt)*) => {
        $crate::__rt::derive_trait!($data, None, ($($path)*), $($trait)*)
    };
    ($data:expr, $super:expr, _, $($trait:tt)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($($trait)*))
            .map_err($crate::Error::from)
            .and_then(|trait_: $crate::__rt::ItemTrait| {
                $crate::__rt::derive_trait!(
                    $data,
                    $super,
                    $crate::__rt::path(Some(trait_.ident.clone().into())),
                    trait_
                )
            })
    };
    ($data:expr, $super:expr, ($($path:tt)*), $($trait:tt)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($($path)*))
            .map_err($crate::Error::from)
            .and_then(|path| {
                let trait_: $crate::__rt::ItemTrait = $crate::__rt::parse2($crate::__rt::quote!($($trait)*))?;
                $crate::__rt::derive_trait!(
                    $data,
                    $super,
                    path,
                    trait_
                )
            })
    };
    ($data:expr, $path:expr, $trait:expr) => {
        $crate::__rt::derive_trait!($data, None, $path, $trait)
    };
    ($data:expr, $super:expr, $path:expr, $trait:expr) => {
        $data.impl_trait_with_capacity($trait.items.len(), $path, $super, $trait).map($crate::build)
    };
}

#[macro_export]
macro_rules! quick_derive {
    (@inner $input:expr, |$ast:ident| $expr:expr) => {
        $crate::__rt::parse($input)
            .map_err($crate::Error::from)
            .and_then(|$ast: $crate::__rt::DeriveInput| $expr)
            .unwrap_or_else(|e| $crate::compile_err(&e.to_string()))
            .into()
    };
    ($input:expr, ($($path:tt)*), trait $($trait:tt)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(data, ($($path)*), trait $($trait)*)
            })
        })
    };
    ($input:expr, $super:ident, ($($path:tt)*), trait $($trait:tt)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(
                    data,
                    Some($crate::__rt::Ident::new(
                        stringify!($super), $crate::__rt::Span::call_site()
                    )),
                    ($($path)*),
                    trait $($trait)*
                )
            })
        })
    };
    ($input:expr, trait $($trait:tt)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(data, _, trait $($trait)*)
            })
        })
    };
    ($input:expr, $super:ident, trait $($trait:tt)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(
                    data,
                    Some($crate::__rt::Ident::new(
                        stringify!($super), $crate::__rt::Span::call_site()
                    )),
                    _,
                    trait $($trait)*
                )
            })
        })
    };
}

#[macro_export]
macro_rules! derive_trait_with_capacity {
    ($data:expr, $capacity:expr, _, $($trait:tt)*) => {
        derive_trait_with_capacity!($data, $capacity, None, _, $($trait)*)
    };
    ($data:expr, $capacity:expr, ($($path:tt)*), $($trait:tt)*) => {
        derive_trait_with_capacity!($data, $capacity, None, ($($path)*), $($trait)*)
    };
    ($data:expr, $capacity:expr, $super:expr, _, $($trait:tt)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($($trait)*))
            .map_err($crate::Error::from)
            .and_then(|trait_: $crate::__rt::ItemTrait| {
                derive_trait_with_capacity!($data, $capacity, $super, $crate::__rt::path(Some(trait_.ident.clone().into())), trait_)
            })
    };
    ($data:expr, $capacity:expr, $super:expr, ($($path:tt)*), $($trait:tt)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($($path)*))
            .map_err($crate::Error::from)
            .and_then(|path| {
                derive_trait_with_capacity!($data, $capacity, $super, path, $crate::__rt::parse2($crate::__rt::quote!($($trait)*))?)
            })
    };
    ($data:expr, $capacity:expr, $path:expr, $trait:expr) => {
        derive_trait_with_capacity!($data, $capacity, None, $path, $trait)
    };
    ($data:expr, $capacity:expr, $super:expr, $path:expr, $trait:expr) => {
        $data.impl_trait_with_capacity($capacity, $path, $super, $trait).map($crate::build)
    };
}
