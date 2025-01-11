# <a href="url"><img src="https://github.com/dev-logs/surreal-derive/assets/27767477/a10ad106-83af-48a2-894f-a599613e0d79" width="48"></a>  Surreal Derive
# Description
- Generates query statements
- Provides easy access to query results via paths
- Supports serialization to `surrealdb::sql::Value` and deserialization from `surrealdb::sql::Value` instead of using serde
- Support enum
- Easy to add custom type
- Supports IDs and nested structs
- Supports relations

# Installation

### 1. Install surreal-devl: https://crates.io/crates/surreal_devl
```console
cargo add surreal_devl
```
### 2. Install surreal-derive:
```console
cargo add surreal_derive_plus
```

# Usage:

### Generate query statement
```rust
use surreal_derive_plus::{SurrealDerive, surreal_quote};
use surrealdb::sql::Value;

// Example 1: Generate query statement
#[derive(SurrealDerive)]
struct User {
    name: String,
    age: i32,
}

let user = User { name: "john".to_string(), age: 30 };
// Generates: CREATE user:john CONTENT { name: 'john', age: 30 }
let query = surreal_quote!("CREATE #record(&user)");
```

### Easy access to query result from path
```rust
let result: SurrealQR = db.query("SELECT * FROM user").await?.take(RPath::from(0));
// Access nested fields
let name: Option<String> = result.get(RPath::from("user").get("name"))?.deserialize()?;
```

### Serialize and Deserialize
```rust
#[derive(SurrealDerive)]
struct User {
    name: String,
    age: i32,
}

// Serialize
let user = User { name: "alice", age: 25 };
let value: Value = user.serialize();

// Deserialize
let user: User = SurrealDeserializer::deserialize(&value)?;
```

### Support id and nested struct
```rust
#[derive(SurrealDerive)]
struct Address {
    street: String,
    city: String,
}

#[derive(SurrealDerive)]
struct User {
    name: String,
    address: Address,  // Nested struct
}

impl SurrealId for User {
    fn id(&self) -> Thing {
        Thing::from(("user", self.name.as_str()))
    }
}

#[derive(SurrealDerive)]
struct Company {
    user: Link<User>,
    address: Address,  // Nested struct
}

let address = Address {
    street: String::from("123 Main St"),
    city: String::from("New York")
};

let user_address = Address {
    street: String::from("122 Main St"),
    city: String::from("New York")
};

let user = User {
    name: String::from("john"),
    address: user_address
};

let company = Company {
    user: Link::from(user),
    address: address
};

// Create user with nested struct address
let query = db.query(surreal_quote!("CREATE #record(&user)")).await?;
// Create company with link to user eg: user = user:john
let query = surreal_quote!("CREATE #record(&company)");

let result: Option<Company> = db.query("SELECT * FROM company").await?.take(RPath::from(0));
```

### Support relation
```rust
#[derive(SurrealDerive)]
struct Employment {
    role: String,
    salary: f64,
}

let edge = Employment { 
    role: "Developer",
    salary: 100000.0 
}.relate(employee, company);

// Creates relation: RELATE employee:john->employment:developer->company:acme
db.query(surreal_quote!("#relate(&edge)")).await?;
```

### Foreign key
```rust
struct User {
   name: String,
   // Link to user by using id, eg: `friend = user:<john>`
   friend: Box<Link<User>>
}

impl SurrealId for User {
    fn id(&self) -> Thing {
        Thing::from(("user", self.name.as_str()))
    }
}
```

### Support enum

#### Basic enum serialization
```rust
#[derive(SurrealDerive)]
enum UserRole {
    Admin,
    User { level: i32 },
    Moderator(String),
}

// Serialization format:
// Admin -> "Admin"
// User { level: 5 } -> { "user": { "level": 5 } }
// Moderator("forums") -> { "moderator": "forums" }

// Example usage with struct
#[derive(SurrealDerive)]
struct User {
    name: String,
    role: UserRole,
}

let admin = User {
    name: "alice".to_string(),
    role: UserRole::Admin,
};

// Creates a record with structure:
// {
//     "name": "alice",
//     "role": "Admin"
// }
let query = surreal_quote!("CREATE #record(&admin)");
```

