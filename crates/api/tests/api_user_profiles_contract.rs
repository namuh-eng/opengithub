use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity,
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

async fn cookie_header(pool: &PgPool, config: &AppConfig, user_id: Uuid) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    identity::upsert_session(
        pool,
        &session_id,
        Some(user_id),
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
    db: Option<PgPool>,
    config: AppConfig,
    method: Method,
    uri: &str,
    cookie: Option<String>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let app = opengithub_api::build_app_with_config(db, config);
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let response = app
        .oneshot(
            builder
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = serde_json::from_slice(&bytes).expect("response body should be JSON");
    (status, headers, body)
}

async fn seed_profile(pool: &PgPool, login: &str) -> Uuid {
    let user = identity::upsert_user_by_email(
        pool,
        &format!("{login}.{}@opengithub.local", Uuid::new_v4()),
        Some("Profile Builder"),
        None,
    )
    .await
    .expect("user should persist");
    sqlx::query(
        "UPDATE users SET username = $1, bio = 'Building profiles', company = 'Namuh', location = 'Seoul', website_url = 'https://namuh.co' WHERE id = $2",
    )
    .bind(login)
    .bind(user.id)
    .execute(pool)
    .await
    .expect("profile fields should persist");
    let repo_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO repositories (owner_user_id, name, description, visibility, created_by_user_id) VALUES ($1, 'opengithub', 'A calmer forge', 'public', $1) RETURNING id",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await
    .expect("repository should persist");
    sqlx::query("INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', 'var(--accent)', 1000) ON CONFLICT DO NOTHING")
        .bind(repo_id)
        .execute(pool)
        .await
        .expect("language should persist");
    sqlx::query("INSERT INTO profile_contribution_days (user_id, contribution_date, contribution_count) VALUES ($1, '2026-01-02', 3) ON CONFLICT DO NOTHING")
        .bind(user.id)
        .execute(pool)
        .await
        .expect("contribution should persist");
    user.id
}

#[tokio::test]
async fn user_profile_returns_public_overview_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let login = format!("profile-{}", Uuid::new_v4().simple());
    seed_profile(&pool, &login).await;

    let (status, headers, body) = send(
        Some(pool),
        app_config(),
        Method::GET,
        &format!("/api/users/{login}/profile?year=2026"),
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["identity"]["login"], login);
    assert_eq!(body["identity"]["bio"], "Building profiles");
    assert_eq!(body["tabs"]["repositories"], 1);
    assert_eq!(body["pinnedItems"][0]["title"], "opengithub");
    assert_eq!(body["contributions"]["total"], 3);
    assert!(body["contributions"]["days"].as_array().is_some_and(|days| days.len() == 365));
    assert!(body.get("email").is_none());
}

#[tokio::test]
async fn user_profile_follow_block_and_report_are_login_gated_writes() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile write scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let target_login = format!("target-{}", Uuid::new_v4().simple());
    seed_profile(&pool, &target_login).await;
    let actor = identity::upsert_user_by_email(
        &pool,
        &format!("viewer.{}@opengithub.local", Uuid::new_v4()),
        Some("Viewer"),
        None,
    )
    .await
    .expect("viewer should persist");
    let cookie = cookie_header(&pool, &config, actor.id).await;

    let (anonymous_status, _, anonymous_body) = send(
        Some(pool.clone()),
        config.clone(),
        Method::PUT,
        &format!("/api/users/{target_login}/follow"),
        None,
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (follow_status, _, follow_body) = send(
        Some(pool.clone()),
        config.clone(),
        Method::PUT,
        &format!("/api/users/{target_login}/follow"),
        Some(cookie.clone()),
        None,
    )
    .await;
    assert_eq!(follow_status, StatusCode::OK);
    assert_eq!(follow_body["following"], true);
    assert_eq!(follow_body["followerCount"], 1);

    let (block_status, _, block_body) = send(
        Some(pool.clone()),
        config.clone(),
        Method::PUT,
        &format!("/api/users/{target_login}/block"),
        Some(cookie.clone()),
        None,
    )
    .await;
    assert_eq!(block_status, StatusCode::OK);
    assert_eq!(block_body["blocked"], true);

    let (report_status, _, report_body) = send(
        Some(pool),
        config,
        Method::POST,
        &format!("/api/users/{target_login}/report"),
        Some(cookie),
        Some(json!({ "reason": "spam", "details": "No secrets here" })),
    )
    .await;
    assert_eq!(report_status, StatusCode::CREATED);
    assert_eq!(report_body["status"], "received");
    assert!(report_body["id"].as_str().is_some());
}

#[tokio::test]
async fn private_profiles_hide_activity_for_other_viewers() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private profile scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let login = format!("private-{}", Uuid::new_v4().simple());
    let user_id = seed_profile(&pool, &login).await;
    sqlx::query("UPDATE users SET private_profile = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("privacy should persist");

    let (status, _, body) = send(
        Some(pool),
        app_config(),
        Method::GET,
        &format!("/api/users/{login}/profile?year=2026"),
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["identity"]["privateProfile"], true);
    assert_eq!(body["identity"]["followerCount"], 0);
    assert_eq!(body["pinnedItems"].as_array().unwrap().len(), 0);
    assert_eq!(body["achievements"].as_array().unwrap().len(), 0);
    assert_eq!(body["contributions"]["total"], 0);
}
