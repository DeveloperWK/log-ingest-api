use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use redis::aio::MultiplexedConnection;

use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::handler::ingest_handler;
mod handler;
mod redis_handler;
mod schema;

#[derive(Debug, Clone)]
pub struct AppState {
    redis: MultiplexedConnection,
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
    let state = AppState { redis: conn };
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
