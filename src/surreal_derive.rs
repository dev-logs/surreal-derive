use quote::quote;
use surreal_devl::config::SurrealDeriveConfig;
use surreal_devl::naming_convention::{camel_to_snake_case, snake_case_to_camel};
use syn::Fields;

pub fn surreal_derive_process_enum(ast: syn::ItemEnum) -> proc_macro::TokenStream {
    let enum_name = &ast.ident;

    let insert_to_every_variant = |token: proc_macro2::TokenStream| {
        let mut matches = vec![];
        for variant in &ast.variants {
            let ident = &variant.ident;
            let fields = &variant.fields;
            if let Fields::Unnamed(supported_enum) = fields {
                if supported_enum.unnamed.iter().count() != 1 as usize {
                    panic!("Only supported Unnamed enum with single value");
                }
            }
            else {
                panic!("Only supported Unnamed enum with single value");
            }

            matches.push(quote::quote! {
                #enum_name::#ident(param) => #token,
            });
        }

        quote! {
            match value {
                #(#matches)*
            }
        }
    };

    let quote1 = insert_to_every_variant(quote::quote! {
      param.into_idiom_value()
    });

    let quote2 = insert_to_every_variant(quote::quote! {
      param.into_idiom_value()
    });

    let quote3 = insert_to_every_variant(quote! {
      surrealdb::sql::Value::Thing(param.into())
    });

    let quote4 = insert_to_every_variant(quote! {
        surrealdb::sql::Value::Thing(param.clone().into())
    });

    let quote5 = insert_to_every_variant(quote! {
        param.clone().into()
    });

    let gen = quote::quote! {
        impl surreal_devl::serialize::SurrealSerialize for #enum_name {
            fn into_idiom_value(&self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> {
                let value = self;
                #quote1
            }
        }

        impl surreal_devl::serialize::SurrealSerialize for &#enum_name {
            fn into_idiom_value(&self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> {
                let value = self;
                #quote2
            }
        }

        impl From<#enum_name> for surrealdb::sql::Value {
            fn from(value: #enum_name) -> Self {
              #quote3
            }
        }

        impl From<&#enum_name> for surrealdb::sql::Value {
            fn from(value: &#enum_name) -> Self {
                #quote4
            }
        }

        impl Into<surrealdb::opt::RecordId> for &#enum_name {
            fn into(self) -> surrealdb::opt::RecordId {
                let value = self;
                #quote5
            }
        }
    };

    gen.into()
}

pub fn surreal_derive_process_struct(ast: syn::ItemStruct) -> proc_macro::TokenStream {
    let config = SurrealDeriveConfig::get();
    let struct_name = &ast.ident;

    let field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Failed to process variable name, the ident could not be empty");
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str())
        };

        // Check if the field's type is Vec.
        if let syn::Type::Path(type_path) = &field.ty {
            if type_path.path.segments.first().unwrap().ident.to_string() == "Vec" {
                return quote::quote! {
                   let mut array_value: std::vec::Vec<surrealdb::sql::Value> = self.#field_name.iter().map(|v| {
                       surrealdb::sql::Value::from(v)
                   })
                   .collect();

                   vec.push((
                       surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                       surrealdb::sql::Value::from(array_value)) // value
                   );
               };
            }
            if type_path.path.segments.iter().find(|s| s.ident.to_string() == "Duration").is_some() {
                return quote::quote! {
                   vec.push((
                       surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                       surrealdb::sql::Value::from(surrealdb::sql::Duration::from(self.#field_name.clone()))) // value
                   );
               };
            }
        }

        quote::quote! {
            vec.push((
                surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
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

        impl surreal_devl::serialize::SurrealSerialize for &#struct_name {
            #into_idiom_value_fn
        }

        impl From<#struct_name> for surrealdb::sql::Value {
            fn from(value: #struct_name) -> Self {
                surrealdb::sql::Value::Thing(value.into())
            }
        }

        impl From<&#struct_name> for surrealdb::sql::Value {
            fn from(value: &#struct_name) -> Self {
               surrealdb::sql::Value::Thing(value.clone().into())
            }
        }

        impl Into<surrealdb::opt::RecordId> for &#struct_name {
            fn into(self) -> surrealdb::opt::RecordId {
                self.clone().into()
            }
        }
    };

    gen.into()
}
