use std::{path::Path, sync::LazyLock};

use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::config::{AppConfig, AuthConfig};
use opengithub_api::domain::{
    identity::{upsert_user_by_email, User},
    repositories::{
        create_repository_with_bootstrap, CreateRepository, RepositoryBootstrapRequest,
        RepositoryOwner, RepositoryVisibility,
    },
    tokens::hash_personal_access_token,
};
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
static GIT_STORAGE_ENV_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

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

    reset_rate_limit_subject(&pool, "ip", "unknown").await;
    Some(pool)
}

async fn reset_rate_limit_subject(pool: &PgPool, subject_type: &str, subject_key: &str) {
    sqlx::query(
        r#"
        DELETE FROM rate_limit_buckets
        WHERE subject_type = $1
          AND subject_key = $2
        "#,
    )
    .bind(subject_type)
    .bind(subject_key)
    .execute(pool)
    .await
    .expect("test rate limit bucket should reset");
}

fn test_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("valid app URL"),
        api_url: Url::parse("http://localhost:3016").expect("valid API URL"),
        auth: Some(AuthConfig {
            google_client_id: "test-google-client".to_owned(),
            google_client_secret: "test-google-secret".to_owned(),
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

async fn create_repository(
    pool: &PgPool,
    owner: &User,
    visibility: RepositoryVisibility,
) -> opengithub_api::domain::repositories::Repository {
    create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("git-security-{}", Uuid::new_v4().simple()),
            description: Some("Git security test repository".to_owned()),
            visibility,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("blank".to_owned()),
            gitignore_template_slug: None,
            license_template_slug: None,
        },
    )
    .await
    .expect("repository should create")
}

async fn create_pat(pool: &PgPool, user_id: Uuid, scopes: &[&str]) -> String {
    let token = format!("oghp_{}_secret", Uuid::new_v4().simple());
    let prefix = token
        .split("_secret")
        .next()
        .expect("token prefix marker")
        .to_owned();
    sqlx::query(
        r#"
        INSERT INTO personal_access_tokens (
            user_id, name, prefix, token_hash, scopes, expires_at, resource_owner_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $1)
        "#,
    )
    .bind(user_id)
    .bind("Git security test token")
    .bind(prefix)
    .bind(hash_personal_access_token(&token))
    .bind(
        scopes
            .iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<_>>(),
    )
    .bind(Some(Utc::now() + Duration::days(1)))
    .execute(pool)
    .await
    .expect("PAT should insert");
    token
}

fn basic_auth_header(token: &str) -> HeaderValue {
    use base64::Engine as _;

    HeaderValue::from_str(&format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"))
    ))
    .expect("basic auth header should build")
}

async fn send(
    app: axum::Router,
    method: Method,
    uri: &str,
    headers: HeaderMap,
    body: Body,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        if let Some(name) = name {
            builder = builder.header(name, value);
        }
    }
    let response = app
        .oneshot(builder.body(body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read")
        .to_vec();
    (status, headers, bytes)
}

#[tokio::test]
async fn git_routes_reject_invalid_services_and_oversized_requests_safely() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping git security scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-security-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-security-owner").await;
    let repository = create_repository(&pool, &owner, RepositoryVisibility::Public).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), test_config());

    let (status, headers, body) = send(
        app.clone(),
        Method::GET,
        &format!(
            "/{}/{}.git/info/refs?service=git-upload-archive",
            repository.owner_login, repository.name
        ),
        HeaderMap::new(),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        headers.get(header::WWW_AUTHENTICATE),
        Some(&HeaderValue::from_static(r#"Basic realm="opengithub Git""#))
    );
    let rendered = String::from_utf8_lossy(&body);
    assert!(rendered.contains("unsupported_git_service"));
    assert!(!rendered.contains("git-upload-archive"));

    let oversized = vec![0_u8; 33 * 1024 * 1024];
    let (status, _, body) = send(
        app,
        Method::POST,
        &format!(
            "/{}/{}.git/git-upload-pack",
            repository.owner_login, repository.name
        ),
        HeaderMap::new(),
        Body::from(oversized),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    let rendered = String::from_utf8_lossy(&body);
    assert!(!rendered.contains(&repository.name));
    assert!(!rendered.contains("README.md"));

    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn private_git_surfaces_do_not_leak_content_tokens_or_hashes() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private git leakage scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-leakage-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-leak-owner").await;
    let repository = create_repository(&pool, &owner, RepositoryVisibility::Private).await;
    let read_token = create_pat(&pool, owner.id, &["repo:read"]).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), test_config());

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth_header(&read_token));
    let (status, _, body) = send(
        app.clone(),
        Method::GET,
        &format!(
            "/{}/{}.git/info/refs?service=git-receive-pack",
            repository.owner_login, repository.name
        ),
        headers.clone(),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let rendered = String::from_utf8_lossy(&body);
    assert!(rendered.contains("authentication_required"));
    assert!(!rendered.contains(&read_token));
    assert!(!rendered.contains("sha256:"));
    assert!(!rendered.contains("README.md"));

    let (status, _, body) = send(
        app.clone(),
        Method::GET,
        &format!(
            "/{}/{}/raw/main/%2e%2e/README.md",
            repository.owner_login, repository.name
        ),
        headers,
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let rendered = String::from_utf8_lossy(&body);
    assert!(!rendered.contains("README.md"));
    assert!(!rendered.contains(&read_token));
    assert!(!rendered.contains("sha256:"));

    let metadata_rows: Vec<serde_json::Value> =
        sqlx::query_scalar("SELECT metadata FROM api_request_logs WHERE path LIKE $1")
            .bind(format!("%/{}%", repository.name))
            .fetch_all(&pool)
            .await
            .expect("request logs should read");
    assert!(!metadata_rows.is_empty());
    let rendered_logs = serde_json::to_string(&metadata_rows).expect("logs serialize");
    assert!(!rendered_logs.contains(&read_token));
    assert!(!rendered_logs.to_ascii_lowercase().contains("authorization"));
    assert!(!rendered_logs.to_ascii_lowercase().contains("cookie"));

    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn public_git_archive_paths_are_sanitized() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping git archive sanitization scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-archive-safe-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-archive-safe-owner").await;
    let repository = create_repository(&pool, &owner, RepositoryVisibility::Public).await;
    let app = opengithub_api::build_app_with_config(Some(pool), test_config());

    let (status, headers, body) = send(
        app,
        Method::GET,
        &format!(
            "/{}/{}/archive/refs/heads/main.zip",
            repository.owner_login, repository.name
        ),
        HeaderMap::new(),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with(b"PK"));
    let disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .expect("content disposition should exist");
    assert!(disposition.contains(&format!("{}-main.zip", repository.name)));
    assert!(!disposition.contains('/'));
    assert!(!disposition.contains('\\'));
    let archive_paths: Vec<String> = walk_storage_paths(&storage_dir);
    assert!(archive_paths
        .iter()
        .any(|path| path.contains("archives") && path.ends_with(".zip")));
    assert!(archive_paths.iter().all(|path| !path.contains("..")));

    let _ = std::fs::remove_dir_all(storage_dir);
}

fn walk_storage_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                paths.extend(walk_storage_paths(&path));
            } else {
                paths.push(path.to_string_lossy().into_owned());
            }
        }
    }
    paths
}
