use proc_macro2::TokenStream;
use quote::quote;

use syn::{token, Expr, Ident};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("Unknown error")]
    Misc,
}

impl From<Error> for proc_macro::TokenStream {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        quote!(compile_error!(#error);).into()
    }
}

pub struct ParsedInput {
    pub context: Expr,
    pub key: Ident,
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<token::Comma>()?;
        let key = input.parse()?;
        Ok(ParsedInput { context, key })
    }
}

pub fn t_macro(tokens: TokenStream, output_type: OutputType) -> Result<TokenStream, Error> {
    let ParsedInput { context, key } = syn::parse2(tokens)?;

    let get_key = quote!(#context.get_strings().#key);
    let build_fn = match output_type {
        OutputType::String => quote!(build_string),
    };
    let inner = quote! {
        {
            #[allow(unused)]
            use mf1::BuildStr;
            let _key = #get_key;
            _key.#build_fn()
        }
    };
    Ok(inner)
}
