use crate::database::perks::add_perk_to_user::add_perk_to_user;
use crate::database::user::get_user_by_email::get_user_opt_by_email;
use crate::database::user::update_user_wallet::update_user_wallet;
use crate::domain::perk::PerkName;
use crate::errors::error::Error;
use crate::middlewares::authentication::Backend;
use crate::startup::application::AppState;
use axum::extract::State;
use axum::{Extension, Json};
use axum_login::AuthSession;
use block_mesh_common::interfaces::server_api::{ConnectWalletRequest, ConnectWalletResponse};
use block_mesh_manager_database_domain::domain::aggregate::AggregateName;
use block_mesh_manager_database_domain::domain::get_or_create_aggregate_by_user_and_name::get_or_create_aggregate_by_user_and_name_str;
use block_mesh_manager_database_domain::domain::update_aggregate::update_aggregate;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(name = "connect_wallet", skip_all)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(pool): Extension<PgPool>,
    Extension(auth): Extension<AuthSession<Backend>>,
    Json(body): Json<ConnectWalletRequest>,
) -> Result<Json<ConnectWalletResponse>, Error> {
    let user = auth.user.ok_or(Error::UserNotFound)?;
    let mut transaction = create_txn(&pool).await?;
    let db_user = get_user_opt_by_email(&mut transaction, &user.email)
        .await?
        .ok_or(Error::UserNotFound)?;
    let signature =
        Signature::try_from(body.signature.as_slice()).map_err(|_| Error::InternalServer)?;
    let pubkey = Pubkey::from_str(body.pubkey.as_str()).unwrap_or_default();
    let message = body.message.as_bytes();
    if signature.verify(pubkey.as_ref(), message) {
        match db_user.wallet_address {
            None => {
                if add_perk_to_user(
                    &mut transaction,
                    user.id,
                    PerkName::Wallet,
                    1.1,
                    0.0,
                    serde_json::from_str("{}").unwrap(),
                )
                .await
                .is_err()
                {
                    return Ok(Json(ConnectWalletResponse {
                        status: 52,
                        message: Some("(52) Cannot add perk".to_string()),
                    }));
                }
                if let Err(e) = update_user_wallet(&mut transaction, user.id, &body.pubkey).await {
                    let msg = if e.to_string().contains("violates unique constraint") {
                        "Wallet address already connected to a different user"
                    } else {
                        "(56) Cannot update user wallet"
                    };
                    return Ok(Json(ConnectWalletResponse {
                        status: 56,
                        message: Some(msg.to_string()),
                    }));
                }
                state
                    .wallet_addresses
                    .insert(user.email.clone(), Some(body.pubkey));
            }
            Some(wallet_address) => {
                let name = format!("{}_{}", AggregateName::WalletChange, Uuid::new_v4());
                let agg = match get_or_create_aggregate_by_user_and_name_str(
                    &mut transaction,
                    &name,
                    &db_user.id,
                )
                .await
                {
                    Ok(a) => a,
                    Err(_) => {
                        return Ok(Json(ConnectWalletResponse {
                            status: 72,
                            message: Some("(72) Cannot find agg".to_string()),
                        }));
                    }
                };
                if update_aggregate(&mut transaction, &agg.id, &Value::from(wallet_address))
                    .await
                    .is_err()
                {
                    return Ok(Json(ConnectWalletResponse {
                        status: 77,
                        message: Some("(77) Cannot update agg".to_string()),
                    }));
                }
                if let Err(e) = update_user_wallet(&mut transaction, user.id, &body.pubkey).await {
                    let msg = if e.to_string().contains("violates unique constraint") {
                        "(82) Wallet address already connected to a different user"
                    } else {
                        "(82) Cannot update user wallet "
                    };
                    return Ok(Json(ConnectWalletResponse {
                        status: 82,
                        message: Some(msg.to_string()),
                    }));
                }
                state
                    .wallet_addresses
                    .insert(user.email.clone(), Some(body.pubkey));
            }
        }
    } else {
        return Ok(Json(ConnectWalletResponse {
            status: 90,
            message: Some("(90) Cannot verify signature".to_string()),
        }));
    }
    commit_txn(transaction).await?;
    Ok(Json(ConnectWalletResponse {
        status: 200,
        message: None,
    }))
}
