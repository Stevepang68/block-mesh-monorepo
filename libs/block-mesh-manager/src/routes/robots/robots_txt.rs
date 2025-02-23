use crate::errors::error::Error;
use crate::startup::application::AppState;
use askama_axum::IntoResponse;
use axum::extract::State;
use http::StatusCode;
use std::sync::Arc;

pub async fn handler(State(_state): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let robots_txt = r#"User-agent: *
Allow: /$
Disallow: /
Crawl-delay: 120
    "#;
    Ok((StatusCode::OK, robots_txt).into_response())
}
