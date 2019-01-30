use std::{error, fmt, result};

use proc_macro2::TokenStream;
use quote::quote;

pub(crate) type StdResult<T, E> = result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

#[inline(never)]
pub fn compile_err(msg: &str) -> TokenStream {
    quote!(compile_error!(#msg);)
}

#[derive(Debug)]
pub enum Error {
    /// [`syn::Error`].
    ///
    /// [`syn::Error`]: https://docs.rs/syn/0.15/syn/struct.Error.html
    Syn(syn::Error),
    /// other error.
    Other(String),
}

impl Error {
    /// Render the error as an invocation of [`compile_error!`].
    ///
    /// [`compile_error!`]: https://doc.rust-lang.org/std/macro.compile_error.html
    pub fn to_compile_error(&self) -> TokenStream {
        match self {
            Error::Syn(e) => e.to_compile_error(),
            Error::Other(msg) => compile_err(msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syn(e) => write!(f, "{}", e),
            Error::Other(s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Syn(e) => e.description(),
            Error::Other(s) => s,
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Syn(e) => Some(e),
            Error::Other(_) => None,
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
