use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity::{self, User},
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
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("migrator could not run cleanly for account security scenario ({error}); applying additive settings migrations directly");
        sqlx::raw_sql(include_str!(
            "../migrations/202605040007_personal_access_token_management.up.sql"
        ))
        .execute(&pool)
        .await
        .ok()?;
        sqlx::raw_sql(include_str!(
            "../migrations/202605040009_account_security_settings.up.sql"
        ))
        .execute(&pool)
        .await
        .ok()?;
        sqlx::raw_sql(include_str!(
            "../migrations/202605070006_account_session_management.up.sql"
        ))
        .execute(&pool)
        .await
        .ok()?;
    }
    Some(pool)
}

#[tokio::test]
async fn active_sessions_list_revoke_and_sign_out_everywhere() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping account sessions scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool, "session-owner").await;
    let (current_session_id, cookie) = cookie_header(&pool, &config, &user).await;
    let other_session_id = Uuid::new_v4().to_string();
    let third_session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    identity::upsert_session(
        &pool,
        &other_session_id,
        Some(user.id),
        json!({ "provider": "google" }),
        expires_at,
    )
    .await
    .expect("other session should persist");
    identity::upsert_session(
        &pool,
        &third_session_id,
        Some(user.id),
        json!({ "provider": "google" }),
        expires_at,
    )
    .await
    .expect("third session should persist");
    sqlx::query(
        r#"
        UPDATE sessions
        SET user_agent = CASE id
              WHEN $2 THEN 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/124.0 Safari/537.36'
              ELSE 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1'
            END,
            ip_inet = CASE id WHEN $2 THEN '127.0.0.1'::inet ELSE '10.1.2.3'::inet END
        WHERE user_id = $1
        "#,
    )
    .bind(user.id)
    .bind(&current_session_id)
    .execute(&pool)
    .await
    .expect("session metadata should update");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = send_json(
        app.clone(),
        Method::GET,
        "/api/settings/security/sessions",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["activeCount"], 3);
    assert_eq!(body["currentSessionId"], current_session_id);
    assert_eq!(body["sessions"][0]["isCurrent"], true);
    assert_eq!(body["sessions"][0]["device"], "Mac · Chrome");
    assert_eq!(body["sessions"][0]["location"], "Localhost");

    let (status, _, body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/security/sessions/{current_session_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "sign_in_method_forbidden");

    let (status, _, body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/security/sessions/{other_session_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["revokedId"], other_session_id);
    assert_eq!(body["sessions"]["activeCount"], 2);

    let (status, _, body) = send_json(
        app,
        Method::POST,
        "/api/settings/security/sessions/sign-out-everywhere",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["revokedCount"], 1);
    assert_eq!(body["sessions"]["activeCount"], 1);
    assert_eq!(body["sessions"]["sessions"][0]["id"], current_session_id);

    let remaining_active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sessions WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("active count should load");
    assert_eq!(remaining_active, 1);

    let audit_text: String =
        sqlx::query_scalar("SELECT COALESCE(string_agg(metadata::text, ' '), '') FROM security_audit_events WHERE actor_user_id = $1 AND event_type LIKE 'session.%'")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .expect("audit rows should load");
    assert!(audit_text.contains(&other_session_id));
    assert!(!audit_text.contains("test-session-secret"));
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
    let unique = Uuid::new_v4();
    let user = identity::upsert_user_by_email(
        pool,
        &format!("{label}-{unique}@opengithub.local"),
        Some("Security Manager"),
        None,
    )
    .await
    .expect("user should persist");
    identity::upsert_oauth_account(
        pool,
        user.id,
        "google",
        &format!("google-{unique}"),
        &user.email,
    )
    .await
    .expect("oauth account should persist");
    user
}

async fn cookie_header(pool: &PgPool, config: &AppConfig, user: &User) -> (String, String) {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    identity::upsert_session(
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
    (
        session_id,
        format!("{}={cookie_value}", config.session_cookie_name),
    )
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
async fn security_settings_reject_anonymous_requests_with_json_401() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, headers, body) =
        send_json(app, Method::GET, "/api/settings/security", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["status"], 401);
    assert_eq!(body["error"]["code"], "not_authenticated");
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn security_settings_enforce_sudo_and_last_identity_protection() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping account security scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool, "security-owner").await;
    let (session_id, cookie) = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, _, initial_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/settings/security",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(initial_body["signInMethods"].as_array().unwrap().len(), 1);
    assert_eq!(initial_body["signInMethods"][0]["canUnlink"], false);
    assert_eq!(initial_body["sudo"]["active"], false);
    assert_eq!(initial_body["twoFactor"]["available"], false);

    let only_id = initial_body["signInMethods"][0]["id"].as_str().unwrap();
    let (status, _, body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/security/sign-in-methods/{only_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "sudo_required");

    let (status, _, body) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/security/sudo",
        Some(&cookie),
        Some(json!({ "confirmation": "wrong@example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "sudo_confirmation_failed");

    let (status, _, sudo_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/security/sudo",
        Some(&cookie),
        Some(json!({ "confirmation": user.email })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sudo_body["sudo"]["active"], true);
    let elevated_until: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT elevated_until FROM sessions WHERE id = $1")
            .bind(&session_id)
            .fetch_one(&pool)
            .await
            .expect("session should query");
    assert!(elevated_until.is_some_and(|expires| expires > Utc::now()));

    let (status, _, body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/security/sign-in-methods/{only_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "last_identity");

    let second_account_id: Uuid = sqlx::query_scalar(
        "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, email) VALUES ($1, 'google', $2, $3) RETURNING id",
    )
    .bind(user.id)
    .bind(format!("second-{}", Uuid::new_v4()))
    .bind("second-google@example.com")
    .fetch_one(&pool)
    .await
    .expect("second account should insert");
    let (status, _, unlink_body) = send_json(
        app,
        Method::DELETE,
        &format!("/api/settings/security/sign-in-methods/{second_account_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(unlink_body["removedId"], second_account_id.to_string());
    assert_eq!(
        unlink_body["settings"]["signInMethods"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(!unlink_body
        .to_string()
        .contains("second-google@example.com"));
}
