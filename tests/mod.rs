#[cfg(test)]
mod test_derive_macro {
    use chrono::{DateTime, Utc};
    use serde_derive::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use surreal_derive_plus::{surreal_quote, SurrealDerive};
    use surreal_devl::{
        proxy::default::{SurrealDeserializer, SurrealSerializer},
        surreal_id::{Link, SurrealId},
    };
    use surrealdb::sql::{Object, Thing, Value};

    #[derive(Clone, PartialEq, Serialize, Deserialize, Debug, SurrealDerive)]
    pub enum UserType {
        Employee
    }

    /// Simple entity with SurrealDerive
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealDerive)]
    struct SimpleEntity {
        name: String,
        age: i32,
        user_type: UserType
    }

    impl SurrealId for SimpleEntity {
        fn id(&self) -> Thing {
            Thing::from(("simple_entity", self.name.as_str()))
        }
    }
    /// Complex entity with optional fields and nested SurrealDerive
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct ComplexEntity {
        title: String,
        tags: Vec<String>,
        optional_note: Option<String>,
        child: Option<Link<SimpleEntity>>, // Link
    }

    // Additional struct to test relationships
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct AnotherEntity {
        data: String,
    }

    impl SurrealId for AnotherEntity {
        fn id(&self) -> Thing {
            Thing::from(("another_entity", self.data.as_str()))
        }
    }

    // Entity with a date/time field
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct TimedEntity {
        timestamp: DateTime<Utc>,
    }

    #[test]
    fn test_1_basic_conversion_into_value() {
        let entity = SimpleEntity {
            name: "Alice".to_string(),
            age: 30,
            user_type: UserType::Employee
        };

        // Convert to SurrealDB Value
        let val: Value = entity.clone().serialize();

        // Convert back
        let new_entity: SimpleEntity = SurrealDeserializer::deserialize(&val).unwrap();

        assert_eq!(entity, new_entity);
    }

    #[test]
    fn test_2_optional_fields_none() {
        let entity = ComplexEntity {
            title: "ComplexTitle".to_string(),
            tags: vec!["tag1".into(), "tag2".into()],
            optional_note: None,
            child: None,
        };

        let val: Value = entity.clone().serialize();
        let new_entity: ComplexEntity = SurrealDeserializer::deserialize(&val).unwrap();

        assert_eq!(entity, new_entity);
    }

    #[test]
    fn test_3_optional_fields_some() {
        let simple_entity = SimpleEntity {
            name: "Bob".to_string(),
            age: 28,
            user_type: UserType::Employee
        };

        let entity = ComplexEntity {
            title: "ComplexTitle2".to_string(),
            tags: vec!["tagA".into(), "tagB".into()],
            optional_note: Some("A note".to_string()),
            child: Some(Link::Record(simple_entity.clone())),
        };

        let val: Value = entity.clone().serialize();
        let new_entity: ComplexEntity = SurrealDeserializer::deserialize(&val).unwrap();

        assert_eq!(entity.title, new_entity.title);
        assert_eq!(entity.tags, new_entity.tags);
        assert_eq!(entity.optional_note, new_entity.optional_note);
        assert_eq!(entity.child.map(|it| Link::Id(it.id())), new_entity.child);
    }

    #[test]
    fn test_4_conversion_from_object() {
        // Manually create a SurrealDB Object
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), Value::from("Charlie"));
        map.insert("age".to_string(), Value::from(45));
        map.insert("user_type".to_string(), Value::from("employee"));
        let object = Object::from(map);
        let entity: SimpleEntity = (&object).try_into().unwrap();

        assert_eq!(entity.name, "Charlie");
        assert_eq!(entity.age, 45);
        assert_eq!(entity.user_type, UserType::Employee);
    }

    #[test]
    fn test_5_conversion_to_object() {
        let entity = SimpleEntity {
            name: "Daisy".to_string(),
            age: 22,
            user_type: UserType::Employee
        };

        let obj: Object = entity.into();
        let name_val = obj.get("name").unwrap();
        let age_val = obj.get("age").unwrap();

        assert_eq!(name_val.to_string(), "'Daisy'"); // SurrealDB strings are quoted
        assert_eq!(age_val.to_string(), "22");
    }

    #[test]
    fn test_6_surreal_quote_basic_usage() {
        let entity = AnotherEntity {
            data: "SomeData".to_string(),
        };

        // This should format #(...) as a placeholder in the final statement
        let statement = surreal_quote!("#(entity.data)");
        // statement should be something like "SomeData" (depending on how you handle the macro expansion)

        // We just ensure it doesn't panic and outputs a string
        assert!(!statement.is_empty());
        assert!(statement.contains("SomeData"));
    }

    #[test]
    fn test_7_surreal_quote_with_multiple_placeholders() {
        let user = SimpleEntity {
            name: "UserOne".to_string(),
            age: 18,
            user_type: UserType::Employee
        };
        let statement = surreal_quote!("#(user.name) and #(user.age)");
        assert!(statement.contains("UserOne"));
        assert!(statement.contains("18"));
    }

    #[test]
    fn test_8_datetime_field_conversion() {
        use chrono::TimeZone;
        let sample_time = Utc.ymd(2023, 1, 1).and_hms(12, 0, 0);

        let timed_entity = TimedEntity {
            timestamp: sample_time,
        };
        let val: Value = timed_entity.clone().serialize();
        let new_timed_entity: TimedEntity = SurrealDeserializer::deserialize(&val).unwrap();

        assert_eq!(timed_entity, new_timed_entity);
    }

    #[test]
    fn test_9_complex_nested_structure() {
        let child = SimpleEntity {
            name: "NestedChild".to_string(),
            age: 10,
            user_type: UserType::Employee
        };

        let complex = ComplexEntity {
            title: "Parent".to_string(),
            tags: vec!["child".into(), "nested".into()],
            optional_note: Some("Testing nested objects".into()),
            child: Some(Link::Record(child.clone())),
        };

        let val: Value = complex.clone().serialize();
        let new_complex: ComplexEntity = SurrealDeserializer::deserialize(&val).unwrap();
        assert_eq!(complex, new_complex);
    }

    #[test]
    fn test_10_empty_vectors() {
        // Entities with empty vectors or optional vectors
        #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
        struct VectorEntity {
            items: Vec<String>,
            optional_items: Option<Vec<i32>>,
        }

        let entity = VectorEntity {
            items: vec![],
            optional_items: None,
        };

        let val: Value = entity.clone().serialize();
        let new_entity: VectorEntity = SurrealDeserializer::deserialize(&val).unwrap();
        assert_eq!(entity, new_entity);
    }
}

