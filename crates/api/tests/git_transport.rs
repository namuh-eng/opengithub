use std::{path::Path, sync::LazyLock};

use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::auth::session;
use opengithub_api::config::{AppConfig, AuthConfig};
use opengithub_api::domain::{
    identity::{upsert_user_by_email, User},
    repositories::{
        create_repository_with_bootstrap, get_repository_by_owner_name,
        repository_overview_for_actor, CreateRepository, RepositoryBootstrapRequest,
        RepositoryOwner, RepositoryVisibility,
    },
    tokens::hash_personal_access_token,
};
use sqlx::PgPool;
use tokio::{net::TcpListener, process::Command};
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
    if MIGRATOR.run(&pool).await.is_err() {
        let has_git_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_git_storage') IS NOT NULL
               AND to_regclass('public.repository_files') IS NOT NULL
               AND to_regclass('public.secret_scanning_alerts') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_git_tables {
            return None;
        }
    }
    Some(pool)
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

async fn session_cookie(pool: &PgPool, config: &AppConfig, user: &User) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(1);
    opengithub_api::domain::identity::upsert_session(
        pool,
        &session_id,
        Some(user.id),
        serde_json::json!({ "provider": "google" }),
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

async fn create_pat(
    pool: &PgPool,
    user_id: Uuid,
    scopes: &[&str],
    expires_at: Option<chrono::DateTime<Utc>>,
) -> String {
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
    .bind("Git transport test token")
    .bind(prefix)
    .bind(hash_personal_access_token(&token))
    .bind(
        scopes
            .iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<_>>(),
    )
    .bind(expires_at)
    .execute(pool)
    .await
    .expect("PAT should insert");
    token
}

async fn send_raw(app: axum::Router, uri: &str) -> (StatusCode, Vec<u8>, String) {
    send_raw_with_headers(app, uri, HeaderMap::new()).await
}

async fn send_raw_with_headers(
    app: axum::Router,
    uri: &str,
    headers: HeaderMap,
) -> (StatusCode, Vec<u8>, String) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    for (name, value) in headers {
        if let Some(name) = name {
            builder = builder.header(name, value);
        }
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read")
        .to_vec();
    (status, bytes, content_type)
}

fn basic_auth_header(token: &str) -> HeaderValue {
    use base64::Engine as _;

    HeaderValue::from_str(&format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"))
    ))
    .expect("basic auth header should build")
}

