extern crate proc_macro;

use std::num::FpCategory::Normal;
use std::ops::{Index, Range};
use quote::ToTokens;
use surreal_devl::macro_state::*;
use surreal_devl::macro_state::Trace::NAKED;

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr).value();

    let mut current_state = State::normal();
    let mut out_states: Vec<Content> = vec![];

    for (i, c) in input.char_indices().into_iter() {
        if let State::NORMAL { block_traces } = &mut current_state {
            match c {
                '#' => { current_state = State::NORMAL { block_traces: vec![Trace::NAKED(i as u32)] } },
                _ => {
                    if !block_traces.is_empty() {
                        match c {
                            '(' => {
                                block_traces.push(Trace::BRACKET(i as u32));
                            },
                            _ => {}
                        }

                        if let Trace::NAKED(first_index) = block_traces.first().unwrap() {
                            current_state = State::matching(first_index.to_owned());
                        }
                    }
                },
            };
        }

        if let State::MATCHING { ref mut current, ref mut content_traces, ref mut block_traces } = &mut current_state {
            macro_rules! found_new_content {
                    () => {
                        current.merge_content(c.to_string());
                        match c {
                            '(' => content_traces.push(Trace::BRACKET(i as u32)),
                            _ => {}
                        };
                    };
                }

            if !content_traces.is_empty() {
                if content_traces.last().expect("The last item in content trace should be empty").consume(&c) {
                    content_traces.pop();
                }

                found_new_content!();
            } else {
                if !block_traces.is_empty() {
                    if block_traces.last().expect("The last item in block traces should be empty").consume(&c) {
                        block_traces.pop();
                        if let Some(NAKED(i)) = block_traces.last() {
                            block_traces.pop();
                        }

                    } else {
                        found_new_content!();
                    }
                }

                if block_traces.is_empty() {
                    let mut matched_thing = current.clone();
                    matched_thing.end = Some((i + 1) as u32);
                    current_state = State::MATCHED(matched_thing);
                }
            }
        };

        if let State::MATCHED(c) = &mut current_state {
            out_states.push(c.clone());
            current_state = State::normal();
        }
    }

    let mut out_str = input.clone();
    let values: Vec<String> = out_states.clone().into_iter().map(|it| {
        it.value
    }).collect();

    for out_state in out_states {
        out_str.replace_range(Range { start: out_state.start as usize, end: out_state.end.unwrap() as usize }, "{}" );
    }

    (quote::quote! {
        format!(#out_str, #(#values)*)
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
