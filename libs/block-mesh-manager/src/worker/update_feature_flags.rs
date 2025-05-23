use block_mesh_common::constants::DeviceType;
use block_mesh_common::feature_flag_client::{get_all_flags, FlagValue};
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[tracing::instrument(name = "feature_flags_loop", skip_all)]
pub async fn feature_flags_loop(client: Client, map: Arc<RwLock<HashMap<String, FlagValue>>>) {
    let sleep = env::var("FEATURE_FLAGS_SLEEP")
        .ok()
        .and_then(|var| var.parse().ok())
        .unwrap_or(60000);
    loop {
        if let Ok(updated_flags) = get_all_flags(&client, DeviceType::AppServer).await {
            for flag in updated_flags {
                if let Ok(mut map) = map.write() {
                    map.insert(flag.0, flag.1);
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(sleep)).await;
    }
}
