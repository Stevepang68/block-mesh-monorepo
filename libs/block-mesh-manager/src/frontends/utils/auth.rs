use crate::frontends::context::auth_context::AuthContext;
use crate::frontends::context::notification_context::NotificationContext;
use crate::frontends::utils::connectors::{pubkey, sign_message};
use anyhow::anyhow;
use leptos::leptos_dom;
#[allow(unused_imports)]
use leptos_dom::tracing;

use block_mesh_common::interfaces::server_api::{
    ConnectWalletRequest, ConnectWalletResponse, GetTokenResponse, LoginForm, RegisterForm,
    RegisterResponse,
};
use block_mesh_common::routes_enum::RoutesEnum;
use js_sys::Uint8Array;
use leptos::*;
use uuid::Uuid;

pub async fn register(blockmesh_url: &str, credentials: &RegisterForm) -> anyhow::Result<()> {
    let url = format!("{}{}", blockmesh_url, RoutesEnum::Static_UnAuth_RegisterApi);
    let client = reqwest::Client::new();
    let response = client.post(&url).form(credentials).send().await?;
    let response: RegisterResponse = response.json().await?;
    if response.status_code == 200 {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to register - {}",
            response.error.unwrap_or_default()
        ))
    }
}

pub async fn login(
    blockmesh_url: &str,
    credentials: &LoginForm,
) -> anyhow::Result<GetTokenResponse> {
    let blockmesh_url = if blockmesh_url.contains("app") {
        blockmesh_url.replace("app", "api")
    } else {
        blockmesh_url.to_string()
    };
    let url = format!("{}/api{}", blockmesh_url, RoutesEnum::Api_GetToken);
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&credentials)
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

pub async fn connect_wallet(
    origin: String,
    connect_wallet_request: ConnectWalletRequest,
) -> anyhow::Result<ConnectWalletResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/connect_wallet", origin))
        .header("Content-Type", "application/json")
        .json(&connect_wallet_request)
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

pub async fn connect_wallet_in_browser(wallet: String) -> bool {
    if wallet.is_empty() {
        return false;
    }
    let msg = Uuid::new_v4().to_string();
    let key = pubkey(&wallet).await;
    let sign = sign_message(&msg, &wallet).await;

    let uint8_array = Uint8Array::new(&sign);
    let mut signature = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut signature[..]);

    let origin = window().origin();

    let pubkey = key.as_string().unwrap();

    let notifications = expect_context::<NotificationContext>();

    match connect_wallet(
        origin,
        ConnectWalletRequest {
            pubkey: pubkey.clone(),
            message: msg.to_string(),
            signature,
        },
    )
    .await
    {
        Ok(_) => {
            let auth = expect_context::<AuthContext>();
            auth.wallet_address.set(Some(pubkey));
            notifications.set_success("Connected successfully");
            true
        }
        Err(_) => {
            notifications.set_error("Failed to connect");
            false
        }
    }
}
