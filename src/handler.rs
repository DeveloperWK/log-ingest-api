use std::net::SocketAddr;

use axum::{
    Json,
    extract::{ConnectInfo, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    AppState,
    schema::{IngestLog, RawLog},
};

pub async fn ingest_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    payload: Result<Json<RawLog>, JsonRejection>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let payload = match payload {
        Ok(p) => p.0,
        Err(err) => {
            let msg = err.body_text();
            let msg = if msg.contains("missing field") {
                let field_name = msg.split('`').nth(1).unwrap_or("unknown");
                format!("Field '{}' is required.", field_name)
            } else {
                "Invalid JSON body.".to_string()
            };
            return Err((StatusCode::BAD_REQUEST, msg));
        }
    };
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
    // let AppState { redis } = state;
    // let redis_client = redis.clone();
    // produce_to_redis_stream(redis_client, &ingest)
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Failed to produce log to Redis stream: {:?}", e);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Failed to persist log data."),
    //         )
    //     })?;
    if let Err(e) = state.redis_batcher.push(ingest.clone()).await {
        eprintln!("Failed to push log to batcher: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to persist log data.".to_string(),
        ));
    }
    tracing::info!("Ingested log ID: {}", ingest.id);
    Ok(StatusCode::ACCEPTED)
}
