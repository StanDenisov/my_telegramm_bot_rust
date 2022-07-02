use crate::models::{EditedMessage, Message, Update};
use crate::pg_service::PgService;
use crate::tg_service::TgClient;
use futures::pin_mut;
use std::sync::Arc;
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
                    match upd.edited_message {
                        Some(wem) => {
                            let update = Update::new(1, upd.update_id)
                                .await
                                .insert(&&postgres_service.pg_pool)
                                .await;
                            match update {
                                Ok(_) => {
                                    let e = EditedMessage::new(wem.message_id, wem.text)
                                        .await
                                        .change_message_text(&&postgres_service.pg_pool)
                                        .await;
                                    match e {
                                        Ok(_) => println!("message edited"),
                                        Err(x) => println!("{:?}", x),
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e)
                                }
                            }
                        }
                        None => {}
                    }
                    match upd.message {
                        Some(wm) => match wm.text.as_str() {
                            "/history" => {
                                println!("history");
                                let update = Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                match update {
                                    Ok(_) => {
                                        tg_client
                                            .history(&&postgres_service.pg_pool, wm.chat.id)
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("{:?}", e)
                                    }
                                }
                            }
                            "/exit" => {
                                println!("exit");
                                let update = Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                match update {
                                    Ok(_) => {
                                        tg_client
                                            .exit(&&postgres_service.pg_pool, wm.chat.id)
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("{:?}", e)
                                    }
                                }
                            }
                            _ => {
                                println!("message");
                                let update = Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                match update {
                                    Ok(_) => {
                                        let message =
                                            Message::new(wm.text, wm.chat.id, wm.message_id)
                                                .await
                                                .insert(&&postgres_service.pg_pool)
                                                .await;
                                        match message {
                                            Ok(_) => {
                                                println!("message saved")
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
                        },
                        None => {}
                    }
                    match upd.callback_query {
                        Some(wc) => match wc.data.as_str() {
                            "/next" => {
                                println!("next");
                                let update = Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                match update {
                                    Ok(_) => {
                                        tg_client
                                            .next(&&postgres_service.pg_pool, wc.message.chat.id)
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("{:?}", e)
                                    }
                                }
                            }
                            _ => {
                                println!("last");
                                let update = Update::new(1, upd.update_id)
                                    .await
                                    .insert(&&postgres_service.pg_pool)
                                    .await;
                                match update {
                                    Ok(_) => {
                                        tg_client
                                            .last(&&postgres_service.pg_pool, wc.message.chat.id)
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("{:?}", e)
                                    }
                                }
                            }
                        },
                        None => {}
                    }
                }
                Err(_) => break,
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
