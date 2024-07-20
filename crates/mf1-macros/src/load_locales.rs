use convert_case::Case::{Pascal, Snake};
use convert_case::Casing;
use mf1_parser::{parse, ArgType, LexerSpan, Token as AstToken, TokenSlice};
use proc_macro2::Ident;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs::File, path::PathBuf};
use std::{io, iter};
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
    pub keys: HashMap<Cow<'a, str>, StringItem<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringItem<'a> {
    String(Cow<'a, str>),
    Subkey(HashMap<Cow<'a, str>, StringItem<'a>>),
}

impl<'a> StringSet<'a> {
    pub fn from_file(name: &'a str, locale_file: File) -> Result<Self, serde_json::Error> {
        let reader = io::BufReader::new(locale_file);
        let mut deser = serde_json::Deserializer::from_reader(reader);
        let keys = HashMap::<Cow<'a, str>, StringItem<'a>>::deserialize(&mut deser)?;
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
        .map(|locale| quote!(Locale::#locale => &#locale));

    let as_str_match_arms = locale_idents
        .iter()
        .zip(locales.iter())
        .map(|(key, l)| (key, l.name))
        .map(|(variant, locale)| quote!(Locale::#variant => #locale));

    let from_str_match_arms = locale_idents
        .iter()
        .zip(locales.iter())
        .map(|(key, l)| (key, l.name))
        .map(|(variant, locale)| quote!(#locale => Ok(Locale::#variant)));

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
    let keys_tokens = generate_keys(
        locales.iter().map(|k| (k.name, k)).collect(),
        base_locale_strings,
        i18n_keys_ident,
    );
    Ok(quote! {
        #locales_enum
        #keys_tokens
    })
}

fn generate_keys(
    locales: HashMap<&str, &StringSet>,
    base_locale_strings: &StringSet,
    i18n_keys_ident: Ident,
) -> TokenStream {
    let base_locale_ident = base_locale_strings.ident();
    let locale_subkeys: HashMap<_, _> = locales
        .iter()
        .map(|(name, string_keys)| {
            let mut keys = HashMap::new();
            string_keys
                .keys
                .iter()
                .filter_map(|(k, v)| match v {
                    StringItem::Subkey(v) => Some((k, v)),
                    _ => None,
                })
                .for_each(|(k, v)| match base_locale_strings.keys.get(k) {
                    Some(StringItem::Subkey(_)) => {
                        keys.insert(k.clone(), v);
                    }
                    Some(_) => eprintln!(
                        "Default locale has incompatible non-subkey key {:?} from locale {}!",
                        k, name
                    ),
                    None => eprintln!(
                        "Default locale is missing key {:?} from locale {}!",
                        k, name
                    ),
                });
            (name, keys)
        })
        .collect();
    let locale_subkeys = base_locale_strings
        .keys
        .iter()
        .filter_map(|(k, v)| match v {
            StringItem::Subkey(v) => Some((k, v)),
            _ => None,
        })
        .map(|(k, v)| {
            let locales: HashMap<&str, StringSet> = locale_subkeys
                .iter()
                .map(|(l, m)| {
                    (
                        **l,
                        StringSet {
                            name: l,
                            keys: m.get(k).map(|s| (*s).clone()).unwrap_or_else(HashMap::new),
                        },
                    )
                })
                .collect();
            (
                k,
                generate_keys(
                    locales.iter().map(|(k, v)| (*k, v)).collect(),
                    &StringSet {
                        name: base_locale_strings.name,
                        keys: v.clone(),
                    },
                    quote::format_ident!("{}", k.to_case(Pascal)),
                ),
            )
        })
        .map(|(k, v)| {
            let k = quote::format_ident!("{}", k.to_case(Snake));
            quote! {
                pub mod #k {
                    #v
                }
            }
        });

    let locale_ast: HashMap<_, _> = locales
        .iter()
        .map(|(name, l)| {
            let mut keys = HashMap::new();
            l.keys
                .iter()
                .filter_map(|(k, v)| match v {
                    StringItem::String(v) => Some((k, v)),
                    StringItem::Subkey(_) => None,
                })
                .for_each(|(k, v)| {
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
            (name, keys)
        })
        .collect();

    let string_keys = base_locale_strings
        .keys
        .iter()
        .filter(|(k, v)| {
            match v{
                StringItem::String(_) => (),
                StringItem::Subkey(_) => return false,
            }
            locale_ast.iter().all(|(_l, v)| match v.get(*k) {
                Some(s) => matches!(s, Ok(r) if r.iter().all(|t| matches!(t, AstToken::Content { value: _ }))),
                None => true,
            })
        })
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    let mut dyn_keys = HashMap::new();
    for (locale, asts) in locale_ast.iter() {
        for (k, ast) in asts.iter().filter(|(k, _)| !string_keys.contains(k)) {
            if base_locale_strings.keys.contains_key(k) {
                if let Ok(ast) = ast {
                    dyn_keys
                        .entry(k)
                        .and_modify(|args| ast.get_args_into(args))
                        .or_insert_with(|| ast.get_args());
                }
            } else {
                // Default locale is missing this key!
                eprintln!(
                    "Default locale is missing key {:?} from locale {}!",
                    k, locale
                )
            }
        }
    }
    let dyn_keys = dyn_keys
        .into_iter()
        .map(|(k, a)| {
            (
                k,
                a.into_iter()
                    .map(|(a, v)| {
                        if v.len() > 1 {
                            eprintln!(
                    "Argument {a:?} from key {k} is used in multiple ways! Picking first type",
                )
                        }
                        let arg_type = *v.iter().next().expect("arguments should have a type");
                        (a, arg_type)
                    })
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    let builder_defs: Vec<TokenStream> = dyn_keys
        .iter()
        .map(|(key, args)| {
            let ident = Ident::new(key, Span::call_site());
            let type_params = args
                .iter()
                .map(|(arg, _)| Ident::new(&format!("__{}", arg), Span::call_site()));
            let concrete_types: Vec<_> = args
                .iter()
                .map(|(_, arg_type)| match arg_type {
                    ArgType::OrdinalArg => quote! {i32},
                    ArgType::PlainArg | ArgType::SelectArg => quote! {&str},
                    ArgType::FunctionArg => todo!()
                }).collect();

            let field_names: Vec<_> = args.iter().map(|(arg, _)| {
                Ident::new(&format!("arg_{}", arg), Span::call_site())
            }).collect();
            let fields = args.iter().map(|(arg, _)| {
                let key = Ident::new(&format!("arg_{}", arg), Span::call_site());
                let type_param = Ident::new(&format!("__{}", arg), Span::call_site());
                quote!(#key: #type_param)
            });
            let default_type_params = args.iter().map(|_| quote!(EmptyValue));
            let default_fields = field_names.iter().map(|key| {
                quote!(#key: EmptyValue)
            });
            let formatter_args = field_names.iter().map(|key| {
                quote!(self.#key)
            });
            let formatter_type = quote!(&'a for<'x, 'y> fn(&'x mut dyn mf1::Formatable<'y>, #(#concrete_types,)*) -> Result<(), Box<dyn std::error::Error>>);
            fn gen_setter<'a>(ident: &syn::Ident, field: &syn::Ident, other_fields: impl Iterator<Item = &'a Ident> + Clone) -> proc_macro2::TokenStream {
                let restructure_others = other_fields.clone();
                quote! {

                    impl<'a> #ident<'a, EmptyValue> {
                        pub fn #field(self, #field: &str) -> #ident<'a, &str> {
                            let #ident { formatter, #(#other_fields,)* .. } = self;
                            #ident { formatter, #field, #(#restructure_others,)*  }
                        }
                    }
                }
            }
    fn split_at<T>(slice: &[T], i: usize) -> (&[T], &T, &[T]) {
        let (left, rest) = slice.split_at(i);
        let (mid, right) = rest.split_first().unwrap();
        (left, mid, right)
    }
            let setters = (0..field_names.len())
            .map(|i| split_at(&field_names, i))
            .map(|(left_fields, field, right_fields)| {
                gen_setter(&ident, field, left_fields.iter().chain(right_fields.iter()))
            });
            quote! {
                #[allow(non_camel_case_types, non_snake_case)]
                // #[derive(Clone, Copy)]
                #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
                // #[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
                pub struct #ident<'a, #(#type_params,)*> {
                    formatter: #formatter_type,
                    #(#fields,)*
                }

                impl<'a> #ident<'a, #(#default_type_params,)*> {
                    pub const fn new(formatter: #formatter_type) -> Self {
                        Self {
                            formatter,
                            #(#default_fields,)*
                        }
                    }
                }
                #(#setters)*
                impl<'a> mf1::BuildStr for #ident<'a, #(#concrete_types,)*> {
                    #[inline]
                    fn build_string(self) -> std::borrow::Cow<'static, str> {
                        std::borrow::Cow::Owned(format!("{}", self))
                    }
                }
                impl<'a> std::fmt::Display for #ident<'a, #(#concrete_types,)*> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match (self.formatter)(f, #(#formatter_args)*) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(*e.downcast().unwrap()),
                        }
                    }
                }

            }
        })
        .collect::<Vec<_>>();

    let subkey_field_defs = base_locale_strings
        .keys
        .iter()
        .filter_map(|(k, v)| match v {
            StringItem::Subkey(_) => {
                let mod_name: Ident = quote::format_ident!("{}", k.to_case(Snake));
                let type_name = quote::format_ident!("{}", k.to_case(Pascal));
                let k = quote::format_ident!("{}", k);
                Some(quote!(pub #k: subkeys::#mod_name::#type_name))
            }
            _ => None,
        });
    let dyn_field_defs = dyn_keys.iter().map(|(key, args)| {
        let key: Ident = quote::format_ident!("{}", key);
        let type_params = args.iter().map(|_| quote!(builders::EmptyValue));
        quote!(pub #key: builders::#key<'static, #(#type_params,)*>)
    });

    let string_field_defs = string_keys
        .iter()
        .map(|key| Ident::new(key, Span::call_site()))
        .map(|key| quote!(pub #key: &'static str));

    let keys_type = quote! {
        #[doc(hidden)]
        pub mod builders {
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub struct EmptyValue;
            #(#builder_defs)*
        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #i18n_keys_ident {
            #(#string_field_defs,)*
            #(#dyn_field_defs,)*
            #(#subkey_field_defs,)*
        }
    };

    let locale_values = locales.iter().map(|locale| {
        let ident = locale.1.ident();
        let subkey_fields = base_locale_strings
        .keys
        .iter()
        .filter_map(|(k, v)| match v {
            StringItem::Subkey(_) => {
                let mod_name: Ident = quote::format_ident!("{}", k.to_case(Snake));
                let k = quote::format_ident!("{}", k);
            Some(quote!(#k: subkeys::#mod_name::#ident))
            },
            _ => None,
        });
        let string_fields = string_keys.iter().map(|key| {
            let key_ident = Ident::new(key, Span::call_site());
            match locale.1.keys.get(*key) {
                Some(StringItem::String(value)) => quote!(#key_ident: #value),
                Some(StringItem::Subkey(_)) => unreachable!(),
                _ => {
                    quote!(#key_ident: #base_locale_ident.#key_ident)
                }
            }
        });
        let formatter_fields = dyn_keys.iter().map(|(key, arg_types)| {
            let key_ident = Ident::new(key, Span::call_site());
            match locale_ast.get(locale.0).unwrap().get(*key) {
                Some(Ok(ast)) => {
                    let args = arg_types
                        .iter()
                        .map(|(name, arg_type)| {
                            let name = Ident::new(name, Span::call_site());
                            match arg_type {
                                ArgType::OrdinalArg => quote! {#name: i32},
                                ArgType::PlainArg | ArgType::SelectArg => quote! {#name: &str},
                                ArgType::FunctionArg => todo!()
                            }
                        });
                    fn gen_items(token: &AstToken<String>) -> impl Iterator<Item = TokenStream> {
                        match token {
                            AstToken::Content { value } => iter::once(quote! {fmt.write_str(#value)?;}),
                            AstToken::PlainArg { arg } => {
                                let arg = Ident::new(arg, Span::call_site());
                                iter::once(quote! {fmt.write_str(#arg)?;})
                            },
                            AstToken::Octothorpe {  } => iter::once(quote! {fmt.write_str("#")?;}),
                            select @ AstToken::Select { arg, cases, plural_offset: _ } => {
                                let case_statements = cases.iter().filter(|case| case.key != "other").map(|case| {
                                    let key = &case.key;
                                    let items = case.tokens.iter().flat_map(gen_items);
                                    quote! { #key => { #(#items)* }}
                                });
                                let other = if let Some(case) = cases.iter().find(|case| case.key == "other") {
                                    let items = case.tokens.iter().flat_map(gen_items);
                                    quote! { _ => { #(#items)* }
                                }} else {
                                    eprintln!("Missing other case in select {select:?}");
                                    quote! {}
                                };
                                let arg = Ident::new(arg, Span::call_site());
                                iter::once(quote! {
                                    match #arg {
                                        #(#case_statements,)*
                                        #other
                                    }
                                })
                            }
                            _ => todo!(),
                        }
                    }
                    let items = ast.iter().flat_map(gen_items);
                    quote!(#key_ident: builders::#key_ident::new(&(|fmt: &mut dyn mf1::Formatable, #(#args,)*| -> Result<(), _> {
                        #(#items)*
                        Ok(())
                    } as _)))
                },
                _ => {
                    quote!(#key_ident: #base_locale_ident.#key_ident)
                }
            }
        });


        quote! {
            #[allow(non_upper_case_globals)]
            pub(crate) const #ident: #i18n_keys_ident = #i18n_keys_ident {
                #(#string_fields,)*
                #(#formatter_fields,)*
                #(#subkey_fields,)*
            };
        }
    });

    let locale_static = quote! {
        #(
            #locale_values
        )*
    };
    quote! {
        #[doc(hidden)]
        pub mod subkeys {
            #(#locale_subkeys)*
        }
        #keys_type
        #locale_static
    }
}
