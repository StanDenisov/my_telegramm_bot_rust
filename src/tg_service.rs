use std::time::Duration;

use crate::models::LinkMessage;
use crate::{Message, Update};
use anyhow::{anyhow, Result};
use futures_core::stream::Stream;
use futures_util::stream;
use serde::Deserialize;
use sqlx::PgPool;

use crate::web::{
    DeleteMessage, InlineKeyboardMarkup, KeyboardButton, WButtons, WMessage, WUpdate, Wrapper,
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

    pub async fn get_updates(&self, pg_pool: &PgPool) -> impl Stream<Item = Result<WUpdate>> + '_ {
        let update = Update::get_last_update(pg_pool).await;
        match update {
            Ok(upd) => {
                let url = format!(
                    "{}getUpdates?timeout={}&offset={}",
                    self.url, CONSUMER_INTERVAL, upd.update_id,
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
            Err(e) => {
                eprintln!("{:?}", e);
                let x = vec![Err(anyhow!(
                    "reconnect after failure on polling pg updates: {:#?}",
                    e
                ))];
                return stream::iter(x);
            }
        }
    }

    pub async fn exit(&self, pg_pool: &PgPool, chat_id: i64) {
        //delete old message and return last message
        let current = LinkMessage::delete_and_return_link(pg_pool, chat_id).await;
        match current {
            Ok(cur) => {
                //need to delete old message from chat
                let deleted_id = cur.id;
                let deleted_chat_id = cur.chat_id;
                let delete_url = format!("{}deleteMessage", self.url);
                let consume = &self.client;
                let delete_message = DeleteMessage::new(deleted_chat_id, deleted_id).await;

                let delete_repsonse = consume
                    .post(delete_url)
                    .json(&delete_message)
                    .timeout(Duration::from_secs(CONSUMER_INTERVAL))
                    .send()
                    .await;
                match delete_repsonse {
                    Ok(_) => {
                        println!("got exit");
                    }
                    Err(e) => {
                        eprintln!("{:?}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    }

    pub async fn next(&self, pg_pool: &PgPool, chat_id: i64) {
        //delete old message and return last message
        let current = LinkMessage::delete_and_return_link(pg_pool, chat_id).await;
        match current {
            Ok(cur) => {
                //need to delete old message from chat
                let deleted_message_id = cur.message_id;
                let deleted_id = cur.id;
                let deleted_chat_id = cur.chat_id;
                let delete_url = format!("{}deleteMessage", self.url);
                let consume = &self.client;
                let delete_message = DeleteMessage::new(deleted_chat_id, deleted_id).await;

                let delete_request = consume
                    .post(delete_url)
                    .json(&delete_message)
                    .timeout(Duration::from_secs(CONSUMER_INTERVAL))
                    .send()
                    .await;
                match delete_request {
                    Ok(_) => {
                        println!("Deleted")
                    }
                    Err(e) => {
                        eprintln!("{:?}", e)
                    }
                }

                let next = Message::select_next_message(chat_id, pg_pool, deleted_message_id).await;

                match next {
                    Ok(nx) => {
                        let buttons = TgClient::create_buttons(deleted_chat_id, nx.text).await;

                        let url = format!("{}SendMessage", self.url);
                        let new_message_id = consume
                            .post(url)
                            .json(&buttons)
                            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
                            .send()
                            .await;
                        match new_message_id {
                            Ok(new_message_id) => {
                                let new_message_id =
                                    new_message_id.json::<Wrapper<WMessage>>().await;
                                match new_message_id {
                                    Ok(new_message_id) => {
                                        let new_message_id = new_message_id.result;
                                        match new_message_id {
                                            None => {
                                                eprintln!("No message id")
                                            }
                                            Some(new_message_id) => {
                                                let new_message_id = new_message_id.message_id;
                                                let link = LinkMessage::new(
                                                    new_message_id,
                                                    "next_link".to_string(),
                                                    deleted_chat_id,
                                                    nx.message_id,
                                                )
                                                .await
                                                .insert(pg_pool)
                                                .await;
                                                match link {
                                                    Ok(_) => {
                                                        println!("message linked")
                                                    }
                                                    Err(e) => {
                                                        eprintln!("{:?}", e)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(e) => {
                                eprintln!("{:?}", e)
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub async fn last(&self, pg_pool: &PgPool, chat_id: i64) {
        //delete old message and return last message
        let current = LinkMessage::delete_and_return_link(pg_pool, chat_id).await;

        match current {
            Ok(cur) => {
                //need to delete old message from chat
                let deleted_id = cur.id;
                let deleted_message_id = cur.message_id;
                let deleted_chat_id = cur.chat_id;
                let delete_url = format!("{}deleteMessage", self.url);
                let consume = &self.client;
                let delete_message = DeleteMessage::new(deleted_chat_id, deleted_id).await;

                let delete_request = consume
                    .post(delete_url)
                    .json(&delete_message)
                    .timeout(Duration::from_secs(CONSUMER_INTERVAL))
                    .send()
                    .await;

                match delete_request {
                    Ok(_) => {
                        println!("Deleted")
                    }
                    Err(e) => {
                        eprintln!("{:?}", e)
                    }
                }

                let last = Message::select_last_message(chat_id, pg_pool, deleted_message_id).await;

                match last {
                    Ok(last) => {
                        let buttons = TgClient::create_buttons(deleted_chat_id, last.text).await;
                        let url = format!("{}SendMessage", self.url);
                        let new_message_id = consume
                            .post(url)
                            .json(&buttons)
                            .timeout(Duration::from_secs(CONSUMER_INTERVAL))
                            .send()
                            .await;
                        match new_message_id {
                            Ok(new_message_id) => {
                                let new_message_id =
                                    new_message_id.json::<Wrapper<WMessage>>().await;
                                match new_message_id {
                                    Ok(new_message_id) => {
                                        let new_message_id = new_message_id.result;
                                        match new_message_id {
                                            None => {
                                                eprintln!("message id is None")
                                            }
                                            Some(new_message_id) => {
                                                let new_message_id = new_message_id.message_id;
                                                let link_message = LinkMessage::new(
                                                    new_message_id,
                                                    "last_link".to_string(),
                                                    deleted_chat_id,
                                                    last.message_id,
                                                )
                                                .await
                                                .insert(pg_pool)
                                                .await;
                                                match link_message {
                                                    Ok(_) => {
                                                        println!("Link Set")
                                                    }
                                                    Err(e) => {
                                                        eprintln!("{:?}", e)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("{:?}", e)
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{:?}", e)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{:?}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    }

    pub async fn history(&self, pg_pool: &PgPool, chat_id: i64) {
        let first_message_from_history =
            Message::select_first_user_message_by_chat_id(chat_id, pg_pool).await;
        match first_message_from_history {
            Ok(first_message_from_history) => {
                let url = format!("{}SendMessage", self.url);
                let buttons =
                    TgClient::create_buttons(chat_id, first_message_from_history.text).await;
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
                        let linked_message = LinkMessage::new(
                            val,
                            "link".parse().unwrap(),
                            chat_id,
                            first_message_from_history.message_id,
                        )
                        .await
                        .insert(pg_pool)
                        .await;

                        match linked_message {
                            Ok(_) => {
                                println!("Linked message send")
                            }
                            Err(e) => {
                                eprintln!("{:?}", e)
                            }
                        }
                    }
                    bad_res => match bad_res {
                        Ok(x) => println!("{:?}", x),
                        Err(e) => println!("{:?}", e),
                    },
                };
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    }

    pub async fn create_buttons(deleted_chat_id: i64, link_text: String) -> WButtons {
        let buttons = WButtons::new(
            deleted_chat_id,
            link_text.parse().unwrap(),
            InlineKeyboardMarkup::new(vec![vec![
                KeyboardButton::new("next".to_string(), "/next".to_string()),
                KeyboardButton::new("last".to_string(), "/last".to_string()),
            ]]),
        );
        return buttons;
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
