#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use serde_derive::{Deserialize, Serialize};
    use surreal_devl::surreal_statement::relate;
    use surrealdb::opt::{RecordId};
    use surrealdb::sql::Id;
    use surrealdb_id::link::Link;
    use surrealdb_id::relation::r#trait::IntoRelation;
    use surreal_derive_plus::{surreal_quote, SurrealDerive};

    // Entity 1
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive)]
    struct User {
        name: String
    }

    // Entity 2
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive)]
    struct BlogPost {
        title: String
    }

    // Relation
    #[derive(Debug, Clone, Serialize, Deserialize, SurrealDerive)]
    struct Discussion {
        content: String,
        created_at: DateTime<Utc>
    }

    impl Into<RecordId> for User {
        fn into(self) -> RecordId {
            ("user", self.name.as_str()).into()
        }
    }

    impl Into<RecordId> for Discussion {
        fn into(self) -> RecordId {
            ("discuss", Id::Number(self.created_at.timestamp_millis())).into()
        }
    }

    impl Into<RecordId> for BlogPost {
        fn into(self) -> RecordId {
            ("blogPost", self.title.as_str()).into()
        }
    }

    #[tokio::test]
    pub async fn should_convert_to_link() -> surrealdb::Result<()> {
        let user: RecordId = RecordId::from(("user", "Devlog"));
        let blogPost: RecordId = RecordId::from(("blogPost", "How to use surrealdb"));
        let relation = Discussion { content: "Hello I really want to know more".to_string(), created_at: Default::default() }.relate(user, blogPost);

        assert_eq!(
            surreal_quote!("#relate(&relation)"),
            "RELATE user:Devlog -> discuss -> blogPost:⟨How to use surrealdb⟩ SET content = 'Hello I really want to know more', created_at = '1970-01-01T00:00:00Z'"
        );

        Ok(())
    }

    #[tokio::test]
    pub async fn should_insert_link() -> surrealdb::Result<()> {
        let user: Link<User> = Link::from(("user", "Devlog"));

        assert_eq!(
            surreal_quote!("#id(&user)"),
            "user:Devlog"
        );

        assert_eq!(
            surreal_quote!("#content(&user)"),
            "id=user:Devlog"
        );

        Ok(())
    }

    #[tokio::test]
    pub async fn should_insert_relation() -> surrealdb::Result<()> {
        let user: Link<User> = Link::from(("user", "Devlog"));
        let blog: Link<BlogPost> = Link::from(("blog", "AAA"));
        let discussion = Discussion { content: "content".to_string(), created_at: Default::default() };

        let relation = discussion.relate(&user, &blog);

        assert_eq!(
            surreal_quote!("#id(&relation)"),
            "discuss:0"
        );

        assert_eq!(
            surreal_quote!("#content(&relation)"),
            "in=user:Devlog,relation=discuss:0,out=blog:AAA"
        );

        Ok(())
    }
}
