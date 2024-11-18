use anyhow::{anyhow, Context};
use block_mesh_common::constants::DeviceType;
use block_mesh_common::feature_flag_client::get_flag_value;
use block_mesh_common::interfaces::server_api::{
    ClientsMetadata, DashboardRequest, DashboardResponse, GetTaskRequest, GetTaskResponse,
    RegisterForm, RegisterResponse, ReportBandwidthRequest, ReportBandwidthResponse,
    ReportUptimeRequest, ReportUptimeResponse, RunTaskResponse, SubmitTaskRequest,
    SubmitTaskResponse,
};
use block_mesh_common::interfaces::server_api::{GetTokenResponse, LoginForm};
use block_mesh_common::reqwest::http_client;
use block_mesh_common::routes_enum::RoutesEnum;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use speed_test::download::test_download;
use speed_test::latency::test_latency;
use speed_test::metadata::fetch_metadata;
use speed_test::upload::test_upload;
use speed_test::Metadata;
use std::cmp;
use std::str::FromStr;
use tracing::Level;
use uuid::Uuid;

#[allow(dead_code)]
pub async fn dashboard(url: &str, credentials: &DashboardRequest) -> anyhow::Result<()> {
    let url = format!(
        "{}/{}/api{}",
        url,
        DeviceType::Cli,
        RoutesEnum::Api_Dashboard
    );
    let client = http_client(DeviceType::Cli);
    let response = client.post(&url).json(credentials).send().await?;
    let response: DashboardResponse = response.json().await?;
    info!("Dashboard data:");
    println!(
        "{}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );
    Ok(())
}

#[allow(dead_code)]
pub async fn register(url: &str, credentials: &RegisterForm) -> anyhow::Result<()> {
    let url = format!("{}{}", url, RoutesEnum::Static_UnAuth_RegisterApi);
    let client = http_client(DeviceType::Cli);
    let response = client.post(&url).form(credentials).send().await?;
    let response: RegisterResponse = response.json().await?;

    if response.status_code == 200 {
        info!("Successfully registered");
        Ok(())
    } else {
        error!(
            "Failed to registered with error : {}",
            response.error.unwrap_or_default()
        );
        Err(anyhow!("Failed to register"))
    }
}

#[allow(dead_code)]
pub async fn login_to_network(url: &str, login_form: LoginForm) -> anyhow::Result<Uuid> {
    let url = if url.contains("app") {
        url.replace("app", "api")
    } else {
        url.to_string()
    };
    let url = format!(
        "{}/{}/api{}",
        url,
        DeviceType::Cli,
        RoutesEnum::Api_GetToken
    );
    let client = http_client(DeviceType::Cli);
    let response: GetTokenResponse = client
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .json(&login_form)
        .send()
        .await?
        .json()
        .await?;
    match response.api_token {
        Some(api_token) => {
            info!("Login successful");
            Ok(api_token)
        }
        None => {
            error!("Failed to login");
            Err(anyhow!("missing api_token"))
        }
    }
}

#[tracing::instrument(name = "report_uptime", skip(api_token), err(level = Level::TRACE))]
pub async fn report_uptime(
    url: &str,
    email: &str,
    api_token: &str,
    session_metadata: ClientsMetadata,
) -> anyhow::Result<()> {
    let api_token = Uuid::from_str(api_token).context("Failed to parse UUID")?;
    let cloudflare_metadata = fetch_metadata().await.unwrap_or_default();

    let query = ReportUptimeRequest {
        email: email.to_string(),
        api_token,
        ip: Some(cloudflare_metadata.ip).filter(|ip| !ip.is_empty()),
    };

    let url = format!(
        "{}/{}/api{}",
        url,
        DeviceType::Cli,
        RoutesEnum::Api_ReportUptime
    );
    info!("Reporting uptime on {}", &url);
    if let Ok(response) = http_client(DeviceType::Cli)
        .post(url)
        .query(&query)
        .json(&session_metadata)
        .send()
        .await
        .inspect_err(|error| debug!("Error occurred while reporting uptime: {error}"))
    {
        let json = response.json::<ReportUptimeResponse>().await?;
        debug!("Uptime response: {json:?}");
        info!("Successfully submitted uptime report");
    } else {
        debug!("Reporting uptime failed");
    }

    Ok(())
}

#[tracing::instrument(name = "get_task", level = "trace", skip(api_token),err(level = Level::TRACE))]
pub async fn get_task(
    base_url: &str,
    email: &str,
    api_token: &Uuid,
) -> anyhow::Result<Option<GetTaskResponse>> {
    let body: GetTaskRequest = GetTaskRequest {
        email: email.to_string(),
        api_token: *api_token,
    };

    let response: Option<GetTaskResponse> = http_client(DeviceType::Cli)
        .post(format!(
            "{}/{}/api{}",
            base_url,
            DeviceType::Cli,
            RoutesEnum::Api_GetToken
        ))
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

#[tracing::instrument(name = "run_task", err(level = Level::TRACE))]
pub async fn run_task(
    url: &str,
    method: &str,
    headers: Option<Value>,
    body: Option<Value>,
) -> anyhow::Result<RunTaskResponse> {
    let client = http_client(DeviceType::Cli);
    let mut client = match method {
        "GET" => client.get(url),
        "POST" => match body {
            Some(v) => client.post(url).json(&v),
            None => client.post(url),
        },
        method => {
            return Err(anyhow!("Unsupported method: {}", method));
        }
    };

    if let Some(headers) = headers {
        let mut headers_map = HeaderMap::new();
        if headers.is_object() {
            headers.as_object().unwrap().into_iter().for_each(|(k, v)| {
                let header_name = HeaderName::from_str(k).unwrap();
                let header_value = HeaderValue::from_str(v.as_str().unwrap()).unwrap();
                headers_map.insert(header_name, header_value);
            });
            client = client.headers(headers_map)
        }
    }

    let response = client.send().await;
    match response {
        Ok(v) => {
            let status = v.status().as_u16();
            let raw = v.text().await?;

            Ok(RunTaskResponse {
                status: status.into(),
                raw,
            })
        }
        Err(e) => Err(anyhow!("run_task error: {e}")),
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(name = "submit_task", skip(api_token, response_raw), err(level = Level::TRACE))]
pub async fn submit_task(
    base_url: &str,
    email: &str,
    api_token: &Uuid,
    task_id: &Uuid,
    response_code: i32,
    response_raw: String,
    metadata: Metadata,
    response_time: f64,
) -> anyhow::Result<SubmitTaskResponse> {
    let Metadata {
        ip,
        country,
        asn,
        colo,
        city: _city,
    } = metadata;
    let query: SubmitTaskRequest = SubmitTaskRequest {
        email: email.to_string(),
        api_token: *api_token,
        task_id: *task_id,
        response_code: Some(response_code),
        country: Option::from(country),
        ip: Option::from(ip),
        asn: Option::from(asn),
        colo: Option::from(colo),
        response_time: Option::from(response_time),
        response_body: None,
    };
    let response = http_client(DeviceType::Cli)
        .post(format!(
            "{}/{}/api{}",
            base_url,
            DeviceType::Cli,
            RoutesEnum::Api_SubmitTask
        ))
        .query(&query)
        .body(response_raw)
        .send()
        .await?;
    Ok(response.json::<SubmitTaskResponse>().await?)
}

#[allow(dead_code)]
pub async fn task_poller(url: &str, email: &str, api_token: &str) -> anyhow::Result<()> {
    let api_token = Uuid::from_str(api_token).context("Failed to parse UUID")?;
    let task = get_task(url, email, &api_token)
        .await
        .inspect_err(|error| debug!("get_task error: {error}"))?;
    let metadata = fetch_metadata().await.unwrap_or_default();
    let task = task.context("Task not found")?;

    let task_start = std::time::Instant::now();
    let finished_task = match run_task(&task.url, &task.method, task.headers, task.body).await {
        Ok(v) => v,
        Err(e) => {
            let response_time = cmp::max(task_start.elapsed().as_millis(), 1) as f64;
            match submit_task(
                url,
                email,
                &api_token,
                &task.id,
                520,
                e.to_string(),
                metadata.clone(),
                response_time,
            )
            .await
            {
                Ok(_) => {
                    info!("successfully submitted failed task");
                }
                Err(e) => {
                    debug!("submit_task: error: {e}");
                }
            }
            return Err(anyhow!("submit_task errored"));
        }
    };
    let response_time = cmp::max(task_start.elapsed().as_millis(), 1) as f64;

    match submit_task(
        url,
        email,
        &api_token,
        &task.id,
        finished_task.status,
        finished_task.raw,
        metadata,
        response_time,
    )
    .await
    {
        Ok(_) => {
            info!("successfully submitted task");
        }
        Err(e) => {
            debug!("submit_task: error: {e}");
        }
    };
    Ok(())
}

#[tracing::instrument(name = "submit_bandwidth", err(level = Level::TRACE))]
pub async fn submit_bandwidth(
    url: &str,
    email: &str,
    api_token: &str,
) -> anyhow::Result<ReportBandwidthResponse> {
    let api_token = Uuid::from_str(api_token).context("Invalid UUID")?;
    let download_speed = test_download(100_000).await.unwrap_or_default();
    let upload_speed = test_upload(100_000).await.unwrap_or_default();
    let latency = test_latency().await.unwrap_or_default();
    let metadata = fetch_metadata().await.unwrap_or_default();

    let body = ReportBandwidthRequest {
        email: email.to_string(),
        api_token,
        download_speed,
        upload_speed,
        latency,
        city: metadata.city,
        country: metadata.country,
        ip: metadata.ip,
        asn: metadata.asn,
        colo: metadata.colo,
    };

    let response = http_client(DeviceType::Cli)
        .post(format!(
            "{}/{}/api{}",
            url,
            DeviceType::Cli,
            RoutesEnum::Api_SubmitBandwidth
        ))
        .json(&body)
        .send()
        .await?;
    let response: ReportBandwidthResponse = response.json().await?;
    Ok(response)
}

#[allow(dead_code)]
pub async fn get_polling_interval() -> f64 {
    let output = match get_flag_value(
        "cli_polling_interval",
        &http_client(DeviceType::Cli),
        DeviceType::Cli,
    )
    .await
    .unwrap_or(Some(Value::from(600_000.0)))
    {
        Some(polling_interval) => {
            if polling_interval.is_number() {
                polling_interval.as_f64().unwrap() / 1000.0
            } else {
                600.0
            }
        }
        None => 600.0,
    };
    if output < 60.0 {
        600.0
    } else {
        output
    }
}

#[test]
fn test_option_filter_none() {
    let ip = String::from("");
    let a = Some(ip.clone()).filter(|ip| !ip.is_empty());
    let b = if ip.is_empty() { None } else { Some(ip) };

    assert_eq!(a, b);
}

#[test]
fn test_option_filter_some() {
    let ip = String::from("some");
    let a = Some(ip.clone()).filter(|ip| !ip.is_empty());
    let b = if ip.is_empty() { None } else { Some(ip) };

    assert_eq!(a, b);
}
