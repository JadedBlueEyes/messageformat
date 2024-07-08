pub use logos::Span;
use logos::{Lexer, Logos};
use std::borrow::Cow;
use std::ops::Deref;

use crate::ast::{SelectCase, Token as AstToken};

#[cfg(test)]
mod test;

type Result<T> = std::result::Result<T, (String, Span)>;
pub use Span as LexerSpan;
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos()]
enum BodyToken {
    // Body
    /// Escapes to a single `'` apostrophe.
    #[token("''")]
    DoubleApostrophe,

    /// Used to escape `{ } #` when they would otherwise be interpreted as syntax.
    #[regex(r"'[{}#]([^'])*'")]
    // This is slightly incompatible with the JS regex
    // which is `'[{}#](?:[^]*?[^'])?'(?!')`. This uses lazy matching and
    // backtracking, neither of witch Logos supports.
    // This results in "I said '{''Wow!''}'", "I said {'Wow!'}" failing.
    Quote,

    /// Enters the 'argument' lexer context
    #[token(r"{")]
    Argument, // Enter argument context

    /// In a select, is the key, otherwise just a literal `#`
    #[token("#")]
    Octothorpe,

    /// Anything but `{ } # '`, may also match a single quote.
    #[regex(r#"([^\{\}#']+|')"#)]
    Content,

    /// Exits the body context - parser should error if unexpected.
    #[token("}")]
    End, // Exit context
}

type PassLexer<'source, S, T> = (Result<S>, Lexer<'source, T>);

fn parse_body<'source, 'a, T>(
    mut lex: Lexer<'source, BodyToken>,
) -> PassLexer<'source, (Vec<AstToken<'source, 'a, T>>, bool), BodyToken>
where
    T: Deref<Target = str> + Clone + From<&'source str>,
{
    let mut ast: Vec<AstToken<T>> = vec![];
    // lex.extras.push(State::Body);

    while let Some(Ok(token)) = lex.next() {
        match token {
            BodyToken::Argument => {
                let (res, tlex) = parse_arg(lex.morph());
                lex = tlex.morph();
                match res {
                    Ok(Some(t)) => ast.push(t),
                    Ok(None) => {}
                    Err(e) => return (Err(e), lex),
                };
            }
            BodyToken::DoubleApostrophe => ast.push(AstToken::Content {
                value: lex.slice()[0..1].into(),
            }),
            BodyToken::Quote => {
                let slice = lex.slice();
                ast.push(AstToken::Content {
                    value: slice[1..slice.len() - 1].into(),
                })
            }
            BodyToken::Octothorpe => ast.push(AstToken::Octothorpe {}),
            BodyToken::Content => ast.push(AstToken::Content {
                value: lex.slice().into(),
            }),
            BodyToken::End => {
                return (Ok((ast, true)), lex);
            }
        }
    }

    (Ok((ast, false)), lex)
}

// For the regexes, `\p{...}` is a unicode category.
// See:
// - https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Regular_expressions/Unicode_character_class_escape
// - https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Regular_expressions/Character_class (`v` mode)
// - https://www.unicode.org/reports/tr18/#Full_Properties
// - https://www.unicode.org/reports/tr44/
// - https://www.unicode.org/reports/tr31/

// #[derive(Default, Debug)]
// struct ArgTokenState {
//     keywords: bool,
// }

// impl From<()> for ArgTokenState {
//     fn from(value: ()) -> Self {
//         Self::default()
//     }
// }
// impl Into<()> for ArgTokenState {
//     fn into(self) -> () {
//         ()
//     }
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(skip r"\p{Pattern_White_Space}+")] // , extras = ArgTokenState
enum ArgToken {
    // Arguments
    #[token("plural")]
    Plural,
    #[token("select")]
    Select,
    #[token("selectordinal")]
    SelectOrdinal,

    #[token(",")]
    Comma,

    /// An argument identifier
    #[regex(r"[\d\p{ID_Start}][\p{ID_Continue}]*")]
    Ident,

    #[token("}")]
    End, // Exit context
}

