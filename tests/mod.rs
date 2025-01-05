#[cfg(test)]
mod test_derive_macro {
    use chrono::{DateTime, Utc};
    use serde_derive::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use surreal_derive_plus::{surreal_quote, SurrealDerive};
    use surreal_devl::{proxy::default::{SurrealDeserializer, SurrealSerializer}, surreal_id::{Link, SurrealId}};
    use surrealdb::sql::{Object, Thing, Value};

    /// Simple entity with SurrealDerive
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct SimpleEntity {
        name: String,
        age: i32,
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
        };

        // Convert to SurrealDB Value
        let val: Value = entity.clone().serialize();

        // Convert back
        let new_entity: SimpleEntity = SurrealDeserializer::deserialize(&val);

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
        let new_entity: ComplexEntity = SurrealDeserializer::deserialize(&val);

        assert_eq!(entity, new_entity);
    }

    #[test]
    fn test_3_optional_fields_some() {
        let simple_entity = SimpleEntity {
            name: "Bob".to_string(),
            age: 28,
        };

        let entity = ComplexEntity {
            title: "ComplexTitle2".to_string(),
            tags: vec!["tagA".into(), "tagB".into()],
            optional_note: Some("A note".to_string()),
            child: Some(Link::Record(simple_entity.clone())),
        };

        let val: Value = entity.clone().serialize();
        let new_entity: ComplexEntity = SurrealDeserializer::deserialize(&val);

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

        let object = Object::from(map);
        let entity: SimpleEntity = object.into();

        assert_eq!(entity.name, "Charlie");
        assert_eq!(entity.age, 45);
    }

    #[test]
    fn test_5_conversion_to_object() {
        let entity = SimpleEntity {
            name: "Daisy".to_string(),
            age: 22,
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
        let new_timed_entity: TimedEntity = SurrealDeserializer::deserialize(&val);

        assert_eq!(timed_entity, new_timed_entity);
    }

    #[test]
    fn test_9_complex_nested_structure() {
        let child = SimpleEntity {
            name: "NestedChild".to_string(),
            age: 10,
        };

        let complex = ComplexEntity {
            title: "Parent".to_string(),
            tags: vec!["child".into(), "nested".into()],
            optional_note: Some("Testing nested objects".into()),
            child: Some(Link::Record(child.clone())),
        };

        let val: Value = complex.clone().serialize();
        let new_complex: ComplexEntity = SurrealDeserializer::deserialize(&val);
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
        let new_entity: VectorEntity = SurrealDeserializer::deserialize(&val);
        assert_eq!(entity, new_entity);
    }
}

#[cfg(test)]
mod test_surreal_quote {
    use std::time::Duration as StdDuration;
    use chrono::{DateTime, Datelike, Utc};
    use serde_derive::{Deserialize, Serialize};
    use surreal_derive_plus::{SurrealDerive, surreal_quote};
    use surrealdb::sql::Thing;

    // You mentioned you have these in your code base:
    use surreal_devl::surreal_id::{SurrealId, Link};
    use surreal_devl::surreal_edge::Edge;

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
        let statement = surreal_quote!("LET arr = #array(&items)");
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
    use surreal_derive_plus::{SurrealDerive, surreal_quote};
    use surrealdb::{
        engine::local::Db, opt::auth::Root, sql::Thing, Surreal
    };
    use surreal_devl::{surreal_id::{Link, SurrealId}, surreal_qr::SurrealQR};
    use surrealdb::engine::local::Mem;
    // --------------------------
    // Sample structs with fields
    // --------------------------
    
    pub async fn create_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        db
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive)]
    struct User {
        username: String,
        friend: Option<Item>,
        nickname: Option<String>,             // Optional fields
        friend2: Box<Link<User>>,
        friend3: Option<Box<Link<User>>>,
        friends: Vec<Link<User>>,             // Vector of link,
        created_at: DateTime<Utc>,            // Date/time field
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


    impl SurrealId for Item {
        fn id(&self) -> Thing {
            Thing::from(("item", self.name.as_str()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive, PartialEq)]
    struct Inventory {
        user: Link<User>,
        items: Vec<Link<Item>>,
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

        let statement = surreal_quote!("CREATE #record(&user)");
        db.query(statement).await.unwrap();

        let result: Option<User> = db.select(("user", "test_user")).await.unwrap();
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

        let updated_user: Option<User> = db.select(("user", "update_user")).await.unwrap();
        assert_eq!(updated_user.unwrap().nickname, Some("UpdatedNickname".to_string()));
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
            .take(0)
            .unwrap();

        assert_eq!(users.len(), 3);
        assert!(users.iter().all(|u| u.nickname == Some("CommonNickname".to_string())));
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
        let result: SurrealQR = db.query(surreal_quote!("DELETE user #id(&user)")).await.unwrap().take(0).unwrap();

        let result: Option<User> = db.select(("user", "delete_user")).await.unwrap();
        assert!(result.is_none());
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

        db.query(surreal_quote!("CREATE #record(&item1)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&item2)"))
            .await
            .unwrap();
        db.query(surreal_quote!("CREATE #record(&user)"))
            .await
            .unwrap();

        let inventory = Inventory {
            user: Link::Record(user.clone()),
            items: vec![Link::Record(item1), Link::Record(item2)],
            note: Some("User's inventory".to_string()),
        };

        db.query(surreal_quote!("CREATE #record(&inventory)"))
            .await
            .unwrap();

        let stored_inventory: Option<Inventory> = db
            .select(("inventory", "inventory_user"))
            .await
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
            .select(("inventory", "inventory_user2"))
            .await
            .unwrap();
        assert_eq!(updated_inventory.unwrap().note, Some("Updated inventory note".to_string()));
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

        let result: Option<User> = db.select(("user", "user_b")).await.unwrap();
        assert!(result.is_some());
        let user_b_retrieved = result.unwrap();
        assert_eq!(user_b_retrieved.friends.len(), 1);
        assert_eq!(user_b_retrieved.friends[0].id(), user_a.id());
    }
}
