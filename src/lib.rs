extern crate proc_macro;
mod attributes;
mod surreal_derive;
mod surreal_quote;
use attributes::SurrealDeriveAttribute;
use darling::FromDeriveInput;
use syn::{parse_macro_input, Data, LitStr};

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();
    surreal_quote::surreal_quote(input)
}

#[proc_macro_derive(SurrealDerive, attributes(surreal_derive))]
pub fn surreal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let usage_input = input.clone();
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let attributes = SurrealDeriveAttribute::from_derive_input(&derive_input).unwrap_or_default();

    if let Data::Enum(_) = derive_input.data {
        panic!("#[derive(SurrealDerive)] only works for struct");
    }
    else {
        let ast: syn::ItemStruct = syn::parse_macro_input!(usage_input as syn::ItemStruct);
        surreal_derive::surreal_derive_process_struct(ast, attributes)
    }
}
