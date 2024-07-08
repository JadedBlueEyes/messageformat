use crate::ast::{SelectCase, Token};
use crate::parser::parse;

pub(crate) fn parse_ui(src: &str) -> Vec<Token<&str>> {
    match parse(src) {
        Ok(value) => value,
        Err((msg, span)) => {
            panic!(
                "Panicked at input {:?} ({:?}): {}\n\n{}",
                src.get(span.start..span.end),
                span,
                msg,
                src
            );
            // use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

            // let mut colors = ColorGenerator::new();

            // let a = colors.next();

            // Report::build(ReportKind::Error, (), 12)
            //     .with_message(format!("Invalid Message"))
            //     .with_label(Label::new(span).with_message(msg).with_color(a))
            //     .finish()
            //     .print(Source::from(src))
            //     .unwrap();
            // panic!();
        }
    }
}

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
        let res = parse::<&str>(&$src).unwrap();
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
fn string_only() {
    parse_assert!("This is a string", "This is a string");
    parse_assert!("☺☺☺☺", "☺☺☺☺");
    parse_assert!("This is \n a string", "This is \n a string");
    parse_assert!("中国话不用彁字。", "中国话不用彁字。");
    parse_assert!(" \t leading whitespace", " \t leading whitespace");
    parse_assert!("trailing whitespace   \n  ", "trailing whitespace   \n  ");
}

#[test]
fn escape_curly_brackets() {
    // TODO: Merge contiguous content spans?
    parse_assert!("'{'test", "{", "test");
    parse_assert!("test'}'", "test", "}");
    parse_assert!("'{test}'", "{test}");
}

#[test]
fn handle_quotes() {
    parse_assert!("This is a dbl quote: \"", "This is a dbl quote: \"");
    parse_assert!("This is a single quote: '", "This is a single quote: ", "'");
}

#[test]
fn keywords_in_body() {
    parse_assert!("select select, ", "select select, ");
    parse_assert!("select offset, offset:1 ", "select offset, offset:1 ");
    parse_assert!("one other, =1 ", "one other, =1 ");
}

#[test]
fn keywords_in_arg() {
    parse_assert!(
        "one {select} ",
        "one ",
        Token::PlainArg { arg: "select" },
        " "
    );
    parse_assert!(
        "one {plural} ",
        "one ",
        Token::PlainArg { arg: "plural" },
        " "
    );
}

#[test]
fn apostrophes_in_body() {
    parse_assert_concat!("I see '{many}'", "I see {many}");
    // parse_assert_concat!("I said '{''Wow!''}'", "I said {'Wow!'}"); // Bad regex, see the token.
    parse_assert_concat!("I said '{''Wow!'''}'", "I said {'Wow!'}"); // Bad regex, see the token.
    parse_assert_concat!("I don't know", "I don't know");
    parse_assert_concat!("I don''t know", "I don't know");
    parse_assert_concat!("A'a''a'A", "A'a'a'A");
    // parse_assert_concat!("A '#' A", "A '#' A"); // Aaa are we supposed to lex differently based in in/out of select!?
    // In the end it doesn't matter - out of select, octothorpe is just a literal when we format.
    parse_assert_concat!("A '|' A", "A '|' A");
}

#[test]
fn arg_single() {
    parse_assert!("{test}", Token::PlainArg { arg: "test" });
    parse_assert!("{0}", Token::PlainArg { arg: "0" });
}

#[test]
fn arg_whitespace() {
    let res = Token::PlainArg { arg: "test" };
    parse_assert!("{test}", res.clone());
    parse_assert!("{test }", res.clone());
    parse_assert!("{test  }", res.clone());
    parse_assert!("{  \ttest}", res.clone());
    parse_assert!("{test \t\n}", res.clone());
    parse_assert!("{ \n  test  \n\n}", res.clone());
}

#[test]
fn arg_body_whitespace() {
    let res = Token::PlainArg { arg: "test" };
    parse_assert!("x{test}", "x", res.clone());
    parse_assert!("\n{test }", "\n", res.clone());
    parse_assert!(" {test  }", " ", res.clone());
    parse_assert!("x {  \ttest}", "x ", res.clone());
    parse_assert!("x{test \t\n} x  ", "x", res.clone(), " x  ");
    parse_assert!("x\n{ \n  test  \n\n}\n", "x\n", res.clone(), "\n");
}

#[test]
fn arg_body_unicode() {
    let res = Token::PlainArg { arg: "test" };
    parse_assert!("☺{test}", "☺", res.clone());
    parse_assert!(
        "中{test }中国话不用彁字。",
        "中",
        res.clone(),
        "中国话不用彁字。"
    );
}

#[test]
fn arg_body_html() {
    let res = Token::PlainArg { arg: "test" };
    parse_assert!(
        "<div class=\"test\">content: {test}</div>",
        "<div class=\"test\">content: ",
        res.clone(),
        "</div>"
    );
}

#[test]
fn select_whitespace_agnostic() {
    let res = Token::Select {
        arg: "VAR",
        plural_offset: None,
        cases: vec![
            SelectCase {
                key: "key",
                tokens: vec![Token::Content { value: "a" }].into(),
            },
            SelectCase {
                key: "other",
                tokens: vec![Token::Content { value: "b" }].into(),
            },
        ]
        .into(),
    };
    parse_assert!("{VAR,select,key{a}other{b}}", res.clone());
    parse_assert!(
        "{    VAR   ,    select   ,    key      {a}   other    {b}    }",
        res.clone()
    );
    parse_assert!(
        "{ \n   VAR  \n , \n   select  \n\n , \n \n  key \n    \n {a}  \n other \n   {b} \n  \n }",
        res.clone()
    );
    parse_assert!(
        "{ \t  VAR  \n , \n\t\r  select  \n\t , \t \n  key \n    \t {a}  \n other \t   {b} \t  \t }",
        res.clone()
    );
}
