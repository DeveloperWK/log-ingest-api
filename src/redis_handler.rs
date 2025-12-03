use redis::{AsyncCommands, RedisResult, aio::MultiplexedConnection};

use crate::schema::IngestLog;

pub async fn produce_to_redis_stream(
    mut con: MultiplexedConnection,
    ingest: &IngestLog,
) -> RedisResult<()> {
    let json_data = serde_json::to_string(&ingest).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::IoError,
            "Serialization failed",
            e.to_string(),
        ))
    })?;

    let _: () = con
        .xadd("ingest_log_stream", "*", &[("logs", &json_data.as_bytes())])
        .await?;

    Ok(())
}
