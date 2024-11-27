use crate::email::{Email, EmailType};
use crate::errors::Error;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use block_mesh_common::interfaces::server_api::SendEmail;
use database_utils::utils::health_check::health_check;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use reqwest::StatusCode;
use std::env;

#[tracing::instrument(name = "db_health", skip_all)]
pub async fn db_health(State(state): State<AppState>) -> Result<impl IntoResponse, Error> {
    let data_sink_db_pool = &state.emails_db_pool;
    let mut transaction = create_txn(data_sink_db_pool).await?;
    health_check(&mut *transaction).await?;
    commit_txn(transaction).await?;
    Ok((StatusCode::OK, "OK"))
}

#[tracing::instrument(name = "server_health", skip_all)]
pub async fn server_health() -> Result<impl IntoResponse, Error> {
    Ok((StatusCode::OK, "OK"))
}

#[tracing::instrument(name = "send_email", skip_all)]
pub async fn send_email(
    State(state): State<AppState>,
    Query(send_params): Query<SendEmail>,
) -> Result<impl IntoResponse, Error> {
    if send_params.code.is_empty()
        || send_params.code != env::var("ADMIN_PARAM").unwrap_or_default()
    {
        Err(Error::InternalServer("Bad admin param".to_string()))
    } else {
        let email_type = EmailType::from(send_params.email_type);
        match email_type {
            EmailType::ConfirmEmail => {
                let mut transaction = create_txn(&state.emails_db_pool).await?;
                let result = state
                    .email_client
                    .send_confirmation_email_aws(&send_params.email_address, &send_params.nonce)
                    .await?;
                Email::create_email(
                    &mut transaction,
                    &send_params.user_id,
                    &email_type,
                    &send_params.email_address,
                    &result.message_id.unwrap_or_default(),
                )
                .await?;
                commit_txn(transaction).await?;
                Ok(StatusCode::OK)
            }
            EmailType::ResetPassword => {
                let mut transaction = create_txn(&state.emails_db_pool).await?;
                let result = state
                    .email_client
                    .send_reset_password_email_aws(&send_params.email_address, &send_params.nonce)
                    .await?;
                Email::create_email(
                    &mut transaction,
                    &send_params.user_id,
                    &email_type,
                    &send_params.email_address,
                    &result.message_id.unwrap_or_default(),
                )
                .await?;
                commit_txn(transaction).await?;
                Ok(StatusCode::OK)
            }
            EmailType::Unknown => Err(Error::InternalServer("Unknown email type".to_string())),
        }
    }
}
#[tracing::instrument(name = "version", skip_all)]
pub async fn version() -> impl IntoResponse {
    (StatusCode::OK, env!("CARGO_PKG_VERSION"))
}
pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(server_health))
        .route("/server_health", get(server_health))
        .route("/db_health", get(db_health))
        .route("/version", get(version))
        .route("/send_email", get(send_email))
        .with_state(state)
}
