use quote::quote;
use surreal_devl::config::SurrealDeriveConfig;
use surreal_devl::naming_convention::{camel_to_snake_case, snake_case_to_camel};

use crate::attributes::SurrealDeriveAttribute;

pub fn surreal_derive_process_struct(
    ast: syn::ItemStruct,
    attributes: SurrealDeriveAttribute,
) -> proc_macro::TokenStream {
    let config = SurrealDeriveConfig::get();
    let struct_name = &ast.ident;

    let from_object_field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Failed to process variable name, the ident could not be empty");
        let field_type = &field.ty;
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str())
        };

        return quote! {
            #field_name: <#field_type as surreal_devl::proxy::default::SurrealDeserializer>::from_option(value_object.get(#db_name).take()),
        };
    });

    let into_object_field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Failed to process variable name, the ident could not be empty");
        let field_type = &field.ty;
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str())
        };

        return quote! {
            map.insert(#db_name.to_owned(), <#field_type as surreal_devl::proxy::default::SurrealSerializer>::serialize(value.#field_name.clone()));
        };
    });

    let field_converters = ast.fields.iter().map(|field| {
        let field_name = field
            .ident
            .as_ref()
            .expect("Failed to process variable name, the ident could not be empty");
        let field_type = &field.ty;
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str()),
        };

        quote::quote! {
            vec.push((
                surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                <#field_type as surreal_devl::proxy::default::SurrealSerializer>::serialize(self.#field_name.clone())
            ));
        }
    });

    let into_idiom_value_fn = quote::quote! {
        fn into_idiom_value(&self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> {
           let mut vec: std::vec::Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)> = std::vec::Vec::new();
           #(#field_converters)*

           return vec;
        }
    };

    let from_object = {
        quote::quote! {
            impl From<surrealdb::sql::Object> for #struct_name {
                fn from(mut value_object: surrealdb::sql::Object) -> Self {
                    return Self {
                        #(#from_object_field_converters)*
                    }
                }
            }
        }
    };

    let into_object = {
        quote::quote! {
            impl From<#struct_name> for surrealdb::sql::Object {
                fn from(mut value: #struct_name) -> Self {
                    let mut map: std::collections::BTreeMap<String, surrealdb::sql::Value> = std::collections::BTreeMap::new();
                    #(#into_object_field_converters)*

                    return Self::from(map)
                }
            }
        }
    };

    let impl_surreal_serialize = {
        quote::quote! {
            impl surreal_devl::serialize::SurrealSerialize for #struct_name {
                #into_idiom_value_fn
            }
        }
    };

    let impl_surreal_serialize_ref = {
        quote::quote! {
            impl surreal_devl::serialize::SurrealSerialize for &#struct_name {
                #into_idiom_value_fn
            }
        }
    };

    let gen = quote::quote! {
        #from_object

        #into_object

        impl surreal_devl::proxy::default::SurrealDeserializer for #struct_name {
            fn deserialize(value: &surrealdb::sql::Value) -> Self {
                match value {
                    surrealdb::sql::Value::Object(obj) => {
                        Self::from(obj.clone())
                    },
                    _ => panic!("Expected an object")
                }
            }
        }

        impl surreal_devl::proxy::default::SurrealSerializer for #struct_name {
            fn serialize(self) -> surrealdb::sql::Value {
                let obj: surrealdb::sql::Object = self.into();
                surrealdb::sql::Value::Object(obj)
            }
        }

        #impl_surreal_serialize

        #impl_surreal_serialize_ref
    };

    gen.into()
}
