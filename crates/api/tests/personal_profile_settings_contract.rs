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

async fn create_user(pool: &PgPool) -> User {
    let unique = Uuid::new_v4();
    let user = upsert_user_by_email(
        pool,
        &format!("settings-{unique}@opengithub.local"),
        Some("Settings User"),
        None,
    )
    .await
    .expect("user should persist");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("settings-{unique}"))
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
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, headers, value)
}

#[tokio::test]
async fn profile_settings_reads_and_saves_identity_privacy_social_and_audit_events() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping personal profile settings scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool).await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, headers, body) = send_json(
        app.clone(),
        Method::GET,
        "/api/user/settings/profile",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["displayName"], "Settings User");
    assert_eq!(body["emails"].as_array().expect("emails").len(), 1);
    let public_email_id = body["emails"][0]["id"].as_str().expect("email id");
    assert_eq!(
        body["socialAccounts"]
            .as_array()
            .expect("social accounts")
            .len(),
        4
    );

    let update = json!({
        "displayName": "Updated Settings User",
        "publicEmailId": public_email_id,
        "bio": "",
        "pronouns": "they/them",
        "websiteUrl": "https://example.com",
        "company": "NamuH",
        "location": "Seoul",
        "displayLocalTime": true,
        "timeZone": "Asia/Seoul",
        "preferredLanguage": "ko",
        "privateProfile": true,
        "showPrivateContributionCount": true,
        "achievementsEnabled": false,
        "socialAccounts": [
            { "provider": "x", "handleOrUrl": "@settings", "position": 1 },
            { "provider": "mastodon", "handleOrUrl": "https://social.example/@settings", "position": 2 },
            { "provider": "linkedin", "handleOrUrl": "", "position": 3 },
            { "provider": "bluesky", "handleOrUrl": "", "position": 4 }
        ]
    });
    let (status, _headers, body) = send_json(
        app.clone(),
        Method::PATCH,
        "/api/user/settings/profile",
        Some(&cookie),
        Some(update),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["displayName"], "Updated Settings User");
    assert_eq!(body["bio"], "");
    assert_eq!(body["privateProfile"], true);
    assert_eq!(body["showPrivateContributionCount"], true);
    assert_eq!(body["achievementsEnabled"], false);
    assert_eq!(body["socialAccounts"][0]["handleOrUrl"], "@settings");

    let visibility: String =
        sqlx::query_scalar("SELECT profile_visibility FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .expect("visibility should read");
    assert_eq!(visibility, "private");
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM security_audit_events WHERE actor_user_id = $1 AND event_type = 'profile.settings.update'",
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("audit should count");
    assert!(audit_count >= 1);
}

#[tokio::test]
async fn appearance_settings_default_validate_and_persist_theme_preferences() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping appearance settings scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool).await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, _headers, body) = send_json(
        app.clone(),
        Method::GET,
        "/api/user/settings/appearance",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["theme"], "system");
    assert_eq!(body["fontSize"], "medium");

    let (status, _headers, body) = send_json(
        app.clone(),
        Method::PATCH,
        "/api/user/settings/appearance",
        Some(&cookie),
        Some(json!({ "theme": "dark-high-contrast", "fontSize": "large" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["theme"], "dark_high_contrast");
    assert_eq!(body["fontSize"], "large");

    let persisted: (String, String) =
        sqlx::query_as("SELECT theme, font_size FROM user_settings WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .expect("appearance settings should persist");
    assert_eq!(
        persisted,
        ("dark_high_contrast".to_owned(), "large".to_owned())
    );

    let (status, _headers, body) = send_json(
        app,
        Method::PATCH,
        "/api/user/settings/appearance",
        Some(&cookie),
        Some(json!({ "theme": "neon", "fontSize": "large" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn avatar_upload_validates_type_size_and_supports_remove() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping personal avatar scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool).await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let invalid = json!({ "action": "upload", "fileName": "bad.txt", "contentType": "text/plain", "byteSize": 12 });
    let (status, _headers, body) = send_json(
        app.clone(),
        Method::PATCH,
        "/api/user/settings/profile/avatar",
        Some(&cookie),
        Some(invalid),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");

    let valid = json!({
        "action": "upload",
        "fileName": "avatar.png",
        "contentType": "image/png",
        "byteSize": 128,
        "previewUrl": "data:image/png;base64,aGVsbG8="
    });
    let (status, _headers, body) = send_json(
        app.clone(),
        Method::PATCH,
        "/api/user/settings/profile/avatar",
        Some(&cookie),
        Some(valid),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["avatar"]["contentType"], "image/png");
    assert!(body["avatar"]["url"]
        .as_str()
        .expect("avatar url")
        .starts_with("data:image/png"));

    let (status, _headers, body) = send_json(
        app,
        Method::PATCH,
        "/api/user/settings/profile/avatar",
        Some(&cookie),
        Some(json!({ "action": "remove" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["avatar"].is_null());
}
