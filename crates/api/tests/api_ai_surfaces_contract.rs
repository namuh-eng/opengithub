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
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
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

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn seed_commit(
    pool: &PgPool,
    repository_id: Uuid,
    author_id: Uuid,
    message: &str,
    days_ago: i64,
) -> (Uuid, String) {
    let oid = format!("{:040x}", Uuid::new_v4().as_u128());
    let row = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, committed_at)
        VALUES ($1, $2, $3, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(&oid)
    .bind(author_id)
    .bind(message)
    .bind(Utc::now() - Duration::days(days_ago))
    .fetch_one(pool)
    .await
    .expect("commit should persist");
    (row.get("id"), oid)
}

async fn seed_release(
    pool: &PgPool,
    repository_id: Uuid,
    author_id: Uuid,
    tag: &str,
    commit_id: Uuid,
) -> Uuid {
    let row = sqlx::query(
        r#"
        INSERT INTO releases (repository_id, tag_name, name, body, author_user_id, target_commit_id)
        VALUES ($1, $2, $2, '', $3, $4)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(tag)
    .bind(author_id)
    .bind(commit_id)
    .fetch_one(pool)
    .await
    .expect("release should persist");
    sqlx::query(
        "INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id) VALUES ($1, $2, 'tag', $3)",
    )
    .bind(repository_id)
    .bind(format!("refs/tags/{tag}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("tag ref should persist");
    row.get("id")
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

#[tokio::test]
async fn ai_changelog_uses_previous_and_target_tag_range_for_cache_key() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping ai-001 changelog range scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "ai-changelog-owner").await;
    let owner_login = owner.username.as_deref().unwrap_or(&owner.email);
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let repo_name = create_repo(&pool, &owner, RepositoryVisibility::Public, true).await;
    let repository_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM repositories WHERE owner_user_id = $1 AND name = $2",
    )
    .bind(owner.id)
    .bind(&repo_name)
    .fetch_one(&pool)
    .await
    .expect("repository should exist");

    let (previous_commit_id, _) =
        seed_commit(&pool, repository_id, owner.id, "Legacy release baseline", 4).await;
    let (_, included_oid) = seed_commit(
        &pool,
        repository_id,
        owner.id,
        "Add ranged changelog support",
        2,
    )
    .await;
    let (target_commit_id, target_oid) =
        seed_commit(&pool, repository_id, owner.id, "Fix changelog editor", 1).await;
    let _ = seed_commit(&pool, repository_id, owner.id, "Future unreleased work", 0).await;

    seed_release(&pool, repository_id, owner.id, "v1.0.0", previous_commit_id).await;
    let release_id = seed_release(&pool, repository_id, owner.id, "v1.1.0", target_commit_id).await;

    let context =
        format!("{target_oid} Fix changelog editor\n{included_oid} Add ranged changelog support");
    let content_hash = hash_content(&format!("{}:{}:{context}", "v1.0.0", "v1.1.0"));
    sqlx::query(
        r#"
        INSERT INTO ai_outputs (kind, scope_type, scope_id, content_hash, prompt_version, model, output, created_by_user_id)
        VALUES ('changelog', 'release', $1, $2, 'ai-001-v1', 'gpt-4o', $3, $4)
        "#,
    )
    .bind(release_id)
    .bind(content_hash)
    .bind("### Added\n- Generated from only the selected tag range.")
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("cached changelog should persist");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, body) = post_json(
        app,
        &format!("/api/ai/repos/{owner_login}/{repo_name}/releases/changelog"),
        Some(&owner_cookie),
        json!({ "previousTag": "v1.0.0", "targetTag": "v1.1.0" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], true);
    assert_eq!(body["output"]["cached"], true);
    assert_eq!(
        body["output"]["output"],
        "### Added\n- Generated from only the selected tag range."
    );
}
