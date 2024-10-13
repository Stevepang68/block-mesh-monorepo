use crate::state::AppState;
use crate::websocket::ws_handler::ws_handler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tracing::instrument(name = "health", skip_all)]
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tracing::instrument(name = "version", skip_all)]
pub async fn version() -> impl IntoResponse {
    (StatusCode::OK, env!("CARGO_PKG_VERSION"))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsResponse {
    queue: usize,
}

#[tracing::instrument(name = "stats", skip_all)]
pub async fn stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let websocket_manager = &state.websocket_manager;
    let queue = websocket_manager.broadcaster.queue.lock().await;
    Json(StatsResponse { queue: queue.len() })
}

pub async fn app(listener: TcpListener, state: Arc<AppState>) {
    let router = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/stats", get(stats))
        .route("/ws", get(ws_handler))
        .with_state(state);

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
