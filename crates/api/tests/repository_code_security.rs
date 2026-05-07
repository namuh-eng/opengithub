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
        repositories::{
            create_repository_with_bootstrap, CreateRepository, RepositoryBootstrapRequest,
            RepositoryOwner, RepositoryVisibility,
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
    let suffix = Uuid::new_v4().simple();
    let user = upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("{label}-{suffix}"))
        .bind(user.id)
        .execute(pool)
        .await
        .expect("username should update");
    user
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

async fn send(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Value, String) {
    send_with_body(app, method, uri, cookie, Body::empty()).await
}

async fn send_with_body(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Body,
) -> (StatusCode, Value, String) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body_text = String::from_utf8(bytes.to_vec()).expect("body should be utf8");
    let body_json = serde_json::from_str(&body_text).expect("response should be json");
    (status, body_json, body_text)
}

fn assert_error_hygiene(body: &Value, text: &str) {
    assert!(body["error"]["code"].is_string());
    assert!(body["error"]["message"].is_string());
    assert!(!text.contains("panicked at"));
    assert!(!text.contains("stack backtrace"));
    assert!(!text.contains("SESSION_SECRET"));
    assert!(!text.contains("DATABASE_URL"));
}

#[tokio::test]
async fn repository_code_routes_reject_auth_bypass_and_private_access() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository code security scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-sec-owner").await;
    let outsider = create_user(&pool, "repo-sec-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let private_repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-sec-{}", Uuid::new_v4().simple()),
            description: Some("Private repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let overview_uri = format!(
        "/api/repos/{}/{}",
        private_repository.owner_login, private_repository.name
    );
    let star_uri = format!("{overview_uri}/star");

    for (method, uri, cookie, expected_status, expected_code) in [
        (
            Method::GET,
            overview_uri.as_str(),
            None,
            StatusCode::FORBIDDEN,
            "forbidden",
        ),
        (
            Method::GET,
            overview_uri.as_str(),
            Some(outsider_cookie.as_str()),
            StatusCode::FORBIDDEN,
            "forbidden",
        ),
        (
            Method::PUT,
            star_uri.as_str(),
            Some(outsider_cookie.as_str()),
            StatusCode::FORBIDDEN,
            "forbidden",
        ),
    ] {
        let (status, body, text) = send(app.clone(), method, uri, cookie).await;
        assert_eq!(status, expected_status);
        assert_eq!(body["error"]["code"], expected_code);
        assert_error_hygiene(&body, &text);
    }

    let (status, body, _) = send(app, Method::GET, &overview_uri, Some(&owner_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], private_repository.name);
}

#[tokio::test]
async fn repository_code_routes_reject_bad_refs_and_path_traversal() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository code path security scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-path-sec-owner").await;
    let cookie = cookie_header(&pool, &config, &owner).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("path-sec-{}", Uuid::new_v4().simple()),
            description: Some("Path security repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);

    for (uri, expected_code) in [
        (format!("{base}/contents/src/..?ref=main"), "not_found"),
        (format!("{base}/contents/%2e%2e?ref=main"), "not_found"),
        (
            format!("{base}/contents/src%5Cmain.rs?ref=main"),
            "not_found",
        ),
        (
            format!("{base}/blobs/src/main.rs?ref=missing"),
            "ref_not_found",
        ),
        (format!("{base}/commits?ref=main&path=src/.."), "not_found"),
    ] {
        let (status, body, text) = send(app.clone(), Method::GET, &uri, Some(&cookie)).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "uri should be rejected: {uri}"
        );
        assert_eq!(body["error"]["code"], expected_code);
        assert_error_hygiene(&body, &text);
    }

    let invalid_watch_body =
        Body::from(r#"{"level":"<script>alert(1)</script>","customEvents":["issues"]}"#);
    let (watch_status, watch_body, watch_text) = send_with_body(
        app,
        Method::PATCH,
        &format!("{base}/watch"),
        Some(&cookie),
        invalid_watch_body,
    )
    .await;
    assert_eq!(watch_status, StatusCode::BAD_REQUEST);
    assert_eq!(watch_body["error"]["code"], "invalid_json");
    assert!(!watch_text.contains("<script>"));
    assert_error_hygiene(&watch_body, &watch_text);
}
