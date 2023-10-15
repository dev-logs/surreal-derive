extern crate proc_macro;

use std::fmt::Display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{LitStr, parse_macro_input};
use surreal_devl::config::SurrealDeriveConfig;

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();
    let config = SurrealDeriveConfig::get();

    let mut chars = input.chars();
    let mut output = String::new();
    let mut values = Vec::new();

    while let Some(c) = chars.next() {
        match c {
            '#' => {
                // Handle '#': This is a special case in the input string.
                output.push_str("{}");
                let mut content = String::new();
                'outer: while let Some(c) = chars.next() {
                    match c {
                        '(' => {
                            // Handle the start of a block.
                            content.push('(');
                            let mut depth = 1;
                            while let Some(c) = chars.next() {
                                content.push(c);
                                match c {
                                    '(' => depth += 1,
                                    ')' => {
                                        depth -= 1;
                                        if depth == 0 {
                                            // End of the block.
                                            break 'outer;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        '\'' | '"' => {
                            output.push(c);
                            break;
                        }
                        ';' | ',' => {
                            output.push(c);
                            output = output.trim().to_string();
                            break;
                        }
                        ' ' | '\n' | '\r' | '\t' => {
                            output = output.trim().to_string();
                            output.push(' ');
                            break;
                        },
                        _ => {
                            content.push(c);
                        }
                    }
                }

                values.push(content);
            }
            ' ' | '\n' | '\r' | '\t' => {
                output = output.trim().to_string();
                output.push(' '); // more optimize
            }
            _ => {
                output.push(c);
            }
        }
    }

    output = output.trim().to_owned();

    let values = values.clone().into_iter().map(|it| {
        return syn::parse_str::<TokenStream>(&it).unwrap();
    });

    let log_namespace = config.namespace;
    let log_fn = syn::parse_str::<TokenStream>(config.info_log_macro.as_str()).unwrap();
    let debug_log = match config.enable_log {
        true => {
            quote! {
                #log_fn!("{}: {}", #log_namespace, statement);
            }
        }
        false => {quote! {}}
    };

    return (quote::quote! {{
        use surreal_devl::surreal_statement::*;
        let statement = format!(#output, #(#values),*);
        #debug_log
        statement
    }}).into();
}

#[proc_macro_derive(SurrealDerive)]
pub fn surreal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    let struct_name = &ast.ident;

    let field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        // Check if the field's type is Vec.
        if let syn::Type::Path(type_path) = &field.ty {
            if type_path.path.segments.first().unwrap().ident.to_string() == "Vec" {
                return quote::quote! {
                   let mut array_value: std::vec::Vec<surrealdb::sql::Value> = self.#field_name.iter().map(|v| {
                       surrealdb::sql::Value::from(v)
                   })
                   .collect();

                   vec.push((
                       surrealdb::sql::Idiom::from(stringify!(#field_name).to_owned()), // field name
                       surrealdb::sql::Value::from(array_value)) // value
                   );
               };
            }
        }

        quote::quote! {
            vec.push((
                surrealdb::sql::Idiom::from(stringify!(#field_name).to_owned()), // field name
                surrealdb::sql::Value::from(self.#field_name.clone())) // value
            );
        }
    });

    let into_idiom_value_fn = quote::quote! {
        fn into_idiom_value(&self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> {
           let mut vec: std::vec::Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> = std::vec::Vec::new();
           #(#field_converters)*

           return vec;
        }
    };

    let gen = quote::quote! {
        impl surreal_devl::serialize::SurrealSerialize for #struct_name {
            #into_idiom_value_fn
        }

        impl From<#struct_name> for surrealdb::sql::Value {
            fn from(value: #struct_name) -> Self {
                surrealdb::sql::Value::Thing(value.into())
            }
        }

        impl From<&#struct_name> for surrealdb::sql::Value {
            fn from(value: &#struct_name) -> Self {
               surrealdb::sql::Value::Thing(<#struct_name as Into<surrealdb::sql::Thing>>::into((value.clone())))
            }
        }
    };

    gen.into()
}
