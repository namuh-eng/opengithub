use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use opengithub_api::{
    auth::{encode_state, google::GoogleUserInfo, session},
    config::{AppConfig, AuthConfig},
    domain::identity::get_oauth_account,
    routes::auth::{callback_google, logout, me, persist_google_login, OAuthCallbackRequest},
    AppState,
};
use sqlx::PgPool;
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

fn auth_config() -> AuthConfig {
    AuthConfig {
        google_client_id: "google-client-id.apps.googleusercontent.com".to_owned(),
        google_client_secret: "google-client-secret".to_owned(),
        session_secret: "test-session-secret-with-enough-entropy".to_owned(),
    }
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("app URL"),
        api_url: Url::parse("http://localhost:3016").expect("api URL"),
        auth: Some(auth_config()),
        session_cookie_name: "__Host-session".to_owned(),
        session_cookie_secure: false,
    }
}

fn google_user(unique: Uuid) -> GoogleUserInfo {
    GoogleUserInfo {
        sub: format!("google-sub-{unique}"),
        email: format!("google-user-{unique}@opengithub.local"),
        name: Some("Google Test User".to_owned()),
        picture: Some("https://example.test/avatar.png".to_owned()),
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

#[tokio::test]
async fn callback_persistence_sets_a_signed_cookie_and_me_reads_the_session() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres auth callback scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let state = AppState {
        db: Some(pool.clone()),
        config: config.clone(),
    };
    let google_user = google_user(Uuid::new_v4());

    let completed = persist_google_login(
        &state,
        google_user.clone(),
        "/dashboard?tab=repos".to_owned(),
    )
    .await
    .expect("Google login should persist");
    assert_eq!(completed.next, "/dashboard?tab=repos");
    assert_eq!(completed.user.email, google_user.email);
    assert!(completed.cookie.contains("HttpOnly"));
    assert!(completed.cookie.contains("SameSite=Lax"));
    assert!(completed.cookie.contains("Secure"));

    let account = get_oauth_account(&pool, "google", &google_user.sub)
        .await
        .expect("oauth account lookup should succeed")
        .expect("oauth account should exist");
    assert_eq!(account.user_id, completed.user.id);

    let auth_me = me(
        State(state.clone()),
        cookie_headers(&config, &completed.cookie),
    )
    .await
    .expect("/api/auth/me should succeed")
    .0;
    assert!(auth_me.authenticated);
    assert_eq!(auth_me.user.expect("auth user").email, google_user.email);
}

#[tokio::test]
async fn logout_revokes_the_current_cookie_and_is_idempotent() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres logout scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let state = AppState {
        db: Some(pool),
        config: config.clone(),
    };
    let completed =
        persist_google_login(&state, google_user(Uuid::new_v4()), "/dashboard".to_owned())
            .await
            .expect("Google login should persist");
    let headers = cookie_headers(&config, &completed.cookie);

    let response = logout(State(state.clone()), headers.clone())
        .await
        .expect("logout should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let expired_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("logout should clear cookie");
    assert!(expired_cookie.contains("Max-Age=0"));

    let anonymous = me(State(state.clone()), headers.clone())
        .await
        .expect("/api/auth/me should succeed after logout")
        .0;
    assert!(!anonymous.authenticated);
    assert!(anonymous.user.is_none());

    logout(State(state), headers)
        .await
        .expect("logout should be idempotent");
}

#[tokio::test]
async fn callback_errors_redirect_to_the_login_card_without_leaking_details() {
    let config = app_config();
    let state = AppState { db: None, config };
    let response = callback_google(
        State(state),
        Query(OAuthCallbackRequest {
            code: None,
            state: None,
            error: Some("access_denied".to_owned()),
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("location header");
    assert_eq!(location, "http://localhost:3015/login?error=oauth_failed");
    assert!(!location.contains("access_denied"));
}

#[tokio::test]
async fn callback_missing_code_redirects_without_calling_google() {
    let config = app_config();
    let auth = config.auth.as_ref().expect("auth config");
    let state_payload = encode_state(auth, "/dashboard").expect("state should encode");
    let response = callback_google(
        State(AppState { db: None, config }),
        Query(OAuthCallbackRequest {
            code: None,
            state: Some(state_payload),
            error: None,
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("location header");
    assert_eq!(location, "http://localhost:3015/login?error=oauth_failed");
}
