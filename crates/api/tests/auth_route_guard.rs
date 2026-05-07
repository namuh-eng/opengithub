use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity::{self, AuthUser},
    routes::auth::{current_user, persist_google_login},
    AppState,
};
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

fn cookie_headers(config: &AppConfig, set_cookie: &str) -> HeaderMap {
    let cookie_value =
        session::cookie_value_from_set_cookie(set_cookie).expect("cookie value should be present");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("{}={cookie_value}", config.session_cookie_name))
            .expect("cookie header"),
    );
    headers
}

fn tampered_cookie_headers(config: &AppConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!(
            "{}=not-a-valid-session",
            config.session_cookie_name
        ))
        .expect("cookie header"),
    );
    headers
}

fn assert_not_authenticated(
    error: (
        StatusCode,
        axum::Json<opengithub_api::api_types::ErrorEnvelope>,
    ),
) {
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error.1 .0.status, StatusCode::UNAUTHORIZED.as_u16());
    assert_eq!(error.1 .0.error.code, "not_authenticated");
    assert!(!error.1 .0.error.message.contains("__Host-session"));
}

#[tokio::test]
async fn protected_route_rejects_missing_cookie_with_json_401() {
    let state = AppState {
        db: None,
        config: app_config(),
    };

    let error = current_user(State(state), HeaderMap::new())
        .await
        .expect_err("missing cookie must not authenticate");

    assert_not_authenticated(error);
}

#[tokio::test]
async fn global_issues_api_rejects_missing_cookie_before_database_lookup() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/issues")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body: serde_json::Value = serde_json::from_slice(&bytes).expect("body should be JSON");
    assert_eq!(body["status"], 401);
    assert_eq!(body["error"]["code"], "not_authenticated");
    assert!(!body.to_string().contains("database"));
}

#[tokio::test]
async fn protected_route_reports_db_unavailable_only_after_a_valid_cookie_is_present() {
    let config = app_config();
    let cookie = session::set_cookie_header(
        &config,
        "session-without-database",
        Utc::now() + Duration::minutes(5),
    )
    .expect("signed cookie should be created");
    let state = AppState {
        db: None,
        config: config.clone(),
    };

    let error = current_user(State(state), cookie_headers(&config, &cookie))
        .await
        .expect_err("valid cookie cannot be checked without database");

    assert_eq!(error.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.1 .0.error.code, "database_unavailable");
}

#[tokio::test]
async fn protected_route_rejects_tampered_cookie_with_json_401() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres route guard scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let state = AppState {
        db: Some(pool),
        config: config.clone(),
    };

    let error = current_user(State(state), tampered_cookie_headers(&config))
        .await
        .expect_err("tampered cookie must not authenticate");

    assert_not_authenticated(error);
}

#[tokio::test]
async fn protected_route_rejects_expired_or_revoked_sessions_with_json_401() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres route guard scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let state = AppState {
        db: Some(pool.clone()),
        config: config.clone(),
    };
    let unique = Uuid::new_v4();
    let user = identity::upsert_user_by_email(
        &pool,
        &format!("expired-{unique}@opengithub.local"),
        Some("Expired User"),
        None,
    )
    .await
    .expect("user should upsert");
    let expired_session_id = format!("expired-{unique}");
    identity::upsert_session(
        &pool,
        &expired_session_id,
        Some(user.id),
        serde_json::json!({ "provider": "google" }),
        Utc::now() - Duration::minutes(1),
    )
    .await
    .expect("expired session should persist");
    let expired_cookie = session::set_cookie_header(
        &config,
        &expired_session_id,
        Utc::now() + Duration::minutes(5),
    )
    .expect("signed expired-session cookie");

    let expired_error = current_user(
        State(state.clone()),
        cookie_headers(&config, &expired_cookie),
    )
    .await
    .expect_err("expired session must not authenticate");
    assert_not_authenticated(expired_error);

    let login = persist_google_login(
        &state,
        opengithub_api::auth::google::GoogleUserInfo {
            sub: format!("revoked-sub-{unique}"),
            email: format!("revoked-{unique}@opengithub.local"),
            name: Some("Revoked User".to_owned()),
            picture: None,
        },
        "/dashboard".to_owned(),
    )
    .await
    .expect("login should persist");
    let revoked_headers = cookie_headers(&config, &login.cookie);
    let revoked_session_id = session::session_id_from_headers(&config, &revoked_headers)
        .expect("revoked cookie should verify")
        .expect("revoked cookie should contain a session id");
    identity::revoke_session(&pool, &revoked_session_id)
        .await
        .expect("session should revoke");

    let revoked_error = current_user(State(state), revoked_headers)
        .await
        .expect_err("revoked session must not authenticate");
    assert_not_authenticated(revoked_error);
}

#[tokio::test]
async fn protected_route_returns_the_session_user_for_a_valid_cookie() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres route guard scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let state = AppState {
        db: Some(pool),
        config: config.clone(),
    };
    let unique = Uuid::new_v4();
    let login = persist_google_login(
        &state,
        opengithub_api::auth::google::GoogleUserInfo {
            sub: format!("valid-sub-{unique}"),
            email: format!("valid-{unique}@opengithub.local"),
            name: Some("Valid User".to_owned()),
            picture: Some("https://example.test/avatar.png".to_owned()),
        },
        "/dashboard".to_owned(),
    )
    .await
    .expect("login should persist");

    let user: AuthUser = current_user(State(state), cookie_headers(&config, &login.cookie))
        .await
        .expect("valid session should authenticate")
        .0;

    assert_eq!(user.id, login.user.id);
    assert_eq!(user.email, login.user.email);
    assert_eq!(user.display_name.as_deref(), Some("Valid User"));
}
