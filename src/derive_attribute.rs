use darling::{FromDeriveInput, FromMeta};

#[derive(FromDeriveInput)]
pub struct SurrealDeriveAttribute {
    pub default_into_thing: bool,
    pub type_id: syn::Type
}

impl Default for SurrealDeriveAttribute {
    fn default() -> Self {
       Self {
           default_into_thing: false,
           type_id: syn::Type::from_string("surrealdb::sql::Value").unwrap()
       }
    }
}
