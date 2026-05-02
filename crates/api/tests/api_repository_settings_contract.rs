use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
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
    let set_cookie =
        session::set_cookie_header(config, &session_id, expires_at).expect("cookie should sign");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn request_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
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
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn repository_settings_are_admin_only_and_audited() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository settings scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let owner = create_user(&pool, "settings-owner").await;
    let reader = create_user(&pool, "settings-reader").await;
    let repo_name = format!("settings-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: Some("Before".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let path = format!("/api/repos/{}/{}/settings", owner.email, repo_name);

    let (reader_status, reader_body) =
        request_json(app.clone(), Method::GET, &path, Some(&reader_cookie), None).await;
    assert_eq!(reader_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_body["error"]["code"], "forbidden");

    let (status, body) =
        request_json(app.clone(), Method::GET, &path, Some(&owner_cookie), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["visibility"], "public");
    assert_eq!(body["features"]["issues"], true);

    let (invalid_status, invalid_body) = request_json(
        app.clone(),
        Method::PATCH,
        &path,
        Some(&owner_cookie),
        Some(json!({ "mergeMethods": { "mergeCommit": false, "squash": false, "rebase": false, "autoMerge": false } })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (patch_status, patch_body) = request_json(
        app,
        Method::PATCH,
        &path,
        Some(&owner_cookie),
        Some(json!({
            "description": "After",
            "features": { "issues": false, "projects": true, "wiki": true },
            "mergeMethods": { "mergeCommit": true, "squash": true, "rebase": true, "autoMerge": false },
            "allowForking": true,
            "webCommitSignoffRequired": true
        })),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["description"], "After");
    assert_eq!(patch_body["features"]["issues"], false);
    assert_eq!(patch_body["webCommitSignoffRequired"], true);
    assert_eq!(patch_body["auditEventCount"], 1);
}
