use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, *};

// =================================================================================================
// Utilities

macro_rules! parse_quote {
    ($($tt:tt)*) => {
        syn::parse2(quote::quote!($($tt)*))
    };
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        return Err(syn::Error::new_spanned(&$span, $msg))
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}

fn param_ident(attrs: Vec<Attribute>, ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs,
        ident,
        colon_token: None,
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    })
}

// =================================================================================================
// EnumElements

/// The elements that compose enums.
pub struct EnumElements<'a> {
    pub attrs: &'a [Attribute],
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub variants: &'a Punctuated<Variant, token::Comma>,
}

/// A type that might be enums.
pub trait MaybeEnum: ToTokens {
    /// Get the elements that compose enums.
    fn elements(&self) -> Result<EnumElements<'_>>;
}

impl<E: ?Sized + MaybeEnum> MaybeEnum for &E {
    fn elements(&self) -> Result<EnumElements<'_>> {
        (**self).elements()
    }
}

impl<E: ?Sized + MaybeEnum> MaybeEnum for &mut E {
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
            _ => error!(self, "may only be used on enums"),
        }
    }
}

impl MaybeEnum for Stmt {
    fn elements(&self) -> Result<EnumElements<'_>> {
        match self {
            Stmt::Item(Item::Enum(item)) => MaybeEnum::elements(item),
            _ => error!(self, "may only be used on enums"),
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
            Data::Struct(_) => error!(self, "cannot be implemented for structs"),
            Data::Union(_) => error!(self, "cannot be implemented for unions"),
        }
    }
}

// =================================================================================================
// EnumData

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
        if elements.variants.len() < 2 {
            error!(maybe_enum, "cannot be implemented for enums with less than two variants");
        }

        parse_variants(elements.variants).map(|(variants, fields)| Self {
            ident: elements.ident.clone(),
            generics: elements.generics.clone(),
            variants,
            fields,
        })
    }

    /// Constructs a new `EnumImpl`.
    pub fn make_impl(&self) -> Result<EnumImpl<'_>> {
        EnumImpl::new(self, Vec::new())
    }

    /// Constructs a new `EnumImpl` with the specified capacity..
    pub fn impl_with_capacity(&self, capacity: usize) -> Result<EnumImpl<'_>> {
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
    pub fn make_impl_trait<I>(
        &self,
        trait_path: Path,
        supertraits_types: I,
        item: ItemTrait,
    ) -> Result<EnumImpl<'_>>
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
    pub fn impl_trait_with_capacity<I>(
        &self,
        capacity: usize,
        trait_path: Path,
        supertraits_types: I,
        item: ItemTrait,
    ) -> Result<EnumImpl<'_>>
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

fn parse_variants(variants: &Punctuated<Variant, token::Comma>) -> Result<(Vec<Ident>, Vec<Type>)> {
    variants.iter().try_fold(
        (Vec::with_capacity(variants.len()), Vec::with_capacity(variants.len())),
        |(mut variants, mut fields), v| {
            if let Some((_, e)) = &v.discriminant {
                error!(e, "an enum with discriminants is not supported")
            }

            match &v.fields {
                Fields::Unnamed(f) => match f.unnamed.len() {
                    1 => fields.push(f.unnamed.iter().next().unwrap().ty.clone()),
                    0 => error!(v.fields, "a variant with zero fields is not supported"),
                    _ => error!(v.fields, "a variant with multiple fields is not supported"),
                },
                Fields::Unit => error!(v, "an enum with units variant is not supported"),
                Fields::Named(_) => error!(v, "an enum with named fields variant is not supported"),
            }

            variants.push(v.ident.clone());
            Ok((variants, fields))
        },
    )
}

