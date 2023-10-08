extern crate proc_macro;

use std::ops::Range;
use quote::ToTokens;
use surreal_devl::macro_state::*;

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr).value();

    let mut current_state = State::NORMAL;
    let mut out_states: Vec<Content> = vec![];

    for (i, c) in input.char_indices().into_iter() {
        current_state = match &mut current_state {
            State::NORMAL => match c {
                '#' => State::MATCHING {
                    current: Content::new(String::from(""), i as u32),
                    traces: vec![Trace::NAKED],
                },
                _ => State::NORMAL,
            },
            State::MATCHING { ref mut current, ref mut traces } => match &(traces.last().expect("The traces must not be empty")).consume(&c) {
                true => {
                    traces.pop();
                    match traces.is_empty() {
                        true => {
                            let mut matched_thing = current.clone();
                            matched_thing.end = Some(i as u32);
                            State::MATCHED(matched_thing)
                        }
                        false => current_state
                    }
                }
                false => {
                    current.merge_content(c.to_string());
                    match c {
                        '(' => traces.push(Trace::BRACKET),
                        _ => {}
                    }

                    State::MATCHING {
                        current: current.to_owned(),
                        traces: traces.to_owned(),
                    }
                }
            },
            _ => current_state
        };

        current_state = match current_state {
            State::MATCHED(c) => {
                out_states.push(c.clone());
                State::NORMAL
            }
            _ => current_state
        };
    };

    let mut out_str = input.clone();
    for out_state in out_states {
        out_str.replace_range(Range { start: out_state.start as usize, end: out_state.end.unwrap() as usize }, "{}");
    }

    (quote::quote! {
        format!(#out_str)
    }).into()
}

#[proc_macro_derive(surreal_derive)]
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
        fn into_idiom_value(self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> {
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
