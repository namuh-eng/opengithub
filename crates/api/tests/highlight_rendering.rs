use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use opengithub_api::domain::highlight::{highlight_code, HighlightCodeInput};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
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

async fn send_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should build");
    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be json");
    (status, value)
}

#[tokio::test]
async fn highlight_detects_language_tokens_and_symbols() {
    let highlighted = highlight_code(
        None,
        HighlightCodeInput {
            source: r#"export function repositoryPath(owner: string): string {
  const suffix = "settings";
  return `/${owner}/${suffix}`;
}
"#
            .to_owned(),
            path: Some("src/repository.ts".to_owned()),
            sha: Some("abc123".to_owned()),
            repository_id: None,
            language: None,
        },
    )
    .await
    .expect("source should highlight");

    assert_eq!(highlighted.language, "typescript");
    assert_eq!(highlighted.sha, "abc123");
    assert!(highlighted.supported_languages.len() >= 50);
    assert!(highlighted
        .lines
        .iter()
        .flat_map(|line| line.tokens.iter())
        .any(|token| token.class_name == "tok-keyword" && token.text.contains("export")));
    assert!(highlighted
        .symbols
        .iter()
        .any(|symbol| symbol.name == "repositoryPath" && symbol.kind == "function"));
}

#[tokio::test]
async fn highlight_language_override_changes_detection() {
    let highlighted = highlight_code(
        None,
        HighlightCodeInput {
            source: "SELECT * FROM repositories WHERE private = false;".to_owned(),
            path: Some("query.txt".to_owned()),
            sha: None,
            repository_id: None,
            language: Some("sql".to_owned()),
        },
    )
    .await
    .expect("source should highlight");

    assert_eq!(highlighted.language, "sql");
    assert!(highlighted.lines[0]
        .tokens
        .iter()
        .any(|token| token.class_name == "tok-keyword" && token.text == "SELECT"));
}

#[tokio::test]
async fn highlight_api_returns_error_envelope_for_empty_source() {
    let app = opengithub_api::build_app_with_config(
        None,
        opengithub_api::config::AppConfig::local_development(),
    );

    let (status, body) = send_json(
        app,
        "/api/highlight/render",
        json!({ "source": "", "path": "empty.rs" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    assert_eq!(body["status"], 422);
}

#[tokio::test]
async fn highlight_api_returns_error_envelope_for_malformed_body() {
    let app = opengithub_api::build_app_with_config(
        None,
        opengithub_api::config::AppConfig::local_development(),
    );

    let (status, body) = send_json(app, "/api/highlight/render", json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_json");
    assert_eq!(body["status"], 400);
}

#[tokio::test]
async fn highlight_cache_returns_cached_hit_with_database() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres highlight cache scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let request = HighlightCodeInput {
        source: "fn main() {\n  println!(\"hello\");\n}".to_owned(),
        path: Some("src/main.rs".to_owned()),
        sha: Some(format!("same-sha-{}", Uuid::new_v4())),
        repository_id: None,
        language: None,
    };

    let first = highlight_code(Some(&pool), request.clone())
        .await
        .expect("first highlight should succeed");
    let second = highlight_code(Some(&pool), request)
        .await
        .expect("second highlight should succeed");

    assert!(!first.cached);
    assert!(second.cached);
    assert_eq!(first.lines, second.lines);
}
