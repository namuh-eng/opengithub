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
    let suffix = Uuid::new_v4().simple();
    upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(&format!("{label}-{suffix}")),
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

async fn send_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

async fn create_repo(
    pool: &PgPool,
    owner: &User,
    visibility: RepositoryVisibility,
    ai_features_enabled: bool,
) -> String {
    let name = format!("ai-surface-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: name.clone(),
            description: Some("AI surface contract repository".to_owned()),
            visibility,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    sqlx::query("UPDATE repositories SET ai_features_enabled = $2 WHERE id = $1")
        .bind(repository.id)
        .bind(ai_features_enabled)
        .execute(pool)
        .await
        .expect("repository AI setting should update");
    name
}

#[tokio::test]
async fn repository_ai_summary_respects_repository_and_user_opt_in() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping ai-001 contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "ai-owner").await;
    let reader = create_user(&pool, "ai-reader").await;
    let owner_login = owner.username.as_deref().unwrap_or(&owner.email);
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let public_repo = create_repo(&pool, &owner, RepositoryVisibility::Public, false).await;
    let (status, body) = send_json(
        app.clone(),
        &format!("/api/ai/repos/{owner_login}/{public_repo}/summary"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], false);
    assert_eq!(
        body["reason"],
        "AI features are disabled for this repository."
    );
    assert_eq!(body["output"], Value::Null);

    let private_repo = create_repo(&pool, &owner, RepositoryVisibility::Private, false).await;
    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role) VALUES ((SELECT id FROM repositories WHERE name = $1), $2, 'read')",
    )
    .bind(&private_repo)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("read permission should insert");
    let (status, body) = send_json(
        app.clone(),
        &format!("/api/ai/repos/{owner_login}/{private_repo}/summary"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], false);
    assert_eq!(
        body["reason"],
        "AI features are disabled for private repository content."
    );

    let enabled_repo = create_repo(&pool, &owner, RepositoryVisibility::Public, true).await;
    sqlx::query(
        r#"
        INSERT INTO user_settings (user_id, ai_features_enabled)
        VALUES ($1, false)
        ON CONFLICT (user_id) DO UPDATE SET ai_features_enabled = EXCLUDED.ai_features_enabled
        "#,
    )
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("user setting should upsert");
    let (status, body) = send_json(
        app,
        &format!("/api/ai/repos/{owner_login}/{enabled_repo}/summary"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], false);
    assert_eq!(
        body["reason"],
        "AI features are disabled in your account settings."
    );
    assert_eq!(body["output"], Value::Null);
}
