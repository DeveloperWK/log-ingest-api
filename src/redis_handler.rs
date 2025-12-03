use crate::schema::IngestLog;
use redis::{AsyncCommands, RedisResult, aio::MultiplexedConnection};
use tokio::sync::Mutex;
use tokio::time::Duration;
#[derive(Debug)]
pub struct StreamBatcher {
    pub buffer: Mutex<Vec<IngestLog>>,
    pub redis_con: Mutex<MultiplexedConnection>,
    pub max_batch_size: usize,
    pub flush_interval: Duration,
}

impl StreamBatcher {
    pub fn new(
        redis_con: MultiplexedConnection,
        max_batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(max_batch_size)),
            redis_con: Mutex::new(redis_con),
            max_batch_size,
            flush_interval,
        }
    }

    pub async fn push(&self, log: IngestLog) -> RedisResult<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(log);

        if buffer.len() >= self.max_batch_size {
            self.flush(&mut buffer).await?;
        }

        Ok(())
    }

    pub async fn flush(&self, buffer: &mut Vec<IngestLog>) -> RedisResult<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut con = self.redis_con.lock().await;

        for log in buffer.drain(..) {
            let json_data = serde_json::to_string(&log).map_err(|e| {
                redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Serialization failed",
                    e.to_string(),
                ))
            })?;

            let _: () = con
                .xadd("ingest_log_stream", "*", &[("logs", json_data.as_bytes())])
                .await?;
        }

        Ok(())
    }
}
