use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;

use crate::{
    api_types::error_response,
    domain::rate_limits::{
        api_version, check_rate_limit, fallback_bucket, resource_for_path, subject_from_headers,
        RateLimitBucket,
    },
    AppState,
};

pub async fn enforce_rate_limit(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_owned();
    let headers = req.headers().clone();
    let version = api_version(&headers);
    let subject = subject_from_headers(state.db.as_ref(), &state.config, &headers).await;
    let resource = resource_for_path(&path);
    let now = Utc::now();

    if let Some(pool) = state.db.as_ref() {
        match check_rate_limit(pool, &subject, resource, now).await {
            Ok(decision) if decision.allowed => {
                let mut response = next.run(req).await;
                add_headers(response.headers_mut(), &decision.bucket, &version);
                response
            }
            Ok(decision) => {
                let (status, Json(body)) = error_response(
                    StatusCode::FORBIDDEN,
                    "rate_limited",
                    format!(
                        "API rate limit exceeded for {} requests",
                        decision.bucket.resource
                    ),
                );
                let mut response = (status, Json(body)).into_response();
                add_headers(response.headers_mut(), &decision.bucket, &version);
                response
            }
            Err(error) => {
                tracing::warn!(%error, "rate limit check failed; allowing request without increment");
                let mut response = next.run(req).await;
                let bucket = fallback_bucket(&subject, resource, now);
                add_headers(response.headers_mut(), &bucket, &version);
                response
            }
        }
    } else {
        let mut response = next.run(req).await;
        let bucket = fallback_bucket(&subject, resource, now);
        add_headers(response.headers_mut(), &bucket, &version);
        response
    }
}

fn add_headers(headers: &mut header::HeaderMap, bucket: &RateLimitBucket, version: &str) {
    insert_i32(headers, "x-ratelimit-limit", bucket.limit);
    insert_i32(headers, "x-ratelimit-remaining", bucket.remaining);
    insert_i64(headers, "x-ratelimit-reset", bucket.reset);
    insert_i32(headers, "x-ratelimit-used", bucket.used);
    insert_str(headers, "x-ratelimit-resource", &bucket.resource);
    insert_str(headers, "x-github-api-version-selected", version);
}

fn insert_i32(headers: &mut header::HeaderMap, name: &'static str, value: i32) {
    insert_str(headers, name, &value.to_string());
}

fn insert_i64(headers: &mut header::HeaderMap, name: &'static str, value: i64) {
    insert_str(headers, name, &value.to_string());
}

fn insert_str(headers: &mut header::HeaderMap, name: &'static str, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        headers.insert(HeaderName::from_static(name), value);
    }
}
