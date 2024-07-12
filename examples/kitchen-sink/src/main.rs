use mf1::{load_locales, t_l_string as t};

load_locales!();

fn main() {
    dbg!(Locale::default());
    dbg!(Locale::en, Locale::en.get_strings());
    dbg!(Locale::es, Locale::es.get_strings());
    println!("{}", t!(Locale::en, message))
}

#[cfg(test)]
use expect_test::{expect, Expect};

#[cfg(test)]
fn check(actual: std::borrow::Cow<'static, str>, expect: &Expect) {
    let actual = actual.to_string();
    expect.assert_eq(&actual);
}

#[test]
fn default_lang() {
    assert_eq!(Locale::default(), Locale::en)
}

#[test]
fn basic_strings() {
    check(t!(Locale::en, message), &expect!["This is a message!"]);
    check(t!(Locale::es, message), &expect!["¡Este es un mensaje!"]);
}

#[test]
fn interpolation() {
    let version = "2000";
    check(
        t!(Locale::en, interpolated_2, version),
        &expect!["Frobnicator 2000"],
    );
    check(
        t!(Locale::es, interpolated_2, version),
        &expect!["Frobnicador 2000"],
    );
}

#[test]
fn fallbacks_string() {
    let actual = expect!["This is a second message!"];
    check(t!(Locale::en, message_2), &actual);
    assert_eq!(t!(Locale::es, message_2), t!(Locale::en, message_2));
}

#[test]
fn fallbacks_dynamic() {
    let string = "a string.";
    let actual = expect!["This has been interpolated with a string."];
    check(t!(Locale::en, interpolated, var = string), &actual);
    assert_eq!(
        t!(Locale::es, interpolated, var = string),
        t!(Locale::en, interpolated, var = string)
    );
}
