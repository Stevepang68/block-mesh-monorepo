use crate::database::nonce::get_nonce_by_user_id::get_nonce_by_user_id_pool;
use crate::database::user::get_user_by_email::get_user_opt_by_email_pool;
use crate::errors::error::Error;
use crate::middlewares::authentication::{Backend, Credentials};
use axum::response::Redirect;
use axum::{Extension, Form};
use axum_login::AuthSession;
use block_mesh_common::interfaces::server_api::LoginForm;
use block_mesh_common::routes_enum::RoutesEnum;
use secret::Secret;
use sqlx::PgPool;

#[tracing::instrument(name = "login_post", skip(form, auth))]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Extension(mut auth): Extension<AuthSession<Backend>>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, Error> {
    let user = get_user_opt_by_email_pool(&pool, &form.email.to_ascii_lowercase())
        .await?
        .ok_or_else(|| Error::UserNotFound)?;
    let nonce = get_nonce_by_user_id_pool(&pool, &user.id)
        .await?
        .ok_or_else(|| Error::NonceNotFound)?;
    let creds: Credentials = Credentials {
        email: form.email.to_ascii_lowercase(),
        password: Secret::from(form.password),
        nonce: nonce.nonce.as_ref().to_string(),
    };
    let session = match auth.authenticate(creds).await {
        Ok(Some(user)) => user,
        _ => {
            return Ok(Error::redirect(
                400,
                "Authentication failed",
                "Authentication failed. Please try again.",
                RoutesEnum::Static_UnAuth_Login.to_string().as_str(),
            ));
        }
    };
    match auth.login(&session).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Login failed: {:?} for user {}", e, user.id);
            return Ok(Error::redirect(
                400,
                "Login Failed",
                "Login failed. Please try again.",
                RoutesEnum::Static_UnAuth_Login.to_string().as_str(),
            ));
        }
    }
    Ok(Redirect::to("/ui/dashboard"))
}
