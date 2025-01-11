use quote::{quote, format_ident};
use surreal_devl::config::SurrealDeriveConfig;
use surreal_devl::naming_convention::{camel_to_snake_case, snake_case_to_camel};

use crate::attributes::SurrealDeriveAttribute;

pub fn surreal_derive_process_struct(
    ast: syn::ItemStruct,
    _attributes: SurrealDeriveAttribute,
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
            #field_name: <#field_type as surreal_devl::proxy::default::SurrealDeserializer>::from_option(value_object.get(#db_name).clone())?,
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

    let from_object = {
        quote::quote! {
            impl TryFrom<&surrealdb::sql::Object> for #struct_name {
                type Error = surreal_devl::surreal_qr::SurrealResponseError;
                fn try_from(mut value_object: &surrealdb::sql::Object) -> Result<Self, Self::Error> {
                    return Ok(Self {
                        #(#from_object_field_converters)*
                    })
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

    let gen = quote::quote! {
        #from_object

        #into_object

        impl surreal_devl::proxy::default::SurrealDeserializer for #struct_name {
            fn deserialize(value: &surrealdb::sql::Value) -> Result<Self, surreal_devl::surreal_qr::SurrealResponseError> {
                let object = match &value {
                    surrealdb::sql::Value::Object(ref value) => value,
                    surrealdb::sql::Value::Array(ref value) => {
                        if value.len() != 1 {
                            return Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnArrayWith1ItemToDeserializeToObject)
                        }
                        else if let Some(surrealdb::sql::Value::Object(ref obj)) = value.0.first() {
                            obj
                        }
                        else {
                            return Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnObject)
                        }
                    }
                    _ => return Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnObject),
                };

                Ok(Self::try_from(object)?)
            }
        }

        impl surreal_devl::proxy::default::SurrealSerializer for #struct_name {
            fn serialize(self) -> surrealdb::sql::Value {
                let obj: surrealdb::sql::Object = self.into();
                surrealdb::sql::Value::Object(obj)
            }
        }
    };

    gen.into()
}

