#![allow(clippy::blocks_in_conditions)]

use crate::database::nonce::get_nonce_by_user_id::get_nonce_by_user_id;
use crate::database::user::get_user_by_email::get_user_opt_by_email;
use crate::errors::error::Error;
use crate::utils::cache_envar::get_envar;
use crate::utils::verify_cache::verify_with_cache;
use anyhow::anyhow;
use async_trait::async_trait;
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayer, AuthManagerLayerBuilder, AuthUser, AuthnBackend, UserId,
};
use block_mesh_manager_database_domain::domain::get_user_opt_by_id::get_user_opt_by_id;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use futures::StreamExt;
use futures_time::future::FutureExt;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisResult, ScanOptions};
use secret::Secret;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: Secret<String>,
    pub nonce: String,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: PgPool,
    con: MultiplexedConnection,
}

impl Backend {
    pub async fn get_expire() -> i64 {
        get_envar("REDIS_EXPIRE").await.parse().unwrap_or(86400)
    }
    pub fn new(db: PgPool, con: MultiplexedConnection) -> Self {
        Self { db, con }
    }

    pub fn authenticate_key_with_password(email: &str, password: &Secret<String>) -> String {
        format!("{}-{}", email, password.expose_secret())
    }
    pub fn authenticate_key_with_api_token(email: &str, api_token: &str) -> String {
        format!("{}-{}", email, api_token)
    }

    pub fn authenticate_key_with_user_id(uuid: &Uuid) -> String {
        uuid.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub email: String,
    pub nonce: String,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = SessionUser;
    type Credentials = Credentials;
    type Error = Error;

    #[tracing::instrument(name = "authenticate", skip_all)]
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let key = Backend::authenticate_key_with_password(&creds.email, &creds.password);
        let mut c = self.con.clone();
        if let Ok(redis_user) = get_user_from_redis(&key, &c).await {
            return Ok(Some(redis_user));
        }
        let pool = self.db.clone();
        let mut transaction = create_txn(&pool).await?;
        let user = match get_user_opt_by_email(&mut transaction, &creds.email).await {
            Ok(u) => u,
            Err(e) => {
                del_from_redis(&key, &mut c).await;
                return Err(Error::Auth(e.to_string()));
            }
        };
        commit_txn(transaction).await?;

        let user = match user {
            Some(u) => u,
            None => {
                del_from_redis(&key, &mut c).await;
                return Err(Error::Auth("User not found".to_string()));
            }
        };
        if !verify_with_cache(creds.password.as_ref(), user.password.as_ref()).await {
            return Err(Error::Auth("Invalid password".to_string()));
        }
        let session_user = SessionUser {
            id: user.id,
            nonce: creds.nonce,
            email: user.email,
        };
        save_to_redis(&key, &session_user, &mut c).await;
        Ok(Option::from(session_user))
    }

    #[tracing::instrument(name = "get_user", skip_all)]
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let key = Backend::authenticate_key_with_user_id(user_id);
        let mut c = self.con.clone();
        if let Ok(redis_user) = get_user_from_redis(&user_id.to_string(), &c).await {
            return Ok(Some(redis_user));
        }
        let pool = self.db.clone();
        let mut transaction = create_txn(&pool).await?;
        let user = match get_user_opt_by_id(&mut transaction, user_id).await {
            Ok(u) => u,
            Err(e) => {
                del_from_redis(&key, &mut c).await;
                return Err(Error::Auth(e.to_string()));
            }
        };

        let user = match user {
            Some(u) => u,
            None => {
                del_from_redis(&key, &mut c).await;
                return Err(Error::Auth("User not found".to_string()));
            }
        };

        let nonce = get_nonce_by_user_id(&mut transaction, &user.id)
            .await?
            .ok_or_else(|| Error::Auth("Nonce not found".to_string()))?;
        let session_user = SessionUser {
            id: user.id,
            email: user.email.clone(),
            nonce: nonce.nonce.as_ref().to_string(),
        };
        save_to_redis(&key, &session_user, &mut c).await;
        commit_txn(transaction).await?;
        Ok(Option::from(session_user))
    }
}

impl AuthUser for SessionUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.nonce.as_bytes() // We use the password hash as the auth
                              // hash--what this means
                              // is when the user changes their password the
                              // auth session becomes invalid.
    }
}

pub async fn authentication_layer(
    pool: &PgPool,
    con: &MultiplexedConnection,
) -> AuthManagerLayer<Backend, PostgresStore> {
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.unwrap();

    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(2)));

    let backend = Backend::new(pool.clone(), con.clone());
    AuthManagerLayerBuilder::new(backend, session_layer).build()
}

#[tracing::instrument(name = "get_user_from_redis", skip_all)]
pub async fn get_user_from_redis(
    key: &str,
    con: &MultiplexedConnection,
) -> anyhow::Result<SessionUser> {
    let mut c = con.clone();
    let redis_user: RedisResult<String> = c.get(key.to_string()).await;
    match redis_user {
        Ok(redis_user) => {
            if let Ok(value) = serde_json::from_str::<SessionUser>(&redis_user) {
                Ok(value)
            } else {
                Err(anyhow!("Cant deserialize user from redis".to_string()))
            }
        }
        Err(_) => Err(anyhow!("User not found".to_string())),
    }
}

#[tracing::instrument(name = "save_to_redis", skip_all)]
pub async fn save_to_redis(key: &str, session_user: &SessionUser, con: &mut MultiplexedConnection) {
    if let Ok(session_user) = serde_json::to_string(session_user) {
        let _: RedisResult<()> = con
            .set_ex(key, session_user, Backend::get_expire().await as u64)
            .await;
    }
}

#[tracing::instrument(name = "del_from_redis", skip_all)]
pub async fn del_from_redis(key: &str, con: &mut MultiplexedConnection) {
    let _: RedisResult<()> = con.del(key).await;
}

#[tracing::instrument(name = "del_from_redis_with_pattern", skip_all)]
pub async fn del_from_redis_with_pattern(
    key: &str,
    pattern: &str,
    con: &mut MultiplexedConnection,
) -> anyhow::Result<()> {
    let opts = ScanOptions::default().with_pattern(format!("{}{}", key, pattern));
    let values = con.scan_options::<String>(opts).await?;
    let values: Vec<_> = values
        .collect()
        .timeout(futures_time::time::Duration::from_millis(100))
        .await?;
    for k in values {
        let _: RedisResult<()> = con.del(k).await;
    }
    Ok(())
}
