use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

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
    pub keys: Vec<Ident>,
    pub interpolations: Option<Vec<InterpolatedValue>>,
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;
        input.parse::<token::Comma>()?;
        let mut keys = Vec::new();
        keys.push(input.parse()?);
        while input.peek(token::Dot) {
            input.parse::<token::Dot>()?;
            keys.push(input.parse()?);
        }
        let interpolations = match input.parse::<token::Comma>() {
            Ok(_) => {
                let interpolations = input
                    .parse_terminated(InterpolatedValue::parse, token::Comma)?
                    .into_iter()
                    .collect();
                Some(interpolations)
            }
            Err(_) if input.is_empty() => None,
            Err(err) => return Err(err),
        };
        Ok(ParsedInput {
            context,
            keys,
            interpolations,
        })
    }
}

pub enum InterpolatedValue {
    Var(Ident),
    AssignedVar { key: Ident, value: Expr },
}
impl syn::parse::Parse for InterpolatedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let value = if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            let value = input.parse()?;
            InterpolatedValue::AssignedVar { key, value }
        } else {
            InterpolatedValue::Var(key)
        };
        Ok(value)
    }
}
pub fn t_macro(tokens: TokenStream, output_type: OutputType) -> Result<TokenStream, Error> {
    let ParsedInput {
        context,
        keys,
        interpolations,
    } = syn::parse2(tokens)?;

    let mut get_key = quote!(#context.get_strings().);
    get_key.append_separated(keys, quote!(.));
    let build_fn = match output_type {
        OutputType::String => quote!(build_string),
    };
    let inner = if let Some(interpolations) = interpolations {
        let (keys, values): (Vec<_>, Vec<_>) = interpolations
            .iter()
            .map(|iv| match iv {
                InterpolatedValue::Var(ident) => (ident.clone(), quote!(#ident)),
                InterpolatedValue::AssignedVar { key, value } => (key.clone(), quote!(#value)),
            })
            .unzip();
        let params = quote! {
            let (#(#keys,)*) = (#(#values,)*);
        };

        let builders = interpolations.iter().map(|inter| {
            let key = match inter {
                InterpolatedValue::Var(key) | InterpolatedValue::AssignedVar { key, .. } => key,
            };
            let builder = Ident::new(&format!("arg_{}", key), Span::call_site());
            quote!(#builder(&#key))
        });
        quote! {
            {
                #params
                #[allow(unused)]
                use mf1::BuildStr;
                let _key = #get_key;
                #(
                    let _key = _key.#builders;
                )*
                #[deny(deprecated)]
                _key.#build_fn()
            }
        }
    } else {
        quote! {
            {
                #[allow(unused)]
                use mf1::BuildStr;
                let _key = #get_key;
                _key.#build_fn()
            }
        }
    };
    Ok(inner)
}
