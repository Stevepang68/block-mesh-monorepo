use crate::database::nonce::get_nonce_by_user_id::get_nonce_by_user_id_pool;
use crate::database::user::get_user_by_email::get_user_opt_by_email_pool;
use crate::errors::error::Error;
use crate::notification::notification_redirect::NotificationRedirect;
use crate::startup::application::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::{Extension, Form};
use block_mesh_common::interfaces::server_api::ResendConfirmEmailForm;
use block_mesh_common::routes_enum::RoutesEnum;
use sqlx::PgPool;
use std::sync::Arc;

#[tracing::instrument(name = "resend_confirm_email_post", skip(form, state))]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    State(state): State<Arc<AppState>>,
    Form(form): Form<ResendConfirmEmailForm>,
) -> Result<Redirect, Error> {
    let user = get_user_opt_by_email_pool(&pool, &form.email.to_ascii_lowercase())
        .await?
        .ok_or_else(|| Error::UserNotFound)?;
    let nonce = get_nonce_by_user_id_pool(&pool, &user.id)
        .await?
        .ok_or_else(|| Error::NonceNotFound)?;
    let _ = state
        .email_client
        .send_confirmation_email(&user.email, nonce.nonce.expose_secret())
        .await;
    Ok(NotificationRedirect::redirect(
        "Email Sent",
        "Please check your email",
        RoutesEnum::Static_UnAuth_Login.to_string().as_str(),
    ))
}
