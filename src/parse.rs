use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{borrow::Cow, mem};
use syn::{punctuated::Punctuated, *};

use crate::ast::EnumData;

/// A function for creating `proc_macro_derive` like deriving trait to enum so long as all variants are implemented that trait.
///
/// # Examples
///
/// ```rust
/// # extern crate proc_macro;
/// #
/// use derive_utils::derive_trait;
/// use proc_macro::TokenStream;
/// use quote::format_ident;
/// use syn::{parse_macro_input, parse_quote};
///
/// # #[cfg(any())]
/// #[proc_macro_derive(Iterator)]
/// # pub fn _derive_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
/// pub fn derive_iterator(input: TokenStream) -> TokenStream {
///     derive_trait(
///         &parse_macro_input!(input),
///         // trait path
///         parse_quote!(std::iter::Iterator),
///         // super trait's associated types
///         None,
///         // trait definition
///         parse_quote! {
///             trait Iterator {
///                 type Item;
///                 fn next(&mut self) -> Option<Self::Item>;
///                 fn size_hint(&self) -> (usize, Option<usize>);
///             }
///         },
///     )
///     .unwrap_or_else(|e| e.to_compile_error())
///     .into()
/// }
///
/// # #[cfg(any())]
/// #[proc_macro_derive(ExactSizeIterator)]
/// # pub fn _derive_exact_size_iterator(_: TokenStream) -> TokenStream { unimplemented!() }
/// pub fn derive_exact_size_iterator(input: TokenStream) -> TokenStream {
///     derive_trait(
///         &parse_macro_input!(input),
///         // trait path
///         parse_quote!(std::iter::ExactSizeIterator),
///         // super trait's associated types
///         Some(format_ident!("Item")),
///         // trait definition
///         parse_quote! {
///             trait ExactSizeIterator: Iterator {
///                 fn len(&self) -> usize;
///             }
///         },
///     )
///     .unwrap_or_else(|e| e.to_compile_error())
///     .into()
/// }
/// ```
pub fn derive_trait<I>(
    data: &EnumData,
    trait_path: Path,
    supertraits_types: I,
    trait_def: ItemTrait,
) -> Result<TokenStream>
where
    I: IntoIterator<Item = Ident>,
    I::IntoIter: ExactSizeIterator,
{
    EnumImpl::from_trait(data, trait_path, supertraits_types, trait_def).map(EnumImpl::build)
}

/// A builder for implementing a trait for enums.
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

