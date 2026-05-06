use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity::{upsert_session, upsert_user_by_email, User},
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

async fn create_profile_user(pool: &PgPool, username: &str, private: bool) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{username}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{username} display")),
        None,
    )
    .await
    .expect("user should upsert");
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1, profile_visibility = $2
        WHERE id = $3
        "#,
    )
    .bind(username)
    .bind(if private { "private" } else { "public" })
    .bind(user.id)
    .execute(pool)
    .await
    .expect("profile columns should update");
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

async fn json_request(
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
    let request_body = if let Some(body) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn follow_and_unfollow_are_idempotent_and_update_viewer_state() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile social scenario; set TEST_DATABASE_URL");
        return;
    };

    let marker = format!("profileact{}", Uuid::new_v4().simple());
    let target = create_profile_user(&pool, &marker, false).await;
    let viewer = create_profile_user(&pool, &format!("{marker}-viewer"), false).await;
    let config = app_config();
    let viewer_cookie = cookie_header(&pool, &config, &viewer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, headers, followed) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{marker}/follow"),
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(followed["viewerState"]["isFollowing"], true);
    assert_eq!(followed["followerCount"], 1);

    let (status, _, followed_again) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{marker}/follow"),
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(followed_again["followerCount"], 1);

    let (status, _, unfollowed) = json_request(
        app,
        Method::DELETE,
        &format!("/api/users/{marker}/follow"),
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(unfollowed["viewerState"]["isFollowing"], false);
    assert_eq!(unfollowed["followerCount"], 0);

    let follows: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM user_follows WHERE followed_user_id = $1")
            .bind(target.id)
            .fetch_one(&pool)
            .await
            .expect("follow count should load");
    assert_eq!(follows, 0);
}

#[tokio::test]
async fn followers_and_following_lists_are_paginated_and_public() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile social list scenario; set TEST_DATABASE_URL");
        return;
    };

    let marker = format!("profilelist{}", Uuid::new_v4().simple());
    let target = create_profile_user(&pool, &marker, false).await;
    let follower = create_profile_user(&pool, &format!("{marker}-follower"), false).await;
    let followed = create_profile_user(&pool, &format!("{marker}-followed"), false).await;
    sqlx::query(
        "INSERT INTO user_follows (follower_user_id, followed_user_id) VALUES ($1, $2), ($3, $4)",
    )
    .bind(follower.id)
    .bind(target.id)
    .bind(target.id)
    .bind(followed.id)
    .execute(&pool)
    .await
    .expect("follow graph should insert");

    let config = app_config();
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, headers, followers) = json_request(
        app.clone(),
        Method::GET,
        &format!("/api/users/{marker}/followers"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(followers["mode"], "followers");
    assert_eq!(followers["total"], 1);
    assert_eq!(followers["items"][0]["login"], format!("{marker}-follower"));
    assert_eq!(followers["items"][0]["href"], format!("/{marker}-follower"));

    let (status, _, following) = json_request(
        app,
        Method::GET,
        &format!("/api/users/{marker}/following?page=1&pageSize=10"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(following["mode"], "following");
    assert_eq!(following["items"][0]["login"], format!("{marker}-followed"));
}

#[tokio::test]
async fn block_and_report_persist_records_and_guard_invalid_actions() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile social scenario; set TEST_DATABASE_URL");
        return;
    };

    let marker = format!("profileact{}", Uuid::new_v4().simple());
    let target = create_profile_user(&pool, &marker, false).await;
    let viewer = create_profile_user(&pool, &format!("{marker}-viewer"), false).await;
    let _private_target = create_profile_user(&pool, &format!("{marker}-private"), true).await;
    let config = app_config();
    let viewer_cookie = cookie_header(&pool, &config, &viewer).await;
    let target_cookie = cookie_header(&pool, &config, &target).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    sqlx::query("INSERT INTO user_follows (follower_user_id, followed_user_id) VALUES ($1, $2)")
        .bind(viewer.id)
        .bind(target.id)
        .execute(&pool)
        .await
        .expect("follow should insert");

    let (status, _, anonymous) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{marker}/follow"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous["error"]["code"], "not_authenticated");

    let (status, _, self_action) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{marker}/follow"),
        Some(&target_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(self_action["error"]["code"], "validation_failed");

    let (status, _, private_action) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{}-private/follow", marker),
        Some(&viewer_cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!private_action.to_string().contains("DATABASE_URL"));

    let (status, _, blocked) = json_request(
        app.clone(),
        Method::PUT,
        &format!("/api/users/{marker}/block"),
        Some(&viewer_cookie),
        Some(json!({ "reason": "Safety review" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(blocked["viewerState"]["isBlocking"], true);
    assert_eq!(blocked["viewerState"]["isFollowing"], false);
    assert_eq!(blocked["followerCount"], 0);

    let block_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM user_blocks WHERE blocker_user_id = $1 AND blocked_user_id = $2",
    )
    .bind(viewer.id)
    .bind(target.id)
    .fetch_one(&pool)
    .await
    .expect("block count should load");
    assert_eq!(block_count, 1);

    let (status, _, blank_report) = json_request(
        app.clone(),
        Method::POST,
        &format!("/api/users/{marker}/reports"),
        Some(&viewer_cookie),
        Some(json!({ "reason": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        blank_report["error"]["message"],
        "report reason is required"
    );

    let (status, _, report) = json_request(
        app,
        Method::POST,
        &format!("/api/users/{marker}/reports"),
        Some(&viewer_cookie),
        Some(json!({ "reason": "spam", "details": "Profile is sending spam." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(report["id"].as_str().is_some());
    assert_eq!(report["viewerState"]["canReport"], true);

    let report_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM user_reports WHERE reporter_user_id = $1 AND reported_user_id = $2",
    )
    .bind(viewer.id)
    .bind(target.id)
    .fetch_one(&pool)
    .await
    .expect("report count should load");
    assert_eq!(report_count, 1);
}
