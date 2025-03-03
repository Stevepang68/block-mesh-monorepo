use block_mesh_manager_database_domain::domain::nonce::Nonce;
use chrono::{Duration, Utc};
use dash_with_expiry::hash_map_with_expiry::HashMapWithExpiry;
use secret::Secret;
use sqlx::{Postgres, Transaction};
use std::env;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;

type CacheType = Arc<RwLock<HashMapWithExpiry<Uuid, Option<Nonce>>>>;
static CACHE: OnceCell<CacheType> = OnceCell::const_new();

#[tracing::instrument(name = "get_nonce_by_user_id", skip_all)]
pub async fn get_nonce_by_user_id(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &Uuid,
) -> anyhow::Result<Option<Nonce>> {
    let cache_flag = env::var("NONCE_CACHE")
        .unwrap_or("true".to_string())
        .parse()
        .unwrap_or(true);
    let cache = CACHE
        .get_or_init(|| async { Arc::new(RwLock::new(HashMapWithExpiry::new())) })
        .await;
    if cache_flag {
        if let Some(out) = cache.read().await.get(user_id).await {
            return Ok(out);
        }
    }
    let output = sqlx::query_as!(
        Nonce,
        r#"
        SELECT
        id,
        created_at,
        user_id,
        nonce as "nonce: Secret<String>"
        FROM nonces
        WHERE user_id = $1
        LIMIT 1"#,
        user_id
    )
    .fetch_optional(&mut **transaction)
    .await?;
    if cache_flag {
        let date = Utc::now() + Duration::milliseconds(60_000);
        cache
            .write()
            .await
            .insert(*user_id, output.clone(), Some(date))
            .await;
    }
    Ok(output)
}
