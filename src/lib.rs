
extern crate proc_macro;
mod surreal_quote;
mod surreal_derive;

use std::fmt::Display;
use quote::ToTokens;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();
    surreal_quote::surreal_quote(input)
}

#[proc_macro_derive(SurrealDerive)]
pub fn surreal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    surreal_derive::surreal_derive_process(ast)
}
