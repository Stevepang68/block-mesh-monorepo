use chrono::{DateTime, Duration, Utc};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUser {
    user_id: Uuid,
    ip: String,
    update_at: DateTime<Utc>,
}

impl RateLimitUser {
    pub fn new(user_id: &Uuid, ip: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            ip: ip.to_string(),
            update_at: Utc::now(),
        }
    }
}

pub async fn get_value_from_redis(
    con: &mut MultiplexedConnection,
    key: &str,
    fallback: &RateLimitUser,
) -> anyhow::Result<RateLimitUser> {
    let redis_user: String = match con.get(key.to_string()).await {
        Ok(u) => u,
        Err(_) => return Ok(fallback.clone()),
    };
    let redis_user = match serde_json::from_str::<RateLimitUser>(&redis_user) {
        Ok(u) => u,
        Err(_) => return Ok(fallback.clone()),
    };
    Ok(redis_user)
}

pub async fn touch_redis_value(con: &mut MultiplexedConnection, user_id: &Uuid, ip: &str) {
    let redis_user = RateLimitUser::new(user_id, ip);
    if let Ok(redis_user) = serde_json::to_string(&redis_user) {
        let _: RedisResult<()> = con.set(&user_id.to_string(), redis_user.clone()).await;
        let _: RedisResult<()> = con.set(&ip, redis_user).await;
    }
}

#[tracing::instrument(name = "filter_request", skip(con))]
pub async fn filter_request(
    con: &mut MultiplexedConnection,
    user_id: &Uuid,
    ip: &str,
) -> anyhow::Result<bool> {
    let now = Utc::now();
    let limit = env::var("FILTER_REQUEST")
        .unwrap_or("2500".to_string())
        .parse()
        .unwrap_or(2500);
    let diff = now - Duration::milliseconds(limit);
    let fallback = RateLimitUser::new(user_id, ip);
    let by_user: RateLimitUser = get_value_from_redis(con, &user_id.to_string(), &fallback).await?;
    let by_ip: RateLimitUser = get_value_from_redis(con, &ip, &fallback).await?;
    touch_redis_value(con, user_id, ip).await;
    Ok(max(by_user.update_at, by_ip.update_at) < diff)
}
