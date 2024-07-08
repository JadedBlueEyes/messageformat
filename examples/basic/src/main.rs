use std::str::FromStr;

use mf1::{load_locales, t_l_string as t};

load_locales!();

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let l = args
        .get(1)
        .map(|l| Locale::from_str(l).unwrap())
        .unwrap_or_default();
    println!("{}", t!(l, message));
    println!("{}", t!(l, message_2));
}
