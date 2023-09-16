// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ops;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Fields, Ident, ItemEnum, Result, Type,
};

/// A structure to make trait implementation to enums more efficient.
pub struct EnumData {
    repr: ItemEnum,
    field_types: Vec<Type>,
}

impl EnumData {
    /// Returns an iterator over field types.
    ///
    /// ```text
    /// enum Enum<TypeA, TypeB> {
    ///     VariantA(TypeA),
    ///              ^^^^^
    ///     VariantB(TypeB),
    ///              ^^^^^
    /// }
    /// ```
    pub fn field_types(&self) -> impl ExactSizeIterator<Item = &Type> + Clone {
        self.field_types.iter()
    }

    /// Returns an iterator over variant names.
    ///
    /// ```text
    /// enum Enum<TypeA, TypeB> {
    ///     VariantA(TypeA),
    ///     ^^^^^^^^
    ///     VariantB(TypeB),
    ///     ^^^^^^^^
    /// }
    /// ```
    pub fn variant_idents(&self) -> impl ExactSizeIterator<Item = &Ident> + Clone {
        self.variants.iter().map(|v| &v.ident)
    }
}

impl ops::Deref for EnumData {
    type Target = ItemEnum;

    fn deref(&self) -> &Self::Target {
        &self.repr
    }
}

impl From<EnumData> for ItemEnum {
    fn from(other: EnumData) -> Self {
        other.repr
    }
}

impl Parse for EnumData {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let item: ItemEnum = input.parse()?;

        if item.variants.is_empty() {
            bail!(item, "may not be used on enums without variants");
        }

        let field_types = item.variants.iter().try_fold(
            Vec::with_capacity(item.variants.len()),
            |mut field_types, v| {
                if let Some((_, e)) = &v.discriminant {
                    bail!(e, "may not be used on enums with discriminants");
                }

                if v.fields.is_empty() {
                    bail!(v, "may not be used on enums with variants with zero fields");
                } else if v.fields.len() != 1 {
                    bail!(v, "may not be used on enums with variants with multiple fields");
                }

                match &v.fields {
                    Fields::Unnamed(f) => {
                        field_types.push(f.unnamed.iter().next().unwrap().ty.clone());
                        Ok(field_types)
                    }
                    Fields::Named(_) => {
                        bail!(v, "may not be used on enums with variants with named fields");
                    }
                    Fields::Unit => unreachable!(),
                }
            },
        )?;

        Ok(Self { repr: item, field_types })
    }
}

impl ToTokens for EnumData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.repr.to_tokens(tokens);
    }
}
