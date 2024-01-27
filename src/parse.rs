// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::mem;
use std::borrow::Cow;

use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, token, Block, FnArg, GenericParam, Generics, Ident, ImplItem, ImplItemFn,
    ItemImpl, ItemTrait, Path, Signature, Stmt, Token, TraitItem, TraitItemFn, TraitItemType, Type,
    TypeParamBound, TypePath, Visibility, WherePredicate,
};

use crate::ast::EnumData;

/// A function for creating `proc_macro_derive` like deriving trait to enum so
/// long as all variants are implemented that trait.
///
/// # Examples
///
/// ```
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
///         &parse_quote!(std::iter::Iterator),
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
///         &parse_quote!(std::iter::ExactSizeIterator),
///         // super trait's associated types
///         Some(format_ident!("Item")),
///         // trait definition
///         parse_quote! {
///             trait ExactSizeIterator: Iterator {
///                 fn len(&self) -> usize;
///             }
///         },
///     )
///     .into()
/// }
/// ```
pub fn derive_trait<I>(
    data: &EnumData,
    trait_path: &Path,
    supertraits_types: I,
    trait_def: ItemTrait,
) -> TokenStream
where
    I: IntoIterator<Item = Ident>,
    I::IntoIter: ExactSizeIterator,
{
    EnumImpl::from_trait(data, trait_path, supertraits_types, trait_def).build()
}

/// A builder for implementing a trait for enums.
pub struct EnumImpl<'a> {
    data: &'a EnumData,
    defaultness: bool,
    unsafety: bool,
    generics: Generics,
    trait_: Option<Path>,
    self_ty: Box<Type>,
    items: Vec<ImplItem>,
}

