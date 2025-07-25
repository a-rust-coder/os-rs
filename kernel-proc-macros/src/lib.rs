use std::str::FromStr;

use proc_macro::TokenStream;

#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let output = format!("writeln!(logger, {}).unwrap()", input);
    TokenStream::from_str(&output).unwrap()
}
