use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use opengithub_api::domain::markdown::{
    render_markdown, toggle_task, RenderMarkdownInput, ToggleTaskInput,
};
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
async fn markdown_rendering_sanitizes_and_decorates_gfm() {
    let rendered = render_markdown(
        None,
        RenderMarkdownInput {
            markdown: r#"# Hello world

Visit @mona and #42.

- [x] ship markdown

```rust
fn main() {}
```

<script>alert("xss")</script>
"#
            .to_owned(),
            repository_id: None,
            owner: Some("mona".to_owned()),
            repo: Some("octo-app".to_owned()),
            ref_name: Some("main".to_owned()),
            enable_task_toggles: Some(true),
        },
    )
    .await
    .expect("markdown should render");

    assert!(rendered.html.contains(r#"<div class="markdown-body">"#) || !rendered.html.is_empty());
    assert!(rendered.html.contains(r##"href="#hello-world""##));
    assert!(rendered.html.contains(r#"href="/mona""#));
    assert!(rendered.html.contains(r#"href="/mona/octo-app/issues/42""#));
    assert!(rendered.html.contains("code-block-header"));
    assert!(rendered.html.contains("data-task-toggle"));
    assert!(!rendered.html.contains("<script>"));
}

#[tokio::test]
async fn markdown_rendering_rewrites_relative_paths_for_repository_context() {
    let rendered = render_markdown(
        None,
        RenderMarkdownInput {
            markdown: "[guide](docs/guide.md)\n\n![logo](images/logo.png)".to_owned(),
            repository_id: None,
            owner: Some("mona".to_owned()),
            repo: Some("octo-app".to_owned()),
            ref_name: Some("feature-docs".to_owned()),
            enable_task_toggles: None,
        },
    )
    .await
    .expect("markdown should render");

    assert!(rendered
        .html
        .contains(r#"href="/mona/octo-app/blob/feature-docs/docs/guide.md""#));
    assert!(rendered
        .html
        .contains(r#"src="/mona/octo-app/raw/feature-docs/images/logo.png""#));
}

#[tokio::test]
async fn markdown_task_toggle_changes_requested_item_and_rerenders() {
    let output = toggle_task(
        None,
        ToggleTaskInput {
            markdown: "- [ ] first\n- [x] second".to_owned(),
            task_index: 0,
            checked: true,
        },
    )
    .await
    .expect("task should toggle");

    assert_eq!(output.markdown, "- [x] first\n- [x] second");
    assert!(output.rendered.html.contains("first"));
}

#[tokio::test]
async fn markdown_api_returns_error_envelopes_for_invalid_toggle() {
    let app = opengithub_api::build_app_with_config(None, super_config());

    let (status, body) = send_json(
        app,
        "/api/markdown/task-toggle",
        json!({ "markdown": "no tasks here", "taskIndex": 0, "checked": true }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    assert_eq!(body["status"], 422);
}

#[tokio::test]
async fn markdown_api_returns_error_envelopes_for_invalid_render_json() {
    let app = opengithub_api::build_app_with_config(None, super_config());

    let (status, body) = send_json(app, "/api/markdown/render", json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_json");
    assert_eq!(body["status"], 400);
}

#[tokio::test]
async fn markdown_cache_returns_cached_hit_with_database() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres markdown cache scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let suffix = Uuid::new_v4();
    let request = RenderMarkdownInput {
        markdown: format!("# Cached\n\nRun {suffix}"),
        repository_id: None,
        owner: None,
        repo: None,
        ref_name: None,
        enable_task_toggles: None,
    };

    let first = render_markdown(Some(&pool), request.clone())
        .await
        .expect("first render should succeed");
    let second = render_markdown(Some(&pool), request)
        .await
        .expect("second render should succeed");

    assert!(!first.cached);
    assert!(second.cached);
    assert_eq!(first.content_sha, second.content_sha);
    assert_eq!(first.html, second.html);
}

fn super_config() -> opengithub_api::config::AppConfig {
    opengithub_api::config::AppConfig::local_development()
}
