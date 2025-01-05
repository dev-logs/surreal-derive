# <a href="url"><img src="https://github.com/dev-logs/surreal-derive/assets/27767477/a10ad106-83af-48a2-894f-a599613e0d79" width="48"></a>  Surreal derive
# Description
- Support serialize and deserialize your struct into `surrealdb::sql::Value` with support for both nested struct and foreign key
- Generate query statement
- Quick access to the query result from path

# Installation
### 1. Install surreal-devl: https://crates.io/crates/surreal_devl
Contains the core logic of the whole library, the main purpose is to act as a bridge between SurrealDb SDK and your defined structs, also support working with **Array**, **ID** or **DateTime**
```console
cargo add sureal_devl
```
### 2. Install surreal-derive:
```console
cargo add surreal_derive_plus
```
### Note:

Current restriction that will be resolved in the future: If your variable names coincide with any of the following supported statements: ["id", "val", "date", "duration", "record", "set", "content", "multi", "array"], kindly consider renaming them.

# Usage
### Serialize and deserialize
```rust
use surreal_derive_plus::SurrealDerive;

#[derive(Debug, SurrealDerive, Clone)]
pub struct User {
    pub name: String,
    pub password: String,
}
```
Then we will able to serialize a struct into value and vice versa
```rust
let user = User {
    name: String::from("tiendang"),
    password: String::from("123123")
};
    
let value: surrealdb::sql::value = user.into();
let new_user: User = value.into();
```

### Nested struct:
```rust
struct User {
   name: String 
}

#[derive(SurrealDerive)]
struct UserFriend {
    // Serialize friend will be friend = { name: "something" }
    friend: User
}
```
### Foreign key
```rust
struct User {
   name: String 
}

impl SurrealId for User {
    fn id(&self) -> Thing {
        Thing::from(("user", self.name.as_str()))
    }
}

#[derive(SurrealDerive)]
struct UserFriend {
    // Serialize friend will always be an id, eg: `friend = user:<john>`
    // Deserialize will be Link::Id or Link:Record if we fetch
    friend: Link<User> 
}
```

### Generate query with surreal_quote macro
#### Struct
```rust
use surreal_derive_plus::surreal_quote;
.... connect to your surreal db ...
    
let new_user = User {
    name: "surreal".to_string(),
    password: "000000".to_string(),
};

let created_user: Option<entities::user::User> = DB.query(surreal_quote!("CREATE #record(&new_user)")).await.unwrap().take(0).unwrap(); => CREATE user:surreal SET name='surreal', password='000000'
```

#### Variable
```rust
let age = 2;
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET age = #age");

assert_eq!(query_statement, "CREATE user SET age = 2");
```
#### Array
```rust
let arr = vec![1,2,3,1];
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET arr = #array(&arr)");

assert_eq!(query_statement, "CREATE user SET arr = [1, 2, 3, 1]");
```
#### Struct Array
```rust
let friends = vec![
    User {
        name: "Ethan".to_string(),
        full_name: "Ethan Sullivan".to_string(),
        password: "123123".to_string(),
    },
    User {
        name: "Olivia".to_string(),
        full_name: "Olivia Anderson".to_string(),
        password: "123123".to_string(),
    }
];

let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET friends = #array(&friends)");
assert_eq!(query_statement, "CREATE user SET friends = [user:Ethan, user:Olivia]");
```
#### DateTime
```rust
let birthday: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET birthday = #date(&birthday)");

assert_eq!(query_statement, "CREATE user SET birthday = '2020-01-01T00:00:00Z'");
```

#### Duration
```rust
let party_duration = Duration::from_millis(2 * 60 * 60 * 1000);
let party_started_at: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 1, 1, 14, 0, 0).unwrap();
let query_statement = surreal_derive_plus::surreal_quote!("CREATE party SET duration = #duration(&party_duration), #date(&party_started_at)");
assert_eq!(query_statement, "CREATE party SET duration = 2h, '2023-01-01T14:00:00Z'");
```

#### Surreal ID
Convert a struct into it's id if it has implement `SurrealId` trait
```rust
let user =  User {
    name: "clay".to_string(),
    full_name: "clay".to_string(),
    password: "123123".to_string(),
};

let query_statement = surreal_derive_plus::surreal_quote!("UPDATE #id(&user) SET age = 10");

assert_eq!(query_statement, "UPDATE user:clay SET age = 10");
```

#### Value
```rust
let str = String::from("string");
let statement = surreal_derive_plus::surreal_quote!("CREATE user SET full_name = #val(&str)");
assert_eq!(statement, "CREATE user SET full_name = 'string'");
```

# SurrealQR: Quick access to query result

```rust

```

# Customize setting
We can customize the setting inside cargo.toml

You might need to call `cargo clean` to take effect
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