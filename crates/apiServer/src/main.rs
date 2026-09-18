// crates/apiServer/src/main.rs
use std::env;
use axum::{
    routing::{delete, get, post},
    Router,
};
use redis::{tokio::ConnectionManager, Client};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use anyhow::{Result};

mod routes;
use routes::*;

#[derive(Clone)]
pub struct AppState {
    pub redis_manager: ConnectionManager,
    pub redis_client: Client,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let redis_url =
            env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_client = Client::open(redis_url)?;
        let redis_manager =
            ConnectionManager::new(redis_client.clone()).await?;

        Ok(Self {
            redis_manager,
            redis_client,
        })
    }
}

pub async fn health() -> StatusCode { StatusCode::OK }

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt::init();
    // if dotenv().is_err() {
    //     error!("Warning: Failed to load .env");
    // } else {
    //     info!("Info: .env success");
    // }

    let redis_url =
        env::var("REDIS_URL")
            .unwrap_or("redis://127.0.0.1:6379".into());

    let frontend_origin =
        env::var("FRONTEND_DOMAIN")
            .unwrap_or("http://localhost:5173".into());

    let state = AppState::new().await?;

    let cors = CorsLayer::new()
        .allow_origin(frontend_origin.parse()?);

    let governor_conf = GovernorConfigBuilder::default()
        .per_hour(10)
        .burst_size(2)
        .finish()
        .unwrap();

    let rate_limit = GovernorLayer {
        config: std::sync::Arc::new(governor_conf),
    };

    let inbox_routes = Router::new()
        .route("/inboxes", post(create_inbox))
        .layer(rate_limit);

    let router = Router::new()
        .merge(inbox_routes)
        .route("/inboxes/:address/messages", get(get_message_from_redis))
        .route("/inboxes/:address/messages/:id", get(get_message))
        .route("/inboxes/:address", delete(delete_inbox))
        .route("/inboxes/:address/stream", get(stream_messages))
        .route("/health", get(health))
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, router).await?;

    Ok(())
}