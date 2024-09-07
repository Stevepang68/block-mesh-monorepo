use crate::db_aggregators::users_ip_aggregator::users_ip_aggregator;
use crate::pg_listener::start_listening;
use block_mesh_common::constants::BLOCKMESH_PG_NOTIFY;
use block_mesh_common::env::load_dotenv::load_dotenv;
use logger_general::tracing::setup_tracing_stdout_only;
use serde_json::Value;
use std::env;

mod call_backs;
mod cron_jobs;
mod db_aggregators;
mod db_calls;
mod domain;
mod pg_listener;
mod utils;

use crate::call_backs::send_to_rx::send_to_rx;
use crate::cron_jobs::rpc_cron::rpc_worker_loop;
use crate::db_aggregators::aggregates_aggregator::aggregates_aggregator;
use crate::db_aggregators::analytics_aggregator::analytics_aggregator;
use crate::db_aggregators::daily_stats_aggregator::daily_stats_aggregator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    setup_tracing_stdout_only();
    tracing::info!("Starting worker");
    let db_pool = sqlx::PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let redis_client = redis::Client::open(env::var("REDIS_URL")?)?;
    let _redis = redis_client.get_multiplexed_async_connection().await?;
    let (tx, _rx) = tokio::sync::broadcast::channel::<Value>(5000);

    let rpc_worker_task = tokio::spawn(rpc_worker_loop(db_pool.clone()));

    let db_listen_task = tokio::spawn(start_listening(
        db_pool.clone(),
        vec![BLOCKMESH_PG_NOTIFY],
        tx.clone(),
        send_to_rx,
    ));
    let db_aggregator_users_ip_task =
        tokio::spawn(users_ip_aggregator(db_pool.clone(), tx.subscribe(), 100, 5));
    let db_aggregates_aggregator_task = tokio::spawn(aggregates_aggregator(
        db_pool.clone(),
        tx.subscribe(),
        100,
        5,
    ));
    let db_analytics_aggregator_task = tokio::spawn(analytics_aggregator(
        db_pool.clone(),
        tx.subscribe(),
        100,
        5,
    ));
    let db_daily_stats_aggregator_task = tokio::spawn(daily_stats_aggregator(
        db_pool.clone(),
        tx.subscribe(),
        100,
        5,
    ));

    tokio::select! {
        o = rpc_worker_task => eprintln!("rpc_worker_task exit {:?}", o),
        o = db_listen_task => eprintln!("db_listen_task exit {:?}", o),
        o = db_aggregator_users_ip_task => eprintln!("db_aggregator_users_ip_task exit {:?}", o),
        o = db_aggregates_aggregator_task => eprintln!("db_aggregates_aggregator_task exit {:?}", o),
        o = db_analytics_aggregator_task => eprintln!("db_analytics_aggregator_task exit {:?}", o),
        o = db_daily_stats_aggregator_task => eprintln!("db_daily_stats_aggregator_task exit {:?}", o)
    }
    Ok(())
}
