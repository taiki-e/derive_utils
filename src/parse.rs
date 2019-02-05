use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, *};

use crate::common::*;
use crate::error::Result;

macro_rules! parse_quote {
    ($($tt:tt)*) => {
        $crate::syn::parse2($crate::quote::quote!($($tt)*))
    };
}

/// The elements that compose enums.
pub struct EnumElements<'a> {
    pub attrs: &'a [Attribute],
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub variants: &'a Punctuated<Variant, Comma>,
}

/// A type that might be enums.
pub trait MaybeEnum {
    /// Get the elements that compose enums.
    fn elements(&self) -> Result<EnumElements<'_>>;
}

impl<'a, E: ?Sized + MaybeEnum> MaybeEnum for &'a mut E {
    fn elements(&self) -> Result<EnumElements<'_>> {
        (**self).elements()
    }
}
impl<'a, E: ?Sized + MaybeEnum> MaybeEnum for &'a E {
    fn elements(&self) -> Result<EnumElements<'_>> {
        (**self).elements()
    }
}
impl MaybeEnum for ItemEnum {
    fn elements(&self) -> Result<EnumElements<'_>> {
        Ok(EnumElements {
            attrs: &self.attrs,
            ident: &self.ident,
            generics: &self.generics,
            variants: &self.variants,
        })
    }
}
impl MaybeEnum for Item {
    fn elements(&self) -> Result<EnumElements<'_>> {
        match self {
            Item::Enum(item) => MaybeEnum::elements(item),
            _ => Err("may only be used on enums")?,
        }
    }
}
impl MaybeEnum for Stmt {
    fn elements(&self) -> Result<EnumElements<'_>> {
        match self {
            Stmt::Item(Item::Enum(item)) => MaybeEnum::elements(item),
            _ => Err("may only be used on enums")?,
        }
    }
}
impl MaybeEnum for DeriveInput {
    fn elements(&self) -> Result<EnumElements<'_>> {
        match &self.data {
            Data::Enum(data) => Ok(EnumElements {
                attrs: &self.attrs,
                ident: &self.ident,
                generics: &self.generics,
                variants: &data.variants,
            }),
            Data::Struct(_) => Err("cannot be implemented for structs")?,
            Data::Union(_) => Err("cannot be implemented for unions")?,
        }
    }
}

/// A structure to make trait implementation to enums more efficient.
pub struct EnumData {
    ident: Ident,
    generics: Generics,
    variants: Vec<Ident>,
    fields: Vec<Type>,
}

impl EnumData {
    /// Constructs a new `EnumData`.
    pub fn new<E: MaybeEnum>(maybe_enum: &E) -> Result<Self> {
        let elements = MaybeEnum::elements(maybe_enum)?;
        parse_variants(elements.variants).map(|(variants, fields)| Self {
            ident: elements.ident.clone(),
            generics: elements.generics.clone(),
            variants,
            fields,
        })
    }

    /// Constructs a new `EnumData` from `&ItemEnum`.
    pub fn from_item(item: &ItemEnum) -> Result<Self> {
        Self::new(item)
    }

    /// Constructs a new `EnumData` from `&DeriveInput`.
    pub fn from_derive(ast: &DeriveInput) -> Result<Self> {
        Self::new(ast)
    }

    /// Constructs a new `EnumImpl`.
    pub fn make_impl<'a>(&'a self) -> Result<EnumImpl<'a>> {
        EnumImpl::new(self, Vec::new())
    }

    /// Constructs a new `EnumImpl` with the specified capacity..
    pub fn impl_with_capacity<'a>(&'a self, capacity: usize) -> Result<EnumImpl<'a>> {
        EnumImpl::new(self, Vec::with_capacity(capacity))
    }

    /// Constructs a new `EnumImpl` from `ItemTrait`.
    ///
    /// `TraitItem::Method` that has the first argument other than the following is error:
    /// - `&self`
    /// - `&mut self`
    /// - `self`
    /// - `mut self`
    /// - `self: Pin<&Self>`
    /// - `self: Pin<&mut Self>`
    ///
    /// The following items are ignored:
    /// - Generic associated types (GAT) (`TraitItem::Method` that has generics)
    /// - `TraitItem::Const`
    /// - `TraitItem::Macro`
    /// - `TraitItem::Verbatim`
    pub fn make_impl_trait<'a, I>(
        &'a self,
        trait_path: Path,
        supertraits_types: I,
        item: ItemTrait,
    ) -> Result<EnumImpl<'a>>
    where
        I: IntoIterator<Item = Ident>,
        I::IntoIter: ExactSizeIterator,
    {
        EnumImpl::from_trait(self, trait_path, Vec::new(), item, supertraits_types)
    }

    /// Constructs a new `EnumImpl` from `ItemTrait` with the specified capacity.
    ///
    /// See [`EnumData::make_impl_trait`] for supported item types.
    ///
    /// [`EnumData::make_impl_trait`]: ./struct.EnumData.html#method.make_impl_trait
    pub fn impl_trait_with_capacity<'a, I>(
        &'a self,
        capacity: usize,
        trait_path: Path,
        supertraits_types: I,
        item: ItemTrait,
    ) -> Result<EnumImpl<'a>>
    where
        I: IntoIterator<Item = Ident>,
        I::IntoIter: ExactSizeIterator,
    {
        EnumImpl::from_trait(
            self,
            trait_path,
            Vec::with_capacity(capacity),
            item,
            supertraits_types,
        )
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    pub fn generics(&self) -> &Generics {
        &self.generics
    }
    pub fn variants(&self) -> &[Ident] {
        &self.variants
    }
    pub fn fields(&self) -> &[Type] {
        &self.fields
    }
}