#### Tagged enum serialization
```rust
#[derive(SurrealDerive)]
#[surreal(tag = "type")]
enum Message {
    Text { content: String },
    Image { url: String, caption: String },
    Audio { url: String, duration: i32 },
}

// Serialization format:
// Text { content: "Hello" } -> 
//   { "type": "text", "value": { "content": "Hello" } }
// Image { url: "pic.jpg", caption: "My photo" } -> 
//   { "type": "image", "value": { "url": "pic.jpg", "caption": "My photo" } }
// Audio { url: "song.mp3", duration: 180 } -> 
//   { "type": "audio", "value": { "url": "song.mp3", "duration": 180 } }

// Example usage:
let text_msg = Message::Text { content: "Hello".to_string() };

// Create message
let query = surreal_quote!("CREATE #record(&text_msg)");

// Query by type
let query = surreal_quote!("SELECT * FROM message WHERE type = 'text'");

// Deserialize
let result: Message = db.query("SELECT * FROM message WHERE type = 'text'").await?.take(0)?;
assert!(matches!(result, Message::Text { .. }));
```

### Support custom type
To support custom types, implement both `SurrealSerializer` and `SurrealDeserializer` traits:

```rust
use chrono::{DateTime, Utc};
use surrealdb::sql::Value;

// Example: Custom DateTime wrapper
struct CustomDateTime(DateTime<Utc>);

impl SurrealSerializer for CustomDateTime {
    fn serialize(&self) -> Value {
        // Convert to SurrealDB datetime value
        Value::from(self.0)
    }
}

impl SurrealDeserializer for CustomDateTime {
    fn deserialize(value: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            Value::DateTime(dt) => Ok(CustomDateTime(*dt)),
            _ => Err("Expected DateTime value".into())
        }
    }
}

// Use in structs
#[derive(SurrealDerive)]
struct Event {
    name: String,
    timestamp: CustomDateTime
}

// Example usage:
let event = Event {
    name: "Meeting".to_string(),
    timestamp: CustomDateTime(Utc::now())
};

// Serialize to SurrealDB
let query = surreal_quote!("CREATE #record(&event)");

// Deserialize from query results
let result: Event = db.query("SELECT * FROM event").await?.take(0)?;
```

### Variables
#### Normal variable
```rust
let age = 2;
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET age = #age");
```
#### Array
```rust
let arr = vec![1,2,3,1];
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET arr = #val(&arr)");
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

let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET friends = #val(&friends)");
```
#### DateTime
```rust
let birthday: DateTime<Utc> = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
let query_statement = surreal_derive_plus::surreal_quote!("CREATE user SET birthday = #val(&birthday)");
```

#### Duration
```rust
let party_duration = Duration::from_millis(2 * 60 * 60 * 1000);
let party_started_at: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 1, 1, 14, 0, 0).unwrap();
let query_statement = surreal_derive_plus::surreal_quote!("CREATE party SET duration = #val(&party_duration), #val(&party_started_at)");
```

#### Surreal ID
Convert a struct into it's id if it has implement `SurrealId` trait
```rust
impl SurrealId for User {
    fn id(&self) -> Thing {
        Thing::from(("user", self.name.as_str()))
    }
}

let user =  User {
    name: "clay".to_string(),
    full_name: "clay".to_string(),
    password: "123123".to_string(),
};

let query_statement = surreal_derive_plus::surreal_quote!("UPDATE #id(&user) SET age = 10");
```

# Custom Settings
You can customize settings inside Cargo.toml

You might need to call `cargo clean` for changes to take effect
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

# License

This project is licensed under the MIT License - see below for details:

```text
MIT License

Copyright (c) 2024 surreal-derive contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