pub fn surreal_derive_process_enum(
    ast: syn::ItemEnum,
    attributes: SurrealDeriveAttribute,
) -> proc_macro::TokenStream {
    let config = SurrealDeriveConfig::get();
    let enum_name = &ast.ident;

    // Determine tag field name based on attributes
    let tag_field = attributes.tag.unwrap_or_else(|| "".to_string());
    if !tag_field.is_empty() && tag_field.ne("type") {
        panic!("Invalid tag field name, only \"type\" is allowed");
    }

    let use_type_value_format = tag_field == "type";

    // Generate match arms for serialization
    let serialize_match_arms = ast.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let db_name = if use_type_value_format {
            variant_name.to_string()
        } else {
            match config.use_camel_case {
                true => snake_case_to_camel(variant_name.to_string().as_str()),
                false => camel_to_snake_case(variant_name.to_string().as_str()),
            }
        };

        match &variant.fields {
            syn::Fields::Unit => {
                let db_name = variant_name.to_string();
                quote! {
                    #enum_name::#variant_name => {
                        surrealdb::sql::Value::from(#db_name.to_string())
                    }
                }
            },
            syn::Fields::Unnamed(fields) => {
                let field_count = fields.unnamed.len();
                let field_names: Vec<_> = (0..field_count).map(|i| format_ident!("_{}", i)).collect();
                let field_serializers = fields.unnamed.iter().map(|field| {
                    let field_type = &field.ty;
                    quote! {
                        <#field_type as surreal_devl::proxy::default::SurrealSerializer>::serialize
                    }
                });

                if use_type_value_format {
                    quote! {
                        #enum_name::#variant_name(#(ref #field_names),*) => {
                            let mut map = std::collections::BTreeMap::new();
                            let values = vec![
                                #(#field_serializers(#field_names.clone())),*
                            ];
                            map.insert("type".to_string(), surrealdb::sql::Value::from(#db_name.to_string()));
                            map.insert("value".to_string(), surrealdb::sql::Value::Array(values.into()));
                            surrealdb::sql::Value::Object(map.into())
                        }
                    }
                } else {
                    quote! {
                        #enum_name::#variant_name(#(ref #field_names),*) => {
                            let mut map = std::collections::BTreeMap::new();
                            let values = vec![
                                #(#field_serializers(#field_names.clone())),*
                            ];
                            map.insert(#db_name.to_string(), surrealdb::sql::Value::Array(values.into()));
                            surrealdb::sql::Value::Object(map.into())
                        }
                    }
                }
            }
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|field| field.ident.as_ref().unwrap()).collect();
                // Handle struct variants (e.g., Custom{r,g,b})
                let field_serializers = fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;
                    let db_field_name = match config.use_camel_case {
                        true => snake_case_to_camel(field_name.to_string().as_str()),
                        false => camel_to_snake_case(field_name.to_string().as_str()),
                    };
                    quote! {
                        inner_map.insert(
                            #db_field_name.to_string(),
                            <#field_type as surreal_devl::proxy::default::SurrealSerializer>::serialize(#field_name.clone())
                        );
                    }
                });

                if use_type_value_format {
                    quote! {
                        #enum_name::#variant_name { #(#field_names),* } => {
                            let mut map = std::collections::BTreeMap::new();
                            let mut inner_map = std::collections::BTreeMap::new();
                            #(#field_serializers)*
                            map.insert("type".to_string(), surrealdb::sql::Value::from(#db_name.to_string()));
                            map.insert("value".to_string(), surrealdb::sql::Value::Object(inner_map.into()));
                            surrealdb::sql::Value::Object(map.into())
                        }
                    }
                } else {
                    quote! {
                        #enum_name::#variant_name { #(#field_names),* } => {
                            let mut map = std::collections::BTreeMap::new();
                            let mut inner_map = std::collections::BTreeMap::new();
                            #(#field_serializers)*
                            map.insert(#db_name.to_string(), surrealdb::sql::Value::Object(inner_map.into()));
                            surrealdb::sql::Value::Object(map.into())
                        }
                    }
                }
            }
        }
    });

    // Generate match arms for deserialization
    let deserialize_match_arms = ast.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let db_name = if use_type_value_format {
            variant_name.to_string()
        } else {
            match config.use_camel_case {
                false => camel_to_snake_case(variant_name.to_string().as_str()),
                true => snake_case_to_camel(variant_name.to_string().as_str()),
            }
        };

        match &variant.fields {
            syn::Fields::Unit => {
                let db_name = variant_name.to_string();
                quote! {
                    #db_name => Ok(#enum_name::#variant_name),
                }
            }
            syn::Fields::Unnamed(fields) => {
                let field_count = fields.unnamed.len();
                let field_deserializers = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let field_type = &field.ty;
                    quote! {
                        <#field_type as surreal_devl::proxy::default::SurrealDeserializer>::from_option(
                            arr.get(#i)
                        )?
                    }
                });

                quote! {
                    #db_name => {
                        if let surrealdb::sql::Value::Array(arr) = variant_value {
                            if arr.len() != #field_count {
                                return Err(surreal_devl::surreal_qr::SurrealResponseError::NumberOfFieldOfLengthOfDbValueNotMatchLengthOfEnum);
                            }
                            Ok(#enum_name::#variant_name(
                                #(#field_deserializers),*
                            ))
                        } else {
                            Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnArray)
                        }
                    }
                }
            }
            syn::Fields::Named(fields) => {
                let field_deserializers = fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;
                    let db_field_name = match config.use_camel_case {
                        true => snake_case_to_camel(field_name.to_string().as_str()),
                        false => camel_to_snake_case(field_name.to_string().as_str()),
                    };
                    quote! {
                        #field_name: <#field_type as surreal_devl::proxy::default::SurrealDeserializer>::from_option(
                            inner_obj.get(#db_field_name)
                        )?
                    }
                });

                quote! {
                    #db_name => {
                        if let surrealdb::sql::Value::Object(inner_obj) = variant_value {
                            Ok(#enum_name::#variant_name {
                                #(#field_deserializers),*
                            })
                        } else {
                            Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnObject)
                        }
                    }
                }
            }
        }
    });

    let gen = quote! {
        impl surreal_devl::proxy::default::SurrealSerializer for #enum_name {
            fn serialize(self) -> surrealdb::sql::Value {
                match self {
                    #(#serialize_match_arms)*
                }
            }
        }

        impl surreal_devl::proxy::default::SurrealDeserializer for #enum_name {
            fn deserialize(value: &surrealdb::sql::Value) -> Result<Self, surreal_devl::surreal_qr::SurrealResponseError> {
                let mut fake_obj = surrealdb::sql::Object::from(std::collections::BTreeMap::<String, surrealdb::sql::Value>::new());
                let obj = match value {
                    surrealdb::sql::Value::Object(obj) => obj,
                    surrealdb::sql::Value::Strand(strand) => {
                        if #use_type_value_format {
                            fake_obj.0.insert("type".to_string(), surrealdb::sql::Value::from(strand.0.clone()));
                            fake_obj.0.insert("value".to_string(), surrealdb::sql::Value::from(strand.0.clone()));
                        } else {
                            fake_obj.0.insert(strand.0.clone(), surrealdb::sql::Value::from(strand.0.clone()));
                        }
                        &fake_obj
                    },
                    _ => return Err(surreal_devl::surreal_qr::SurrealResponseError::ExpectedAnObject),
                };

                let (variant_name, variant_value) = if #use_type_value_format {
                    let type_value = obj.get("type")
                        .ok_or(surreal_devl::surreal_qr::SurrealResponseError::TypeEnumMustBeString)?;
                    let variant_value = obj.get("value")
                        .unwrap_or(type_value);
                    
                    match type_value {
                        surrealdb::sql::Value::Strand(s) => (s.0.as_str(), variant_value),
                        _ => return Err(surreal_devl::surreal_qr::SurrealResponseError::InvalidEnumFormat),
                    }
                } else {
                    if obj.len() != 1 {
                        return Err(surreal_devl::surreal_qr::SurrealResponseError::InvalidEnumFormat);
                    }
                    let (name, value) = obj.iter().next().unwrap();
                    (name.as_str(), value)
                };

                match variant_name {
                    #(#deserialize_match_arms)*
                    _ => Err(surreal_devl::surreal_qr::SurrealResponseError::UnknownVariant),
                }
            }
        }
    };

    gen.into()
}

