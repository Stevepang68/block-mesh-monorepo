use crate::domain::perk::Perk;
#[allow(unused_imports)]
use chrono::{Duration, Utc};
use dash_with_expiry::hash_map_with_expiry::HashMapWithExpiry;
use sqlx::{query_as, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;

#[allow(dead_code)]
struct Id {
    id: Uuid,
}

type CacheType = Arc<RwLock<HashMapWithExpiry<Uuid, Vec<Perk>>>>;
#[allow(dead_code)]
static CACHE: OnceCell<CacheType> = OnceCell::const_new();

pub async fn get_user_perks(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &Uuid,
) -> anyhow::Result<Vec<Perk>> {
    // let cache = CACHE
    //     .get_or_init(|| async { Arc::new(RwLock::new(HashMapWithExpiry::new())) })
    //     .await;
    // if let Some(out) = cache.read().await.get(user_id).await {
    //     return Ok(out);
    // }
    let perks = query_as!(
        Perk,
        r#"
        SELECT
        id, user_id, name, created_at, multiplier, one_time_bonus, data, updated_at
        FROM perks
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(&mut **transaction)
    .await?;
    // let date = Utc::now() + Duration::milliseconds(60_000);
    // cache
    //     .write()
    //     .await
    //     .insert(*user_id, perks.clone(), Some(date))
    //     .await;
    Ok(perks)
}