#[cfg(test)]
mod test_surreal_quote {
    use chrono::{DateTime, Datelike, Utc};
    use serde_derive::{Deserialize, Serialize};
    use std::time::Duration as StdDuration;
    use surreal_derive_plus::{surreal_quote, SurrealDerive};
    use surrealdb::sql::Thing;

    // You mentioned you have these in your code base:
    use surreal_devl::surreal_edge::Edge;
    use surreal_devl::surreal_id::{Link, SurrealId};

    // -----------------------------------------------
    // Sample structs using SurrealDerive + SurrealId
    // -----------------------------------------------

    #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
    struct Schedule {
        name: String,
        waiting_time: StdDuration,
        start_time: DateTime<Utc>,
    }

    impl SurrealId for Schedule {
        fn id(&self) -> Thing {
            Thing::from(("schedule", self.name.as_str()))
        }
    }

    #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
    struct Relationship {
        kind: String,
    }

    impl SurrealId for Relationship {
        fn id(&self) -> Thing {
            Thing::from(("relationship", self.kind.as_str()))
        }
    }

    #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
    struct User {
        name: String,
    }

    impl SurrealId for User {
        fn id(&self) -> Thing {
            Thing::from(("user", self.name.as_str()))
        }
    }

    #[derive(Clone, Serialize, Deserialize, SurrealDerive)]
    struct Service {
        id: String,
        users: Vec<Link<User>>,
    }

    impl SurrealId for Service {
        fn id(&self) -> Thing {
            Thing::from(("service", self.id.as_str()))
        }
    }

    // -----------------------------------------------------------
    // 1) Test #record(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_record_statement() {
        let alex = User {
            name: "Alex".to_owned(),
        };
        let john = User {
            name: "John".to_owned(),
        };
        let service = Service {
            id: "serviceA".to_string(),
            users: vec![Link::Record(alex), Link::Record(john)],
        };

