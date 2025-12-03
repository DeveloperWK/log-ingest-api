use std::net::SocketAddr;

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    AppState,
    redis_handler::produce_to_redis_stream,
    schema::{IngestLog, RawLog},
};

pub async fn ingest_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<RawLog>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ingest = IngestLog {
        id: uuid::Uuid::new_v4(),
        client_ip: addr.ip().to_string(),
        host: payload.host,
        level: payload.level,
        message: payload.message,
        metadata: payload.metadata,
        received_at: chrono::Utc::now().timestamp_millis(),
        service: payload.service,
        span_id: payload.span_id,
        timestamp: payload.timestamp,
        trace_id: payload.trace_id,
    };
    let AppState { redis } = state;
    let redis_client = redis.clone();
    produce_to_redis_stream(redis_client, &ingest)
        .await
        .map_err(|e| {
            eprintln!("Failed to produce log to Redis stream: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to persist log data."),
            )
        })?;
    tracing::info!("Ingested log ID: {}", ingest.id);
    Ok(StatusCode::ACCEPTED)
}
