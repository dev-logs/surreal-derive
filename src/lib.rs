extern crate proc_macro;
use surreal_devl::macro_state::*;
#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr);
    let mut current_state= surreal_devl::macro_state::State::NORMAL;
    let mut out_states: Vec<surreal_devl::macro_state::Content> = vec![];

    for (i, c) in input.value().char_indices().into_iter() {
        current_state = match &mut current_state {
            surreal_devl::macro_state::State::NORMAL => match c {
                '#' => State::MATCHING {
                    current: Content::new(String::from(""), i as u32),
                    traces: vec![Trace::NAKED]
                },
                _ => State::NORMAL,
            },
            State::MATCHING {ref mut current, ref mut traces } => match &(traces.last().unwrap()).consume(&c) {
                true => {
                    traces.pop();
                    if traces.is_empty() {
                        let mut matched_thing = current.clone();
                        matched_thing.end = Some(i as u32);
                        // State::MATCHED(matched_thing)
                    }

                    current_state
                },
                false => {
                    current.merge_content(c.to_string());
                    match c {
                        '(' => traces.push(Trace::BRACKET),
                        _ => {}
                    }

                    State::MATCHING {
                        current: current.to_owned(), traces: traces.to_owned()
                    }
                }
            },
            _ => current_state
        };

        current_state = match current_state {
            State::MATCHED(c) => {
                out_states.push(c.clone());
                State::NORMAL
            },
            _ => current_state
        };
    }

    proc_macro::TokenStream::new()
}

#[proc_macro_derive(surreal_derive)]
pub fn create(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro::TokenStream::from(input);

    let ast: syn::ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    let struct_name = &ast.ident;

    let field_convert_btreemap_quote = ast.fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        return quote::quote! {
            map.insert("#field_name".to_owned(), surrealdb::sql::Value::from(self.#field_name.clone()));
        };
    });

    let field_convert_set_expressions_quote: std::vec::Vec<proc_macro2::TokenStream> = vec![("create", "Equal"), ("update", "Equal")].iter().map(|(fn_name, operator_name)| {
        let fn_ident = syn::Ident::new(&format!("into_{}_expressions", fn_name), proc_macro2::Span::call_site());
        let operator_ident = syn::Ident::new(operator_name, proc_macro2::Span::call_site());

        let field_quotes = ast.fields.iter().map(|field| {
            let field_name = field.ident.as_ref().unwrap();

            return match field.ty {
                syn::Type::Path(..) => {
                    quote::quote! {
                        vec.push((
                            surrealdb::sql::Idiom::from(stringify!(#field_name).to_owned()), // field name
                            surrealdb::sql::Operator::#operator_ident, // operator
                            surrealdb::sql::Value::from(self.#field_name)) // value
                        );
                    }
                },
                _ => {
                    quote::quote! {
                        vec.push((
                            surrealdb::sql::Idiom::from(stringify!(#field_name).to_owned()), // field name
                            surrealdb::sql::Operator::#operator_ident, // operator
                            surrealdb::sql::Value::from(self.#field_name)) // value
                        );
                    }
                }
            };
        });

        return quote::quote! {
            pub fn #fn_ident(self) -> std::vec::Vec<(surrealdb::sql::Idiom, surrealdb::sql::Operator, surrealdb::sql::Value)> {
               let mut vec: std::vec::Vec<(surrealdb::sql::Idiom, surrealdb::sql::Operator, surrealdb::sql::Value)> = std::vec::Vec::new();
               #(#field_quotes)*

               return vec;
           }
        };
    }).collect();

    let gen = quote::quote! {
        impl std::convert::Into<std::collections::BTreeMap<String, surrealdb::sql::Value>> for #struct_name {
            fn into(self) -> std::collections::BTreeMap<String, surrealdb::sql::Value> {
                let mut map: std::collections::BTreeMap<String, surrealdb::sql::Value> = std::collections::BTreeMap::new();
                #(#field_convert_btreemap_quote)*
                return map;
            }
        }

        impl #struct_name {
           #(#field_convert_set_expressions_quote)*
        }
    };

    gen.into()
}

/*
    surreal_ql! {
        begin_transaction!()
        create!(issuer)
        create!(token)
        update!()
        commit_transaction!()
    }
*/
