use futures::future;

use tgbot::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
    future::pending::<()>().await;
}
