mod api_key_validate;
mod handler;
mod redis_handler;
mod schema;

use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;

use crate::{handler::ingest_handler, redis_handler::StreamBatcher};

#[derive(Debug, Clone)]
pub struct AppState {
    redis_batcher: Arc<StreamBatcher>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let conn = client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis");

    let batcher = Arc::new(StreamBatcher::new(conn, 3, Duration::from_secs(5)));
    let batcher_clone = batcher.clone();
    let state = AppState {
        redis_batcher: batcher.clone(),
    };
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(batcher_clone.flush_interval).await;

            let mut buffer = batcher_clone.buffer.lock().await;
            if !buffer.is_empty() {
                if let Err(e) = batcher_clone.flush(&mut buffer).await {
                    eprintln!("Failed to flush logs: {:?}", e);
                }
            }
        }
    });
    let app = Router::new()
        .route("/health", get(health))
        .route("/ingest", post(ingest_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();
    println!("🚀 Server running on http://127.0.0.1:5000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn health() -> Result<impl IntoResponse, (StatusCode, String)> {
    let response = Json(json!({"message":"Hello world"}));
    Ok(response)
}
