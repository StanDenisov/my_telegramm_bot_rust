use std::time::Duration;

use crate::models::LinkMessage;
use crate::{Message, Update};
use anyhow::{anyhow, Result};
use futures_core::stream::Stream;
use futures_util::stream;
use serde::Deserialize;
use sqlx::PgPool;

use crate::web::{
    DeleteMessage, KeyboardButton, NewLinkMessage, ReplyKeyboardMarkup, WButtons, WMessage,
    WUpdate, Wrapper,
};

const CONSUMER_INTERVAL: u64 = 2;

#[derive(Debug, Deserialize, Clone)]
pub struct TgClientConfig {
    pub api_url: String,
    pub bot_id: i64,
    pub bot_secret: String,
}

pub struct TgClient {
    url: String,
    client: reqwest::Client,
}

impl TgClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("failed to create http client");
        Self {
            url: dotenv::var("TG").unwrap(),
            client,
        }
    }

    // Unfortunately, no easy way to store such a stream in a struct's field.
    // This is not necessary anyway.
    // But its ok to assign to a variable without hand-written type (i.e. let it be auto-derived)
    // and leave OOP behind.
    pub async fn get_updates(&self, pg_pool: &PgPool) -> impl Stream<Item = Result<WUpdate>> + '_ {
        let update_id = Update::get_last_update(pg_pool).await;
        let url = format!(
            "{}getUpdates?timeout={}&offset={}",
            self.url, CONSUMER_INTERVAL, update_id,
        );
        let consumer = &self.client;
        // we need to find a way to reconnect during long polling
        let a = match consumer
            .get(url.clone())
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
        {
            Ok(rs) if rs.status().is_success() => {
                let val = rs
                    .json::<Wrapper<Vec<WUpdate>>>()
                    .await
                    .map(|w|
                        // bad unwrap - we loose debug info
                        w.result.unwrap_or(Vec::new()))
                    .map(|v| v.into_iter().map(Ok).collect::<Vec<_>>())
                    .map_err(anyhow::Error::from)
                    .unwrap_or_else(|e| vec![Err(e)]);
                val
            }
            bad_res => {
                vec![Err(anyhow!(
                    "reconnect after failure on polling tg updates: {:#?}",
                    bad_res // have to extract valuable debug info, e.g. on wrong secret
                ))]
            }
        };
        stream::iter(a)
    }

    pub async fn next(&self, pg_pool: &PgPool) {
        //delete old message and return last message
        let current = LinkMessage::delete_and_return_link(pg_pool).await;
        println!("{:?}", current.reply_to_message_id);
        //need to delete old message from chat
        let deleted_id = current.id;

        //old link id need to + 1 to get next
        let deleted_reference_id = current.reply_to_message_id;

        let deleted_chat_id = current.chat_id;
        let delete_url = format!("{}deleteMessage", self.url);
        let consume = &self.client;
        let delete_message = DeleteMessage::new(deleted_chat_id, deleted_id).await;

        consume
            .post(delete_url)
            .json(&delete_message)
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
            .unwrap();
        println!("{:?}", deleted_reference_id + 1);
        let buttons = WButtons::new(
            deleted_chat_id,
            "next_link".parse().unwrap(),
            deleted_reference_id + 1,
            ReplyKeyboardMarkup::new(vec![vec![
                KeyboardButton::new("/next".to_string()),
                KeyboardButton::new("/last".to_string()),
            ]]),
        );
        let url = format!("{}SendMessage", self.url);
        let new_message_id = consume
            .post(url)
            .json(&buttons)
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
            .unwrap()
            .json::<Wrapper<WMessage>>()
            .await
            .unwrap()
            .result
            .expect("omg its sheeeeetttttt!!!!")
            .message_id;

        LinkMessage::new(
            new_message_id,
            "next_link".to_string(),
            deleted_chat_id,
            deleted_reference_id + 1,
            Option::from(true),
        )
        .await
        .insert(pg_pool)
        .await;
    }

    pub async fn last(&self, pg_pool: &PgPool) {
        //delete old message and return last message
        let current = LinkMessage::delete_and_return_link(pg_pool).await;

        //need to delete old message from chat
        let deleted_id = current.id;

        //old link id need to + 1 to get next
        let deleted_reference_id = current.reply_to_message_id;

        //
        let deleted_chat_id = current.chat_id;
        let delete_url = format!("{}deleteMessage", self.url);
        let consume = &self.client;
        let delete_message = DeleteMessage::new(deleted_chat_id, deleted_id).await;

        consume
            .post(delete_url)
            .json(&delete_message)
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
            .unwrap();

        let buttons = WButtons::new(
            deleted_chat_id,
            "last_link".parse().unwrap(),
            deleted_reference_id - 1,
            ReplyKeyboardMarkup::new(vec![vec![
                KeyboardButton::new("/next".to_string()),
                KeyboardButton::new("/last".to_string()),
            ]]),
        );
        let url = format!("{}SendMessage", self.url);
        let new_message_id = consume
            .post(url)
            .json(&buttons)
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
            .unwrap()
            .json::<Wrapper<WMessage>>()
            .await
            .unwrap()
            .result
            .unwrap()
            .message_id;

        LinkMessage::new(
            new_message_id,
            "last_link".to_string(),
            deleted_chat_id,
            deleted_reference_id - 1,
            Option::from(true),
        )
        .await
        .insert(pg_pool)
        .await;
    }

    pub async fn history(&self, pg_pool: &PgPool, chat_id: i64) -> i64 {
        let text = "link";
        let reply_message_id =
            Message::select_first_user_message_by_chat_id(chat_id, pg_pool).await;
        let url = format!("{}SendMessage", self.url);
        let buttons = WButtons::new(
            chat_id,
            text.parse().unwrap(),
            reply_message_id,
            ReplyKeyboardMarkup::new(vec![vec![
                KeyboardButton::new("/next".to_string()),
                KeyboardButton::new("/last".to_string()),
            ]]),
        );
        let consumer = &self.client;
        match consumer
            .post(url)
            .json(&buttons)
            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
            .send()
            .await
        {
            Ok(rs) if rs.status().is_success() => {
                let val = rs
                    .json::<Wrapper<WMessage>>()
                    .await
                    .unwrap()
                    .result
                    .unwrap()
                    .message_id;
                LinkMessage::new(
                    val,
                    text.parse().unwrap(),
                    chat_id,
                    reply_message_id,
                    Option::from(true),
                )
                .await
                .insert(pg_pool)
                .await;
                return val;
            }
            bad_res => {
                println!("{:?}", bad_res.unwrap());
                return 1;
            }
        };
    }
}

#[cfg(test)]
mod test {
    use futures_util::stream::StreamExt;

    use super::*;

    fn tg_client() -> TgClient {
        TgClient::new()
    }

    #[test]
    fn synchronous_test_example() {
        tg_client();
        assert!(true);
    }
}
