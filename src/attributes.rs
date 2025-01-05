use darling::FromDeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(surreal_derive))]
pub struct SurrealDeriveAttribute {
    pub untagged: bool,
}

impl Default for SurrealDeriveAttribute {
    fn default() -> Self {
        Self { untagged: false }
    }
}
