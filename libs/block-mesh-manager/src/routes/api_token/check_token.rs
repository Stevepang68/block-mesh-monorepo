use crate::errors::error::Error;
use crate::startup::application::AppState;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::{Extension, Json};
use block_mesh_common::interfaces::server_api::{
    CheckTokenRequest, CheckTokenResponseEnum, GetTokenResponse,
};
use block_mesh_manager_database_domain::domain::get_user_and_api_token_by_email::get_user_and_api_token_by_email;
use dashmap::try_result::TryResult::Present;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use http::StatusCode;
use sqlx::PgPool;
use std::sync::Arc;

#[tracing::instrument(name = "check_token", skip(pool, state))]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CheckTokenRequest>,
) -> Result<impl IntoResponse, Error> {
    let email = body.email.clone().to_ascii_lowercase();
    let key = (email.clone(), body.api_token);
    let check_token_map = &state.check_token_map;
    if let Present(entry) = check_token_map.try_get(&key) {
        return match entry.value() {
            CheckTokenResponseEnum::ApiTokenMismatch => {
                Ok((StatusCode::OK, "Api Token Mismatch").into_response())
            }
            CheckTokenResponseEnum::UserNotFound => {
                Ok((StatusCode::OK, "User Not Found").into_response())
            }
            CheckTokenResponseEnum::ApiTokenNotFound => {
                Ok((StatusCode::OK, "Api Token Not Found").into_response())
            }
            CheckTokenResponseEnum::GetTokenResponse(r) => Ok(Json(r.clone()).into_response()),
        };
    }
    let mut transaction = create_txn(&pool).await?;
    let user_and_api_token = match get_user_and_api_token_by_email(&mut transaction, &email).await {
        Ok(user_and_api_token) => match user_and_api_token {
            Some(user) => user,
            None => {
                commit_txn(transaction).await?;
                check_token_map.insert(key, CheckTokenResponseEnum::UserNotFound);
                return Ok((StatusCode::OK, "User Not Found").into_response());
            }
        },
        Err(_) => {
            commit_txn(transaction).await?;
            check_token_map.insert(key, CheckTokenResponseEnum::UserNotFound);
            return Ok((StatusCode::OK, "User Not Found").into_response());
        }
    };
    if *user_and_api_token.token.as_ref() != body.api_token {
        commit_txn(transaction).await?;
        check_token_map.insert(key, CheckTokenResponseEnum::ApiTokenMismatch);
        return Ok((StatusCode::OK, "Api Token Mismatch").into_response());
    }
    let response = GetTokenResponse {
        api_token: Some(*user_and_api_token.token.as_ref()),
        message: None,
    };
    commit_txn(transaction).await?;
    check_token_map.insert(
        key,
        CheckTokenResponseEnum::GetTokenResponse(response.clone()),
    );
    Ok(Json(response).into_response())
}
