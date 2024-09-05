use crate::db_aggregators::users_ip_aggregator::users_ip_aggregator;
use crate::pg_listener::start_listening;
use block_mesh_common::constants::BLOCKMESH_PG_NOTIFY;
use block_mesh_common::env::load_dotenv::load_dotenv;
use logger_general::tracing::setup_tracing_stdout_only;
use serde_json::Value;
use std::env;

mod call_backs;
mod db_aggregators;
mod pg_listener;
mod utils;

use crate::call_backs::send_to_rx::send_to_rx;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    setup_tracing_stdout_only();
    tracing::info!("Starting worker");
    let db_pool = sqlx::PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let redis_client = redis::Client::open(env::var("REDIS_URL")?)?;
    let _redis = redis_client.get_multiplexed_async_connection().await?;
    let (tx, rx) = flume::bounded::<Value>(5000);

    let db_listen_task = tokio::spawn(start_listening(
        db_pool.clone(),
        vec![BLOCKMESH_PG_NOTIFY],
        tx.clone(),
        send_to_rx,
    ));
    let db_aggregator_users_ip_task =
        tokio::spawn(users_ip_aggregator(db_pool.clone(), rx.clone(), 100, 5));
    tokio::select! {
        // o = db_channel_task => eprintln!("db_channel_task exit {:?}", o),
        o = db_listen_task => eprintln!("db_listen_task exit {:?}", o),
        o = db_aggregator_users_ip_task => eprintln!("db_aggregator_users_ip_task exit {:?}", o)
    }
    Ok(())
}