impl<'a> EnumImpl<'a> {
    /// Constructs a new `EnumImpl`.
    pub fn new(data: &'a EnumData) -> Self {
        let ident = &data.ident;
        let ty_generics = data.generics.split_for_impl().1;
        Self {
            data,
            defaultness: false,
            unsafety: false,
            generics: data.generics.clone(),
            trait_: None,
            self_ty: Box::new(syn::parse_quote!(#ident #ty_generics)),
            items: Vec::new(),
            unsafe_code: false,
        }
    }

    /// Constructs a new `EnumImpl` from a trait definition.
    ///
    /// `TraitItem::Method` that has the first argument other than the following is error:
    /// * `&self`
    /// * `&mut self`
    /// * `self`
    /// * `mut self`
    /// * `self: Pin<&Self>`
    /// * `self: Pin<&mut Self>`
    ///
    /// The following items are ignored:
    /// * Generic associated types (GAT) (`TraitItem::Method` that has generics)
    /// * `TraitItem::Const`
    /// * `TraitItem::Macro`
    /// * `TraitItem::Verbatim`
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body.
    pub fn from_trait<I>(
        data: &'a EnumData,
        trait_path: Path,
        supertraits_types: I,
        mut trait_def: ItemTrait,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Ident>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut generics = data.generics.clone();
        let trait_ = {
            if trait_def.generics.params.is_empty() {
                trait_path.clone()
            } else {
                let ty_generics = trait_def.generics.split_for_impl().1;
                syn::parse_quote!(#trait_path #ty_generics)
            }
        };

        let fst = data.field_types().next();
        let mut types: Vec<_> = trait_def
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Type(ty) => Some((false, Cow::Borrowed(&ty.ident))),
                _ => None,
            })
            .collect();

        let supertraits_types = supertraits_types.into_iter();
        if supertraits_types.len() > 0 {
            if let Some(TypeParamBound::Trait(_)) = trait_def.supertraits.iter().next() {
                types.extend(supertraits_types.map(|ident| (true, Cow::Owned(ident))));
            }
        }

        let where_clause = &mut generics.make_where_clause().predicates;
        where_clause.push(syn::parse_quote!(#fst: #trait_));
        data.field_types()
            .skip(1)
            .map(|variant| {
                if types.is_empty() {
                    syn::parse_quote!(#variant: #trait_)
                } else {
                    let types = types.iter().map(|(supertraits, ident)| {
                        match trait_def.supertraits.iter().next() {
                            Some(TypeParamBound::Trait(trait_)) if *supertraits => {
                                quote!(#ident = <#fst as #trait_>::#ident)
                            }
                            _ => quote!(#ident = <#fst as #trait_>::#ident),
                        }
                    });
                    if trait_def.generics.params.is_empty() {
                        syn::parse_quote!(#variant: #trait_path<#(#types),*>)
                    } else {
                        let generics = trait_def.generics.params.iter().map(|param| match param {
                            GenericParam::Lifetime(def) => def.lifetime.to_token_stream(),
                            GenericParam::Type(param) => param.ident.to_token_stream(),
                            GenericParam::Const(param) => param.ident.to_token_stream(),
                        });
                        syn::parse_quote!(#variant: #trait_path<#(#generics),*, #(#types),*>)
                    }
                }
            })
            .for_each(|p| where_clause.push(p));

        if !trait_def.generics.params.is_empty() {
            generics.params.extend(mem::replace(&mut trait_def.generics.params, Punctuated::new()));
        }

        if let Some(old) = trait_def.generics.where_clause.as_mut() {
            if !old.predicates.is_empty() {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(mem::replace(&mut old.predicates, Punctuated::new()));
            }
        }

        let ident = &data.ident;
        let ty_generics = data.generics.split_for_impl().1;
        let mut impls = Self {
            data,
            defaultness: false,
            unsafety: trait_def.unsafety.is_some(),
            generics,
            trait_: Some(Trait { path: trait_path, ty: trait_ }),
            self_ty: Box::new(syn::parse_quote!(#ident #ty_generics)),
            items: Vec::with_capacity(trait_def.items.len()),
            unsafe_code: false,
        };
        impls.append_items_from_trait(trait_def).map(|_| impls)
    }

    pub fn set_trait(&mut self, path: Path) {
        self.trait_ = Some(Trait::new(path));
    }

    /// Appends a generic type parameter to the back of generics.
    pub fn push_generic_param(&mut self, param: GenericParam) {
        self.generics.params.push(param);
    }

    /// Appends a predicate to the back of `where`-clause.
    pub fn push_where_predicate(&mut self, predicate: WherePredicate) {
        self.generics.make_where_clause().predicates.push(predicate);
    }

    /// Appends an item to impl items.
    pub fn push_item(&mut self, item: ImplItem) {
        self.items.push(item);
    }

    fn arms(&self, f: impl FnMut(&Ident) -> TokenStream) -> TokenStream {
        let arms = self.data.variant_idents().map(f);
        quote!(#(#arms,)*)
    }

    fn trait_path(&self) -> Option<&Path> {
        self.trait_.as_ref().map(|t| &t.path)
    }

    /// Appends a method to impl items.
    ///
    /// A method that has the first argument other than the following is error:
    /// * `&self`
    /// * `&mut self`
    /// * `self`
    /// * `mut self`
    /// * `self: Pin<&Self>`
    /// * `self: Pin<&mut Self>`
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body.
    pub fn push_method(&mut self, item: TraitItemMethod) -> Result<()> {
        assert!(item.default.is_none(), "trait method `{}` has a body", item.sig.ident);

        let self_ty = SelfType::parse(&item.sig)?;
        let mut args = Vec::with_capacity(item.sig.inputs.len());
        item.sig.inputs.iter().skip(1).try_for_each(|arg| match arg {
            FnArg::Typed(arg) => {
                args.push(&arg.pat);
                Ok(())
            }
            _ => Err(error!(arg, "unsupported arguments type")),
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
                syn::parse_quote!(match self { #arms })
            }

            SelfType::Pin(mode, pin) => {
                self.unsafe_code = true;
                let trait_ = self.trait_path();
                let arms = if trait_.is_none() {
                    self.arms(|v| {
                        quote! {
                            #ident::#v(x) => #pin::new_unchecked(x).#method(#(#args),*)
                        }
                    })
                } else {
                    self.arms(|v| {
                        quote! {
                            #ident::#v(x) => #trait_::#method(#pin::new_unchecked(x) #(,#args)*)
                        }
                    })
                };

                match mode {
                    CaptureMode::Ref { mutability: false } => {
                        if self.unsafety || item.sig.unsafety.is_some() {
                            syn::parse_quote! {
                                match self.get_ref() { #arms }
                            }
                        } else {
                            syn::parse_quote! {
                                unsafe { match self.get_ref() { #arms } }
                            }
                        }
                    }
                    CaptureMode::Ref { mutability: true } => {
                        if self.unsafety || item.sig.unsafety.is_some() {
                            syn::parse_quote! {
                                match self.get_unchecked_mut() { #arms }
                            }
                        } else {
                            syn::parse_quote! {
                                unsafe { match self.get_unchecked_mut() { #arms } }
                            }
                        }
                    }
                }
            }
        };

        self.push_item(ImplItem::Method(ImplItemMethod {
            attrs: item.attrs,
            vis: Visibility::Inherited,
            defaultness: None,
            sig: item.sig,
            block: Block { brace_token: token::Brace::default(), stmts: vec![Stmt::Expr(method)] },
        }));

        Ok(())
    }

    /// Appends items from a trait definition to impl items.
    ///
    /// See [`EnumImpl::from_trait`] for supported item types.
    ///
    /// [`EnumImpl::from_trait`]: ./struct.EnumImpl.html#method.from_trait
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body.
    pub fn append_items_from_trait(&mut self, trait_def: ItemTrait) -> Result<()> {
        let fst = self.data.field_types().next();
        trait_def.items.into_iter().try_for_each(|item| match item {
            // The TraitItemType::generics field (Generic associated types (GAT)) are not supported
            TraitItem::Type(TraitItemType { ident, .. }) => {
                let trait_ = self.trait_.as_ref().map(|t| &t.ty);
                let ty = syn::parse_quote!(type #ident = <#fst as #trait_>::#ident;);
                self.push_item(ImplItem::Type(ty));
                Ok(())
            }
            TraitItem::Method(method) => self.push_method(method),
            _ => Ok(()),
        })
    }

    pub fn build(self) -> TokenStream {
        self.build_impl().to_token_stream()
    }

    pub fn build_impl(self) -> ItemImpl {
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

struct Trait {
    /// `AsRef`
    path: Path,
    /// `AsRef<T>`
    ty: Path,
}

impl Trait {
    fn new(path: Path) -> Self {
        Self { path: remove_last_path_args(path.clone()), ty: path }
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
    fn parse(sig: &Signature) -> Result<Self> {
        fn get_ty_path(ty: &Type) -> Option<&Path> {
            if let Type::Path(TypePath { qself: None, path }) = ty { Some(path) } else { None }
        }

        match sig.receiver() {
            None => Err(error!(sig.inputs, "methods without receiver are not supported")),
            Some(FnArg::Receiver(_)) => Ok(SelfType::None),
            Some(FnArg::Typed(pat)) => {
                // (mut) self: <path>
                if let Some(path) = get_ty_path(&pat.ty) {
                    let ty = path.segments.last().unwrap();
                    if let PathArguments::AngleBracketed(args) = &ty.arguments {
                        // (mut) self: [<path>::]<ty><&(mut) <elem>..>
                        if let Some(GenericArgument::Type(Type::Reference(TypeReference {
                            mutability,
                            elem,
                            ..
                        }))) = args.args.first()
                        {
                            // (mut) self: (<path>::)Pin<&(mut) Self>
                            if args.args.len() == 1
                                && ty.ident == "Pin"
                                && get_ty_path(elem).map_or(false, |path| path.is_ident("Self"))
                            {
                                return Ok(SelfType::Pin(
                                    CaptureMode::Ref { mutability: mutability.is_some() },
                                    remove_last_path_args(path.clone()),
                                ));
                            }
                        }
                    }
                }

                Err(error!(pat.ty, "unsupported receiver type: {}", pat.ty.to_token_stream()))
            }
        }
    }
}

fn remove_last_path_args(mut path: Path) -> Path {
    path.segments.last_mut().unwrap().arguments = PathArguments::None;
    path
}
