#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use block_mesh_manager::database::spam_email::get_spam_emails::init_spam_emails_cache;
    use logger_general::tracing::get_sentry_layer;
    use dash_with_expiry::hash_map_with_expiry::HashMapWithExpiry;
    use dash_with_expiry::hash_set_with_expiry::HashSetWithExpiry;
    use std::sync::RwLock;
    use std::collections::HashMap;
    use database_utils::utils::connection::write_pool::write_pool;
    use block_mesh_common::email_client::client::EmailClient;
    use database_utils::utils::connection::channel_pool::channel_pool;
    use database_utils::utils::connection::follower_pool::follower_pool;
    use database_utils::utils::connection::unlimited_pool::unlimited_pool;
    use block_mesh_common::constants::DeviceType;
    use block_mesh_manager::worker::update_feature_flags::feature_flags_loop;
    use block_mesh_manager::utils::cache_envar::get_envar;
    use database_utils::utils::migrate::migrate;
    use std::process;
    use dashmap::DashMap;
    use block_mesh_common::interfaces::server_api::{CheckTokenResponseMap, GetTokenResponseMap};
    use std::mem;
    use logger_general::tracing::setup_tracing_stdout_only_with_sentry;
    use block_mesh_manager::database::user::create_test_user::create_test_user;
    use block_mesh_common::reqwest::http_client;
    use block_mesh_common::env::app_env_var::AppEnvVar;
    use block_mesh_common::env::env_var::EnvVar;
    use block_mesh_common::env::get_env_var_or_panic::get_env_var_or_panic;
    use block_mesh_common::env::load_dotenv::load_dotenv;
    use std::env;
    #[allow(unused_imports)]
    use logger_general::tracing::setup_tracing_stdout_only;
    use block_mesh_common::feature_flag_client::get_all_flags;
    #[cfg(not(target_env = "msvc"))]
    use tikv_jemallocator::Jemalloc;
    #[cfg(not(target_env = "msvc"))]
    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;
    use block_mesh_manager::configuration::get_configuration::get_configuration;
    use block_mesh_manager::startup::application::{AppState, Application};
    use secret::Secret;
    use std::sync::Arc;
}}

#[cfg(feature = "ssr")]
#[tracing::instrument(name = "main", skip_all)]
fn main() {
    let sentry_layer = get_sentry_layer();
    let sentry_url = env::var("SENTRY").unwrap_or_default();
    let sentry_sample_rate = env::var("SENTRY_SAMPLE_RATE")
        .unwrap_or("0.1".to_string())
        .parse()
        .unwrap_or(0.1);
    if sentry_layer {
        let _guard = sentry::init((
            sentry_url,
            sentry::ClientOptions {
                debug: env::var("APP_ENVIRONMENT").unwrap_or_default() == "local",
                sample_rate: sentry_sample_rate,
                traces_sample_rate: sentry_sample_rate,
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));
        mem::forget(_guard);
    }

    let _ = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { run().await });
    process::exit(1);
}