        // Surreal statement might be something like:
        // "CREATE service:serviceA SET id = 'serviceA', users = [user:Alex, user:John]"
        let statement = surreal_quote!("CREATE #record(&service)");
        assert_eq!(
            "CREATE service:serviceA SET id = 'serviceA', users = [user:Alex, user:John]",
            statement
        );
    }

    // -----------------------------------------------------------
    // 2) Test #id(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_id_statement() {
        let service = Service {
            id: "serviceB".to_string(),
            users: vec![],
        };
        // #id(&service) might produce "service:serviceB"
        let statement = surreal_quote!("SELECT #id(&service)");
        assert_eq!("SELECT service:serviceB", statement);
    }

    // -----------------------------------------------------------
    // 3) Test #set(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_set_statement() {
        let user = User {
            name: "Alice".to_owned(),
        };
        // #set(&user) might produce "SET name = 'Alice'"
        let statement = surreal_quote!("UPDATE user:Alice #set(&user)");
        assert_eq!("UPDATE user:Alice SET name = 'Alice'", statement);
    }

    // -----------------------------------------------------------
    // 4) Test #content(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_content_statement() {
        let user = User {
            name: "Bob".to_owned(),
        };

        let statement = surreal_quote!("CREATE user SET #content(&user)");
        assert_eq!("CREATE user SET name='Bob'", statement);
    }

    // -----------------------------------------------------------
    // 5) Test #val(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_val_statement() {
        let user_name = "Carol".to_owned();
        let statement = surreal_quote!("LET name=#val(&user_name)");
        assert_eq!("LET name='Carol'", statement);
    }

    // -----------------------------------------------------------
    // 6) Test #array(...) usage
    // -----------------------------------------------------------
    #[test]
    fn test_array_statement() {
        let items = vec!["one".to_owned(), "two".to_owned(), "three".to_owned()];
        // #array(&items) might produce "['one','two','three']"
        let statement = surreal_quote!("LET arr = #val(&items)");
        assert_eq!("LET arr = ['one', 'two', 'three']", statement);
    }

    // -----------------------------------------------------------
    // 7) Test #date(...) usage with chrono::DateTime
    // -----------------------------------------------------------
    #[test]
    fn test_date_statement() {
        let now = Utc::now();
        // #date(&now) might produce something like "2025-01-03T12:34:56Z"
        let statement = surreal_quote!("SELECT #val(&now)");

        // We can't do an exact equality check because the date is dynamic,
        // but we can verify that it starts with "SELECT " and includes the current year.
        assert!(statement.starts_with("SELECT "));
        let current_year = now.year().to_string();
        assert!(statement.contains(&current_year));
    }

    // -----------------------------------------------------------
    // 8) Test #relate(...) usage (Edge-based)
    // -----------------------------------------------------------
    #[test]
    fn test_relate_statement() {
        let user_a = User {
            name: "Adam".to_string(),
        };

        let user_b = User {
            name: "Betty".to_string(),
        };

        let relationship = Relationship {
            kind: "friendship".to_string(),
        };

        let edge = Edge {
            r#in: Some(Link::Record(user_a)),
            out: Some(Link::Record(user_b)),
            data: relationship,
        };

        // #relate(&edge) might produce:
        // "RELATE user:Adam -> relationship:friendship -> user:Betty SET kind = 'friendship'"
        let statement = surreal_quote!("#relate(&edge)");
        assert_eq!(
            "RELATE user:Adam -> relationship:friendship -> user:Betty SET kind = 'friendship'",
            statement
        );
    }

    // -----------------------------------------------------------
    // 9) Test #duration(...) usage with std::time::Duration
    // -----------------------------------------------------------
    #[test]
    fn test_duration_from_std() {
        let schedule = Schedule {
            name: "DailyCheck".to_string(),
            waiting_time: StdDuration::from_secs(3600), // 1 hour
            start_time: Utc::now(),
        };

        // #duration(&schedule.waiting_time) might produce "1h", "3600s", etc.
        let statement = surreal_quote!("LET wait = #val(&schedule.waiting_time)");
        // Adjust assertion based on how your code actually formats durations, e.g. "1h".
        assert_eq!("LET wait = 1h", statement);
    }

    // -----------------------------------------------------------
    // 10) Another test with #date(...) or combined usage
    // -----------------------------------------------------------
    #[test]
    fn test_combined_duration_and_datetime() {
        let schedule = Schedule {
            name: "CombinedTest".to_string(),
            waiting_time: StdDuration::from_secs(7200), // 2 hours
            start_time: Utc::now(),
        };

        let statement = surreal_quote!(
            "LET info = {{ duration: #val(&schedule.waiting_time), start: #val(&schedule.start_time) }}"
        );
        // e.g.: "LET info = { duration: 2h, start: 2025-01-03T12:34:56Z }"

        assert!(statement.contains("duration:"));
        assert!(statement.contains("start:"));
    }
}

#[cfg(test)]
mod test_in_memory_integration {
    use chrono::{DateTime, Utc};
    use serde_derive::{Deserialize, Serialize};
    use surreal_derive_plus::{surreal_quote, SurrealDerive};
    use surreal_devl::surreal_qr::RPath;
    use surreal_devl::{
        surreal_id::{Link, SurrealId},
        surreal_qr::SurrealQR,
    };
    use surrealdb::engine::local::Mem;
    use surrealdb::sql::Value;
    use surrealdb::{engine::local::Db, opt::auth::Root, sql::Thing, Surreal};

    // --------------------------
    // Sample structs with fields
    // --------------------------

    pub async fn create_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        db
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct User {
        username: String,
        friend: Option<Item>,
        nickname: Option<String>, // Optional fields
        friend2: Box<Link<User>>,
        friend3: Option<Box<Link<User>>>,
        friends: Vec<Link<User>>,  // Vector of link,
        created_at: DateTime<Utc>, // Date/time field
    }

