use sqlx::postgres::PgQueryResult;
use sqlx::{Error, PgPool};

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
    pub async fn insert(&self, pg_pool: &PgPool) -> Result<PgQueryResult, Error> {
        return sqlx::query(
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
        .await;
    }

    pub async fn select_next_message(
        chat_id: i64,
        pg_pool: &PgPool,
        id: i64,
    ) -> Result<Message, Error> {
        let record = sqlx::query_as!(
            Message,
            r#"SELECT * FROM message WHERE chat_id = $1 AND message_id > $2  LIMIT 1"#,
            chat_id,
            id
        )
        .fetch_one(pg_pool)
        .await;
        return record;
    }

    pub async fn select_last_message(
        chat_id: i64,
        pg_pool: &PgPool,
        id: i64,
    ) -> Result<Message, Error> {
        let record = sqlx::query_as!(
            Message,
            r#"SELECT text, chat_id, message_id FROM message WHERE chat_id = $1 AND message_id = (select MAX(message_id) from message WHERE message_id < $2)  LIMIT 1"#,
            chat_id,
            id
        )
        .fetch_one(pg_pool)
        .await;
        return record;
    }

    pub async fn select_first_user_message_by_chat_id(
        chat_id: i64,
        pg_pool: &PgPool,
    ) -> Result<Message, Error> {
        let record = sqlx::query_as!(
            Message,
            r#"SELECT * FROM message WHERE chat_id = $1 LIMIT 1"#,
            chat_id
        )
        .fetch_one(pg_pool)
        .await;
        return record;
    }
}

#[derive(Debug, Clone)]
pub struct LinkMessage {
    pub id: i64,
    pub text: String,
    pub chat_id: i64,
    pub message_id: i64,
}

impl LinkMessage {
    pub async fn new(id: i64, text: String, chat_id: i64, message_id: i64) -> Self {
        Self {
            id,
            text,
            chat_id,
            message_id,
        }
    }

    pub async fn insert(&self, pg_pool: &PgPool) -> Result<PgQueryResult, Error> {
        return sqlx::query(
            r#"
    INSERT INTO link_message (id,text,chat_id, message_id)
    VALUES ( $1::bigint,$2,$3::bigint, $4::bigint)
    ON CONFLICT DO NOTHING
        "#,
        )
        .bind(&self.id)
        .bind(&self.text)
        .bind(&self.chat_id)
        .bind(&self.message_id)
        .execute(pg_pool)
        .await;
    }

    pub async fn delete_and_return_link(
        pg_pool: &PgPool,
        chat_id: i64,
    ) -> Result<LinkMessage, Error> {
        let select_element_after_delete = sqlx::query_as!(
            LinkMessage,
            r#"DELETE FROM link_message
                    WHERE chat_id = $1
                    RETURNING *
                    "#,
            chat_id
        )
        .fetch_one(pg_pool)
        .await;
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

    pub async fn change_message_text(&self, pg_pool: &PgPool) -> Result<PgQueryResult, Error> {
        return sqlx::query(
            r#"
                    UPDATE message
                    SET text = $1
                    WHERE message_id = $2
                    "#,
        )
        .bind(&self.text)
        .bind(&self.message_id)
        .execute(pg_pool)
        .await;
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

    pub async fn insert(&self, pg_pool: &PgPool) -> Result<PgQueryResult, Error> {
        return sqlx::query(
            r#"
    UPDATE update
    SET update_id = $1::bigint
    WHERE id = 1;
        "#,
        )
        .bind(self.update_id + 1)
        .execute(pg_pool)
        .await;
    }

    pub async fn get_last_update(pg_pool: &PgPool) -> Result<Update, Error> {
        sqlx::query_as!(Update, r#"SELECT * FROM UPDATE WHERE ID = 1"#)
            .fetch_one(pg_pool)
            .await
    }
}
