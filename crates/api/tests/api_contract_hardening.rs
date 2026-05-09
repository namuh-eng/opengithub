use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
    },
};
use serde_json::{json, Value};
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
    reset_rate_limit_subject(&pool, "ip", "unknown").await;
    reset_rate_limit_subject(&pool, "ip", "203.0.113.44").await;
    Some(pool)
}

async fn reset_rate_limit_subject(pool: &PgPool, subject_type: &str, subject_key: &str) {
    sqlx::query(
        r#"
        DELETE FROM rate_limit_buckets
        WHERE subject_type = $1
          AND subject_key = $2
        "#,
    )
    .bind(subject_type)
    .bind(subject_key)
    .execute(pool)
    .await
    .expect("test rate limit bucket should reset");
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert")
}

async fn cookie_header(pool: &PgPool, config: &AppConfig, user: &User) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    upsert_session(
        pool,
        &session_id,
        Some(user.id),
        json!({ "provider": "google" }),
        expires_at,
    )
    .await
    .expect("session should persist");
    let set_cookie = session::set_cookie_header(config, &session_id, expires_at)
        .expect("signed cookie should be created");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let body = body.map(|value| value.to_string());
    send_raw(app, method, uri, cookie, body.as_deref(), HeaderMap::new()).await
}

async fn send_raw(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<&str>,
    extra_headers: HeaderMap,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::USER_AGENT, "api-contract-hardening-test/1.0")
        .header(header::ACCEPT, "application/json")
        .header("x-forwarded-for", "203.0.113.44");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    for (name, value) in extra_headers.iter() {
        builder = builder.header(name, value);
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .expect("request should build")
    } else {
        builder.body(Body::empty()).expect("request should build")
    };

    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!(
                "response should be JSON, got {error}: {}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

fn assert_error(body: &Value, status: u16, code: &str) {
    assert_eq!(body["status"], status);
    assert_eq!(body["error"]["code"], code);
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|value| !value.trim().is_empty()));
}

#[tokio::test]
async fn api_families_share_error_envelopes_and_redacted_request_logs() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping api-001 hardening scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "api-hardening-owner").await;
    let outsider = create_user(&pool, "api-hardening-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repo_name = format!("api-hardening-{}", Uuid::new_v4().simple());
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: Some("Final API contract hardening fixture".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    let marker = format!("needle{}", Uuid::new_v4().simple());
    upsert_search_document(
        &pool,
        owner.id,
        UpsertSearchDocument {
            repository_id: Some(repo.id),
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Code,
            resource_id: format!("api-hardening-code-{}", repo.id),
            title: format!("API hardening {marker}"),
            body: Some(format!("Searchable API hardening marker {marker}")),
            path: Some("src/lib.rs".to_owned()),
            language: Some("rust".to_owned()),
            branch: Some("main".to_owned()),
            visibility: RepositoryVisibility::Private,
            metadata: json!({ "feature": "api-001" }),
        },
    )
    .await
    .expect("search document should persist");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", owner.email, repo_name);

    let (status, headers, body) =
        send_json(app.clone(), Method::GET, "/api/user", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_json(&headers);
    assert_error(&body, 401, "not_authenticated");

    let mut secret_headers = HeaderMap::new();
    secret_headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer ogp_secret_token_value"),
    );
    secret_headers.insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_static("api-hardening-redaction"),
    );
    let (status, headers, body) = send_raw(
        app.clone(),
        Method::POST,
        "/api/repos",
        Some(&owner_cookie),
        Some("{\"name\":"),
        secret_headers,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_json(&headers);
    assert_error(&body, 400, "invalid_json");
    assert!(!body.to_string().contains("ogp_secret_token_value"));
    assert!(!body.to_string().contains(&owner_cookie));

    let (status, _headers, body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/issues"),
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&body, 403, "forbidden");

    let (status, _headers, body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues"),
        Some(&owner_cookie),
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error(&body, 422, "validation_failed");

    let (status, _headers, issue_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues"),
        Some(&owner_cookie),
        Some(json!({ "title": "Final contract issue" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(issue_body["number"], 1);

    let (status, _headers, pull_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/pulls"),
        Some(&owner_cookie),
        Some(json!({
            "title": "Final contract pull",
            "headRef": "feature/final-api-contract",
            "baseRef": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(pull_body["pull_request"]["number"], 2);

    let (status, _headers, workflow_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/actions/workflows"),
        Some(&owner_cookie),
        Some(json!({
            "name": "Contract CI",
            "path": ".github/workflows/contract.yml",
            "triggerEvents": ["push"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let workflow_id = workflow_body["id"].as_str().expect("workflow id");

    let (status, _headers, run_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/actions/workflows/{workflow_id}/runs"),
        Some(&owner_cookie),
        Some(json!({
            "headBranch": "main",
            "headSha": "abcdef0123456789",
            "event": "push"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(run_body["run_number"], 1);

    let (status, _headers, package_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages"),
        Some(&owner_cookie),
        Some(json!({
            "name": "opengithub-contract",
            "packageType": "container",
            "visibility": "private"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let package_id = package_body["id"].as_str().expect("package id");

    let (status, _headers, version_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages/{package_id}/versions"),
        Some(&owner_cookie),
        Some(json!({ "version": "1.0.0" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(version_body["version"], "1.0.0");

    let (status, _headers, conflict_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages/{package_id}/versions"),
        Some(&owner_cookie),
        Some(json!({ "version": "1.0.0" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&conflict_body, 409, "conflict");

    let (status, _headers, missing_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/packages/{}", Uuid::new_v4()),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&missing_body, 404, "not_found");

    let (status, _headers, search_body) = send_json(
        app,
        Method::GET,
        &format!("/api/search?q={marker}&kind=code&page=0&pageSize=1000"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(search_body["page"], 1);
    assert_eq!(search_body["pageSize"], 50);
    assert_eq!(search_body["total"], 1);
    assert!(search_body["items"].is_array());

    let rows: Vec<(String, String, i32, i32, Option<String>, Value)> = sqlx::query_as(
        r#"
        SELECT method, path, status, duration_ms, user_agent, metadata
        FROM api_request_logs
        WHERE path LIKE $1 OR request_id = 'api-hardening-redaction'
        ORDER BY created_at ASC
        "#,
    )
    .bind(format!("%{repo_name}%"))
    .fetch_all(&pool)
    .await
    .expect("request logs should read");
    assert!(rows.len() >= 8);
    assert!(rows.iter().any(|(method, path, status, _, _, _)| {
        method == "POST" && path.ends_with("/issues") && *status == 201
    }));
    assert!(rows.iter().any(|(method, path, status, _, _, _)| {
        method == "POST" && path == "/api/repos" && *status == 400
    }));
    for (method, path, status, duration_ms, user_agent, metadata) in &rows {
        assert!(!method.trim().is_empty());
        assert!(!path.trim().is_empty());
        assert!((100..=599).contains(status));
        assert!(*duration_ms >= 0);
        assert_eq!(
            user_agent.as_deref(),
            Some("api-contract-hardening-test/1.0")
        );
        assert!(metadata.get("method").is_some());
    }

    let rendered_logs = serde_json::to_string(&rows).expect("logs serialize");
    assert!(!rendered_logs.contains("ogp_secret_token_value"));
    assert!(!rendered_logs.contains(&owner_cookie));
    assert!(!rendered_logs.to_ascii_lowercase().contains("authorization"));
    assert!(!rendered_logs.to_ascii_lowercase().contains("cookie"));
    assert!(!rendered_logs.contains("test-session-secret"));
}
