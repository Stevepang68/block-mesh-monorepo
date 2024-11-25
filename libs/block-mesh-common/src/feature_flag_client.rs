use crate::constants::{DeviceType, BLOCK_MESH_FEATURE_FLAGS};
use dashmap::try_result::TryResult::Present;
use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FLAGS: [&str; 1] = ["polling_interval"];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlagValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl TryInto<bool> for FlagValue {
    type Error = ();

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            FlagValue::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

impl TryInto<f64> for FlagValue {
    type Error = ();

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            FlagValue::Number(n) => Ok(n),
            _ => Err(()),
        }
    }
}

#[tracing::instrument(name = "get_all_flags", skip_all, ret, err)]
pub async fn get_all_flags(
    client: &Client,
    device_type: DeviceType,
) -> anyhow::Result<DashMap<String, FlagValue>> {
    let flags: DashMap<String, FlagValue> = DashMap::new();
    for flag in FLAGS {
        tracing::info!("Fetching flag {:?}", flag);
        let value = match get_flag_value(flag, client, device_type).await {
            Ok(v) => v,
            Err(_e) => continue,
        };
        let value = match value {
            Some(v) => v,
            None => continue,
        };

        tracing::info!("Fetching flag {:?} from http , value = {:?}", flag, value);
        if value.is_boolean() {
            flags.insert(
                flag.to_string(),
                FlagValue::Boolean(value.as_bool().unwrap()),
            );
        } else if value.is_string() {
            flags.insert(flag.to_string(), FlagValue::String(value.to_string()));
        } else if value.is_number() {
            flags.insert(flag.to_string(), FlagValue::Number(value.as_f64().unwrap()));
        }
        tracing::info!("Finished fetching flag {:?} , value = {:?}", flag, value);
    }
    Ok(flags)
}

pub fn get_flag_value_from_map(
    map: &DashMap<String, FlagValue>,
    flag: &str,
    default: FlagValue,
) -> FlagValue {
    match map.try_get(flag) {
        Present(value) => value.value().clone(),
        _ => default,
    }
}

#[tracing::instrument(name = "get_flag_value", skip_all, ret, err)]
pub async fn get_flag_value(
    flag: &str,
    client: &Client,
    device_type: DeviceType,
) -> anyhow::Result<Option<Value>> {
    let url = format!(
        "{}/{}/read-flag/{}",
        BLOCK_MESH_FEATURE_FLAGS, device_type, flag
    );
    let response = client.get(&url).send().await?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(anyhow::anyhow!(response.text().await?));
    }
    let response: Value = response.json().await?;
    Ok(Some(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reqwest::http_client;
    use std::sync::Arc;
    use tracing_test::traced_test;
    use uuid::Uuid;

    #[tokio::test]
    #[traced_test]
    async fn test_test_boolean_false() {
        let client = http_client(DeviceType::Unknown);
        let value = get_flag_value("test_boolean_false", &client, DeviceType::Unknown).await;
        assert!(value.is_ok());
        let value = value.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(false, value);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_test_boolean_true() {
        let client = http_client(DeviceType::Unknown);
        let value = get_flag_value("test_boolean_true", &client, DeviceType::Unknown).await;
        assert!(value.is_ok());
        let value = value.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(true, value);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_missing_value() {
        let client = http_client(DeviceType::Unknown);
        let uuid = Uuid::new_v4();
        let value = get_flag_value(&uuid.to_string(), &client, DeviceType::Unknown).await;
        assert!(value.is_err());
    }

    #[tokio::test]
    #[traced_test]
    async fn test_polling_value() {
        let client = http_client(DeviceType::Unknown);
        let value = get_flag_value("polling_interval", &client, DeviceType::Unknown).await;
        assert!(value.is_ok());
        let value = value.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert!(value.is_number());
        let _ = FlagValue::Number(value.as_f64().unwrap());
    }

    #[tokio::test]
    #[traced_test]
    async fn test_all_values() {
        let client = http_client(DeviceType::Unknown);
        let _values = get_all_flags(&client, DeviceType::Unknown).await.unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_clone() {
        let flags1: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        flags1.insert("hello".to_string(), "world".to_string());
        let value1 = flags1.get("hello").unwrap().value().to_string();
        assert_eq!("world".to_string(), value1);
        let flags2 = flags1.clone();
        let value2 = flags2.get("hello").unwrap().value().to_string();
        assert_eq!("world".to_string(), value2);
        flags2.insert("hello".to_string(), "world2".to_string());
        let value1 = flags1.get("hello").unwrap().value().to_string();
        let value2 = flags2.get("hello").unwrap().value().to_string();
        assert_eq!(value1, value2);
        assert_eq!("world2".to_string(), value1);
        assert_eq!("world2".to_string(), value2);
    }
}
