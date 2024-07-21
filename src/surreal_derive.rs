use quote::quote;
use surreal_devl::config::SurrealDeriveConfig;
use surreal_devl::naming_convention::{camel_to_snake_case, snake_case_to_camel};
use syn::Fields;

pub fn surreal_derive_process_enum(ast: syn::ItemEnum) -> proc_macro::TokenStream {
    let enum_name = &ast.ident;

    let insert_to_every_variant = |id_token: proc_macro2::TokenStream, object_token: proc_macro2::TokenStream| {
        let mut matches = vec![];
        if ast.variants.iter().count() != 2 {
            panic!("The enum must have exact two state");
        }

        for variant in &ast.variants {
            let ident = &variant.ident;
            if ident.clone().to_string() != "Object" && ident.clone().to_string() != "Id" {
                panic!("The enum must have exact two state id and object got {:?}", ident.to_string());
            }

            let fields = &variant.fields;
            if let Fields::Unnamed(supported_enum) = fields {
                if supported_enum.unnamed.iter().count() != 1 {
                    panic!("Only supported Unnamed enum with single value");
                }
            } else {
                panic!("Only supported Unnamed enum with single value");
            }

            matches.push(quote::quote! {
                #enum_name::Id(param) => {
                    #id_token
                },
                #enum_name::Object(param) => {
                  #object_token
                }
            });
        }

        quote! {
            match value {
                #(#matches)*
            }
        }
    };

    let quote1 = insert_to_every_variant(
        quote::quote! {param.into_idiom_value()},
        quote::quote! {param.into_idiom_value()}
    );

    let quote2 = insert_to_every_variant(
        quote::quote! {param.into_idiom_value()},
        quote::quote! {param.into_idiom_value()},
    );

    let quote3 = insert_to_every_variant(
        quote! {surrealdb::sql::Value::Thing(param.into())},
        quote! {surrealdb::sql::Value::Thing(param.into())}
    );

    let quote4 = insert_to_every_variant(
        quote! {surrealdb::sql::Value::Thing(param.into())},
        quote! {surrealdb::sql::Value::Thing(param.into())}
    );

    let quote5 = insert_to_every_variant(
        quote! {param.into()},
        quote! {param.into()}
    );

    let quote6 = insert_to_every_variant(
        quote! {surrealdb::sql::Thing::from(Into::<surrealdb::opt::RecordId>::into(param))},
        quote! {surrealdb::sql::Thing::from(Into::<surrealdb::opt::RecordId>::into(param))},
    );

    let gen = quote::quote! {
        impl From<surrealdb::sql::Value> for #enum_name {
            fn from(value: surrealdb::sql::Value) -> Self {
                match value {
                    surrealdb::sql::Value::Thing(_) => {
                        return Self::Id(value.into());
                    },
                    surrealdb::sql::Value::Object(_) => {
                        return Self::Object(value.into());
                    },
                    _ => {
                        panic!("Unsupported type != Thing and Object");
                    }
                }
            }
        }

        impl From<Option<surrealdb::sql::Value>> for #enum_name {
            fn from(value: Option<surrealdb::sql::Value>) -> Self {
                Self::from(value.unwrap())
            }
        }

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
                let value = value;
                #quote4
            }
        }

        impl Into<surrealdb::opt::RecordId> for &#enum_name {
            fn into(self) -> surrealdb::opt::RecordId {
                let value = self;
                #quote5
            }
        }

        impl From<#enum_name> for surrealdb::sql::Thing {
           fn from(value: #enum_name) -> Self {
               #quote6
           }
        }
    };

    gen.into()
}

pub fn surreal_derive_process_struct(ast: syn::ItemStruct) -> proc_macro::TokenStream {
    let config = SurrealDeriveConfig::get();
    let struct_name = &ast.ident;

    let from_value_field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Failed to process variable name, the ident could not be empty");
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str())
        };

        if let syn::Type::Path(type_path) = &field.ty {
            let type_name = type_path.path.segments.iter().map(|it| {
                it.ident.to_string()
            }).collect::<Vec<_>>().join("::");
            match type_name.as_str() {
                "Option" | "core::option::Option" => {
                    return quote! {
                        #field_name: value_object.get_mut(#db_name).take().map(|it| it.to_owned().try_into().unwrap()).into(),
                    }
                },
                "Vec" => {
                    return quote! {
                        #field_name: match value_object.get_mut(#db_name) {
                            Some(surrealdb::sql::Value::Array(ref mut array_obj)) => {
                                array_obj.to_owned().into_iter().map(|item_obj| item_obj.to_owned().into()).collect()
                            },
                            _ => {
                                panic!("Expected an array");
                            },
                        }
                    }
                },
                _ => {}
            };
        }

        return quote! {
            #field_name: value_object.get_mut(#db_name).take().map(|it| it.to_owned().try_into().unwrap()).unwrap(),
        };
    });

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
            if type_path.path.segments.iter().find(|s| s.ident.to_string() == "Option").is_some() {
                return quote::quote! {
                   vec.push((
                       surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                       self.#field_name.clone().map(|it| surrealdb::sql::Value::from(it)).unwrap_or(surrealdb::sql::Value::None))
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

    let from_value_field_converters_i1 = from_value_field_converters.clone();
    let from_value_field_converters_i2 = from_value_field_converters.clone();

    let gen = quote::quote! {
        impl From<surrealdb::sql::Object> for #struct_name {
            fn from(mut value_object: surrealdb::sql::Object) -> Self {
                return Self {
                    #(#from_value_field_converters_i1)*
                }
            }
        }

        impl From<surrealdb::sql::Value> for #struct_name {
            fn from(value: surrealdb::sql::Value) -> Self {
                let mut value_object = match value {
                    surrealdb::sql::Value::Object(mut value_object) => {
                        value_object
                    }
                    surrealdb::sql::Value::Array(surrealdb::sql::Array(array)) => {
                        if let surrealdb::sql::Value::Object(mut value_object) = array.first().take().unwrap().to_owned() {
                            value_object
                        }
                        else {
                            panic!("Expected an object or array with one item is object");
                        }
                    }
                    _ => {
                        panic!("Expected an object or array with one item")
                    }
                };

                return Self {
                    #(#from_value_field_converters_i2)*
                }
            }
        }

        impl From<Option<surrealdb::sql::Value>> for #struct_name {
            fn from(value: Option<surrealdb::sql::Value>) -> Self {
                Self::from(value.unwrap())
            }
        }

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
