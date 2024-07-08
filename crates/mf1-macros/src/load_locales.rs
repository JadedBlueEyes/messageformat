use convert_case::Case::Snake;
use convert_case::Casing;
use mf1_parser::{parse, LexerSpan, Token as AstToken};
use proc_macro2::Ident;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use std::io;
use std::{borrow::Cow, collections::HashMap, fs::File, path::PathBuf};
use thiserror::Error;
use toml::Value;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error, can't access env variable \"CARGO_MANIFEST_DIR\": {0}")]
    CargoDirEnvNotPresent(std::env::VarError),
    #[error("Error accessing Cargo.toml: {0}")]
    ManifestNotFound(std::io::Error),
    #[error("Error parsing Cargo.toml: {0}")]
    ConfigFileDeser(toml::de::Error),
    #[error("No locales found")]
    NoDefaultLocale,
    #[error("No locale file: {0}")]
    NoLocaleFile(std::io::Error),
    #[error("Parsing of file {path:?} failed: {err}")]
    LocaleFileDeser {
        path: PathBuf,
        err: serde_json::Error,
    },
    #[error("Parsing of key {key} in {locale} failed: {message} ({span:?})")]
    ParseKeyErr {
        locale: String,
        key: String,
        src: String,
        message: String,
        span: LexerSpan,
    },
    #[error("Unknown error")]
    Misc,
}

impl From<Error> for proc_macro::TokenStream {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        quote!(compile_error!(#error);).into()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    pub locales_dir: Option<String>,
    pub base_locale: Option<String>,
    pub locales: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringSet<'a> {
    pub name: &'a str,
    pub keys: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> StringSet<'a> {
    pub fn from_file(name: &'a str, locale_file: File) -> Result<Self, serde_json::Error> {
        let reader = io::BufReader::new(locale_file);
        let mut deser = serde_json::Deserializer::from_reader(reader);
        let keys = HashMap::<Cow<'a, str>, Cow<'a, str>>::deserialize(&mut deser)?;
        Ok(Self { name, keys })
    }
    pub fn ident(&self) -> Ident {
        quote::format_ident!("{}", self.name.to_case(Snake))
    }
}

pub fn load_locales() -> Result<TokenStream, Error> {
    let cargo_manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(Error::CargoDirEnvNotPresent)?
        .into();

    let cargo_manifest = std::fs::read_to_string(cargo_manifest_dir.join("Cargo.toml"))
        .map_err(Error::ManifestNotFound)?
        .parse::<toml::Table>()
        .map_err(Error::ConfigFileDeser)?;

    let meta: ConfigFile = cargo_manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|p| p.get("mf1"))
        .and_then(|p| Value::try_into(p.clone()).ok())
        .unwrap_or_default();

    let mut manifest_dir_path: PathBuf = meta
        .locales_dir
        .clone()
        .map(|d| d.into())
        .unwrap_or_else(|| cargo_manifest_dir.join("locales"));

    let mut locales = Vec::with_capacity(meta.locales.len());
    for locale in meta.locales.iter() {
        manifest_dir_path.push(locale);
        manifest_dir_path.set_extension("json");
        let locale_file = std::fs::File::open(&manifest_dir_path).map_err(Error::NoLocaleFile)?;
        let locale =
            StringSet::from_file(locale, locale_file).map_err(|err| Error::LocaleFileDeser {
                path: manifest_dir_path.clone(),
                err,
            })?;
        locales.push(locale);
        manifest_dir_path.pop();
    }

    let i18n_keys_ident = quote::format_ident!("Mf1Keys");

    let default_locale = meta
        .base_locale
        .as_ref()
        .or_else(|| meta.locales.first())
        .ok_or(Error::NoDefaultLocale)?;

    let base_locale_strings = locales
        .iter()
        .find(|l| l.name == default_locale)
        .ok_or(Error::NoDefaultLocale)?;
    let base_locale_ident = base_locale_strings.ident();

    let locale_idents: Vec<_> = locales.iter().map(StringSet::ident).collect();

    let get_strings_match_arms = locale_idents
        .iter()
        .map(|locale| quote!(Locale::#locale => &#locale))
        .collect::<Vec<_>>();

    let as_str_match_arms = locale_idents
        .iter()
        .zip(locales.iter())
        .map(|(key, l)| (key, l.name))
        .map(|(variant, locale)| quote!(Locale::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locale_idents
        .iter()
        .zip(locales.iter())
        .map(|(key, l)| (key, l.name))
        .map(|(variant, locale)| quote!(#locale => Ok(Locale::#variant)))
        .collect::<Vec<_>>();

    let locales_enum = quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum Locale {
            #(#locale_idents,)*
        }

        impl Locale {
            fn get_strings(self) -> &'static #i18n_keys_ident {
                match self {
                    #(#get_strings_match_arms,)*
                }
            }

            fn as_str(self) -> &'static str {
                match self {
                    #(#as_str_match_arms,)*
                }
            }
        }

        impl std::str::FromStr for Locale {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim() {
                    #(#from_str_match_arms,)*
                    _ => Err(())
                }
            }
        }

        impl Default for Locale {
            fn default() -> Self {
                Locale::#base_locale_ident
            }
        }
    };

    let locale_ast: HashMap<_, _> = locales
        .iter()
        .map(|l| {
            let mut keys = HashMap::new();
            l.keys.iter().for_each(|(k, v)| {
                keys.insert(
                    k.clone(),
                    parse::<String>(v).map_err(|(message, span)| Error::ParseKeyErr {
                        locale: l.name.to_string(),
                        key: k.to_string(),
                        src: v.to_string(),
                        message,
                        span,
                    }),
                );
            });
            (l.name, keys)
        })
        .collect();

    let string_keys = base_locale_strings
        .keys
        .keys()
        .filter(|k| {
            locale_ast.iter().all(|(_l, v)| match v.get(*k) {
                Some(s) => matches!(s, Ok(r) if r.iter().all(|t| matches!(t, AstToken::Content { value: _ }))),
                None => true,
            })
        })
        .collect::<Vec<_>>();

    let string_field_defs: Vec<TokenStream> = string_keys
        .iter()
        .map(|key| Ident::new(key, Span::call_site()))
        .map(|key| quote!(pub #key: &'static str))
        .collect::<Vec<_>>();

    let keys_type = quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #i18n_keys_ident {
            #(#string_field_defs,)*
        }
    };

    let locale_values = locales.iter().map(|locale| {
        let fields = string_keys.iter().map(|key| {
            let key_ident = Ident::new(key, Span::call_site());
            match locale.keys.get(*key) {
                Some(value) => quote!(#key_ident: #value),
                _ => {
                    quote!(#key_ident: #base_locale_ident.#key_ident)
                    // global_locale
                    //     .keys
                    //     .get(key)
                    //     .map(|value| quote!(#key: #value))
                }
            }
        });
        let ident = locale.ident();

        quote! {
            #[allow(non_upper_case_globals)]
            const #ident: #i18n_keys_ident = #i18n_keys_ident {
                #(#fields,)*
            };
        }
    });

    let locale_static = quote! {
        #(
            #locale_values
        )*
    };
    // #(#builder_fields,)*
    // #(#subkeys_fields,)*
    Ok(quote! {
        #locales_enum
        #keys_type
        #locale_static
    })
    // }
    // Err(Error::Misc)
}