#[doc(hidden)]
pub struct Trait {
    /// `AsRef`
    path: Path,
    /// `AsRef<T>`
    ty: Path,
}

impl Trait {
    #[doc(hidden)]
    pub fn new(path: Path, ty: Path) -> Self {
        Self { path, ty }
    }
}

/// A builder for implementing traits for enums.
pub struct EnumImpl<'a> {
    data: &'a EnumData,
    defaultness: bool,
    unsafety: bool,
    generics: Generics,
    trait_: Option<Trait>,
    self_ty: Box<Type>,
    items: Vec<ImplItem>,
    unsafe_code: bool,
}

#[doc(hidden)]
pub fn build(impls: EnumImpl<'_>) -> TokenStream {
    impls.build()
}

#[doc(hidden)]
pub fn build_item(impls: EnumImpl<'_>) -> ItemImpl {
    impls.build_item()
}

impl<'a> EnumImpl<'a> {
    fn new(data: &'a EnumData, items: Vec<ImplItem>) -> Result<Self> {
        let ident = &data.ident;
        let ty_generics = &data.generics;
        parse_quote!(#ident #ty_generics)
            .map(|self_ty| Self {
                data,
                defaultness: false,
                unsafety: false,
                generics: data.generics.clone(),
                trait_: None,
                self_ty: Box::new(self_ty),
                items,
                unsafe_code: false,
            })
            .map_err(|e| e.into())
    }

    #[doc(hidden)]
    pub fn trait_(&mut self) -> &mut Option<Trait> {
        &mut self.trait_
    }
    pub fn self_ty(&mut self) -> &mut Type {
        &mut *self.self_ty
    }

    pub fn push_generic_param(&mut self, param: GenericParam) {
        self.generics.params.push(param);
    }
    pub fn push_generic_param_ident(&mut self, ident: Ident) {
        self.push_generic_param(param_ident(Vec::with_capacity(0), ident));
    }

    /// Appends a predicate to the back of `where`-clause.
    pub fn push_where_predicate(&mut self, predicate: WherePredicate) {
        self.generics.make_where_clause().predicates.push(predicate);
    }

    /// Appends an item to impl items.
    pub fn push_item(&mut self, item: ImplItem) {
        self.items.push(item);
    }

    fn arms<F: FnMut(&Ident) -> TokenStream>(&self, f: F) -> TokenStream {
        let arms = self.data.variants.iter().map(f);
        quote!(#(#arms,)*)
    }

    fn trait_path(&self) -> Option<&Path> {
        self.trait_.as_ref().map(|t| &t.path)
    }

    /// Appends a method from `TraitItemMethod` to impl items.
    ///
    /// A method that has the first argument other than the following is error:
    /// - `&self`
    /// - `&mut self`
    /// - `self`
    /// - `mut self`
    /// - `self: Pin<&Self>`
    /// - `self: Pin<&mut Self>`
    pub fn push_method(&mut self, item: TraitItemMethod) -> Result<()> {
        let method = {
            let mut args = item.sig.decl.inputs.iter();
            let self_ty = SelfTypes::parse(args.next())?;
            let args = &args
                .map(|arg| match arg {
                    FnArg::Captured(arg) => Ok(&arg.pat),
                    _ => Err("unsupported arguments type")?,
                })
                .collect::<Result<Vec<_>>>()?;

            let method = &item.sig.ident;
            let ident = &self.data.ident;
            match self_ty {
                SelfTypes::None => {
                    let trait_ = self.trait_path();
                    let arms = if trait_.is_none() {
                        self.arms(|v| quote!(#ident::#v(x) => x.#method(#(#args),*)))
                    } else {
                        self.arms(|v| quote!(#ident::#v(x) => #trait_::#method(x #(,#args)*)))
                    };
                    parse_quote!(match self { #arms })?
                }

                SelfTypes::Pin(self_pin, pin) => {
                    self.unsafe_code = true;
                    let trait_ = self.trait_path();
                    let arms = if trait_.is_none() {
                        self.arms(
                            |v| quote!(#ident::#v(x) => #pin::new_unchecked(x).#method(#(#args),*)),
                        )
                    } else {
                        self.arms(|v| quote!(#ident::#v(x) => #trait_::#method(#pin::new_unchecked(x) #(,#args)*)))
                    };

                    match self_pin {
                        SelfPin::Ref => {
                            if self.unsafety || item.sig.unsafety.is_some() {
                                parse_quote!(match #pin::get_ref(self) { #arms })?
                            } else {
                                parse_quote!(unsafe { match #pin::get_ref(self) { #arms } })?
                            }
                        }
                        SelfPin::Mut => {
                            if self.unsafety || item.sig.unsafety.is_some() {
                                parse_quote!(match #pin::get_unchecked_mut(self) { #arms })?
                            } else {
                                parse_quote!(unsafe { match #pin::get_unchecked_mut(self) { #arms } })?
                            }
                        }
                    }
                }
            }
        };

        self.push_item(ImplItem::Method(method_from_method(
            item,
            block(vec![Stmt::Expr(method)]),
        )));

        Ok(())
    }

    /// Appends items from `ItemTrait` to impl items.
    ///
    /// See [`EnumData::make_impl_trait`] for supported item types.
    ///
    /// [`EnumData::make_impl_trait`]: ./struct.EnumData.html#method.make_impl_trait
    pub fn append_items_from_trait(&mut self, item: ItemTrait) -> Result<()> {
        let fst = self.data.fields.iter().next();
        item.items.into_iter().try_for_each(|item| match item {
            TraitItem::Const(_) | TraitItem::Macro(_) | TraitItem::Verbatim(_) => Ok(()),

            TraitItem::Type(TraitItemType {
                ident, generics, ..
            }) => {
                // Generic associated types (GAT) are not supported
                if generics.params.is_empty() {
                    {
                        let trait_ = self.trait_.as_ref().map(|t| &t.ty);
                        parse_quote!(type #ident = <#fst as #trait_>::#ident;)
                    }
                    .map(|ty| self.push_item(ImplItem::Type(ty)))?;
                }
                Ok(())
            }

            TraitItem::Method(method) => self.push_method(method),
        })
    }

    fn from_trait<I>(
        data: &'a EnumData,
        path: Path,
        items: Vec<ImplItem>,
        mut item: ItemTrait,
        supertraits_types: I,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Ident>,
        I::IntoIter: ExactSizeIterator,
    {
        fn generics_params<'a, I>(iter: I) -> impl Iterator<Item = Cow<'a, GenericParam>>
        where
            I: Iterator<Item = &'a GenericParam>,
        {
            iter.map(|param| match param {
                GenericParam::Type(ty) => {
                    Cow::Owned(param_ident(ty.attrs.clone(), ty.ident.clone()))
                }
                param => Cow::Borrowed(param),
            })
        }

        let mut generics = data.generics.clone();
        let trait_ = {
            if item.generics.params.is_empty() {
                path.clone()
            } else {
                let generics = generics_params(item.generics.params.iter());
                parse_quote!(#path<#(#generics),*>)?
            }
        };

        {
            let fst = data.fields.iter().next();
            let mut types: Vec<_> = item
                .items
                .iter()
                .filter_map(|item| match item {
                    TraitItem::Type(ty) => Some((false, Cow::Borrowed(&ty.ident))),
                    _ => None,
                })
                .collect();

            let supertraits_types = supertraits_types.into_iter();
            if supertraits_types.len() > 0 {
                if let Some(TypeParamBound::Trait(_)) = item.supertraits.iter().next() {
                    types.extend(supertraits_types.map(|ident| (true, Cow::Owned(ident))));
                }
            }

            let where_clause = &mut generics.make_where_clause().predicates;
            where_clause.push(parse_quote!(#fst: #trait_)?);
            data.fields
                .iter()
                .skip(1)
                .map(|variant| {
                    if types.is_empty() {
                        parse_quote!(#variant: #trait_)
                    } else {
                        let types = types.iter().map(|(supertraits, ident)| {
                            match item.supertraits.iter().next() {
                                Some(TypeParamBound::Trait(trait_)) if *supertraits => {
                                    quote!(#ident = <#fst as #trait_>::#ident)
                                }
                                _ => quote!(#ident = <#fst as #trait_>::#ident),
                            }
                        });
                        if item.generics.params.is_empty() {
                            parse_quote!(#variant: #path<#(#types),*>)
                        } else {
                            let generics = generics_params(item.generics.params.iter());
                            parse_quote!(#variant: #path<#(#generics),*, #(#types),*>)
                        }
                    }
                })
                .try_for_each(|res| res.map(|f| where_clause.push(f)))?;
        }

        if !item.generics.params.is_empty() {
            mem::replace(&mut item.generics.params, Punctuated::new())
                .into_iter()
                .for_each(|param| generics.params.push(param));
        }

        if let Some(old) = item.generics.where_clause.as_mut() {
            if !old.predicates.is_empty() {
                let where_clause = &mut generics.make_where_clause().predicates;
                mem::replace(&mut old.predicates, Punctuated::new())
                    .into_iter()
                    .for_each(|param| where_clause.push(param));
            }
        }

        let ident = &data.ident;
        let ty_generics = &data.generics;
        let mut impls = parse_quote!(#ident #ty_generics).map(|self_ty| Self {
            data,
            defaultness: false,
            unsafety: item.unsafety.is_some(),
            generics,
            trait_: Some(Trait::new(path, trait_)),
            self_ty: Box::new(self_ty),
            items,
            unsafe_code: false,
        })?;

        impls.append_items_from_trait(item).map(|_| impls)
    }

    pub fn build(self) -> TokenStream {
        self.build_item().into_token_stream()
    }

    pub fn build_item(self) -> ItemImpl {
        ItemImpl {
            attrs: if self.unsafe_code {
                vec![Attribute {
                    pound_token: default(),
                    style: AttrStyle::Outer,
                    bracket_token: default(),
                    path: path(Some(ident_call_site("allow").into())),
                    tts: quote!((unsafe_code)),
                }]
            } else {
                Vec::with_capacity(0)
            },
            defaultness: if self.defaultness {
                Some(default())
            } else {
                None
            },
            unsafety: if self.unsafety { Some(default()) } else { None },
            impl_token: default(),
            generics: self.generics,
            trait_: self.trait_.map(|Trait { ty, .. }| (None, ty, default())),
            self_ty: self.self_ty,
            brace_token: default(),
            items: self.items,
        }
    }
}

fn method_from_method(method: TraitItemMethod, block: Block) -> ImplItemMethod {
    ImplItemMethod {
        attrs: method.attrs,
        vis: Visibility::Inherited,
        defaultness: None,
        sig: method.sig,
        block,
    }
}

enum SelfTypes {
    /// `&self`, `&mut self`, `self` or `mut self`
    None,
    /// `self: Pin<&Self>` or `self: Pin<&mut Self>`
    Pin(SelfPin, Path),
}

enum SelfPin {
    /// `self: Pin<&Self>`
    Ref,
    /// `self: Pin<&mut Self>`
    Mut,
}

impl SelfTypes {
    fn parse(arg: Option<&FnArg>) -> Result<Self> {
        fn remove_last_path_arg(path: &Path) -> Path {
            let mut path = path.clone();
            path.segments.last_mut().unwrap().into_value().arguments = PathArguments::None;
            path
        }

        const ERR: &str = "unsupported first argument type";

        match arg {
            Some(FnArg::SelfRef(_)) | Some(FnArg::SelfValue(_)) => Ok(SelfTypes::None),

            Some(FnArg::Captured(ArgCaptured {
                pat: Pat::Ident(PatIdent { ident, .. }),
                ty: Type::Path(TypePath { qself: None, path }),
                ..
            })) if ident == "self" => match &*path.segments[path.segments.len() - 1]
                .clone()
                .into_token_stream()
                .to_string()
            {
                "Pin < & Self >" => Ok(SelfTypes::Pin(SelfPin::Ref, remove_last_path_arg(path))),
                "Pin < & mut Self >" => {
                    Ok(SelfTypes::Pin(SelfPin::Mut, remove_last_path_arg(path)))
                }
                _ => Err(ERR)?,
            },

            _ => Err(ERR)?,
        }
    }
}

fn parse_variants(
    punctuated: &Punctuated<Variant, token::Comma>,
) -> Result<(Vec<Ident>, Vec<Type>)> {
    #[inline(never)]
    fn err(msg: &str) -> Result<()> {
        Err(format!("cannot be implemented for enums with {}", msg).into())
    }

    if punctuated.len() < 2 {
        err("less than two variants")?;
    }

    let mut variants = Vec::with_capacity(punctuated.len());
    let mut fields = Vec::with_capacity(punctuated.len());
    punctuated
        .iter()
        .try_for_each(|v| {
            if v.discriminant.is_some() {
                err("discriminants")?;
            }

            match &v.fields {
                Fields::Unnamed(f) => match f.unnamed.len() {
                    1 => fields.push(f.unnamed.iter().next().unwrap().ty.clone()),
                    0 => err("zero fields")?,
                    _ => err("multiple fields")?,
                },
                Fields::Unit => err("with units")?,
                Fields::Named(_) => err("named fields")?,
            }

            variants.push(v.ident.clone());
            Ok(())
        })
        .map(|_| (variants, fields))
}