fn parse_arg<'source, 'a, T>(
    mut lex: Lexer<'source, ArgToken>,
) -> PassLexer<'source, Option<AstToken<'source, 'a, T>>, ArgToken>
where
    T: Deref<Target = str> + Clone + From<&'source str>,
{
    let mut arg = None;
    let next = lex.next();
    if let Some(Ok(token)) = next {
        // First, we expect an identifier
        match token {
            ArgToken::Ident => arg = Some(lex.slice()),
            // Keywords are identifiers in this context.
            ArgToken::Plural | ArgToken::Select | ArgToken::SelectOrdinal => {
                arg = Some(lex.slice())
            }
            // If we just get a close, we have something like ` { } `,
            // which the user probs didn't mean, but we'll accept anyway
            ArgToken::End => return (Ok(None), lex),
            // Otherwise, we got something unexpected.
            _ => {
                return (
                    Err(("Unexpected token in argument".to_owned(), lex.span())),
                    lex,
                )
            }
        };
    } else {
        // A stand-alone opening bracket?
        dbg!(next, arg);
        if next.is_some() {
            return (
                Err(("Unexpected token in argument".to_owned(), lex.span())),
                lex,
            );
        } else {
            return (
                Err((
                    "Message unexpectedly ended within argument".to_owned(),
                    lex.span(),
                )),
                lex,
            );
        }
    }
    if let Some(Ok(token)) = lex.next() {
        match token {
            ArgToken::End => {
                // Just a simple arg
                if let Some(arg) = arg {
                    return (Ok(Some(AstToken::PlainArg { arg: arg.into() })), lex);
                } else {
                    unreachable!() // At least, it should be...
                }
            }
            ArgToken::Comma => {} // We got some more coming!
            _ => {
                return (
                    Err((
                        "Unexpected token in argument (expected comma or closing bracket)"
                            .to_owned(),
                        lex.span(),
                    )),
                    lex,
                )
            }
        }
    }

    if let Some(Ok(token)) = lex.next() {
        match token {
            select @ (ArgToken::Plural | ArgToken::Select | ArgToken::SelectOrdinal) => {
                let (res, tlex) = parse_select(select, arg.unwrap(), lex.morph());
                lex = tlex.morph();
                match res {
                    Ok(t) => (Ok(Some(t)), lex),
                    // Ok(None) => {}
                    Err(e) => (Err(e), lex),
                }
            }

            ArgToken::Ident => todo!(),

            ArgToken::End => {
                // Just a simple arg, but with an end comma
                if let Some(arg) = arg {
                    (Ok(Some(AstToken::PlainArg { arg: arg.into() })), lex)
                } else {
                    unreachable!() // At least, it should be...
                }
            }
            // ArgToken::Comma => { }, // We got some more coming!
            _ => (
                Err(("Unexpected token in argument".to_owned(), lex.span())),
                lex,
            ),
        }
    } else {
        (Err(("Unexpected end of input".to_owned(), lex.span())), lex)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(skip r"\p{Pattern_White_Space}+")]
enum SelectToken {
    // Select
    #[token("offset")]
    Offset,

    #[token(":")]
    Colon,

    #[regex(r"\d+", priority = 4)]
    Int,

    #[regex(r"[\d\p{ID_Start}][\p{ID_Continue}]*", priority = 2)]
    Ident,

    #[token(",")]
    Comma,

    #[token("{")]
    Open,

    #[token("}")]
    End, // Exit context
}

fn parse_select<'source, 'a, T>(
    parent_type: ArgToken,
    arg: &'source str,
    mut lex: Lexer<'source, SelectToken>,
) -> PassLexer<'source, AstToken<'source, 'a, T>, SelectToken>
where
    T: Deref<Target = str> + Clone + From<&'source str>,
{
    let mut cases = vec![];
    let mut offset = (None, None);
    let mut expect_colon = false;
    let mut expect_comma = true;
    let mut key = None;

    while let Some(Ok(token)) = lex.next() {
        match token {
            SelectToken::Offset => {
                if offset.1.is_none() && !expect_comma {
                    offset.1 = Some(lex.slice());
                    expect_colon = true;
                    expect_comma = true; // This may still be an ident
                } else {
                    return (
                        Err(("Unexpected offset keyword".to_owned(), lex.span())),
                        lex,
                    );
                }
            }
            SelectToken::Colon => {
                if expect_colon {
                    expect_colon = false;
                    expect_comma = false; // Not an ident now
                } else {
                    return (Err(("Unexpected colon".to_owned(), lex.span())), lex);
                }
            }
            SelectToken::Int => {
                if offset.1.is_some() && !expect_colon {
                    // We are expecting an offset
                    match lex.slice().parse::<i32>() {
                        Ok(i) => {
                            offset.0 = Some(i);
                            offset.1 = None
                        }
                        Err(e) => {
                            return (
                                Result::Err((format!("Bad integer: {}", e), lex.span())),
                                lex,
                            )
                        }
                    };
                } else if offset.1.is_none() && !expect_comma && !expect_colon {
                    // this is a key
                    key = Some(lex.slice());
                } else {
                    return (Err(("Unexpected integer".to_owned(), lex.span())), lex);
                }
            }
            SelectToken::Ident => {
                if offset.1.is_none() && !expect_comma && !expect_colon {
                    key = Some(lex.slice());
                } else {
                    return (Err(("Unexpected identifier".to_owned(), lex.span())), lex);
                }
            }
            SelectToken::Comma => {
                if expect_comma {
                    expect_comma = false;
                    expect_colon = false; // No longer expecting offset
                } else {
                    return (Err(("Unexpected comma".to_owned(), lex.span())), lex);
                }
            }
            SelectToken::Open => {
                if let Some(key_inner) = key {
                    let (res, tlex) = parse_body(lex.morph());
                    lex = tlex.morph();
                    match res {
                        Ok((t, true)) => {
                            cases.push(SelectCase {
                                key: key_inner.into(),
                                tokens: Cow::Owned(t),
                            });
                            key = None
                        }
                        Ok((_, false)) => {
                            return (Err(("Unexpected end of input".to_owned(), lex.span())), lex);
                        }
                        // Ok(None) => {}
                        Err(e) => return (Err(e), lex),
                    };
                }
            }
            SelectToken::End => {
                if !expect_colon {
                    return (
                        match parent_type {
                            ArgToken::Plural => {
                                let _token: std::result::Result<AstToken<T>, ()> =
                                    Ok(AstToken::Plural {
                                        arg: arg.into(),
                                        cases: Cow::Owned(vec![]),
                                        plural_offset: offset.0,
                                    });
                                todo!()
                            }
                            ArgToken::SelectOrdinal => {
                                let _token: std::result::Result<AstToken<T>, ()> =
                                    Ok(AstToken::SelectOrdinal {
                                        arg: arg.into(),
                                        cases: Cow::Owned(vec![]),
                                        plural_offset: offset.0,
                                    });
                                todo!()
                            }
                            ArgToken::Select => Ok(AstToken::Select {
                                arg: arg.into(),
                                cases: Cow::Owned(cases),
                                plural_offset: offset.0,
                            }),
                            _ => Err(("Unexpected parent token type".to_owned(), lex.span())),
                        },
                        lex,
                    );
                } else {
                    return (
                        Err(("Unexpected end of select".to_owned(), lex.span())),
                        lex,
                    );
                }
            }
        }
    }
    todo! {}
}

// enum Modes<'source> {
//     BodyToken(Lexer<'source, BodyToken>),
//     ArgToken(Lexer<'source, ArgToken>),
//     SelectToken(Lexer<'source, SelectToken>),
// }

// impl<'source> Modes<'source> {
//     fn new(s: &'source str) -> Self {
//         Self::BodyToken(BodyToken::lexer(s))
//     }
// }

pub fn parse<'source, 'a, T>(src: &'source str) -> Result<Vec<AstToken<'source, 'a, T>>>
where
    T: Deref<Target = str> + Clone + From<&'source str>,
{
    let lex = BodyToken::lexer(src);

    let (res, lex) = parse_body(lex);
    match res {
        Ok((_tok, true)) => Err(("Unexpected end of body".to_owned(), lex.span())),
        Ok((tok, false)) => Ok(tok),

        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod inline_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::test::*;
    use super::*;
    use crate::parser::SelectCase;
    use crate::Token;
    // use crate::{ast::SelectCase, ast::Token};

    macro_rules! parse_assert {
        ( $src:literal, $( $i:expr ),* ) => {
            {
                assert_eq!(
                    parse_ui($src),
                    vec![
                        $(
                            parse_assert! (token, $i)
                        ),+
                    ]
                );
            }
        };
        ( token, $str:literal ) => {
            crate::ast::Token::Content {
                        value: $str
            }
        };

        ( token, $tree:expr ) => {
            $tree
        }
    }

    macro_rules! parse_assert_concat {
        ( $src:literal, $res:literal ) => {{
            let res = parse::<&str>($src).unwrap();
            let text: String = res
                .iter()
                .map(|t| match t {
                    Token::Content { value } => *value,
                    _ => panic!(),
                })
                .collect();
            assert_eq!(&text, $res);
        }};
    }

    #[test]
    fn test_body_simple() {
        parse_assert!("This is a message", "This is a message");
    }

    // This test is wrong - it should all be content in the original impl.
    #[test]
    fn test_body_octothorpe() {
        parse_assert!(
            "This is # an octothorpe",
            "This is ",
            Token::Octothorpe {},
            " an octothorpe"
        );
    }

    #[test]
    fn test_body_doublequote() {
        parse_assert_concat!("This is a doublequote: ''", "This is a doublequote: '");
    }

    #[test]
    fn test_body_quote_escape() {
        parse_assert_concat!(
            "This is an '{escaped}' string, with some more escapes: '{', '}'",
            "This is an {escaped} string, with some more escapes: {, }"
        );
    }

    #[test]
    fn test_body_quote_no_escape() {
        parse_assert_concat!("This is a 'quoted' string", "This is a 'quoted' string");
    }

    #[test]
    #[should_panic]
    fn test_body_unexpected_close() {
        let _ = parse::<&str>("This is an unexpected close: }").unwrap();
    }

    #[test]
    fn test_arg_simple() {
        parse_assert!(
            "This is a {simple} replace.",
            "This is a ",
            Token::PlainArg { arg: "simple" },
            " replace."
        );
    }

    #[test]
    fn test_arg_keyword() {
        parse_assert!(
            "This has a keyword {select} replace.",
            "This has a keyword ",
            Token::PlainArg { arg: "select" },
            " replace."
        );
    }

    #[test]
    fn test_arg_select() {
        parse_assert!(
            "This is a {varname, select, this{...} that{...} other{...}}",
            "This is a ",
            Token::Select {
                arg: "varname",
                plural_offset: None,
                cases: vec![
                    SelectCase {
                        key: "this",
                        tokens: vec![Token::Content { value: "..." }].into()
                    },
                    SelectCase {
                        key: "that",
                        tokens: vec![Token::Content { value: "..." }].into()
                    },
                    SelectCase {
                        key: "other",
                        tokens: vec![Token::Content { value: "..." }].into()
                    }
                ]
                .into()
            }
        );
    }
}
