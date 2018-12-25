use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, *};

use common::*;
use error::Result;

pub struct EnumData {
    ident: Ident,
    generics: Generics,
    variants: Stack<Ident>,
    fields: Stack<Type>,
}

impl EnumData {
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

pub struct Trait {
    /// `AsRef`
    path: Path,
    /// `AsRef<T>`
    ty: Path,
}

impl Trait {
    pub fn new(path: Path, ty: Path) -> Self {
        Trait { path, ty }
    }
}

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

pub fn build(impls: EnumImpl<'_>) -> TokenStream {
    impls.build()
}

impl<'a> EnumImpl<'a> {
    fn new(data: &'a EnumData, items: Vec<ImplItem>) -> Result<Self> {
        let ident = &data.ident;
        let (_, ty_generics, _) = data.generics.split_for_impl();
        syn::parse2(quote!(#ident #ty_generics))
            .map(|self_ty| EnumImpl {
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
        self.push_generic_param(param_ident(ident));
    }

    pub fn push_where_predicate(&mut self, predicate: WherePredicate) {
        self.generics.make_where_clause().predicates.push(predicate);
    }

    pub fn push_item(&mut self, item: ImplItem) {
        self.items.push(item);
    }

    #[deprecated(note = "Use `push_method` instead")]
    pub fn push_method_pin_ref(&mut self, item: TraitItemMethod) -> Result<()> {
        self.push_method(item)
    }
    #[deprecated(note = "Use `push_method` instead")]
    pub fn push_method_pin_mut(&mut self, item: TraitItemMethod) -> Result<()> {
        self.push_method(item)
    }

    fn arms<F: FnMut(&Ident) -> TokenStream>(&self, f: F) -> TokenStream {
        let arms = self.data.variants.iter().map(f);
        quote!(#(#arms,)*)
    }

    fn trait_path(&self) -> Option<&Path> {
        self.trait_.as_ref().map(|t| &t.path)
    }

    /// Appends a method from `TraitItemMethod`.
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
            let self_ty = match args.next() {
                Some(FnArg::SelfRef(_)) | Some(FnArg::SelfValue(_)) => None,
                Some(FnArg::Captured(ArgCaptured {
                    pat: Pat::Ident(ref pat),
                    ty: Type::Path(ref path),
                    ..
                })) if pat.ident == "self" && path.qself.is_none() => {
                    match &*path.path.clone().into_token_stream().to_string() {
                        "Pin < & Self >"
                        | ":: std :: pin :: Pin < & Self >"
                        | ":: core :: pin :: Pin < & Self >"
                        | "std :: pin :: Pin < & Self >"
                        | "core :: pin :: Pin < & Self >" => Some(SelfTypes::Pin(SelfPin::Ref)),
                        "Pin < & mut Self >"
                        | ":: std :: pin :: Pin < & mut Self >"
                        | ":: core :: pin :: Pin < & mut Self >"
                        | "std :: pin :: Pin < & mut Self >"
                        | "core :: pin :: Pin < & mut Self >" => Some(SelfTypes::Pin(SelfPin::Mut)),
                        _ => Err("unsupported first argument type")?,
                    }
                }
                _ => Err("unsupported first argument type")?,
            };
            let args: &Stack<_> = &args
                .map(|arg| match arg {
                    FnArg::Captured(arg) => Ok(&arg.pat),
                    _ => Err("unsupported arguments type")?,
                })
                .collect::<Result<_>>()?;

            let method = &item.sig.ident;
            let ident = &self.data.ident;
            match self_ty {
                None => {
                    let trait_ = self.trait_path();
                    let arms = if trait_.is_none() {
                        self.arms(|v| quote!(#ident::#v(x) => x.#method(#(#args),*)))
                    } else {
                        self.arms(|v| quote!(#ident::#v(x) => #trait_::#method(x #(,#args)*)))
                    };
                    quote!(match self { #arms })
                }
                Some(SelfTypes::Pin(self_pin)) => {
                    self.unsafe_code = true;
                    let root = std_root();
                    let pin = quote!(#root::pin::Pin);
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
                                quote!(match #pin::get_ref(self) { #arms })
                            } else {
                                quote!(unsafe { match #pin::get_ref(self) { #arms } })
                            }
                        }
                        SelfPin::Mut => {
                            if self.unsafety || item.sig.unsafety.is_some() {
                                quote!(match #pin::get_unchecked_mut(self) { #arms })
                            } else {
                                quote!(unsafe { match #pin::get_unchecked_mut(self) { #arms } })
                            }
                        }
                    }
                }
            }
        };

        self.push_item(ImplItem::Method(method_from_method(
            item,
            block(vec![Stmt::Expr(syn::parse2(method)?)]),
        )));

        Ok(())
    }

    /// Appends items from `ItemTrait`.
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
                        syn::parse2(quote!(type #ident = <#fst as #trait_>::#ident;))
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
        trait_path: Path,
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
                GenericParam::Type(ty) => Cow::Owned(GenericParam::Type(TypeParam {
                    attrs: ty.attrs.clone(),
                    ident: ty.ident.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                    eq_token: None,
                    default: None,
                })),
                param => Cow::Borrowed(param),
            })
        }

        let path = trait_path.clone();
        let mut generics = data.generics.clone();
        let trait_ = {
            if item.generics.params.is_empty() {
                path.clone()
            } else {
                let generics = generics_params(item.generics.params.iter());
                syn::parse2(quote!(#path<#(#generics),*>))?
            }
        };

        {
            let fst = data.fields.iter().next();
            let mut types: Stack<_> = item
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
            where_clause.push(syn::parse2(quote!(#fst: #trait_))?);
            data.fields.iter().skip(1).try_for_each(|variant| {
                if types.is_empty() {
                    syn::parse2(quote!(#variant: #trait_)).map(|f| where_clause.push(f))
                } else {
                    let types = types.iter().map(|(supertraits, ident)| {
                        if *supertraits {
                            match item.supertraits.iter().next() {
                                Some(TypeParamBound::Trait(trait_)) => {
                                    quote!(#ident = <#fst as #trait_>::#ident)
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            quote!(#ident = <#fst as #trait_>::#ident)
                        }
                    });
                    if item.generics.params.is_empty() {
                        syn::parse2(quote!(#variant: #path<#(#types),*>))
                            .map(|f| where_clause.push(f))
                    } else {
                        let generics = generics_params(item.generics.params.iter());
                        syn::parse2(quote!(#variant: #path<#(#generics),*, #(#types),*>))
                            .map(|f| where_clause.push(f))
                    }
                }
            })?;
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
        let mut impls = syn::parse2(quote!(#ident #ty_generics)).map(|self_ty| EnumImpl {
            data,
            defaultness: false,
            unsafety: item.unsafety.is_some(),
            generics,
            trait_: Some(Trait::new(trait_path, trait_)),
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

#[derive(PartialEq, Eq)]
enum SelfTypes {
    /// `self: Pin<&Self>` or `self: Pin<&mut Self>`
    Pin(SelfPin),
}

#[derive(PartialEq, Eq)]
enum SelfPin {
    /// `self: Pin<&Self>`
    Ref,
    /// `self: Pin<&mut Self>`
    Mut,
}

impl EnumData {
    pub fn from_item(item: &ItemEnum) -> Result<Self> {
        parse_variants(&item.variants).map(|(variants, fields)| EnumData {
            ident: item.ident.clone(),
            generics: item.generics.clone(),
            variants,
            fields,
        })
    }

    pub fn from_derive(ast: &DeriveInput) -> Result<Self> {
        let data = match &ast.data {
            Data::Enum(data) => data,
            Data::Struct(_) => Err("doesn't work with structs")?,
            Data::Union(_) => Err("doesn't work with unions")?,
        };

        parse_variants(&data.variants).map(|(variants, fields)| EnumData {
            ident: ast.ident.clone(),
            generics: ast.generics.clone(),
            variants,
            fields,
        })
    }
}

fn parse_variants(
    punctuated: &Punctuated<Variant, token::Comma>,
) -> Result<(Stack<Ident>, Stack<Type>)> {
    if punctuated.len() < 2 {
        Err("cannot be implemented for enums with less than two variants")?;
    }

    let mut variants = Stack::with_capacity(punctuated.len());
    let mut fields = Stack::with_capacity(punctuated.len());
    punctuated
        .iter()
        .try_for_each(|v| {
            if v.discriminant.is_some() {
                Err("cannot be implemented for enums with discriminants")?;
            }

            match &v.fields {
                Fields::Unnamed(f) => match f.unnamed.len() {
                    1 => fields.push(f.unnamed.iter().next().unwrap().ty.clone()),
                    0 => Err("cannot be implemented for enums with zero fields")?,
                    _ => Err("cannot be implemented for enums with multiple fields")?,
                },
                Fields::Unit => Err("cannot be implemented for enums with units")?,
                Fields::Named(_) => Err("cannot be implemented for enums with named fields")?,
            }

            variants.push(v.ident.clone());
            Ok(())
        })
        .map(|_| (variants, fields))
}
