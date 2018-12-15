use std::{fmt, result};

use proc_macro2::TokenStream;
use quote::quote;

pub(crate) type StdResult<T, E> = result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub fn compile_err(msg: &str) -> TokenStream {
    quote!(compile_error!(#msg);)
}

#[derive(Debug)]
pub enum Error {
    /// `syn::Error`.
    Syn(syn::Error),
    /// other error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Error::*;
        match self {
            Syn(e) => write!(f, "{}", e),
            Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Self {
        Error::Other(s.into())
    }
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Self {
        Error::Syn(e)
    }
}
