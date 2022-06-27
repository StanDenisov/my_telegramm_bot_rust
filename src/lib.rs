use crate::models::{EditedMessage, Message, Update};
use crate::pg_service::PgService;
use crate::tg_service::TgClient;
use crate::web::WUpdate;
use futures::pin_mut;
use futures_core::Stream;
use std::sync::Arc;
use std::thread::sleep;
use tokio::pin;
use tokio_stream::StreamExt;

mod models;
mod pg_service;
mod tg_service;
mod web;

pub async fn start_server() {
    let tg_client = Arc::new(TgClient::new());
    let postgres_service = Arc::new(PgService::new().await);
    tokio::spawn({
        let tg_client = tg_client.clone();
        let postgres_service = postgres_service.clone();
        async move { process_updates(&tg_client, &postgres_service).await }
    });
}

async fn process_updates(tg_client: &TgClient, postgres_service: &PgService) {
    loop {
        let x = tg_client.get_updates(&&postgres_service.pg_pool).await;
        pin_mut!(x);
        while let Some(upd) = x.next().await {
            match upd {
                Ok(upd) => {
                    println!("got message");
                    match upd.edited_message {
                        Some(wem) => {
                            Update::new(1, upd.update_id)
                                .await
                                .insert(&&postgres_service.pg_pool)
                                .await;
                            EditedMessage::new(wem.message_id, wem.text)
                                .await
                                .change_message_text(&&postgres_service.pg_pool)
                                .await;
                        }
                        None => {}
                    }
                    match upd.message {
                        Some(wm) => match wm.text.as_str() {
                            "/history" => {
                                println!("history");
                                Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                let x = tg_client
                                    .history(&&postgres_service.pg_pool, wm.chat.id)
                                    .await;
                                println!("{:?}", x);
                            }
                            "/next" => {
                                println!("next");
                                Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                tg_client.next(&&postgres_service.pg_pool).await;
                            }
                            "/last" => {
                                println!("last");
                                Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                tg_client.last(&&postgres_service.pg_pool).await;
                            }
                            _ => {
                                println!("message");
                                Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                Message::new(wm.text, wm.chat.id, wm.message_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                            }
                        },
                        None => {}
                    }
                }
                Err(_) => break,
            }
        }
    }
}
