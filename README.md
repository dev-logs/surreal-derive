# <a href="url"><img src="https://github.com/dev-logs/surreal-derive/assets/27767477/a10ad106-83af-48a2-894f-a599613e0d79" width="48"></a>  Surreal derive
Simple library for writing [**SurrealQL** ](https://surrealdb.com/docs/surrealql), this is just a verythin layer on top of [SurrealDb Rust SDK](https://surrealdb.com/docs/integration/sdks/rust). **You will never pay any runtime performance for what you don't do**
# Installation
### 1. Install surreal-devl: https://github.com/dev-logs/surreal-devl
Contains the core logic of the whole library, the main purpose is to act as a bridge between SurrealDb SDK and the your defined Struct, also support working with **Array**, **ID** or **DateTime**
```console
cargo add sureal_devl
```
### 2. Install surreal-derive:
```console
cargo add surreal_derive_plus
```
# Usage
### Mark your struct as surreal_derive
This will will automatically generate code that could convert your struct into [surrealdb idioms](https://docs.rs/surrealdb/1.0.0/surrealdb/sql/struct.Idiom.html)
```rust
use serde::{Deserialize, Serialize};
use surreal_derive_plus::SurrealDerive;

#[derive(Debug, Serialize, Deserialize, SurrealDerive, Clone)]
pub struct User {
    pub name: String,
    pub password: String,
}
```

### Implement the Into/<surrealdb::value::RecordId> trait
```rust
use surrealdb::opt::RecordId;
use crate::entities::user::User;

impl Into<RecordId> for User {
    fn into(self) -> RecordId {
        return RecordId::from(("user", self.name.as_str()));
    }
}
```

### Write query by using surreal_derive_plus::surreal_quote! macro
#### Struct
```rust
    use surreal_derive_plus::surreal_quote;
    .... connect to your surreal db ...
    
    let new_user = User {
        name: "surreal".to_string(),
        password: "000000".to_string(),
    };

    let created_user: Option<entities::user::User> = DB.query(surreal_quote!("CREATE #record(&user)")).await.unwrap().take(0).unwrap(); => CREATE user:surreal SET name='surreal', password='000000'
```

#### Variable
```rust
        let age = 2;
        let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET age = #age");

        assert_eq!(query_statement, "CREATE user SET age = 2")
```
#### Array
```rust
        let arr = vec![1,2,3,1];
        let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET arr = #array(&arr)");

        assert_eq!(query_statement, "CREATE user SET arr = [1, 2, 3, 1]")
```
#### Struct Array
```rust
        let friends = vec![
            User {
                name: "clay".to_string(),
                full_name: "clay clay".to_string(),
                password: "123123".to_string(),
            },
            User {
                name: "joih".to_string(),
                full_name: "joih joih".to_string(),
                password: "123123".to_string(),
            }
        ];
        let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET friends= #array(&friends)");
        assert_eq!(query_statement, "CREATE user SET friends= [user:clay, user:joih]");
```
#### DateTime
```rust
        let dateBirth: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET brithday = #date(&dateBirth)");
        assert_eq!(query_statement, "CREATE user SET brithday = '2020-01-01T00:00:00Z");
```

#### Surreal ID
```rust
        let user =  User {
            name: "clay".to_string(),
            full_name: "clay clay".to_string(),
            password: "123123".to_string(),
        };

        let query_statement = surreal_derive_plus::surreal_quote!("UPDATE #id(&user) SET age = 10");
        assert_eq!(query_statement, "UPDATE user:clay SET age = 10");
```

# Customize setting
You can customize the setting inside cargo.toml
it is neccessary for call cargo clean to apply all of these configuration
```cargo.toml
[package.metadata]
# Will log the query command at runtime
surreal_enable_log = false
# Will log the generated code at build time
surreal_enable_compile_log = false
# Change the naming convention of generated statement into camel case
surreal_use_camel_case = false
# The log namespace, apply for both build time log and runtime log
surreal_namespace = "surrealql-derive"
# The macro name that use for info log, for example
surreal_info_log_macro = "println"
# The macro name that use for warning log, for example
surreal_warn_log_macro = "println"
```
