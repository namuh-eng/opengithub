use axum::{
    body::{to_bytes, Body},
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use opengithub_api::{
    auth::{self, session, OAuthStatePayload},
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
    },
    routes::auth::{callback_google, logout, OAuthCallbackRequest},
    AppState,
};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

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

fn auth_config() -> AuthConfig {
    AuthConfig {
        google_client_id: "google-client-id.apps.googleusercontent.com".to_owned(),
        google_client_secret: "google-client-secret".to_owned(),
        session_secret: "test-session-secret-with-enough-entropy".to_owned(),
    }
}

fn app_config(api_url: &str) -> AppConfig {
    let api_url = Url::parse(api_url).expect("api URL");
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("app URL"),
        session_cookie_secure: !matches!(api_url.host_str(), Some("localhost" | "127.0.0.1")),
        api_url,
        auth: Some(auth_config()),
        session_cookie_name: "__Host-session".to_owned(),
    }
}

fn signed_state(auth: &AuthConfig, payload: OAuthStatePayload) -> String {
    let payload_part = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).expect("payload JSON"));
    let mut mac = HmacSha256::new_from_slice(auth.session_secret.as_bytes()).expect("HMAC key");
    mac.update(payload_part.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{payload_part}.{signature}")
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
    headers: Vec<(&str, String)>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        builder = builder.header(name, value);
    }

    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, value)
}

#[test]
fn next_paths_reject_open_redirect_and_control_character_shapes() {
    assert_eq!(auth::sanitize_next_url(Some("/dashboard")), "/dashboard");
    assert_eq!(
        auth::sanitize_next_url(Some("https://evil.example")),
        "/dashboard"
    );
    assert_eq!(
        auth::sanitize_next_url(Some("//evil.example")),
        "/dashboard"
    );
    assert_eq!(auth::sanitize_next_url(Some("/\\evil")), "/dashboard");
    assert_eq!(
        auth::sanitize_next_url(Some("/dashboard\r\nLocation: https://evil.example")),
        "/dashboard"
    );
}

#[test]
fn state_expiry_and_signature_are_enforced_without_leaking_payloads() {
    let config = auth_config();
    let expired = signed_state(
        &config,
        OAuthStatePayload {
            nonce: "expired-nonce".to_owned(),
            next: "/dashboard".to_owned(),
            expires_at: Utc::now() - Duration::minutes(1),
        },
    );
    let error = auth::decode_state(&config, &expired).expect_err("expired state is rejected");
    assert_eq!(error.to_string(), "OAuth state has expired");

    let valid = auth::encode_state(&config, "/dashboard").expect("valid state");
    let tampered = format!("{valid}x");
    let error = auth::decode_state(&config, &tampered).expect_err("tampered state is rejected");
    assert_eq!(error.to_string(), "OAuth state signature is invalid");
}

#[test]
fn session_cookie_attributes_follow_local_and_production_security_policy() {
    let local = app_config("http://localhost:3016");
    let production = AppConfig {
        app_url: Url::parse("https://opengithub.namuh.co").expect("app URL"),
        api_url: Url::parse("https://opengithub.namuh.co").expect("api URL"),
        session_cookie_secure: true,
        ..app_config("https://opengithub.namuh.co")
    };
    let expires_at = Utc::now() + Duration::minutes(30);

    let local_cookie =
        session::set_cookie_header(&local, "local-session", expires_at).expect("local cookie");
    assert!(local_cookie.contains("__Host-session="));
    assert!(local_cookie.contains("HttpOnly"));
    assert!(local_cookie.contains("SameSite=Lax"));
    assert!(local_cookie.contains("Path=/"));
    assert!(local_cookie.contains("Secure"));

    let production_cookie =
        session::set_cookie_header(&production, "prod-session", expires_at).expect("prod cookie");
    assert!(production_cookie.contains("Secure"));
}

#[test]
fn invalid_session_cookie_gets_generic_error_without_leaking_cookie_value() {
    let config = app_config("http://localhost:3016");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_static("__Host-session=invalid-cookie"),
    );

    let error = session::session_id_from_headers(&config, &headers)
        .expect_err("invalid cookie must not authenticate");

    assert_eq!(error.to_string(), "session cookie is malformed");
    assert!(!error.to_string().contains("invalid-cookie"));
}

#[tokio::test]
async fn callback_errors_and_logout_do_not_leak_provider_or_session_details() {
    let config = app_config("http://localhost:3016");
    let response = callback_google(
        State(AppState {
            db: None,
            config: config.clone(),
        }),
        Query(OAuthCallbackRequest {
            code: None,
            state: None,
            error: Some("access_denied_from_provider".to_owned()),
        }),
    )
    .await;
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("callback redirect location");

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location, "http://localhost:3015/login?error=oauth_failed");
    assert!(!location.contains("access_denied"));

    let logout_response = logout(State(AppState { db: None, config }), HeaderMap::new())
        .await
        .expect("logout without cookie should still succeed");
    assert_eq!(logout_response.status(), StatusCode::OK);
    assert!(logout_response.headers().contains_key(header::SET_COOKIE));
}

#[tokio::test]
async fn protected_api_auth_failures_use_json_envelopes_without_stack_traces() {
    let config = app_config("http://localhost:3016");
    let app = opengithub_api::build_app_with_config(None, config);

    let (current_user_status, current_user_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/auth/current-user",
        vec![
            ("authorization", "Bearer not-a-real-token".to_owned()),
            ("cookie", "__Host-session=not-a-valid-session".to_owned()),
        ],
    )
    .await;
    assert_eq!(current_user_status, StatusCode::UNAUTHORIZED);
    assert_eq!(current_user_body["error"]["code"], "not_authenticated");
    assert_eq!(current_user_body["status"], 401);
    assert!(!current_user_body
        .to_string()
        .contains("not-a-valid-session"));
    assert!(!current_user_body.to_string().contains("stack"));

    let (repos_status, repos_body) = send_json(app, Method::GET, "/api/repos", vec![]).await;
    assert_eq!(repos_status, StatusCode::UNAUTHORIZED);
    assert_eq!(repos_body["error"]["code"], "not_authenticated");
    assert_eq!(repos_body["status"], 401);
}

#[tokio::test]
async fn private_repository_denial_uses_json_403_without_hiding_auth_state() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres auth security scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config("http://localhost:3016");
    let owner = create_user(&pool, "private-owner").await;
    let reader = create_user(&pool, "private-reader").await;
    let repo_name = format!("private-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");

    let cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app,
        Method::GET,
        &format!("/api/repos/{}/{}", owner.email, repo_name),
        vec![("cookie", cookie)],
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");
    assert_eq!(body["status"], 403);
    assert!(!body.to_string().contains("DATABASE_URL"));
    assert!(!body.to_string().contains("backtrace"));
}
