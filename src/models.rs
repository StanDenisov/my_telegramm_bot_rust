use sqlx::PgPool;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub chat_id: i64,
    pub message_id: i64,
}
impl Message {
    pub async fn new(text: String, chat_id: i64, message_id: i64) -> Self {
        Self {
            text,
            chat_id,
            message_id,
        }
    }
    pub async fn insert(&self, pg_pool: &PgPool) {
        sqlx::query(
            r#"
    INSERT INTO message (message_id,text,chat_id)
    VALUES ( $1::bigint,$2,$3::bigint)
    ON CONFLICT DO NOTHING
        "#,
        )
        .bind(&self.message_id)
        .bind(&self.text)
        .bind(&self.chat_id)
        .execute(pg_pool)
        .await
        .expect("insert message");
    }

    pub async fn select_first_user_message_by_chat_id(chat_id: i64, pg_pool: &PgPool) -> i64 {
        let record = sqlx::query_as!(
            Message,
            r#"SELECT * FROM message WHERE chat_id = $1 LIMIT 1"#,
            chat_id
        )
        .fetch_one(pg_pool)
        .await
        .expect("get last update");
        return record.message_id;
    }
}

#[derive(Debug, Clone)]
pub struct LinkMessage {
    pub id: i64,
    pub text: String,
    pub chat_id: i64,
    pub reply_to_message_id: i64,
    pub link_unique: Option<bool>,
}

impl LinkMessage {
    pub async fn new(
        id: i64,
        text: String,
        chat_id: i64,
        reply_to_message_id: i64,
        link_unique: Option<bool>,
    ) -> Self {
        Self {
            id,
            text,
            chat_id,
            reply_to_message_id,
            link_unique,
        }
    }

    pub async fn insert(&self, pg_pool: &PgPool) {
        sqlx::query(
            r#"
    INSERT INTO link_message (id,text,chat_id,reply_to_message_id, link_unique)
    VALUES ( $1::bigint,$2,$3::bigint, $4::bigint, $5)
    ON CONFLICT DO NOTHING
        "#,
        )
        .bind(&self.id)
        .bind(&self.text)
        .bind(&self.chat_id)
        .bind(&self.reply_to_message_id)
        .bind(true)
        .execute(pg_pool)
        .await
        .expect("insert link_message");
    }

    pub async fn delete_and_return_link(pg_pool: &PgPool) -> LinkMessage {
        let select_element_after_delete = sqlx::query_as!(
            LinkMessage,
            "
                    DELETE FROM link_message
                    WHERE link_unique = true
                    RETURNING *
                    "
        )
        .fetch_one(pg_pool)
        .await
        .unwrap();
        return select_element_after_delete;
    }
}

#[derive(Debug, Clone)]
pub struct EditedMessage {
    pub message_id: i64,
    pub text: String,
}
impl EditedMessage {
    pub async fn new(message_id: i64, text: String) -> Self {
        Self { message_id, text }
    }

    pub async fn change_message_text(&self, pg_pool: &PgPool) {
        sqlx::query(
            r#"
                    UPDATE message
                    SET text = $1
                    WHERE message_id = $2
                    "#,
        )
        .bind(&self.text)
        .bind(&self.message_id)
        .execute(pg_pool)
        .await
        .expect("OMG cant update message");
    }
}

#[derive(Debug, Clone)]
pub struct Update {
    pub id: i64,
    pub update_id: i64,
}

impl Update {
    pub async fn new(id: i64, update_id: i64) -> Self {
        return Self { id, update_id };
    }

    pub async fn insert(&self, pg_pool: &PgPool) {
        sqlx::query(
            r#"
    UPDATE update
    SET update_id = $1::bigint
    WHERE id = 1;
        "#,
        )
        .bind(self.update_id + 1)
        .execute(pg_pool)
        .await
        .expect("insert update");
    }

    pub async fn get_last_update(pg_pool: &PgPool) -> i64 {
        let record = sqlx::query_as!(Update, r#"SELECT * FROM UPDATE WHERE ID = 1"#)
            .fetch_one(pg_pool)
            .await
            .expect("get last update");
        println!("{:?}", record.update_id);
        return record.update_id;
    }
}