impl<'a> EnumImpl<'a> {
    /// Creates a new `EnumImpl`.
    pub fn new(data: &'a EnumData) -> Self {
        let ident = &data.ident;
        let ty_generics = data.generics.split_for_impl().1;
        Self {
            data,
            defaultness: false,
            unsafety: false,
            generics: data.generics.clone(),
            trait_: None,
            self_ty: Box::new(parse_quote!(#ident #ty_generics)),
            items: vec![],
        }
    }

    /// Creates a new `EnumImpl` from a trait definition.
    ///
    /// The following items are ignored:
    /// - Generic associated types (GAT) ([`TraitItem::Type`] that has generics)
    /// - [`TraitItem::Const`]
    /// - [`TraitItem::Macro`]
    /// - [`TraitItem::Verbatim`]
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body, no receiver, or a receiver other
    /// than the following:
    ///
    /// - `&self`
    /// - `&mut self`
    /// - `self`
    pub fn from_trait<I>(
        data: &'a EnumData,
        trait_path: &Path,
        supertraits_types: I,
        mut trait_def: ItemTrait,
    ) -> Self
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
                parse_quote!(#trait_path #ty_generics)
            }
        };

        let fst = data.field_types().next().unwrap();
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

        // https://github.com/taiki-e/derive_utils/issues/47
        let type_params = generics.type_params().map(|p| p.ident.to_string()).collect::<Vec<_>>();
        let has_method = trait_def.items.iter().any(|i| matches!(i, TraitItem::Fn(..)));
        if !has_method || !type_params.is_empty() {
            struct HasTypeParam<'a>(&'a [String]);

            impl HasTypeParam<'_> {
                fn check_ident(&self, ident: &Ident) -> bool {
                    let ident = ident.to_string();
                    self.0.contains(&ident)
                }

                fn visit_type(&self, ty: &Type) -> bool {
                    if let Type::Path(node) = ty {
                        if node.qself.is_none() {
                            if let Some(ident) = node.path.get_ident() {
                                return self.check_ident(ident);
                            }
                        }
                    }
                    self.visit_token_stream(ty.to_token_stream())
                }

                fn visit_token_stream(&self, tokens: TokenStream) -> bool {
                    for tt in tokens {
                        match tt {
                            TokenTree::Ident(ident) => {
                                if self.check_ident(&ident) {
                                    return true;
                                }
                            }
                            TokenTree::Group(group) => {
                                let content = group.stream();
                                if self.visit_token_stream(content) {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                    false
                }
            }

            let visitor = HasTypeParam(&type_params);
            let where_clause = &mut generics.make_where_clause().predicates;
            if !has_method || visitor.visit_type(fst) {
                where_clause.push(parse_quote!(#fst: #trait_));
            }
            if data.field_types().len() > 1 {
                let fst_tokens = fst.to_token_stream().to_string();
                where_clause.extend(data.field_types().skip(1).filter_map(
                    |variant| -> Option<WherePredicate> {
                        if has_method && !visitor.visit_type(variant) {
                            return None;
                        }
                        if variant.to_token_stream().to_string() == fst_tokens {
                            return None;
                        }
                        if types.is_empty() {
                            return Some(parse_quote!(#variant: #trait_));
                        }
                        let types = types.iter().map(|(supertraits, ident)| {
                            match trait_def.supertraits.iter().next() {
                                Some(TypeParamBound::Trait(trait_)) if *supertraits => {
                                    quote!(#ident = <#fst as #trait_>::#ident)
                                }
                                _ => quote!(#ident = <#fst as #trait_>::#ident),
                            }
                        });
                        if trait_def.generics.params.is_empty() {
                            Some(parse_quote!(#variant: #trait_path<#(#types),*>))
                        } else {
                            let generics =
                                trait_def.generics.params.iter().map(|param| match param {
                                    GenericParam::Lifetime(def) => def.lifetime.to_token_stream(),
                                    GenericParam::Type(param) => param.ident.to_token_stream(),
                                    GenericParam::Const(param) => param.ident.to_token_stream(),
                                });
                            Some(parse_quote!(#variant: #trait_path<#(#generics),*, #(#types),*>))
                        }
                    },
                ));
            }
        }

        if !trait_def.generics.params.is_empty() {
            generics.params.extend(mem::take(&mut trait_def.generics.params));
        }

        if let Some(old) = trait_def.generics.where_clause.as_mut() {
            if !old.predicates.is_empty() {
                generics.make_where_clause().predicates.extend(mem::take(&mut old.predicates));
            }
        }

        let ident = &data.ident;
        let ty_generics = data.generics.split_for_impl().1;
        let mut impls = Self {
            data,
            defaultness: false,
            unsafety: trait_def.unsafety.is_some(),
            generics,
            trait_: Some(trait_),
            self_ty: Box::new(parse_quote!(#ident #ty_generics)),
            items: Vec::with_capacity(trait_def.items.len()),
        };
        impls.append_items_from_trait(trait_def);
        impls
    }

    pub fn set_trait(&mut self, path: Path) {
        self.trait_ = Some(path);
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

    /// Appends a method to impl items.
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body, no receiver, or a receiver other
    /// than the following:
    ///
    /// - `&self`
    /// - `&mut self`
    /// - `self`
    pub fn push_method(&mut self, item: TraitItemFn) {
        assert!(item.default.is_none(), "method `{}` has a body", item.sig.ident);

        let self_ty = ReceiverKind::new(&item.sig);
        let mut args = Vec::with_capacity(item.sig.inputs.len());
        item.sig.inputs.iter().skip(1).for_each(|arg| match arg {
            FnArg::Typed(arg) => args.push(&arg.pat),
            FnArg::Receiver(_) => panic!(
                "method `{}` has a receiver in a position other than the first argument",
                item.sig.ident
            ),
        });

        let method = &item.sig.ident;
        let ident = &self.data.ident;
        let method = match self_ty {
            ReceiverKind::Normal => match &self.trait_ {
                None => {
                    let arms = self.data.variant_idents().map(|v| {
                        quote! {
                            #ident::#v(x) => x.#method(#(#args),*),
                        }
                    });
                    parse_quote!(match self { #(#arms)* })
                }
                Some(trait_) => {
                    let arms =
                        self.data.variant_idents().zip(self.data.field_types()).map(|(v, ty)| {
                            quote! {
                                #ident::#v(x) => <#ty as #trait_>::#method(x #(,#args)*),
                            }
                        });
                    parse_quote!(match self { #(#arms)* })
                }
            },
        };

        self.push_item(ImplItem::Fn(ImplItemFn {
            attrs: item.attrs,
            vis: Visibility::Inherited,
            defaultness: None,
            sig: item.sig,
            block: Block {
                brace_token: token::Brace::default(),
                stmts: vec![Stmt::Expr(method, None)],
            },
        }));
    }

    /// Appends items from a trait definition to impl items.
    ///
    /// # Panics
    ///
    /// Panics if a trait method has a body, no receiver, or a receiver other
    /// than the following:
    ///
    /// - `&self`
    /// - `&mut self`
    /// - `self`
    pub fn append_items_from_trait(&mut self, trait_def: ItemTrait) {
        let fst = self.data.field_types().next();
        trait_def.items.into_iter().for_each(|item| match item {
            // The TraitItemType::generics field (Generic associated types (GAT)) are not supported
            TraitItem::Type(TraitItemType { ident, .. }) => {
                let trait_ = &self.trait_;
                let ty = parse_quote!(type #ident = <#fst as #trait_>::#ident;);
                self.push_item(ImplItem::Type(ty));
            }
            TraitItem::Fn(method) => self.push_method(method),
            _ => {}
        });
    }

    pub fn build(self) -> TokenStream {
        self.build_impl().to_token_stream()
    }

    pub fn build_impl(self) -> ItemImpl {
        ItemImpl {
            attrs: vec![],
            defaultness: if self.defaultness { Some(<Token![default]>::default()) } else { None },
            unsafety: if self.unsafety { Some(<Token![unsafe]>::default()) } else { None },
            impl_token: token::Impl::default(),
            generics: self.generics,
            trait_: self.trait_.map(|trait_| (None, trait_, <Token![for]>::default())),
            self_ty: self.self_ty,
            brace_token: token::Brace::default(),
            items: self.items,
        }
    }
}

enum ReceiverKind {
    /// `&(mut) self`, `(mut) self`, `(mut) self: &(mut) Self`, or `(mut) self: Self`
    Normal,
}

impl ReceiverKind {
    fn new(sig: &Signature) -> Self {
        fn get_ty_path(ty: &Type) -> Option<&Path> {
            if let Type::Path(TypePath { qself: None, path }) = ty {
                Some(path)
            } else {
                None
            }
        }

        match sig.receiver() {
            None => panic!("method `{}` has no receiver", sig.ident),
            Some(receiver) => {
                if receiver.colon_token.is_none() {
                    return ReceiverKind::Normal;
                }
                match &*receiver.ty {
                    Type::Path(TypePath { qself: None, path }) => {
                        // (mut) self: Self
                        if path.is_ident("Self") {
                            return ReceiverKind::Normal;
                        }
                    }
                    Type::Reference(ty) => {
                        // (mut) self: &(mut) Self
                        if get_ty_path(&ty.elem).map_or(false, |path| path.is_ident("Self")) {
                            return ReceiverKind::Normal;
                        }
                    }
                    _ => {}
                }
                panic!(
                    "method `{}` has unsupported receiver type: {}",
                    sig.ident,
                    receiver.ty.to_token_stream()
                );
            }
        }
    }
}
