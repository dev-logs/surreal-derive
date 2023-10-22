use std::any::{Any, TypeId};
use surreal_devl::config::SurrealDeriveConfig;
use proc_macro2::{TokenStream};
use quote::quote;

pub fn surreal_quote(input: String) -> proc_macro::TokenStream {
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
        let result = syn::parse_str::<TokenStream> (&it);
        match result.unwrap().type_id() {
            TypeId::of::<surreal_devl::surreal_statement::content<i32>>() => {}
            TypeId { t } => {}
        };

        return syn::parse_str::<TokenStream>(format!("surrealdb::sql::Value::from({})", &it).as_str()).unwrap();
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

    let output: proc_macro::TokenStream = (quote::quote! {{
        use surreal_devl::surreal_statement::*;
        let statement = format!(#output, #(#values),*);
        #debug_log
        statement
    }}).into();

    if config.enable_compile_log {
        println!("DEBUG: {}  {}", log_namespace, output);
    }

    output
}