use crate::database::api_token::find_token::find_token;
use crate::database::user::get_user_by_email::get_user_opt_by_email;
use crate::errors::error::Error;
use crate::startup::application::AppState;
use crate::ws::handle_socket::handle_socket;
use anyhow::Context;
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    tracing::info!("query => {:#?}", query);
    let email = query
        .get("email")
        .ok_or(Error::Auth("Missing email".to_string()))?
        .clone();
    let api_token = query
        .get("api_token")
        .ok_or(Error::Auth("Missing token".to_string()))?;
    let api_token = Uuid::from_str(api_token).context("Cannot deserialize UUID")?;
    let mut transaction = state.pool.begin().await.map_err(Error::from)?;
    let user = get_user_opt_by_email(&mut transaction, &email)
        .await?
        .ok_or(Error::Auth(String::from("User email is not present in DB")))?;
    let api_token = find_token(&mut transaction, &api_token)
        .await?
        .ok_or(Error::ApiTokenNotFound)?;
    if user.id != api_token.user_id {
        return Err(Error::UserNotFound);
    }
    tracing::info!("ws_handle => connected {:#?}", query);
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, addr, state, email, Uuid::new_v4())))
    // FIXME replace new_v4 with actual value
}
