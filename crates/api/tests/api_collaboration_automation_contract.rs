use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
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
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
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
        serde_json::from_slice(&bytes).expect("response should be json")
    };
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn collaboration_and_automation_routes_use_session_auth_and_standard_envelopes() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping collaboration/automation API contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "api-collab-owner").await;
    let outsider = create_user(&pool, "api-collab-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repo_name = format!("api-collab-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: Some("API collaboration contract fixture".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let base = format!("/api/repos/{}/{}", owner.email, repo_name);

    let (anonymous_status, anonymous_headers, anonymous_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/issues"),
        None,
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::FORBIDDEN);
    assert_json(&anonymous_headers);
    assert_eq!(anonymous_body["error"]["code"], "forbidden");

    let (forbidden_status, _forbidden_headers, forbidden_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/issues"),
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden_body["status"], 403);

    let (invalid_issue_status, _invalid_issue_headers, invalid_issue_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues"),
        Some(&owner_cookie),
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(invalid_issue_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_issue_body["error"]["code"], "validation_failed");

    let (issue_status, issue_headers, issue_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues"),
        Some(&owner_cookie),
        Some(json!({
            "title": "REST issue contract",
            "body": "Created through the public API contract"
        })),
    )
    .await;
    assert_eq!(issue_status, StatusCode::CREATED);
    assert_json(&issue_headers);
    assert_eq!(issue_body["number"], 1);
    assert_eq!(issue_body["author_user_id"], owner.id.to_string());

    let (issues_status, _issues_headers, issues_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/issues?page=0&page_size=1000"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(issues_status, StatusCode::OK);
    assert_eq!(issues_body["page"], 1);
    assert_eq!(issues_body["pageSize"], 100);
    assert_eq!(issues_body["total"], 1);

    let (comment_status, _comment_headers, comment_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues/1/comments"),
        Some(&owner_cookie),
        Some(json!({ "body": "Confirmed through session-auth API." })),
    )
    .await;
    assert_eq!(comment_status, StatusCode::CREATED);
    assert_eq!(comment_body["eventType"], "commented");
    assert_eq!(
        comment_body["actor"]["login"],
        owner.username.as_deref().unwrap_or(&owner.email)
    );
    assert_eq!(
        comment_body["comment"]["body"],
        "Confirmed through session-auth API."
    );

    let (reaction_status, _reaction_headers, reaction_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/issues/1/reactions"),
        Some(&owner_cookie),
        Some(json!({ "content": "thumbs_up" })),
    )
    .await;
    assert_eq!(reaction_status, StatusCode::CREATED);
    assert_eq!(reaction_body["user_id"], owner.id.to_string());

    let (pull_status, _pull_headers, pull_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/pulls"),
        Some(&owner_cookie),
        Some(json!({
            "title": "REST pull contract",
            "body": "Expose pull requests through the API.",
            "headRef": "feature/api-contract",
            "baseRef": "main"
        })),
    )
    .await;
    assert_eq!(pull_status, StatusCode::CREATED);
    assert_eq!(pull_body["pull_request"]["number"], 2);

    let (pulls_status, _pulls_headers, pulls_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/pulls?pageSize=1000"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(pulls_status, StatusCode::OK);
    assert_eq!(pulls_body["pageSize"], 100);
    assert_eq!(pulls_body["total"], 1);

    let (workflow_status, _workflow_headers, workflow_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/actions/workflows"),
        Some(&owner_cookie),
        Some(json!({
            "name": "CI",
            "path": ".github/workflows/ci.yml",
            "triggerEvents": ["push", "pull_request"]
        })),
    )
    .await;
    assert_eq!(workflow_status, StatusCode::CREATED);
    assert_eq!(workflow_body["name"], "CI");
    let workflow_id = workflow_body["id"].as_str().expect("workflow id");

    let (run_status, _run_headers, run_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/actions/workflows/{workflow_id}/runs"),
        Some(&owner_cookie),
        Some(json!({
            "headBranch": "main",
            "headSha": "0123456789abcdef",
            "event": "push"
        })),
    )
    .await;
    assert_eq!(run_status, StatusCode::CREATED);
    assert_eq!(run_body["run_number"], 1);
    let run_id = run_body["id"].as_str().expect("run id");

    let (runs_status, _runs_headers, runs_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{base}/actions/runs?page=1&pageSize=5"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(runs_status, StatusCode::OK);
    assert_eq!(runs_body["total"], 1);
    assert_eq!(runs_body["items"][0]["id"], run_id);

    let (transition_status, _transition_headers, transition_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{base}/actions/runs/{run_id}"),
        Some(&owner_cookie),
        Some(json!({ "status": "completed", "conclusion": "success" })),
    )
    .await;
    assert_eq!(transition_status, StatusCode::OK);
    assert_eq!(transition_body["status"], "completed");
    assert_eq!(transition_body["conclusion"], "success");

    let (package_status, _package_headers, package_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages"),
        Some(&owner_cookie),
        Some(json!({
            "name": "opengithub-api",
            "packageType": "container",
            "visibility": "private"
        })),
    )
    .await;
    assert_eq!(package_status, StatusCode::CREATED);
    let package_id = package_body["id"].as_str().expect("package id");

    let (version_status, _version_headers, version_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages/{package_id}/versions"),
        Some(&owner_cookie),
        Some(json!({
            "version": "sha-0123456",
            "manifest": { "image": "opengithub-api", "tag": "sha-0123456" },
            "blobKey": "packages/container/opengithub-api/sha-0123456",
            "sizeBytes": 128
        })),
    )
    .await;
    assert_eq!(version_status, StatusCode::CREATED);
    assert_eq!(version_body["version"], "sha-0123456");

    let (duplicate_status, _duplicate_headers, duplicate_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{base}/packages/{package_id}/versions"),
        Some(&owner_cookie),
        Some(json!({ "version": "SHA-0123456" })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let serialized_error = duplicate_body.to_string();
    assert!(!serialized_error.contains(&owner_cookie));
    assert!(!serialized_error.contains("test-session-secret"));
    assert!(!serialized_error.to_lowercase().contains("stack"));
}
