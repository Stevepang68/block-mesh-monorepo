use crate::database::call_to_action::get_or_create_call_to_action::get_or_create_call_to_action;
use crate::domain::call_to_action::CallToActionName;
use crate::errors::error::Error;
use crate::middlewares::authentication::Backend;
use axum::response::Redirect;
use axum::{Extension, Form};
use axum_login::AuthSession;
use block_mesh_common::interfaces::server_api::CallToActionForm;
use block_mesh_common::routes_enum::RoutesEnum;
use sqlx::PgPool;

#[tracing::instrument(name = "call_to_action_post", skip(auth, pool))]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Extension(auth): Extension<AuthSession<Backend>>,
    Form(form): Form<CallToActionForm>,
) -> Result<Redirect, Error> {
    let mut transaction = pool.begin().await.map_err(Error::from)?;
    let user = auth.user.ok_or(Error::UserNotFound)?;
    get_or_create_call_to_action(
        &mut transaction,
        user.id,
        CallToActionName::from(form.name),
        form.status,
    )
    .await
    .map_err(Error::from)?;
    transaction.commit().await.map_err(Error::from)?;
    Ok(Redirect::to(&format!(
        "/ui{}",
        RoutesEnum::Static_Auth_Dashboard.to_string().as_str()
    )))
}
