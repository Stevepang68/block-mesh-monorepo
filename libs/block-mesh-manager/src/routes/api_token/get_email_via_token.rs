use crate::database::nonce::get_nonce_by_nonce::get_nonce_by_nonce_pool;
use crate::database::user::get_user_by_id::get_user_opt_by_id;
use crate::errors::error::Error;
use axum::{Extension, Json};
use block_mesh_common::interfaces::server_api::{
    GetEmailViaTokenRequest, GetEmailViaTokenResponse,
};
use sqlx::PgPool;

#[tracing::instrument(name = "get_email_via_token", skip(body))]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<GetEmailViaTokenRequest>,
) -> Result<Json<GetEmailViaTokenResponse>, Error> {
    let mut transaction = pool.begin().await.map_err(Error::from)?;
    let token = body.token;
    let nonce = get_nonce_by_nonce_pool(&pool, &token)
        .await?
        .ok_or_else(|| Error::NonceNotFound)?;
    let user = get_user_opt_by_id(&mut transaction, &nonce.user_id)
        .await?
        .ok_or_else(|| Error::UserNotFound)?;
    transaction.commit().await.map_err(Error::from)?;
    Ok(Json(GetEmailViaTokenResponse { email: user.email }))
}
