// crates/smtp-server/src/main.rs
use tokio::net::TcpListener;
use tokio::spawn;
use tracing_subscriber;
use tracing;
use cores::types;
use cores::lib::handle_connection;
use apiServer::AppState;
use storage::{store_email_in_redis,inbox_exists};
use anyhow::{Result};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:2525").await?;
    let state = AppState::new().await?;

    loop {
        let (socket, _) = listener.accept().await?;

        let state = state.clone();

        spawn(async move {
            if let Err(err) = handle_connection(socket, state).await {
                tracing::error!("connection error: {:?}", err);
            }
        });
    }
}