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
    match t_macro::t_macro(tokens.into(), OutputType::String) {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}
