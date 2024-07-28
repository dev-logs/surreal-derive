use quote::quote;
use regex::Regex;
use surreal_devl::config::SurrealDeriveConfig;
use surreal_devl::naming_convention::{camel_to_snake_case, snake_case_to_camel};
use syn::{Fields, Type};

use crate::attributes::SurrealDeriveAttribute;

pub fn surreal_derive_process_enum(ast: syn::ItemEnum) -> proc_macro::TokenStream {
    let enum_name = &ast.ident;

    let insert_to_every_variant =
        |id_token: proc_macro2::TokenStream, object_token: proc_macro2::TokenStream| {
            let mut matches = vec![];
            if ast.variants.iter().count() != 2 {
                panic!("The enum must have exact two state");
            }

            for variant in &ast.variants {
                let ident = &variant.ident;
                if ident.clone().to_string() != "Object" && ident.clone().to_string() != "Id" {
                    panic!(
                        "The enum must have exact two state id and object got {:?}",
                        ident.to_string()
                    );
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
        quote::quote! {param.into_idiom_value()},
    );

    let quote2 = insert_to_every_variant(
        quote::quote! {param.into_idiom_value()},
        quote::quote! {param.into_idiom_value()},
    );

    let quote3 = insert_to_every_variant(
        quote! {surrealdb::sql::Value::Thing(param.into())},
        quote! {surrealdb::sql::Value::Thing(param.into())},
    );

    let quote4 = insert_to_every_variant(
        quote! {surrealdb::sql::Value::Thing(param.into())},
        quote! {surrealdb::sql::Value::Thing(param.into())},
    );

    let quote5 = insert_to_every_variant(quote! {param.into()}, quote! {param.into()});

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

pub fn surreal_derive_process_struct(
    ast: syn::ItemStruct,
    attributes: SurrealDeriveAttribute,
) -> proc_macro::TokenStream {
    let config = SurrealDeriveConfig::get();
    let struct_name = &ast.ident;
    let vec_regex = Regex::new(r"^(?:::)?(?:std::vec::)?Vec\s*<.*>$").unwrap();
    let option_vec_regex = Regex::new(
        r"^(?:::)?(?:core::option::)?(?:std::option::)?Option\s*<(?:std::vec::)?Vec\s*<.*>>$",
    )
    .unwrap();
    let option_of_any_regex =
        Regex::new(r"^(?:::)?(?:core::option::)?(?:std::option::)?Option\s*<.*>$").unwrap();
    let duration_regex = Regex::new(r"^(?:::)?(?:std::time::)?Duration$").unwrap();

    if attributes.untagged && ast.fields.len() != 1 {
        panic!("Untagged require struct with a single field");
    }

    let from_object_field_converters = ast.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Failed to process variable name, the ident could not be empty");
        let db_name: String = match config.use_camel_case {
            true => snake_case_to_camel(field_name.to_string().as_str()),
            false => camel_to_snake_case(field_name.to_string().as_str())
        };

        let type_name = type_to_string(&field.ty);
        if (&vec_regex).is_match(&type_name) {
            return quote! {
                #field_name: match value_object.get_mut(#db_name) {
                    Some(surrealdb::sql::Value::Array(ref mut array_obj)) => {
                        array_obj.to_owned().into_iter().map(|item_obj| item_obj.to_owned().try_into().unwrap()).collect()
                    },
                    _ => {
                        panic!("Expected an array");
                    },
                },
            }
        }

        if (&option_vec_regex).is_match(&type_name) {
            return quote! {
                #field_name: match value_object.get_mut(#db_name) {
                    Some(surrealdb::sql::Value::Array(ref mut array_obj)) => {
                        Some(array_obj.to_owned().into_iter().map(|item_obj| item_obj.to_owned().try_into().unwrap()).collect())
                    },
                    _ => None,
                },
            }
        }

        if (&option_of_any_regex).is_match(&type_name) {
            return quote! {
                #field_name: value_object.get_mut(#db_name).take().map(|it| it.to_owned().try_into().unwrap()).into(),
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
        let type_name = type_to_string(&field.ty);
        if (&vec_regex).is_match(&type_name) {
            return quote::quote! {
                let mut array_value: std::vec::Vec<surrealdb::sql::Value> = self.#field_name.iter().map(|v| {
                    surrealdb::sql::Value::from(v.to_owned())
                })
                .collect();

                vec.push((
                    surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                    surrealdb::sql::Value::from(array_value)) // value
                );
            };
        }

        if (&option_vec_regex).is_match(&type_name) {
            return quote::quote! {
                if (&self.#field_name).is_none() {
                    vec.push((
                        surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                        surrealdb::sql::Value::None) // value
                    );
                }
                else {
                    let mut array_value: std::vec::Vec<surrealdb::sql::Value> = self.#field_name.as_ref().unwrap().iter().map(|v| {
                        surrealdb::sql::Value::from(v)
                    })
                    .collect();

                    vec.push((
                        surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                        surrealdb::sql::Value::from(array_value)) // value
                    );
                }
            };
        }

        if (&duration_regex).is_match(&type_name) {
            return quote::quote! {
                vec.push((
                    surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                    surrealdb::sql::Value::from(surrealdb::sql::Duration::from(self.#field_name.clone()))) // value
                );
            };
        }

        if (&option_of_any_regex).is_match(&type_name) {
            return quote::quote! {
                vec.push((
                    surrealdb::sql::Idiom::from(#db_name.to_owned()), // field name
                    self.#field_name.clone().map(|it| surrealdb::sql::Value::from(it)).unwrap_or(surrealdb::sql::Value::None))
                );
            };
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

    let from_value_field_converters_i1 = from_object_field_converters.clone();
    let from_value_field_converters_i2 = from_object_field_converters.clone();

    let from_object = {
        if attributes.untagged {
            // not supported
            quote::quote! {}
        }
        else {
            quote::quote! {
                impl From<surrealdb::sql::Object> for #struct_name {
                    fn from(mut value_object: surrealdb::sql::Object) -> Self {
                        return Self {
                            #(#from_value_field_converters_i1)*
                        }
                    }
                }
            }
        }
    };

    let from_value = {
        if !attributes.untagged {
            quote::quote! {
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
        else {
            let assignment = {
                let field = ast.fields.iter().nth(0).unwrap();
                let field_name = field
                .ident
                .as_ref()
                .expect("Failed to process variable name, the ident could not be empty");

                let type_name = type_to_string(&field.ty);
                if (&vec_regex).is_match(&type_name) {
                    quote! {
                        #field_name: match value {
                            surrealdb::sql::Value::Array(ref mut array_obj) => {
                                array_obj.to_owned().into_iter().map(|item_obj| item_obj.to_owned().try_into().unwrap()).collect()
                            },
                            _ => {
                                panic!("Expected an array");
                            },
                        },
                    }
                }
                else if (&option_vec_regex).is_match(&type_name) {
                    quote! {
                        #field_name: match &value {
                            surrealdb::sql::Value::Array(ref mut array_obj) => {
                                Some(array_obj.into_iter().map(|item_obj| item_obj.to_owned().try_into().unwrap()).collect())
                            },
                            _ => None,
                        },
                    }
                }
                else if (&option_of_any_regex).is_match(&type_name) {
                    quote! {
                        #field_name: match value {
                            surrealdb::sql::Value::None => None,
                            _ => Some(value.to_owned().try_into().unwrap())
                        },
                    }
                }
                else {
                    quote! {
                        #field_name: value.clone().try_into().unwrap(),
                    }
                }
            };

            quote! {
                return Self {
                    #assignment
                }
            }
        }
    };

    let from_option_value = {
        quote::quote! {
            Self::from(value.unwrap())
        }
    };

    let impl_surreal_serialize = {
        if attributes.untagged {
           quote::quote! {}
        }
        else {
            quote::quote! {
                impl surreal_devl::serialize::SurrealSerialize for #struct_name {
                    #into_idiom_value_fn
                }
            }
        }
    };

    let impl_surreal_serialize_ref = {
        if attributes.untagged {
           quote::quote! {}
        }
        else {
            quote::quote! {
                impl surreal_devl::serialize::SurrealSerialize for &#struct_name {
                    #into_idiom_value_fn
                }
            }
        }
    };

    let impl_from_struct_to_value = {
        if attributes.untagged {
            let field = ast.fields.iter().nth(0).unwrap();
            let field_name = field
            .ident
            .as_ref()
            .expect("Failed to process variable name, the ident could not be empty");
            let type_name = type_to_string(&field.ty);
            if (&option_of_any_regex).is_match(&type_name) {
                quote::quote! {
                    match value.#field_name {
                        core::option::Option::Some(value) => value.into(),
                        core::option::Option::None => surrealdb::sql::Value::None
                    }
                }
            }
            else {
                quote::quote! {
                    value.#field_name.into()
                }
            }
        }
        else {
            quote::quote! {
                surrealdb::sql::Value::Thing(value.into())
            }
        }
    };

    let impl_from_ref_struct_to_value = {
        quote::quote! {
            surrealdb::sql::Value::Thing(value.clone().into())
        }
    };

    let impl_into_record_id = {
        quote::quote! {
            self.clone().into()
        }
    };

    let gen = quote::quote! {
        #from_object

        impl From<surrealdb::sql::Value> for #struct_name {
            fn from(mut value: surrealdb::sql::Value) -> Self {
                #from_value
            }
        }

        impl From<Option<surrealdb::sql::Value>> for #struct_name {
            fn from(value: Option<surrealdb::sql::Value>) -> Self {
                #from_option_value
            }
        }

        #impl_surreal_serialize

        #impl_surreal_serialize_ref

        impl From<#struct_name> for surrealdb::sql::Value {
            fn from(value: #struct_name) -> Self {
                #impl_from_struct_to_value
            }
        }

        impl From<&#struct_name> for surrealdb::sql::Value {
            fn from(value: &#struct_name) -> Self {
                #impl_from_ref_struct_to_value
            }
        }

        impl Into<surrealdb::opt::RecordId> for &#struct_name {
            fn into(self) -> surrealdb::opt::RecordId {
                #impl_into_record_id
            }
        }
    };

    gen.into()
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let mut type_name = type_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");

            if let Some(last_segment) = type_path.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    let inner_types: Vec<String> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let syn::GenericArgument::Type(ty) = arg {
                                Some(type_to_string(ty))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !inner_types.is_empty() {
                        let inner_str = inner_types.join(", ");
                        type_name = format!("{}<{}>", type_name, inner_str);
                    }
                }
            }

            type_name
        }
        _ => format!("{:?}", ty), // For simplicity, handle only Type::Path here
    }
}

