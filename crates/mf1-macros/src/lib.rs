use proc_macro2;
use t_macro::OutputType;

mod load_locales;
mod t_macro;

#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales() {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}
#[proc_macro]
pub fn t_l_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens_2 = proc_macro2::TokenStream::from(tokens);
    match t_macro::t_macro(tokens_2, OutputType::String) {
        Ok(ts) => proc_macro::TokenStream::from(ts),
        Err(err) => err.into(),
    }
}
