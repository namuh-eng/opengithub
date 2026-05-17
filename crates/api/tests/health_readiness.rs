use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use opengithub_api::config::{AppConfig, AuthConfig};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

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

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be json");
    (status, value)
}

#[tokio::test]
async fn health_stays_liveness_ok_when_database_is_unavailable() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, body) = get_json(app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["database"]["status"], "degraded");
}

#[tokio::test]
async fn ready_fails_when_database_is_unavailable() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, body) = get_json(app, "/ready").await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "unavailable");
    assert_eq!(body["database"]["status"], "unavailable");
}

#[tokio::test]
async fn ready_passes_when_database_accepts_queries() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping readiness happy path; set TEST_DATABASE_URL");
        return;
    };

    let app = opengithub_api::build_app_with_config(Some(pool), app_config());
    let (status, body) = get_json(app, "/ready").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"]["status"], "ok");
}