// =================================================================================================
// EnumImpl

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
        parse_quote!(#ident #ty_generics).map(|self_ty| Self {
            data,
            defaultness: false,
            unsafety: false,
            generics: data.generics.clone(),
            trait_: None,
            self_ty: Box::new(self_ty),
            items,
            unsafe_code: false,
        })
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
        self.push_generic_param(param_ident(Vec::new(), ident));
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
        let self_ty = SelfType::parse(item.sig.decl.inputs.iter().next())?;
        let mut args = Vec::with_capacity(item.sig.decl.inputs.len());
        item.sig.decl.inputs.iter().skip(1).try_for_each(|arg| match arg {
            FnArg::Captured(arg) => {
                args.push(&arg.pat);
                Ok(())
            }
            _ => error!(arg, "unsupported arguments type"),
        })?;
        let args = &args;

        let method = &item.sig.ident;
        let ident = &self.data.ident;
        let method = match self_ty {
            SelfType::None => {
                let trait_ = self.trait_path();
                let arms = if trait_.is_none() {
                    self.arms(|v| quote!(#ident::#v(x) => x.#method(#(#args),*)))
                } else {
                    self.arms(|v| quote!(#ident::#v(x) => #trait_::#method(x #(,#args)*)))
                };
                parse_quote!(match self { #arms })
            }

            SelfType::Pin(mode, pin) => {
                self.unsafe_code = true;
                let trait_ = self.trait_path();
                let arms = if trait_.is_none() {
                    self.arms(
                        |v| quote!(#ident::#v(x) => #pin::new_unchecked(x).#method(#(#args),*)),
                    )
                } else {
                    self.arms(|v| quote!(#ident::#v(x) => #trait_::#method(#pin::new_unchecked(x) #(,#args)*)))
                };

                match mode {
                    CaptureMode::Ref { mutability: false } => {
                        if self.unsafety || item.sig.unsafety.is_some() {
                            parse_quote!(match #pin::get_ref(self) { #arms })
                        } else {
                            parse_quote!(unsafe { match #pin::get_ref(self) { #arms } })
                        }
                    }
                    CaptureMode::Ref { mutability: true } => {
                        if self.unsafety || item.sig.unsafety.is_some() {
                            parse_quote!(match #pin::get_unchecked_mut(self) { #arms })
                        } else {
                            parse_quote!(unsafe { match #pin::get_unchecked_mut(self) { #arms } })
                        }
                    }
                }
            }
        };

        method.map(|method| {
            self.push_item(ImplItem::Method(ImplItemMethod {
                attrs: item.attrs,
                vis: Visibility::Inherited,
                defaultness: None,
                sig: item.sig,
                block: Block {
                    brace_token: token::Brace::default(),
                    stmts: vec![Stmt::Expr(method)],
                },
            }))
        })
    }

    /// Appends items from an iterator of `TraitItem` to impl items.
    ///
    /// See [`EnumData::make_impl_trait`] for supported item types.
    ///
    /// [`EnumData::make_impl_trait`]: ./struct.EnumData.html#method.make_impl_trait
    pub fn append_items<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = TraitItem>,
    {
        let fst = self.data.fields.iter().next();
        items.into_iter().try_for_each(|item| match item {
            TraitItem::Const(_) | TraitItem::Macro(_) | TraitItem::Verbatim(_) => Ok(()),

            // The TraitItemType::generics field (Generic associated types (GAT)) are not supported
            TraitItem::Type(TraitItemType { ident, .. }) => {
                let trait_ = self.trait_.as_ref().map(|t| &t.ty);
                parse_quote!(type #ident = <#fst as #trait_>::#ident;)
                    .map(|ty| self.push_item(ImplItem::Type(ty)))
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
        #[allow(single_use_lifetimes)]
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

        if !item.generics.params.is_empty() {
            generics.params.extend(mem::replace(&mut item.generics.params, Punctuated::new()));
        }

        if let Some(old) = item.generics.where_clause.as_mut() {
            if !old.predicates.is_empty() {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(mem::replace(&mut old.predicates, Punctuated::new()));
            }
        }

        let ident = &data.ident;
        let ty_generics = &data.generics;
        parse_quote!(#ident #ty_generics)
            .map(|self_ty| Self {
                data,
                defaultness: false,
                unsafety: item.unsafety.is_some(),
                generics,
                trait_: Some(Trait::new(path, trait_)),
                self_ty: Box::new(self_ty),
                items,
                unsafe_code: false,
            })
            .and_then(|mut impls| impls.append_items(item.items).map(|_| impls))
    }

    pub fn build(self) -> TokenStream {
        self.build_item().into_token_stream()
    }

    pub fn build_item(self) -> ItemImpl {
        ItemImpl {
            attrs: if self.unsafe_code {
                vec![syn::parse_quote!(#[allow(unsafe_code)])]
            } else {
                Vec::new()
            },
            defaultness: if self.defaultness { Some(token::Default::default()) } else { None },
            unsafety: if self.unsafety { Some(token::Unsafe::default()) } else { None },
            impl_token: token::Impl::default(),
            generics: self.generics,
            trait_: self.trait_.map(|Trait { ty, .. }| (None, ty, token::For::default())),
            self_ty: self.self_ty,
            brace_token: token::Brace::default(),
            items: self.items,
        }
    }
}

enum SelfType {
    /// `&self`, `&mut self`, `self` or `mut self`
    None,
    /// `self: Pin<&Self>` or `self: Pin<&mut Self>`
    Pin(CaptureMode, Path),
}

enum CaptureMode {
    // `self: Type<Self>`
    // Value,
    /// `self: Type<&Self>` or `self: Type<&mut Self>`
    Ref { mutability: bool },
}

impl SelfType {
    fn parse(arg: Option<&FnArg>) -> Result<Self> {
        fn remove_last_path_args(mut path: Path) -> Path {
            path.segments.last_mut().unwrap().into_value().arguments = PathArguments::None;
            path
        }

        fn arg_to_string(arg: Option<&FnArg>) -> String {
            arg.unwrap().clone().into_token_stream().to_string()
        }

        match arg {
            Some(FnArg::SelfRef(_)) | Some(FnArg::SelfValue(_)) => Ok(SelfType::None),

            Some(FnArg::Captured(ArgCaptured {
                pat: Pat::Ident(PatIdent { ident, .. }),
                ty: Type::Path(TypePath { qself: None, path }),
                ..
            })) if ident == "self" => {
                let ty = &path.segments[path.segments.len() - 1];
                if let PathArguments::AngleBracketed(args) = &ty.arguments {
                    if args.args.len() == 1 && ty.ident == "Pin" {
                        if let GenericArgument::Type(Type::Reference(TypeReference {
                            mutability,
                            elem,
                            ..
                        })) = &args.args[0]
                        {
                            match &**elem {
                                Type::Path(TypePath { path: p, qself: None })
                                    if p.is_ident("Self") =>
                                {
                                    return Ok(SelfType::Pin(
                                        CaptureMode::Ref { mutability: mutability.is_some() },
                                        remove_last_path_args(path.clone()),
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                }

                error!(arg, "unsupported first argument type: {}", arg_to_string(arg))
            }

            Some(_) => error!(
                arg,
                "methods that do not have `self` argument are not supported: {}",
                arg_to_string(arg)
            ),
            None => error!(arg, "methods without arguments are not supported"),
        }
    }
}