mod test {
    #[test]
    pub fn regex_test() {
        use regex::Regex;
        let vec_regex = Regex::new(r"^(?:::)?(?:std::vec::)?Vec\s*<.*>$").unwrap();
        let option_vec_regex = Regex::new(
            r"^(?:::)?(?:core::option::)?(?:std::option::)?Option\s*<(?:std::vec::)?Vec\s*<.*>>$",
        )
        .unwrap();
        let option_of_any_regex =
            Regex::new(r"^(?:::)?(?:core::option::)?(?:std::option::)?Option\s*<.*>$").unwrap();
        let duration_regex = Regex::new(r"^(?:::)?(?:std::time::)?Duration$").unwrap();

        assert_eq!(
            option_of_any_regex.is_match(
                "::core::option::Option<super::super::super::surrealdb::links::AuthorLink,>"
            ),
            true
        );
        assert_eq!(
            option_of_any_regex.is_match(
                "core::option::Option<super::super::super::surrealdb::links::AuthorLink,>"
            ),
            true
        );
        assert_eq!(
            option_of_any_regex.is_match(
                "std::option::Option<super::super::super::surrealdb::links::AuthorLink,>"
            ),
            true
        );
        assert_eq!(
            option_of_any_regex.is_match("std::option::Option<post_link::Link>"),
            true
        );
        assert_eq!(
            option_of_any_regex.is_match(
                "::core::option::Option<super::super::super::surrealdb::links::UserLink>"
            ),
            true
        );
    }
}
