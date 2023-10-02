extern crate proc_macro;

#[proc_macro_derive(surreal_derive)]
pub fn create(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro::TokenStream::from(input);

    let ast: syn::ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    let struct_name = &ast.ident;

    let mut final_tokens = proc_macro2::TokenStream::new();

    let field_convert_quote = ast.fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        return quote::quote! {
            map.insert("#field_name".to_owned(), surrealdb::sql::Value::from(self.#field_name.clone()));
        };
    });

    let gen = quote::quote! {
        impl #struct_name {
            pub fn into_btreemap(&self) -> std::collections::BTreeMap<String, surrealdb::sql::Value> {
                let mut map: std::collections::BTreeMap<String, surrealdb::sql::Value> = std::collections::BTreeMap::new();
                #(#field_convert_quote)*
                return map;
            }
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
