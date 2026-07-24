use super::handler::handle_request;
use crate::config::AppConfig;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};

pub async fn serve_stdio(cfg: AppConfig) {
    let state = Arc::new(cfg);
    info!("Starting stdio transport");
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(request) => {
                let state = state.clone();
                tokio::spawn(async move {
                    let response = handle_request(state, request).await;
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{json}");
                    }
                });
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC: {e}");
            }
        }
    }
}

pub async fn serve_http(host: String, port: u16, cfg: AppConfig) {
    let state = Arc::new(cfg);
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let addr = format!("{host}:{port}");
    info!("Starting HTTP transport on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn mcp_handler(
    State(state): State<Arc<AppConfig>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(handle_request(state, request).await))
}

async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": chrono::Utc::now().to_rfc3339()
    }))
}

use axum::routing::get;
