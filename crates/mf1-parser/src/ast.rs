use icu_plurals::PluralCategory;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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
