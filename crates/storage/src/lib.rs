// crates/storage/src/lib.rs
use redis::{AsyncCommands};
use redis::aio::{MultiplexedConnection as ConnectionManager};
use chrono::Utc;

use cores::types::Email;
use cores::error::Error;

pub async fn create_inbox(
    conn: &ConnectionManager,
    address: &str,
    ttl_secs: usize,
) -> Result<(), Error> {
    let mut conn = conn.clone();

    let inbox_key = format!("inbox:{address}");
    let now = Utc::now().timestamp();
    let expires_at = now + ttl_secs as i64;

    let _: () = conn
        .hset_multiple(
            &inbox_key,
            &[
                ("created_at", now),
                ("expires_at", expires_at),
            ],
        )
        .await
        .map_err(|_| Error::RedisError)?;

    conn.expire(&inbox_key, ttl_secs as i64)
        .await
        .map_err(|_| Error::RedisError)?;

    Ok(())
}


pub async fn inbox_exists(
    conn: &ConnectionManager,
    address: &str,
) -> Result<bool, Error> {
    let mut conn = conn.clone();

    let key = format!("inbox:{address}");

    conn.exists(key)
        .await
        .map_err(|_| Error::RedisError)
}


pub async fn delete_inbox(
    conn: &ConnectionManager,
    address: &str,
) -> Result<(), Error> {
    let mut conn = conn.clone();

    let inbox_key = format!("inbox:{address}");
    let emails_key = format!("emails:{address}");

    let _: () = conn
        .del(&[inbox_key, emails_key])
        .await
        .map_err(|_| Error::RedisError)?;

    Ok(())
}


pub async fn store_email_in_redis(
    conn: &ConnectionManager,
    email: &Email,
) -> Result<(), Error> {

    let mut conn = conn.clone();

    for recipient in &email.recipients {

        let inbox_key = format!("inbox:{recipient}");
        let emails_key = format!("emails:{recipient}");
        let channel_key = format!("channel:inbox:{recipient}");

        let exists: bool = conn
            .exists(&inbox_key)
            .await
            .map_err(|_| Error::RedisError)?;

        if !exists {
            return Err(Error::InboxNotFound);
        }

        let json =
            serde_json::to_string(email)
                .map_err(|_| Error::RedisError)?;

        let count: i64 = conn
            .zcard(&emails_key)
            .await
            .map_err(|_| Error::RedisError)?;

        // limit emails per inbox
        if count >= 50 {
            return Err(Error::InboxFull);
        }

        let score = Utc::now().timestamp();

        conn.zadd(&emails_key, &json, score)
            .await
            .map_err(|_| Error::RedisError)?;

        let ttl: i64 = conn
            .ttl(&inbox_key)
            .await
            .map_err(|_| Error::RedisError)?;

        if ttl > 0 {
            conn.expire(&emails_key, ttl as i64)
                .await
                .map_err(|_| Error::RedisError)?;
        }

        // notify SSE listeners
        let _: () = conn
            .publish(channel_key, &json)
            .await
            .map_err(|_| Error::RedisError)?;
    }

    Ok(())
}