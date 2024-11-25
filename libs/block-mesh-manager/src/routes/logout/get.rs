use crate::errors::error::Error;
use crate::middlewares::authentication::Backend;
use crate::startup::application::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::Extension;
use axum_login::AuthSession;
use block_mesh_common::routes_enum::RoutesEnum;
use std::sync::Arc;

#[tracing::instrument(name = "logout", skip_all)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(mut auth): Extension<AuthSession<Backend>>,
) -> Result<Redirect, Error> {
    let user = auth
        .logout()
        .await
        .map_err(|e| Error::Auth(e.to_string()))?;
    if let Some(session_user) = user {
        state.wallet_addresses.remove(&session_user.email);
    }
    Ok(Redirect::to(
        RoutesEnum::Static_UnAuth_Login.to_string().as_str(),
    ))
}
