use proc_macro2::TokenStream;
use quote::ToTokens;
use std::ops::Deref;
use syn::{
    parse::{Parse, ParseStream},
    *,
};

/// A structure to make trait implementation to enums more efficient.
pub struct EnumData {
    repr: ItemEnum,
    field_types: Vec<Type>,
}

impl EnumData {
    /// Returns an iterator over field types.
    ///
    /// ```txt
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
    /// ```txt
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

impl Deref for EnumData {
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
            return Err(Error::new(
                item.brace_token.span,
                "may not be used on enums without variants",
            ));
        }

        let field_types = item.variants.iter().try_fold(
            Vec::with_capacity(item.variants.len()),
            |mut field_types, v| {
                if let Some((_, e)) = &v.discriminant {
                    return Err(error!(e, "may not be used on enums with discriminants"));
                }

                match &v.fields {
                    Fields::Unnamed(f) => match f.unnamed.len() {
                        1 => {
                            field_types.push(f.unnamed.iter().next().unwrap().ty.clone());
                            Ok(field_types)
                        }
                        0 => Err(error!(f, "a variant with zero fields is not supported")),
                        _ => Err(error!(f, "a variant with multiple fields is not supported")),
                    },
                    Fields::Unit => Err(error!(v, "may not be used on enums with unit variants")),
                    Fields::Named(_) => {
                        Err(error!(v, "may not be used on enums with struct variants"))
                    }
                }
            },
        )?;

        Ok(Self { repr: item, field_types })
    }
}

impl ToTokens for EnumData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.repr.to_tokens(tokens)
    }
}
