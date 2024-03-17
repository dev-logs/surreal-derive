
extern crate proc_macro;
mod surreal_quote;
mod surreal_derive;

use syn::{LitStr, parse_macro_input};

/// Read and generate valid SurrealDb query command at compile time
/// it is necessary to know that the main logic of generating query still depends on
/// original Rust SDK of SurrealDb, so that this also offering a seamless integration with the original SurrealDb Rust SDK,
/// which can be found in the documentation at https://docs.rs/surrealdb/1.0.0/surrealdb.
/// USAGES:
/// ```
/// use surreal_derive::SurrealDerive;
/// use surrealdb::sql::serde;
/// use serde::Deserialize;
/// use serde::Serialize;
/// #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
/// struct Person {
///     name: String,
///     age: i32
/// }
///
/// // It is necessary for a struct to specify what is its primary key
/// impl From<Person> for surrealdb::sql::Value {
///     fn from(value: Person) -> Self {
///         ("person", value.name);
///     }
/// }
///
/// fn main() {
///     use surreal_derive::surreal_quote;
///     let p = Person {name: "surrealdb".to_string(), age: 20};
///     let sql_statement = surreal_quote!("CREATE #record(&person)");
///     assert!(sql_statement, "CREATE person:surrealdb SET name='surrealdb', age=10");
/// }
/// ```
#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();
    surreal_quote::surreal_quote(input)
}

/// The `surreal_derive` macro generates implementations to facilitate the conversion of a struct into `surrealdb::sql::Value`.
/// For example, after derive, it will generate a method: @{into_idiom_value(&self) -> Vec<(surrealdb::sql::Idiom, surrealdb::sql::Value)>}
/// it allow us to reuse the logic of Rust SDK
/// For details, refer to the documentation at https://docs.rs/surrealdb/1.0.0/surrealdb/sql/enum.Value.html.
/// USAGES:
/// ```
/// use surreal_derive::SurrealDerive;
/// use surrealdb::sql::serde;
/// use serde::Deserialize;
/// use serde::Serialize;
/// #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
/// struct Person {
///     name: String,
///     age: i32
/// }
///
/// // It is necessary for a struct to specify what is its primary key
/// impl From<Person> for surrealdb::sql::Value {
///     fn from(value: Person) -> Self {
///         ("person", value.name);
///     }
/// }
///
/// // Then we can convert the struct into part of the query statement
/// fn main() {
///     use surreal_derive::surreal_quote;
///     let p = Person {name: "surrealdb".to_string(), age: 20};
///     let sql_statement = surreal_quote!("CREATE #record(&person)");
///     assert!(sql_statement, "CREATE person:surrealdb SET name='surrealdb', age=10");
/// }
/// ```
#[proc_macro_derive(SurrealDerive)]
pub fn surreal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    surreal_derive::surreal_derive_process(ast)
}