#[tokio::test]
async fn public_repository_supports_smart_http_clone() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping git transport scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir = std::env::temp_dir().join(format!("opengithub-git-test-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("cloneable-{}", Uuid::new_v4().simple()),
            description: Some("Cloneable public repository".to_owned()),
            visibility: RepositoryVisibility::Public,
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
    .expect("repository should create");

    let storage_path = sqlx::query_scalar::<_, String>(
        "SELECT storage_path FROM repository_git_storage WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("storage row should exist");
    assert!(Path::new(&storage_path).join("HEAD").exists());

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), test_config());
    let (status, body, content_type) = send_raw(
        app.clone(),
        &format!(
            "/{}/{}.git/info/refs?service=git-upload-pack",
            repository.owner_login, repository.name
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/x-git-upload-pack-advertisement");
    let advertisement = String::from_utf8_lossy(&body);
    assert!(advertisement.contains("# service=git-upload-pack"));
    assert!(advertisement.contains("refs/heads/main"));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("local addr should read");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });
    let checkout_dir = std::env::temp_dir().join(format!("opengithub-clone-{}", Uuid::new_v4()));
    let remote = format!(
        "http://{}/{}/{}.git",
        address, repository.owner_login, repository.name
    );
    let output = Command::new("git")
        .args(["clone", "--depth", "1", "--", &remote])
        .arg(&checkout_dir)
        .output()
        .await
        .expect("git clone should run");
    server.abort();
    assert!(
        output.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let readme =
        std::fs::read_to_string(checkout_dir.join("README.md")).expect("README should be cloned");
    assert!(readme.contains(&repository.name));
    let branch = Command::new("git")
        .current_dir(&checkout_dir)
        .args(["branch", "--show-current"])
        .output()
        .await
        .expect("branch query should run");
    assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), "main");
    let _ = std::fs::remove_dir_all(checkout_dir);
    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn private_repository_denies_anonymous_upload_pack() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping git transport private scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-private-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-private-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-clone-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
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
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), test_config());
    let (status, body, _) = send_raw(
        app,
        &format!(
            "/{}/{}.git/info/refs?service=git-upload-pack",
            repository.owner_login, repository.name
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let error: serde_json::Value =
        serde_json::from_slice(&body).expect("error body should be json");
    assert_eq!(error["error"]["code"], "authentication_required");
    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn private_repository_supports_token_and_session_upload_pack() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private git auth scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-private-auth-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let config = test_config();
    let owner = create_user(&pool, "git-private-token-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-auth-clone-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
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
    .expect("repository should create");

    let token = create_pat(&pool, owner.id, &["repo:read"], None).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let mut token_headers = HeaderMap::new();
    token_headers.insert(header::AUTHORIZATION, basic_auth_header(&token));
    let (status, body, content_type) = send_raw_with_headers(
        app.clone(),
        &format!(
            "/{}/{}.git/info/refs?service=git-upload-pack",
            repository.owner_login, repository.name
        ),
        token_headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/x-git-upload-pack-advertisement");
    assert!(String::from_utf8_lossy(&body).contains("refs/heads/main"));

    let cookie = session_cookie(&pool, &config, &owner).await;
    let mut cookie_headers = HeaderMap::new();
    cookie_headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&cookie).expect("cookie header should build"),
    );
    let (status, body, _) = send_raw_with_headers(
        app.clone(),
        &format!(
            "/{}/{}.git/info/refs?service=git-upload-pack",
            repository.owner_login, repository.name
        ),
        cookie_headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8_lossy(&body).contains("refs/heads/main"));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("local addr should read");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });
    let checkout_dir =
        std::env::temp_dir().join(format!("opengithub-private-clone-{}", Uuid::new_v4()));
    let remote = format!(
        "http://x-access-token:{}@{}/{}/{}.git",
        token, address, repository.owner_login, repository.name
    );
    let output = Command::new("git")
        .args(["clone", "--depth", "1", "--", &remote])
        .arg(&checkout_dir)
        .output()
        .await
        .expect("git clone should run");
    server.abort();
    assert!(
        output.status.success(),
        "private git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let readme =
        std::fs::read_to_string(checkout_dir.join("README.md")).expect("README should be cloned");
    assert!(readme.contains(&repository.name));

    let last_used_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT last_used_at FROM personal_access_tokens WHERE token_hash = $1")
            .bind(hash_personal_access_token(&token))
            .fetch_one(&pool)
            .await
            .expect("last_used_at should read");
    assert!(last_used_at.is_some(), "PAT use should update last_used_at");

    let _ = std::fs::remove_dir_all(checkout_dir);
    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn public_repository_streams_raw_files_and_reuses_zip_archives() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping raw/archive scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-archive-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-archive-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("archiveable-{}", Uuid::new_v4().simple()),
            description: Some("Archiveable public repository".to_owned()),
            visibility: RepositoryVisibility::Public,
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
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), test_config());
    let (raw_status, raw_body, raw_content_type) = send_raw(
        app.clone(),
        &format!(
            "/{}/{}/raw/main/README.md",
            repository.owner_login, repository.name
        ),
    )
    .await;
    assert_eq!(raw_status, StatusCode::OK);
    assert_eq!(raw_content_type, "text/markdown; charset=utf-8");
    assert!(String::from_utf8_lossy(&raw_body).contains(&repository.name));

    let archive_uri = format!(
        "/{}/{}/archive/refs/heads/main.zip",
        repository.owner_login, repository.name
    );
    let (archive_status, archive_body, archive_content_type) =
        send_raw(app.clone(), &archive_uri).await;
    assert_eq!(archive_status, StatusCode::OK);
    assert_eq!(archive_content_type, "application/zip");
    assert!(archive_body.starts_with(b"PK"));

    let (second_status, second_body, _) = send_raw(app, &archive_uri).await;
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(second_body, archive_body);

    let archive_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM repository_archives WHERE repository_id = $1 AND ref_name = 'main' AND format = 'zip'",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("archive count should read");
    assert_eq!(archive_count, 1);

    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn private_repository_denies_anonymous_raw_and_archives() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private raw/archive denial scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-archive-deny-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-archive-deny-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-archive-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
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
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), test_config());
    for uri in [
        format!(
            "/{}/{}/raw/main/README.md",
            repository.owner_login, repository.name
        ),
        format!(
            "/{}/{}/archive/refs/heads/main.zip",
            repository.owner_login, repository.name
        ),
    ] {
        let (status, body, _) = send_raw(app.clone(), &uri).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let rendered = String::from_utf8_lossy(&body);
        assert!(rendered.contains("authentication_required"));
        assert!(!rendered.contains("README.md"));
    }

    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn private_repository_rejects_invalid_expired_or_unscoped_tokens_without_leaking_secret() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private git token denial scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-token-denial-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-token-denial-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-token-denial-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
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
    .expect("repository should create");
    let expired_token = create_pat(
        &pool,
        owner.id,
        &["repo:read"],
        Some(Utc::now() - Duration::minutes(1)),
    )
    .await;
    let unscoped_token = create_pat(&pool, owner.id, &["user:read"], None).await;

    let app = opengithub_api::build_app_with_config(Some(pool), test_config());
    for token in ["not-a-real-token".to_owned(), expired_token, unscoped_token] {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth_header(&token));
        let (status, body, _) = send_raw_with_headers(
            app.clone(),
            &format!(
                "/{}/{}.git/info/refs?service=git-upload-pack",
                repository.owner_login, repository.name
            ),
            headers,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let rendered = String::from_utf8_lossy(&body);
        assert!(rendered.contains("authentication_required"));
        assert!(!rendered.contains(&token));
        assert!(!rendered.contains("sha256:"));
    }

    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn authorized_token_push_updates_repository_snapshot_and_activity() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping git receive-pack scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir = std::env::temp_dir().join(format!("opengithub-git-push-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-push-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("push-target-{}", Uuid::new_v4().simple()),
            description: Some("Push target".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest::default(),
    )
    .await
    .expect("empty repository should create");
    sqlx::query(
        r#"
        INSERT INTO repository_security_feature_settings (
            repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        VALUES ($1, 'secret_scanning', 'enabled', 'Secret scanning is monitoring pushed content.', 0, 0, '/settings/security_analysis')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("secret scanning setting should insert");

    let token = create_pat(&pool, owner.id, &["repo:write"], None).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), test_config());
    let mut token_headers = HeaderMap::new();
    token_headers.insert(header::AUTHORIZATION, basic_auth_header(&token));
    let (status, body, content_type) = send_raw_with_headers(
        app.clone(),
        &format!(
            "/{}/{}.git/info/refs?service=git-receive-pack",
            repository.owner_login, repository.name
        ),
        token_headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/x-git-receive-pack-advertisement");
    assert!(String::from_utf8_lossy(&body).contains("# service=git-receive-pack"));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("local addr should read");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });

    let worktree =
        std::env::temp_dir().join(format!("opengithub-push-worktree-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&worktree).expect("worktree should create");
    git_in(&worktree, ["init", "-b", "main"]).await;
    git_in(&worktree, ["config", "user.email", "push@example.com"]).await;
    git_in(&worktree, ["config", "user.name", "Push User"]).await;
    std::fs::write(
        worktree.join("README.md"),
        format!("# {}\n\nPushed over opengithub HTTPS.\n", repository.name),
    )
    .expect("README should write");
    std::fs::create_dir_all(worktree.join("src")).expect("src should create");
    std::fs::write(
        worktree.join("src/lib.rs"),
        "pub fn pushed() -> bool { true }\n",
    )
    .expect("source should write");
    let pushed_secret = format!("ghp_{}{}", "C".repeat(20), Uuid::new_v4().simple());
    std::fs::write(worktree.join(".env"), format!("TOKEN={pushed_secret}\n"))
        .expect("secret fixture should write");
    git_in(&worktree, ["add", "."]).await;
    git_in(&worktree, ["commit", "-m", "Push repository contents"]).await;
    let remote = format!(
        "http://x-access-token:{}@{}/{}/{}.git",
        token, address, repository.owner_login, repository.name
    );
    git_in(&worktree, ["remote", "add", "origin", &remote]).await;
    git_in(&worktree, ["push", "-u", "origin", "main"]).await;
    sqlx::query(
        r#"
        INSERT INTO repository_branch_protection_rules (
            repository_id, pattern, restricts_pushes
        )
        VALUES ($1, 'main', true)
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("branch policy should be inserted");
    std::fs::write(
        worktree.join("README.md"),
        format!(
            "# {}\n\nThis protected branch update should be blocked.\n",
            repository.name
        ),
    )
    .expect("protected README should write");
    git_in(&worktree, ["add", "README.md"]).await;
    git_in(&worktree, ["commit", "-m", "Blocked protected branch push"]).await;
    let blocked_push = Command::new("git")
        .current_dir(&worktree)
        .args(["push", "origin", "main"])
        .output()
        .await
        .expect("blocked git push should run");
    assert!(
        !blocked_push.status.success(),
        "protected branch push unexpectedly succeeded"
    );
    server.abort();

    let repository = get_repository_by_owner_name(&pool, &repository.owner_login, &repository.name)
        .await
        .expect("repository lookup should run")
        .expect("repository should exist");
    let overview = repository_overview_for_actor(&pool, repository.clone(), owner.id)
        .await
        .expect("overview should load");
    assert_eq!(
        overview
            .latest_commit
            .as_ref()
            .map(|commit| commit.message.as_str()),
        Some("Push repository contents")
    );
    let policy_evaluations: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM repository_rule_evaluations WHERE repository_id = $1 AND operation = 'push'",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("push policy evaluation count should read");
    assert_eq!(policy_evaluations, 0);
    assert!(overview.files.iter().any(|file| file.path == "README.md"));
    assert!(overview.files.iter().any(|file| file.path == "src/lib.rs"));
    assert!(overview
        .root_entries
        .iter()
        .any(|entry| entry.name == "src"));

    let feed_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM feed_events WHERE repository_id = $1 AND actor_user_id = $2 AND event_type = 'push'",
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("feed count should read");
    assert!(feed_count >= 1);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE target_id = $1 AND actor_user_id = $2 AND event_type = 'repository.push'",
    )
    .bind(repository.id.to_string())
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should read");
    assert!(audit_count >= 1);
    let secret_scan_payload: String = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT jsonb_agg(payload)::text
        FROM (
            SELECT jsonb_build_object(
                'redactedSecret', secret_scanning_alerts.redacted_secret,
                'redactedContext', secret_scanning_alerts.redacted_context,
                'path', secret_scanning_alert_locations.path,
                'bypassReason', push_protection_bypasses.reason
            ) AS payload
            FROM secret_scanning_alerts
            JOIN secret_scanning_alert_locations
              ON secret_scanning_alert_locations.alert_id = secret_scanning_alerts.id
            LEFT JOIN push_protection_bypasses
              ON push_protection_bypasses.alert_id = secret_scanning_alerts.id
            WHERE secret_scanning_alerts.repository_id = $1
        ) rows
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("secret scanning payload should read")
    .unwrap_or_default();
    assert!(secret_scan_payload.contains(".env"));
    assert!(secret_scan_payload.contains("push_protection_bypass"));
    assert!(!secret_scan_payload.contains(&pushed_secret));

    let _ = std::fs::remove_dir_all(worktree);
    let _ = std::fs::remove_dir_all(storage_dir);
}

#[tokio::test]
async fn receive_pack_requires_write_scope_or_write_permission() {
    let _env_guard = GIT_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping receive-pack auth scenario; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-git-push-deny-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);

    let owner = create_user(&pool, "git-push-deny-owner").await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("push-deny-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
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
    .expect("repository should create");
    let read_token = create_pat(&pool, owner.id, &["repo:read"], None).await;
    let app = opengithub_api::build_app_with_config(Some(pool), test_config());
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth_header(&read_token));
    let (status, body, _) = send_raw_with_headers(
        app,
        &format!(
            "/{}/{}.git/info/refs?service=git-receive-pack",
            repository.owner_login, repository.name
        ),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let rendered = String::from_utf8_lossy(&body);
    assert!(rendered.contains("authentication_required"));
    assert!(!rendered.contains(&read_token));
    let _ = std::fs::remove_dir_all(storage_dir);
}

async fn git_in<const N: usize>(current_dir: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .current_dir(current_dir)
        .args(args)
        .output()
        .await
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
