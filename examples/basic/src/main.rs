use std::str::FromStr;

use mf1::load_locales;

load_locales!();

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let t = args
        .get(1)
        .map(|l| Locale::from_str(l).unwrap())
        .unwrap_or_default()
        .get_strings();
    println!("{}", t.message);
    println!("{}", t.message_2);
}