    impl SurrealId for User {
        fn id(&self) -> Thing {
            Thing::from(("user", self.username.as_str()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Item {
        name: String,
        description: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Inventory {
        user: Link<User>,
        items: Vec<Item>,
        note: Option<String>,
    }

    impl SurrealId for Inventory {
        fn id(&self) -> Thing {
            // e.g. "inventory:alice"
            Thing::from(("inventory", self.user.id().id))
        }
    }

    #[tokio::test]
    async fn test_insert_user() {
        let db = create_db().await;

        let user = User {
            username: "test_user".to_string(),
            friend: None,
            nickname: Some("Tester".to_string()),
            friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        let user: Option<User> = db
            .query(surreal_quote!("CREATE #record(&user)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        let user = user.unwrap();
        let result: Option<User> = db
            .query(surreal_quote!("SELECT * FROM #id(&user)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "test_user");
    }

    #[tokio::test]
    async fn test_update_user_nickname() {
        let db = create_db().await;

        // Insert a user
        let mut user = User {
            username: "update_user".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        db.query(surreal_quote!("CREATE #record(&user)"))
            .await
            .unwrap();

        // Update nickname
        user.nickname = Some("UpdatedNickname".to_string());
        db.query(surreal_quote!("UPDATE #id(&user) #set(&user)"))
            .await
            .unwrap();

        let updated_user: Option<User> = db
            .query(surreal_quote!("SELECT * FROM #id(&user)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        assert_eq!(
            updated_user.unwrap().nickname,
            Some("UpdatedNickname".to_string())
        );
    }

    #[tokio::test]
    async fn test_query_users_with_nickname() {
        let db = create_db().await;

        // Insert users
        for username in &["user1", "user2", "user3"] {
            let user = User {
                username: username.to_string(),
                friend: None,
                nickname: Some("CommonNickname".to_string()),
                friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
                friend3: None,
                friends: vec![],
                created_at: Utc::now(),
            };
            db.query(surreal_quote!("CREATE #record(&user)"))
                .await
                .unwrap();
        }

        let users: Vec<User> = db
            .query("SELECT * FROM user WHERE nickname = 'CommonNickname'")
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        assert_eq!(users.len(), 3);
        assert!(users
            .iter()
            .all(|u| u.nickname == Some("CommonNickname".to_string())));
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = create_db().await;

        // Insert a user
        let user = User {
            username: "delete_user".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        db.query(surreal_quote!("CREATE #record(&user)"))
            .await
            .unwrap();

        // Delete the user
        let deleted_user: surrealdb::Value = db
            .query(surreal_quote!("DELETE #id(&user)"))
            .await
            .unwrap()
            .take(0)
            .unwrap();

        println!("{:?}", deleted_user);
        //assert_eq!(deleted_user, Some(user));
    }

    #[tokio::test]
    async fn test_create_inventory_with_items() {
        let db = create_db().await;

        let item1 = Item {
            name: "item1".to_string(),
            description: Some("First item".to_string()),
        };
        let item2 = Item {
            name: "item2".to_string(),
            description: Some("Second item".to_string()),
        };

        let user = User {
            username: "inventory_user".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        let inventory = Inventory {
            user: Link::Record(user.clone()),
            items: vec![item1, item2],
            note: Some("User's inventory".to_string()),
        };

        db.query(surreal_quote!("CREATE #record(&inventory)"))
            .await
            .unwrap();

        let stored_inventory: Option<Inventory> = db
            .query(surreal_quote!("SELECT * FROM #id(&inventory)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        assert!(stored_inventory.is_some());
        assert_eq!(stored_inventory.unwrap().items.len(), 2);
    }

    #[tokio::test]
    async fn test_update_inventory_note() {
        let db = create_db().await;

        let user = User {
            username: "inventory_user2".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "friend_2")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        db.query(surreal_quote!("CREATE #record(&user)"))
            .await
            .unwrap();

        let mut inventory = Inventory {
            user: Link::Record(user.clone()),
            items: vec![],
            note: None,
        };

        db.query(surreal_quote!("CREATE #record(&inventory)"))
            .await
            .unwrap();

        // Update note
        inventory.note = Some("Updated inventory note".to_string());
        db.query(surreal_quote!("UPDATE #id(&inventory) #set(&inventory)"))
            .await
            .unwrap();

        let updated_inventory: Option<Inventory> = db
            .query(surreal_quote!("SELECT * FROM #id(&inventory)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        assert_eq!(
            updated_inventory.unwrap().note,
            Some("Updated inventory note".to_string())
        );
    }

    #[tokio::test]
    async fn test_nested_relationships() {
        let db = create_db().await;

        let user_a = User {
            username: "user_a".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "user_b")))),
            friend3: None,
            friends: vec![],
            created_at: Utc::now(),
        };

        let user_b = User {
            username: "user_b".to_string(),
            friend: None,
            nickname: None,
            friend2: Box::new(Link::Id(Thing::from(("user", "user_a")))),
            friend3: None,
            friends: vec![Link::Record(user_a.clone())],
            created_at: Utc::now(),
        };

        db.query(surreal_quote!("CREATE #record(&user_a)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&user_b)"))
            .await
            .unwrap();

        let result: Option<User> = db
            .query(surreal_quote!("SELECT * FROM #id(&user_b)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        let mut result_1: SurrealQR = db
            .query(surreal_quote!("SELECT * FROM #id(&user_b) FETCH friend2"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        let user_b_name: Option<String> = result_1
            .get(RPath::from("username"))
            .unwrap()
            .deserialize()
            .unwrap();
        let friend_name: Option<String> = result_1
            .get(RPath::from("friend2").get("username"))
            .unwrap()
            .deserialize()
            .unwrap();
        assert_eq!(user_b_name, Some("user_b".to_owned()));
        assert_eq!(friend_name, Some("user_a".to_owned()));
        assert!(result.is_some());
        let user_b_retrieved = result.unwrap();
        assert_eq!(user_b_retrieved.friends.len(), 1);
        assert_eq!(user_b_retrieved.friends[0].id(), user_a.id());
    }
}

#[cfg(test)]
mod test_edge_relationships {
    use chrono::{DateTime, Utc};
    use serde_derive::{Deserialize, Serialize};
    use surreal_derive_plus::{surreal_quote, SurrealDerive};
    use surreal_devl::surreal_edge::{Edge, IntoRelation};
    use surreal_devl::surreal_id::{Link, SurrealId};
    use surreal_devl::surreal_qr::{RPath, SurrealQR};
    use surrealdb::sql::Thing;
    use surrealdb::{
        engine::local::{Db, Mem},
        Surreal,
    };

    pub async fn create_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    // Define test entities
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Person {
        name: String,
        age: i32,
    }

    impl SurrealId for Person {
        fn id(&self) -> Thing {
            Thing::from(("person", self.name.as_str()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Company {
        name: String,
        location: String,
        current_ceo: Option<Box<Edge<Person, Employment, Company>>>, // Nested edge field
    }

    impl SurrealId for Company {
        fn id(&self) -> Thing {
            Thing::from(("company", self.name.as_str()))
        }
    }

    // Define relationship types
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Employment {
        role: String,
        start_date: DateTime<Utc>,
        salary: f64,
    }

    impl SurrealId for Employment {
        fn id(&self) -> Thing {
            Thing::from((
                "employment",
                format!("{}_{}", self.role, self.start_date.timestamp()).as_str(),
            ))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Friendship {
        strength: i32,
        since: DateTime<Utc>,
    }

    impl SurrealId for Friendship {
        fn id(&self) -> Thing {
            Thing::from(("friendship", self.since.timestamp().to_string().as_str()))
        }
    }

    #[tokio::test]
    async fn test_nested_edge() {
        let db = create_db().await;

        let ceo = Person {
            name: "John CEO".to_string(),
            age: 45,
        };

        let employment = Employment {
            role: "CEO".to_string(),
            start_date: Utc::now(),
            salary: 250000.0,
        };

        let company = Company {
            name: "Tech Corp".to_string(),
            location: "Silicon Valley".to_string(),
            current_ceo: Some(Box::new(employment.relate(
                ceo.clone(),
                Company {
                    name: "Tech Corp".to_string(),
                    location: "Silicon Valley".to_string(),
                    current_ceo: None,
                },
            ))),
        };

        // Insert records
        db.query(surreal_quote!("CREATE #record(&ceo)"))
            .await
            .unwrap();
        db.query(surreal_quote!(
            "#relate(&company.current_ceo.as_ref().unwrap())"
        ))
        .await
        .unwrap();
        db.query(surreal_quote!("CREATE #record(&company)"))
            .await
            .unwrap();

        // Query company with nested CEO edge
        let result: SurrealQR = db
            .query(surreal_quote!(
                "SELECT * 
             FROM #id(&company)
             FETCH current_ceo, current_ceo.in"
            ))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        let ceo_name: Option<String> = result
            .get(RPath::from("current_ceo").get("in").get("name"))
            .unwrap()
            .deserialize()
            .unwrap();

        assert_eq!(ceo_name, Some("John CEO".to_string()));
    }

    #[tokio::test]
    async fn test_create_and_query_employment_edge() {
        let db = create_db().await;

        // Create test data
        let employee = Person {
            name: "Jane Smith".to_string(),
            age: 35,
        };

        let company1 = Company {
            name: "Company A".to_string(),
            location: "New York".to_string(),
            current_ceo: None,
        };

        let company2 = Company {
            name: "Company B".to_string(),
            location: "London".to_string(),
            current_ceo: None,
        };

        // Create employment edges
        let edge1 = Edge {
            r#in: Some(Link::Record(employee.clone())),
            out: Some(Link::Record(company1.clone())),
            data: Employment {
                role: "Developer".to_string(),
                start_date: Utc::now(),
                salary: 90000.0,
            },
        };

        let edge2 = Edge {
            r#in: Some(Link::Record(employee.clone())),
            out: Some(Link::Record(company2.clone())),
            data: Employment {
                role: "Senior Developer".to_string(),
                start_date: Utc::now(),
                salary: 120000.0,
            },
        };

        // Insert records
        db.query(surreal_quote!("CREATE #record(&employee)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&company1)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&company2)"))
            .await
            .unwrap();
        let edge1_created: Option<Edge<Person, Employment, Company>> = db
            .query(surreal_quote!("#relate(&edge1)"))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&edge2)"))
            .await
            .unwrap();

        // Query edge with both endpoints
        let result: SurrealQR = db
            .query(surreal_quote!(
                "SELECT * 
             FROM #id(&edge1)
             FETCH in, out"
            ))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        let role: Option<String> = result
            .get(RPath::from("role"))
            .unwrap()
            .deserialize()
            .unwrap();
        let employee_name: Option<String> = result
            .get(RPath::from("in").get("name"))
            .unwrap()
            .deserialize()
            .unwrap();
        let employer_name: Option<String> = result
            .get(RPath::from("out").get("name"))
            .unwrap()
            .deserialize()
            .unwrap();

        assert_eq!(role, Some("Developer".to_string()));
        assert_eq!(employee_name, Some("Jane Smith".to_string()));
        assert_eq!(employer_name, Some("Company A".to_string()));
    }

    #[tokio::test]
    async fn test_bidirectional_friendship_edge() {
        let db = create_db().await;

        // Create test persons
        let person1 = Person {
            name: "Alice".to_string(),
            age: 25,
        };

        let person2 = Person {
            name: "Bob".to_string(),
            age: 27,
        };

        let friendship = Friendship {
            strength: 8,
            since: Utc::now(),
        };

        db.query(surreal_quote!("CREATE #record(&person1)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&person2)"))
            .await
            .unwrap();
        // Create bidirectional friendship edge
        let friendship_edge = friendship.relate(person1.clone(), person2);

        // Insert nodes and edge
        db.query(surreal_quote!("#relate(&friendship_edge)"))
            .await
            .unwrap();

        // Query friends of person1
        let result: SurrealQR = db
            .query(surreal_quote!(
                "SELECT *
             FROM #id(&friendship_edge)
             WHERE in = #id(&person1)
             FETCH out"
            ))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        let friend_name: Option<String> = result
            .get(RPath::from("out").get("name"))
            .unwrap()
            .deserialize()
            .unwrap();
        assert_eq!(friend_name, Some("Bob".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_employment_edges() {
        let db = create_db().await;

        let employee = Person {
            name: "Jane Smith".to_string(),
            age: 35,
        };

        // Add the missing companies
        let company1 = Company {
            name: "Company A".to_string(),
            location: "New York".to_string(),
            current_ceo: None,
        };

        let company2 = Company {
            name: "Company B".to_string(),
            location: "London".to_string(),
            current_ceo: None,
        };

        // Create employment edges
        let edge1 = Edge {
            r#in: Some(Link::Record(employee.clone())),
            out: Some(Link::Record(company1.clone())),
            data: Employment {
                role: "Developer".to_string(),
                start_date: Utc::now(),
                salary: 90000.0,
            },
        };

        let edge2 = Edge {
            r#in: Some(Link::Record(employee.clone())),
            out: Some(Link::Record(company2.clone())),
            data: Employment {
                role: "Senior Developer".to_string(),
                start_date: Utc::now(),
                salary: 120000.0,
            },
        };

        // Insert all records
        db.query(surreal_quote!("CREATE #record(&employee)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&company1)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&company2)"))
            .await
            .unwrap();
        db.query(surreal_quote!("#relate(&edge1)")).await.unwrap();
        db.query(surreal_quote!("#relate(&edge2)")).await.unwrap();

        let result: SurrealQR = db
            .query(surreal_quote!(
                "RETURN array::group(
                SELECT out as company, role, salary 
                FROM employment 
                WHERE in = #id(&employee)
                FETCH company 
            )"
            ))
            .await
            .unwrap()
            .take(RPath::from(0))
            .unwrap();

        // Verify both employments using array indices
        for i in 0..2 {
            let company_name: Option<String> = result
                .get(RPath::from(i).get("company").get("name"))
                .unwrap()
                .deserialize()
                .unwrap();
            let role: Option<String> = result
                .get(RPath::from(i).get("role"))
                .unwrap()
                .deserialize()
                .unwrap();

            assert!(company_name.is_some());
            assert!(role.is_some());
            assert!(matches!(
                company_name.as_deref(),
                Some("Company A") | Some("Company B")
            ));
            assert!(matches!(
                role.as_deref(),
                Some("Developer") | Some("Senior Developer")
            ));
        }
    }
}

#[cfg(test)]
mod test_enum_serialization {
    use serde_derive::{Deserialize, Serialize};
    use surreal_devl::proxy::default::{SurrealDeserializer, SurrealSerializer};
    use surrealdb::sql::Value;
    use surreal_derive_plus::SurrealDerive;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SurrealDerive)]
    pub enum UserType {
        Employee,
        Manager,
        Admin,
    }

    #[test]
    fn test_enum_serialization() {
        // Test serialization for all variants
        let test_cases = vec![
            (UserType::Employee, "employee"),
            (UserType::Manager, "manager"),
            (UserType::Admin, "admin"),
        ];

        for (user_type, expected) in test_cases {
            let value: Value = user_type.serialize();
            assert_eq!(value, Value::from(expected));
        }
    }

    #[test]
    fn test_enum_deserialization() {
        // Test successful deserialization
        let test_cases = vec![
            ("employee", Ok(UserType::Employee)),
            ("manager", Ok(UserType::Manager)),
            ("admin", Ok(UserType::Admin)),
            ("invalid", Err(surreal_devl::surreal_qr::SurrealResponseError::UnknownVariant)),
        ];

        for (input, expected) in test_cases {
            let value = Value::from(input);
            let result = UserType::deserialize(&value);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_enum_deserialization_wrong_type() {
        // Test deserialization with wrong value types
        let wrong_types = vec![
            Value::from(42),
            Value::from(true),
            Value::from(3.14),
        ];

        for value in wrong_types {
            assert!(UserType::deserialize(&value).is_err());
        }
    }

    #[test]
    fn test_enum_roundtrip() {
        // Test serialization followed by deserialization
        let original_values = vec![
            UserType::Employee,
            UserType::Manager,
            UserType::Admin,
        ];

        for original in original_values {
            let serialized = original.clone().serialize();
            let deserialized = UserType::deserialize(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }
    }
}

#[cfg(test)]
mod test_complex_enum_serialization {
    use serde_derive::{Deserialize, Serialize};
    use surreal_devl::proxy::default::{SurrealDeserializer, SurrealSerializer};
    use surrealdb::sql::{Array, Object, Value};
    use surreal_derive_plus::SurrealDerive;

    // Nested struct for testing
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SurrealDerive)]
    struct Address {
        street: String,
        city: String,
        country: String,
    }

    // Complex enum with different variant types
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SurrealDerive)]
    pub enum ComplexUserType {
        // Unit variant
        Guest,
        // Unnamed tuple variant
        Basic(String, i32),
        // Named variant with multiple fields
        Premium {
            level: i32,
            subscription_type: String,
            address: Address,
        },
        // Variant with nested enum
        Staff(StaffRole),
    }

    // Nested enum for testing
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SurrealDerive)]
    pub enum StaffRole {
        Junior,
        Senior { years_experience: i32 },
        Lead(String), // department
    }

    // Struct with nested enum
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, SurrealDerive)]
    struct User {
        name: String,
        user_type: ComplexUserType,
        backup_role: Option<StaffRole>,
    }

    #[test]
    fn test_complex_enum_serialization() {
        let address = Address {
            street: "123 Main St".to_string(),
            city: "Tech City".to_string(),
            country: "Codeland".to_string(),
        };

        let test_cases = vec![
            (ComplexUserType::Guest, Value::from("guest")),
            (
                ComplexUserType::Basic("john".to_string(), 25),
                {
                    let mut obj = Object::default();
                    let mut arr = Array::default();
                    arr.push(Value::from("john"));
                    arr.push(Value::from(25));
                    obj.insert("basic".into(), Value::Array(arr));
                    Value::Object(obj)
                }
            ),
            (
                ComplexUserType::Premium {
                    level: 3,
                    subscription_type: "gold".to_string(),
                    address: address.clone(),
                },
                {
                    let mut obj = Object::default();
                    let mut premium_obj = Object::default();
                    premium_obj.insert("level".into(), Value::from(3));
                    premium_obj.insert("subscription_type".into(), Value::from("gold"));
                    let mut addr_obj = Object::default();
                    addr_obj.insert("street".into(), Value::from("123 Main St"));
                    addr_obj.insert("city".into(), Value::from("Tech City"));
                    addr_obj.insert("country".into(), Value::from("Codeland"));
                    premium_obj.insert("address".into(), Value::Object(addr_obj));
                    obj.insert("premium".into(), Value::Object(premium_obj));
                    Value::Object(obj)
                }
            ),
            (
                ComplexUserType::Staff(StaffRole::Junior),
                {
                    let mut obj = Object::default();
                    obj.insert("staff".into(), Value::from(vec![Value::from("junior")]));
                    Value::Object(obj)
                }
            ),
            (
                ComplexUserType::Staff(StaffRole::Senior { years_experience: 5 }),
                {
                    let mut obj = Object::default();
                    let mut senior_obj = Object::default();
                    senior_obj.insert("years_experience".into(), Value::from(5));
                    let mut staff_obj = Object::default();
                    staff_obj.insert("senior".into(), Value::Object(senior_obj));
                    obj.insert("staff".into(), Value::from(vec![Value::Object(staff_obj)]));
                    Value::Object(obj)
                }
            ),
            (
                ComplexUserType::Staff(StaffRole::Lead("Engineering".to_string())),
                {
                    let mut obj = Object::default();
                    let mut staff_obj = Object::default();
                    staff_obj.insert("lead".into(), Value::from(vec![Value::from("Engineering")]));
                    obj.insert("staff".into(), Value::from(vec![Value::Object(staff_obj)]));
                    Value::Object(obj)
                }
            ),
        ];

        for (user_type, expected) in test_cases {
            let value: Value = user_type.serialize();
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn test_complex_enum_deserialization() {
        use surrealdb::sql::{Array, Object};
        
        let mut address = Object::default();
        address.insert("street".into(), Value::from("123 Main St"));
        address.insert("city".into(), Value::from("Tech City"));
        address.insert("country".into(), Value::from("Codeland"));

        let test_cases = vec![
            (
                {
                    let mut obj = Object::default();
                    obj.insert("guest".into(), Value::Array(Array::default()));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Guest)
            ),
            (
                {
                    let mut obj = Object::default();
                    let mut arr = Array::default();
                    arr.push(Value::from("john"));
                    arr.push(Value::from(25));
                    obj.insert("basic".into(), Value::Array(arr));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Basic("john".to_string(), 25))
            ),
            (
                {
                    let mut obj = Object::default();
                    let mut premium_obj = Object::default();
                    premium_obj.insert("level".into(), Value::from(3));
                    premium_obj.insert("subscription_type".into(), Value::from("gold"));
                    premium_obj.insert("address".into(), Value::Object(address));
                    obj.insert("premium".into(), Value::Object(premium_obj));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Premium {
                    level: 3,
                    subscription_type: "gold".to_string(),
                    address: Address {
                        street: "123 Main St".to_string(),
                        city: "Tech City".to_string(),
                        country: "Codeland".to_string(),
                    },
                })
            ),
            (
                {
                    let mut obj = Object::default();
                    let mut arr = Array::default();
                    arr.push(Value::from("junior"));
                    obj.insert("staff".into(), Value::Array(arr));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Staff(StaffRole::Junior))
            ),
            (
                {
                    let mut obj = Object::default();
                    let mut arr = Array::default();
                    let mut senior_obj = Object::default();
                    senior_obj.insert("senior".into(), Value::Object({
                        let mut obj = Object::default();
                        obj.insert("years_experience".into(), Value::from(5));
                        obj
                    }));
                    arr.push(Value::Object(senior_obj));
                    obj.insert("staff".into(), Value::Array(arr));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Staff(StaffRole::Senior { years_experience: 5 }))
            ),
            (
                {
                    let mut obj = Object::default();
                    let mut arr = Array::default();
                    let mut lead_obj = Object::default();
                    let mut lead_arr = Array::default();
                    lead_arr.push(Value::from("Engineering"));
                    lead_obj.insert("lead".into(), Value::Array(lead_arr));
                    arr.push(Value::Object(lead_obj));
                    obj.insert("staff".into(), Value::Array(arr));
                    Value::Object(obj)
                },
                Ok(ComplexUserType::Staff(StaffRole::Lead("Engineering".to_string())))
            ),
            (
                Value::from("invalid"),
                Err(surreal_devl::surreal_qr::SurrealResponseError::UnknownVariant)
            ),
        ];

        for (value, expected) in test_cases {
            let result = ComplexUserType::deserialize(&value);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_nested_struct_with_enum() {
        let user = User {
            name: "Alice".to_string(),
            user_type: ComplexUserType::Premium {
                level: 2,
                subscription_type: "silver".to_string(),
                address: Address {
                    street: "456 Tech Ave".to_string(),
                    city: "Silicon Valley".to_string(),
                    country: "USA".to_string(),
                },
            },
            backup_role: Some(StaffRole::Senior { years_experience: 3 }),
        };

        // Test serialization
        let serialized = user.clone().serialize();
        
        // Test deserialization
        let deserialized: User = User::deserialize(&serialized).unwrap();
        
        // Verify roundtrip
        assert_eq!(user, deserialized);
    }
}
