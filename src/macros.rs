#[macro_export]
macro_rules! derive_trait {
    ($data:expr, _, $trait:item $(,)*) => {
        $crate::__rt::derive_trait!($data, None, _, $trait)
    };
    ($data:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__rt::derive_trait!($data, None, ($($path)*), $trait)
    };
    ($data:expr, $super:expr, _, $trait:item $(,)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($trait))
            .map_err($crate::Error::from)
            .and_then(|trait_: $crate::__rt::ItemTrait| {
                let path = $crate::__rt::path(Some(trait_.ident.clone().into()));
                $crate::__rt::derive_trait!($data, $super, path, trait_)
            })
    };
    ($data:expr, $super:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__rt::parse2($crate::__rt::quote!($($path)*))
            .map_err($crate::Error::from)
            .and_then(|path| {
                let trait_: $crate::__rt::ItemTrait = $crate::__rt::parse2($crate::__rt::quote!($trait))?;
                $crate::__rt::derive_trait!($data, $super, path, trait_)
            })
    };
    ($data:expr, $path:expr, $trait:expr $(,)*) => {{
        $crate::__rt::derive_trait!($data, None, $path, $trait)
    }};
    ($data:expr, $super:expr, $path:expr, $trait:expr $(,)*) => {{
        let trait_: $crate::__rt::ItemTrait = $trait;
        $data.impl_trait_with_capacity(trait_.items.len(), $path, $super, trait_).map($crate::build)
    }};
}

/// A macro for to make easy to write `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
#[macro_export]
macro_rules! quick_derive {
    (@inner $input:expr, |$ast:ident| $expr:expr) => {
        $crate::__rt::parse($input)
            .map_err($crate::Error::from)
            .and_then(|$ast: $crate::__rt::DeriveInput| $expr)
            .unwrap_or_else(|e| $crate::compile_err(&e.to_string()))
            .into()
    };
    ($input:expr, ($($path:tt)*), $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(data, ($($path)*), $trait)
            })
        })
    };
    ($input:expr, $super:ident, ($($path:tt)*), $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(
                    data,
                    Some($crate::__rt::ident_call_site(stringify!($super))),
                    ($($path)*),
                    $trait
                )
            })
        })
    };
    ($input:expr, $super:ident, $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(
                    data,
                    Some($crate::__rt::ident_call_site(stringify!($super))),
                    _,
                    $trait
                )
            })
        })
    };
    ($input:expr, $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::from_derive(&ast).and_then(|data| {
                $crate::__rt::derive_trait!(data, _, $trait)
            })
        })
    };
}
