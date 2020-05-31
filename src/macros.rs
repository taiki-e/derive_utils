#[doc(hidden)]
#[macro_export]
macro_rules! derive_trait_internal {
    ($data:expr, _, $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, None, _, $trait)
    };
    ($data:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, None, ($($path)*), $trait)
    };
    ($data:expr, $super:expr, _, $trait:item $(,)*) => {
        $crate::__private::parse2($crate::__private::quote!($trait))
            .and_then(|trait_: $crate::__private::ItemTrait| {
                let path = $crate::__private::Path {
                    leading_colon: None,
                    segments: Some($crate::__private::PathSegment::from(trait_.ident.clone()))
                        .into_iter()
                        .collect(),
                };
                $crate::__private::derive_trait_internal!($data, $super, path, trait_)
            })
    };
    ($data:expr, $super:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__private::parse2($crate::__private::quote!($($path)*))
            .and_then(|path| {
                let trait_: $crate::__private::ItemTrait = $crate::__private::parse2($crate::__private::quote!($trait))?;
                $crate::__private::derive_trait_internal!($data, $super, path, trait_)
            })
    };
    ($data:expr, $path:expr, $trait:expr $(,)*) => {
        $crate::__private::derive_trait_internal!($data, None, $path, $trait)
    };
    ($data:expr, $super:expr, $path:expr, $trait:expr $(,)*) => {{
        let trait_: $crate::__private::ItemTrait = $trait;
        $data
            .impl_trait_with_capacity(trait_.items.len(), $path, $super, trait_)
            .map($crate::__private::build_item)
    }};
}

#[macro_export]
macro_rules! derive_trait {
    ($data:expr, _, $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, _, $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
    ($data:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, ($($path)*), $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
    ($data:expr, $super:expr, _, $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, $super, _, $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
    ($data:expr, $super:expr, ($($path:tt)*), $trait:item $(,)*) => {
        $crate::__private::derive_trait_internal!($data, $super, ($($path)*), $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
    ($data:expr, $path:expr, $trait:expr $(,)*) => {
        $crate::__private::derive_trait_internal!($data, $path, $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
    ($data:expr, $super:expr, $path:expr, $trait:expr $(,)*) => {
        $crate::__private::derive_trait_internal!($data, $super, $path, $trait)
            .map($crate::__private::ToTokens::into_token_stream)
    };
}

/// A macro for to make easy to write `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
#[macro_export]
macro_rules! quick_derive {
    (@inner $input:expr, |$ast:ident| $expr:expr) => {
        $crate::__private::parse($input)
            .and_then(|$ast: $crate::__private::DeriveInput| $expr)
            .unwrap_or_else(|e| e.to_compile_error())
            .into()
    };
    ($input:expr, ($($path:tt)*), $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::new::<$crate::__private::DeriveInput>(&ast).and_then(|data| {
                $crate::__private::derive_trait!(data, ($($path)*), $trait)
            })
        })
    };
    ($input:expr, $super:ident, ($($path:tt)*), $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::new::<$crate::__private::DeriveInput>(&ast).and_then(|data| {
                $crate::__private::derive_trait!(
                    data,
                    Some($crate::__private::format_ident!(stringify!($super))),
                    ($($path)*),
                    $trait
                )
            })
        })
    };
    ($input:expr, $super:ident, $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::new::<$crate::__private::DeriveInput>(&ast).and_then(|data| {
                $crate::__private::derive_trait!(
                    data,
                    Some($crate::__private::format_ident!(stringify!($super))),
                    _,
                    $trait
                )
            })
        })
    };
    ($input:expr, $trait:item $(,)*) => {
        quick_derive!(@inner $input, |ast| {
            $crate::EnumData::new::<$crate::__private::DeriveInput>(&ast).and_then(|data| {
                $crate::__private::derive_trait!(data, _, $trait)
            })
        })
    };
}
