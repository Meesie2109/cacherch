use crate::{errors::CacherchError, types::SearchResult};
use redis::AsyncCommands;
use serde_json;

const REDIS_URL: &str = "redis://127.0.0.1/";

pub async fn get_connection() -> Result<redis::aio::MultiplexedConnection, CacherchError> {
    let client = redis::Client::open(REDIS_URL)?;
    Ok(client.get_multiplexed_async_connection().await?)
}

pub async fn get_cached_results(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<Option<Vec<SearchResult>>, CacherchError> {
    if let Ok(cached) = conn.get::<_, String>(key).await {
        let results = serde_json::from_str(&cached)?;
        Ok(Some(results))
    } else {
        Ok(None)
    }
}

pub async fn set_cached_results(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
    results: &[SearchResult],
    ttl: usize,
) -> Result<(), CacherchError> {
    let json = serde_json::to_string(results)?;
    let _: () = conn.set_ex(key, json, ttl as u64).await?;
    Ok(())
}
