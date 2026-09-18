// crates/apiServer/src/routes.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};

use futures::{Stream, StreamExt};
use nanoid::nanoid;
use redis::AsyncCommands;

use apiServer::AppState;
use cores::types::Email;

use chrono::{Duration, Utc};

use std::convert::Infallible;


pub async fn create_inbox(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    let address = nanoid!();
    let expires_at = Utc::now() + Duration::hours(1);

    let mut conn = state.redis_manager.clone();

    let key = format!("inbox:{address}");

    let _: () = conn
        .hset_multiple(
            &key,
            &[
                ("created_at", Utc::now().timestamp()),
                ("expires_at", expires_at.timestamp()),
            ],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.expire(&key, 3600)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "address": address,
        "expires_at": expires_at
    })))
}

//
// GET /inboxes/:address/messages
//

pub async fn get_message_from_redis(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Vec<Email>>, StatusCode> {

    ensure_inbox_exists(&state, &address).await?;

    let mut conn = state.redis_manager.clone();

    let key = format!("emails:{address}");

    let data: Vec<String> =
        conn.zrevrange(key, 0, -1)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let emails = data
        .into_iter()
        .filter_map(|j| serde_json::from_str(&j).ok())
        .collect();

    Ok(Json(emails))
}

//
// GET /inboxes/:address/messages/:id
//

pub async fn get_message(
    State(state): State<AppState>,
    Path((address, id)): Path<(String, String)>,
) -> Result<Json<Email>, StatusCode> {

    ensure_inbox_exists(&state, &address).await?;

    let mut conn = state.redis_manager.clone();

    let key = format!("emails:{address}");

    let data: Vec<String> =
        conn.zrange(key, 0, -1)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for msg in data {
        let parsed: Email =
            serde_json::from_str(&msg).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if parsed.id == id {
            return Ok(Json(parsed));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

//
// DELETE /inboxes/:address
//

pub async fn delete_inbox(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<StatusCode, StatusCode> {

    ensure_inbox_exists(&state, &address).await?;

    let mut conn = state.redis_manager.clone();

    let _: () = conn
        .del([
            format!("inbox:{address}"),
            format!("emails:{address}")
        ])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

//
// GET /inboxes/:address/stream
//

pub async fn stream_messages(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {

    ensure_inbox_exists(&state, &address).await?;

    let mut pubsub = state
        .redis_client
        .get_async_pubsub()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let channel = format!("channel:inbox:{address}");

    pubsub
        .subscribe(channel)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = pubsub.on_message().map(|msg| {
        let payload: String = msg.get_payload().unwrap_or_default();

        Ok(Event::default().data(payload))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}


async fn ensure_inbox_exists(
    state: &AppState,
    address: &str,
) -> Result<(), StatusCode> {

    let mut conn = state.redis_manager.clone();

    let exists: bool =
        conn.exists(format!("inbox:{address}"))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}