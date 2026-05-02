use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use opengithub_api::{build_app_with_config, config::AppConfig};
use serde_json::Value;
use tower::ServiceExt;

async fn request(
    path: &str,
    forwarded_for: &str,
    version: Option<&str>,
) -> axum::response::Response {
    let app = build_app_with_config(None, AppConfig::local_development());
    let mut builder = Request::builder()
        .uri(path)
        .header("x-forwarded-for", forwarded_for);
    if let Some(version) = version {
        builder = builder.header("x-github-api-version", version);
    }
    app.oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run")
}

#[tokio::test]
async fn api_responses_include_rate_limit_and_version_headers() {
    let response = request("/health", "198.51.100.10", Some("2022-11-28")).await;

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers["x-ratelimit-limit"], "60");
    assert_eq!(headers["x-ratelimit-remaining"], "59");
    assert_eq!(headers["x-ratelimit-used"], "1");
    assert_eq!(headers["x-ratelimit-resource"], "core");
    assert_eq!(headers["x-github-api-version"], "2022-11-28");
    assert!(headers.contains_key("x-ratelimit-reset"));
}

#[tokio::test]
async fn missing_api_routes_still_receive_rate_limit_headers() {
    let response = request("/api/unknown-rate-limited-resource", "198.51.100.11", None).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()["x-ratelimit-limit"], "60");
    assert_eq!(response.headers()["x-ratelimit-resource"], "core");
    assert_eq!(response.headers()["x-github-api-version"], "2022-11-28");
}

#[tokio::test]
async fn authenticated_bearer_requests_use_core_authenticated_hourly_tier() {
    let app = build_app_with_config(None, AppConfig::local_development());
    let request = Request::builder()
        .uri("/health")
        .header("authorization", "Bearer test-api-token")
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(request).await.expect("request should run");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-ratelimit-limit"], "5000");
    assert_eq!(response.headers()["x-ratelimit-resource"], "core");
}

#[tokio::test]
async fn search_resource_is_limited_to_thirty_requests_per_minute() {
    let app = build_app_with_config(None, AppConfig::local_development());
    let mut last = None;
    for _ in 0..31 {
        let request = Request::builder()
            .uri("/api/search?q=repo")
            .header("x-forwarded-for", "198.51.100.31")
            .body(Body::empty())
            .expect("request should build");
        last = Some(
            app.clone()
                .oneshot(request)
                .await
                .expect("request should run"),
        );
    }
    let response = last.expect("response should exist");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers()["x-ratelimit-limit"], "30");
    assert_eq!(response.headers()["x-ratelimit-remaining"], "0");
    assert_eq!(response.headers()["x-ratelimit-used"], "31");
    assert_eq!(response.headers()["x-ratelimit-resource"], "search");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body: Value = serde_json::from_slice(&body).expect("body should be json");
    assert_eq!(body["error"]["code"], "rate_limited");
}

#[tokio::test]
async fn rate_limit_endpoint_exposes_current_bucket_state() {
    let response = request("/rate_limit", "198.51.100.44", None).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-ratelimit-resource"], "core");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body: Value = serde_json::from_slice(&body).expect("body should be json");
    assert_eq!(body["resources"]["core"]["limit"], 60);
    assert_eq!(body["resources"]["core"]["used"], 1);
    assert_eq!(body["resources"]["search"]["limit"], 30);
    assert_eq!(body["rate"]["resource"], "core");
}
