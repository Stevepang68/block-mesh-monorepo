use crate::errors::Error;
use crate::state::WsAppState;
use crate::websocket::ws_handler::ws_handler;
use anyhow::anyhow;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use block_mesh_common::constants::BLOCKMESH_WS_REDIS_COUNT_KEY;
use database_utils::utils::health_check::health_check;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

#[tracing::instrument(name = "db_health", skip_all)]
pub async fn db_health(State(state): State<Arc<WsAppState>>) -> Result<impl IntoResponse, Error> {
    let pool = state.pool.clone();
    let mut transaction = create_txn(&pool).await?;
    health_check(&mut *transaction).await?;
    commit_txn(transaction).await?;
    Ok((StatusCode::OK, "OK"))
}

#[tracing::instrument(name = "server_health", skip_all)]
pub async fn server_health() -> Result<impl IntoResponse, Error> {
    Ok((StatusCode::OK, "OK"))
}

#[tracing::instrument(name = "health_follower", skip_all)]
pub async fn health_follower(
    State(state): State<Arc<WsAppState>>,
) -> Result<impl IntoResponse, Error> {
    let pool = state.follower_pool.clone();
    let mut transaction = create_txn(&pool).await?;
    health_check(&mut *transaction).await?;
    commit_txn(transaction).await?;
    Ok((StatusCode::OK, "OK"))
}

#[derive(Deserialize)]
pub struct AdminParam {
    code: String,
    user_id: Option<Uuid>,
    email: Option<String>,
}

pub async fn summary(
    State(state): State<Arc<WsAppState>>,
    Query(admin_param): Query<AdminParam>,
) -> Result<Json<Value>, Error> {
    if admin_param.code.is_empty()
        || admin_param.code != env::var("ADMIN_PARAM").unwrap_or_default()
    {
        Err(Error::InternalServer("Bad admin param".to_string()))
    } else {
        let sockets: Vec<String> = state
            .emails
            .read()
            .await
            .iter()
            .map(|i| i.to_string())
            .collect();
        Ok(Json(Value::from(sockets)))
    }
}

pub async fn status(
    State(state): State<Arc<WsAppState>>,
    Query(admin_param): Query<AdminParam>,
) -> Result<Json<Value>, Error> {
    if admin_param.code.is_empty()
        || admin_param.code != env::var("ADMIN_PARAM").unwrap_or_default()
    {
        Err(Error::InternalServer("Bad admin param".to_string()))
    } else {
        let mut output: Vec<String> = Vec::new();
        if let Some(user_id) = admin_param.user_id {
            if let Some(user) = state.user_ids.read().await.get(&user_id) {
                output.push(user.to_string());
            }
        }
        if let Some(email) = admin_param.email {
            if let Some(email) = state.emails.read().await.get(&email) {
                output.push(email.to_string());
            }
        }
        Ok(Json(Value::from(output)))
    }
}

#[tracing::instrument(name = "version", skip_all)]
pub async fn version() -> impl IntoResponse {
    (StatusCode::OK, env!("CARGO_PKG_VERSION"))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsResponse {
    pub counts: Vec<(String, i64)>,
    pub total: i64,
}

#[tracing::instrument(name = "stats", skip_all)]
pub async fn stats(State(state): State<Arc<WsAppState>>) -> Result<impl IntoResponse, Error> {
    let mut redis = state.redis.clone();
    let mut counts: Vec<(String, i64)> = Vec::with_capacity(50);
    let mut total = 0;
    let redis_results: Vec<String> = redis
        .keys(format!("{}*", BLOCKMESH_WS_REDIS_COUNT_KEY))
        .await
        .map_err(|e| Error::from(anyhow!(e.to_string())))?;
    for key in redis_results {
        let redis_count: RedisResult<i64> = redis.get(key.clone()).await;
        if let Ok(count) = redis_count {
            counts.push((key, count));
            total += count;
        }
    }
    Ok(Json(StatsResponse { counts, total }))
}

pub async fn app(listener: TcpListener, state: Arc<WsAppState>) {
    let router = Router::new()
        .route("/", get(server_health))
        .route("/server_health", get(server_health))
        .route("/db_health", get(db_health))
        .route("/health_follower", get(health_follower))
        .route("/version", get(version))
        .route("/stats", get(stats))
        .route("/summary", get(summary))
        .route("/status", get(status))
        .route("/ws", get(ws_handler))
        .route("/ext/ws", get(ws_handler))
        .with_state(state);

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