#[cfg(feature = "ssr")]
#[tracing::instrument(name = "run", skip_all, ret, err)]
async fn run() -> anyhow::Result<()> {
    load_dotenv();
    // setup_tracing_stdout_only();
    // console_subscriber::init(); // tokio-console
    setup_tracing_stdout_only_with_sentry();
    let configuration = get_configuration().expect("Failed to read configuration");
    tracing::info!("Starting with configuration {:#?}", configuration);
    let _gmail_password = get_env_var_or_panic(AppEnvVar::GmailAppPassword);
    let database_url = get_env_var_or_panic(AppEnvVar::DatabaseUrl);
    let _database_url = <EnvVar as AsRef<Secret<String>>>::as_ref(&database_url);
    let mailgun_token = get_env_var_or_panic(AppEnvVar::MailgunSendKey);
    let _mailgun_token = <EnvVar as AsRef<Secret<String>>>::as_ref(&mailgun_token);
    let db_pool = write_pool(None).await;
    let channel_pool = channel_pool(Some("CHANNEL_DATABASE_URL".to_string())).await;
    let env = get_envar("APP_ENVIRONMENT").await;
    tracing::info!("Database migration started");
    let unlimited_pg_pool = unlimited_pool(None).await;
    migrate(&unlimited_pg_pool, env)
        .await
        .expect("Failed to migrate database");
    tracing::info!("Database migration complete");
    let email_client = Arc::new(EmailClient::new(configuration.application.base_url.clone()).await);
    let client = http_client(DeviceType::AppServer);
    tracing::info!("Starting to get feature flags");
    let flags = Arc::new(RwLock::new(
        get_all_flags(&client, DeviceType::AppServer)
            .await
            .unwrap_or(HashMap::new()),
    ));
    tracing::info!("Finished getting feature flags");
    let redis_url = env::var("REDIS_URL")?;
    let redis_url = if redis_url.ends_with("#insecure") {
        redis_url
    } else {
        format!("{}#insecure", redis_url)
    };
    tracing::info!("Starting redis client");
    let redis_client = redis::Client::open(redis_url)?;
    tracing::info!("Found redis client URL");
    let redis = redis_client.get_multiplexed_async_connection().await?;
    tracing::info!("Finished redis client");
    let _ = create_test_user(&db_pool).await;
    let _ = init_spam_emails_cache(&db_pool).await;
    let check_token_map: CheckTokenResponseMap = Arc::new(DashMap::new());
    let get_token_map: GetTokenResponseMap = Arc::new(DashMap::new());
    let submit_bandwidth_limit = env::var("APP_BW_LIMIT")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let rate_limit = env::var("APP_RATE_LIMIT")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let task_limit = env::var("APP_TASK_LIMIT")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let follower_pool = follower_pool(Some("FOLLOWER_DATABASE_URL".to_string())).await;
    let invite_codes = HashMapWithExpiry::new();
    let wallet_addresses = HashMapWithExpiry::new();
    let cf_site_key = env::var("CF_SITE_KEY")?;
    let cf_secret_key = env::var("CF_SECRET_KEY")?;
    let cf_enforce = env::var("CF_ENFORCE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let recaptcha_site_key_v2 = env::var("RECAPTCHA_SITE_KEY_V2")?;
    let recaptcha_secret_key_v2 = env::var("RECAPTCHA_SECRET_KEY_V2")?;
    let recaptcha_site_key_v3 = env::var("RECAPTCHA_SITE_KEY_V3")?;
    let recaptcha_secret_key_v3 = env::var("RECAPTCHA_SECRET_KEY_V3")?;
    let hcaptcha_site_key = env::var("HCAPTCHA_SITE_KEY")?;
    let hcaptcha_secret_key = env::var("HCAPTCHA_SECRET_KEY")?;
    let enable_hcaptcha = env::var("ENABLE_HCAPTCHA")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let enable_recaptcha = env::var("ENABLE_RECAPTCHA")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let enable_proof_of_humanity = env::var("ENABLE_PROOF_OF_HUMANITY")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let app_state = Arc::new(AppState {
        wallet_login_nonce: HashMapWithExpiry::new(),
        rate_limiter: HashSetWithExpiry::new(),
        enable_hcaptcha,
        enable_recaptcha,
        enable_proof_of_humanity,
        hcaptcha_site_key,
        hcaptcha_secret_key,
        recaptcha_site_key_v2,
        recaptcha_secret_key_v2,
        recaptcha_site_key_v3,
        recaptcha_secret_key_v3,
        cf_enforce,
        cf_secret_key,
        cf_site_key,
        wallet_addresses,
        invite_codes,
        submit_bandwidth_limit,
        task_limit,
        rate_limit,
        check_token_map,
        get_token_map,
        email_client,
        pool: db_pool.clone(),
        follower_pool,
        channel_pool,
        client: client.clone(),
        flags: flags.clone(),
        redis,
    });
    tracing::info!("Starting application server");
    let application = Application::build(configuration, app_state.clone(), db_pool.clone()).await;
    let application_task = tokio::spawn(application.run());
    let feature_flags_update_task = tokio::spawn(feature_flags_loop(client, flags.clone()));

    tokio::select! {
        o = application_task => panic!("API {:?}", o),
        o = feature_flags_update_task => panic!("feature_flags_update_task {:?}", o)
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
