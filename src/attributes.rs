use darling::FromDeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(surreal_derive))]
#[warn(dead_code)]
pub struct SurrealDeriveAttribute {}

impl Default for SurrealDeriveAttribute {
    fn default() -> Self {
        Self {}
    }
}
