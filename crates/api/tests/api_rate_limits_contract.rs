use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode},
};
use opengithub_api::config::{AppConfig, AuthConfig};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

async fn database_pool() -> Option<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|value| !value.trim().is_empty())?;

    let pool = opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
        .ok()?;
    MIGRATOR.run(&pool).await.ok()?;
    Some(pool)
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("app URL"),
        api_url: Url::parse("http://localhost:3016").expect("api URL"),
        auth: Some(AuthConfig {
            google_client_id: "google-client-id.apps.googleusercontent.com".to_owned(),
            google_client_secret: "google-client-secret".to_owned(),
            session_secret: "test-session-secret-with-enough-entropy".to_owned(),
        }),
        session_cookie_name: "__Host-session".to_owned(),
        session_cookie_secure: false,
    }
}

async fn get_json(
    app: axum::Router,
    uri: &str,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    for (name, value) in headers.iter() {
        builder = builder.header(name, value);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, value)
}

fn header_i32(headers: &HeaderMap, name: &str) -> i32 {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or_else(|| panic!("{name} should be an integer header"))
}

#[tokio::test]
async fn api_responses_include_rate_limit_and_version_headers_without_database() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-github-api-version"),
        HeaderValue::from_static("2022-11-28"),
    );

    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, response_headers, body) = get_json(app, "/health", headers).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["database"]["status"], "degraded");
    assert_eq!(header_i32(&response_headers, "x-ratelimit-limit"), 60);
    assert_eq!(header_i32(&response_headers, "x-ratelimit-used"), 0);
    assert_eq!(header_i32(&response_headers, "x-ratelimit-remaining"), 60);
    assert_eq!(
        response_headers
            .get("x-ratelimit-resource")
            .and_then(|value| value.to_str().ok()),
        Some("core")
    );
    assert_eq!(
        response_headers
            .get("x-github-api-version-selected")
            .and_then(|value| value.to_str().ok()),
        Some("2022-11-28")
    );
}

#[tokio::test]
async fn invalid_bearer_token_keeps_anonymous_rate_limit_bucket() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static("Bearer definitely-not-a-real-token"),
    );
    headers.insert(
        HeaderName::from_static("x-forwarded-for"),
        HeaderValue::from_static("198.51.100.42"),
    );

    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, response_headers, body) = get_json(app, "/api/user", headers).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "not_authenticated");
    assert_eq!(header_i32(&response_headers, "x-ratelimit-limit"), 60);
    assert_eq!(header_i32(&response_headers, "x-ratelimit-remaining"), 60);
    assert_eq!(
        response_headers
            .get("x-ratelimit-resource")
            .and_then(|value| value.to_str().ok()),
        Some("core")
    );
}

#[tokio::test]
async fn anonymous_search_requests_are_limited_and_report_current_buckets() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping api-003 rate limit scenario; set TEST_DATABASE_URL");
        return;
    };

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());
    let client_ip = format!("203.0.113.{}", Uuid::new_v4().as_u128() % 200 + 1);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-forwarded-for"),
        HeaderValue::from_str(&client_ip).expect("ip header"),
    );
    headers.insert(
        HeaderName::from_static("x-github-api-version"),
        HeaderValue::from_static("2022-11-28"),
    );

    let mut last_status = StatusCode::OK;
    let mut last_headers = HeaderMap::new();
    let mut last_body = Value::Null;
    for _ in 0..31 {
        let (status, response_headers, body) =
            get_json(app.clone(), "/api/search/code?q=router", headers.clone()).await;
        last_status = status;
        last_headers = response_headers;
        last_body = body;
    }

    assert_eq!(last_status, StatusCode::FORBIDDEN);
    assert_eq!(last_body["error"]["code"], "rate_limited");
    assert_eq!(header_i32(&last_headers, "x-ratelimit-limit"), 30);
    assert_eq!(header_i32(&last_headers, "x-ratelimit-remaining"), 0);
    assert_eq!(header_i32(&last_headers, "x-ratelimit-used"), 31);
    assert_eq!(
        last_headers
            .get("x-ratelimit-resource")
            .and_then(|value| value.to_str().ok()),
        Some("search")
    );

    let (status, _headers, body) = get_json(app, "/rate_limit", headers).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["resources"]["search"]["limit"], 30);
    assert_eq!(body["resources"]["search"]["remaining"], 0);
    assert_eq!(body["resources"]["search"]["used"], 31);
    assert_eq!(body["resources"]["core"]["limit"], 60);
}
