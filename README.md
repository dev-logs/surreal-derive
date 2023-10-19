# <a href="url"><img src="https://github.com/dev-logs/surreal-derive/assets/27767477/a10ad106-83af-48a2-894f-a599613e0d79" width="48"></a>  Surreal derive
Simple library for writing [**SurrealQL** ](https://surrealdb.com/docs/surrealql), this is just a verythin layer on top of [SurrealDb Rust SDK](https://surrealdb.com/docs/integration/sdks/rust). **You will never pay any runtime performance for what you don't do**
# Installation
### 1. Install surreal-devl: https://github.com/dev-logs/surreal-devl
Contains the core logic of the whole library, the main purpose is to act as a bridge between SurrealDb SDK and the your defined Struct, also support working with **Array**, **ID** or **DateTime**
```console
cargo add sureal-devl
```
### 2. Install surreal-derive:
```console
cargo add surreal-derive
```
# Usage
### use surreal_quote!

### Mark your struct as surreal_derive
```rust
use serde::{Deserialize, Serialize};
use surreal_derive::SurrealDerive;

#[derive(Debug, Serialize, Deserialize, SurrealDerive, Clone)]
pub struct User {
    pub name: String,
    pub password: String,
}
```

### Implement the Into<surrealdb::value::RecordId> trait
```rust
```
