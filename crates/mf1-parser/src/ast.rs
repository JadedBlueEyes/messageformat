use icu_plurals::PluralCategory;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
#[serde(bound(
    serialize = "StrMaybeOwned: Serialize",
    deserialize = "StrMaybeOwned: Deserialize<'de>"
))]
pub enum Token<'a, 'b, StrMaybeOwned>
where
    StrMaybeOwned: Deref<Target = str> + Clone,
{
    #[serde(rename = "content")]
    Content { value: StrMaybeOwned },

    #[serde(rename = "argument")]
    PlainArg { arg: StrMaybeOwned },
    #[serde(rename = "function")]
    FunctionArg {
        arg: StrMaybeOwned,
        key: StrMaybeOwned,
        param: Option<Cow<'b, [Token<'a, 'b, StrMaybeOwned>]>>,
    },

    //   type: 'plural' | 'select' | 'selectordinal';
    #[serde(rename = "plural")]
    Plural {
        arg: StrMaybeOwned,
        cases: Cow<'b, [PluralCase<'a, 'b, StrMaybeOwned>]>,
        #[serde(default, rename = "pluralOffset")]
        plural_offset: Option<i32>,
    },
    #[serde(rename = "select")]
    Select {
        arg: StrMaybeOwned,
        cases: Cow<'b, [SelectCase<'a, 'b, StrMaybeOwned>]>,
        #[serde(default, rename = "pluralOffset")]
        plural_offset: Option<i32>,
    },
    #[serde(rename = "selectordinal")]
    SelectOrdinal {
        arg: StrMaybeOwned,
        cases: Cow<'b, [PluralCase<'a, 'b, StrMaybeOwned>]>,
        #[serde(default, rename = "pluralOffset")]
        plural_offset: Option<i32>,
    },

    #[serde(rename = "octothorpe")]
    Octothorpe {},
}

// #[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(bound(
    serialize = "StrMaybeOwned: Serialize",
    deserialize = "StrMaybeOwned: Deserialize<'de>"
))]
pub struct PluralCase<'a, 'b, StrMaybeOwned>
where
    StrMaybeOwned: Deref<Target = str> + Clone,
{
    pub key: PluralCategory,
    pub tokens: Cow<'b, [Token<'a, 'b, StrMaybeOwned>]>,
}

impl<'a, 'b, StrMaybeOwned> TryFrom<SelectCase<'a, 'b, StrMaybeOwned>>
    for PluralCase<'a, 'b, StrMaybeOwned>
where
    StrMaybeOwned: Deref<Target = str> + Clone,
{
    type Error = ();

    fn try_from(value: SelectCase<'a, 'b, StrMaybeOwned>) -> Result<Self, Self::Error> {
        Ok(PluralCase {
            key: PluralCategory::get_for_cldr_bytes(value.key.as_bytes()).ok_or(())?,
            tokens: value.tokens,
        })
    }
}

// #[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(bound(
    serialize = "StrMaybeOwned: Serialize",
    deserialize = "StrMaybeOwned: Deserialize<'de>"
))]
pub struct SelectCase<'a, 'b, StrMaybeOwned>
where
    StrMaybeOwned: 'a + Deref<Target = str> + Clone,
{
    pub key: StrMaybeOwned,
    pub tokens: Cow<'b, [Token<'a, 'b, StrMaybeOwned>]>,
}

#[derive(Debug, Copy, Clone)]
pub enum ArgType {
    OrdinalArg,
    PlainArg,
    SelectArg,
    FunctionArg,
}

pub trait TokenSlice<'a, T> {
    fn get_args(&'a self) -> HashMap<&'a T, Vec<ArgType>>
    where
        T: Deref<Target = str> + Clone,
    {
        let mut args = HashMap::new();

        self.get_args_into(&mut args);
        args
    }
    fn get_args_into(&'a self, args: &mut HashMap<&'a T, Vec<ArgType>>)
    where
        T: Deref<Target = str> + Clone;
}

impl<'a, T> TokenSlice<'a, T> for [Token<'_, 'a, T>]
where
    T: Deref<Target = str>
        + Clone
        + for<'b> std::cmp::PartialEq<&'b str>
        + std::cmp::Eq
        + std::hash::Hash,
{
    fn get_args_into(&'a self, args: &mut HashMap<&'a T, Vec<ArgType>>)
    where
        T: Deref<Target = str> + Clone,
    {
        for t in self.iter() {
            match t {
                Token::Content { value: _ } => {}
                Token::PlainArg { arg } => {
                    args.entry(arg).or_default().push(ArgType::PlainArg);
                }
                Token::FunctionArg {
                    arg,
                    key: _,
                    param: _,
                } => {
                    args.entry(arg).or_default().push(ArgType::FunctionArg);
                }
                Token::Plural {
                    arg,
                    cases,
                    plural_offset: _,
                }
                | Token::SelectOrdinal {
                    arg,
                    cases,
                    plural_offset: _,
                } => {
                    args.entry(arg).or_default().push(ArgType::OrdinalArg);
                    for case in cases.iter() {
                        case.tokens.get_args_into(args)
                    }
                }
                Token::Select {
                    arg,
                    cases,
                    plural_offset: _,
                } => {
                    args.entry(arg).or_default().push(ArgType::SelectArg);
                    for case in cases.iter() {
                        case.tokens.get_args_into(args)
                    }
                }
                Token::Octothorpe {} => {}
            };
        }
    }
}
