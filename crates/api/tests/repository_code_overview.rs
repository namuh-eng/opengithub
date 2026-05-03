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
            create_repository_with_bootstrap, CreateRepository, RepositoryBootstrapRequest,
            RepositoryOwner, RepositoryVisibility,
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

async fn send_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    send_json_with_method(app, Method::GET, uri, cookie).await
}

async fn send_json_with_method(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Value) {
    send_json_with_method_and_body(app, method, uri, cookie, Body::empty()).await
}

async fn send_json_with_method_and_body(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Body,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(body).expect("request should build"))
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

async fn send_json_body(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
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

#[tokio::test]
async fn repository_code_overview_returns_root_workspace_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository code overview scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-code-owner").await;
    let cookie = cookie_header(&pool, &config, &owner).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("code-overview-{}", Uuid::new_v4().simple()),
            description: Some("Code overview repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            gitignore_template_slug: Some("rust".to_owned()),
            license_template_slug: Some("mit".to_owned()),
        },
    )
    .await
    .expect("repository should create");
    sqlx::query(
        r#"
        INSERT INTO repository_languages (repository_id, language, color, byte_count)
        VALUES ($1, 'Rust', '#dea584', 1200), ($1, 'TOML', '#9c4221', 300)
        ON CONFLICT (repository_id, lower(language)) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(owner.id)
        .bind(repository.id)
        .execute(&pool)
        .await
        .expect("star should insert");
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM commits WHERE repository_id = $1 ORDER BY committed_at DESC LIMIT 1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("commit id should exist");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, 'refs/heads/feature-sidebar', 'branch', $2),
               ($1, 'refs/tags/v1.0.0', 'tag', $2)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("extra refs should insert");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app.clone(),
        &format!("/api/repos/{}/{}", repository.owner_login, repository.name),
        Some(&cookie),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], repository.name);
    assert_eq!(body["viewerPermission"], "owner");
    assert_eq!(body["branchCount"], 2);
    assert_eq!(body["tagCount"], 1);
    assert_eq!(body["latestCommit"]["message"], "Initial commit");
    assert_eq!(body["readme"]["path"], "README.md");
    assert!(body["rootEntries"]
        .as_array()
        .expect("root entries should be an array")
        .iter()
        .any(|entry| entry["kind"] == "folder" && entry["name"] == "src"));
    assert!(body["rootEntries"]
        .as_array()
        .expect("root entries should be an array")
        .iter()
        .any(|entry| entry["kind"] == "file" && entry["name"] == "Cargo.toml"));
    assert_eq!(body["sidebar"]["starsCount"], 1);
    assert_eq!(body["sidebar"]["languages"][0]["language"], "Rust");
    assert!(body["cloneUrls"]["https"]
        .as_str()
        .expect("clone url should exist")
        .ends_with(".git"));

    let (refs_status, refs_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/refs",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(refs_status, StatusCode::OK);
    assert_eq!(refs_body["total"], 3);
    assert!(refs_body["items"]
        .as_array()
        .expect("refs should be an array")
        .iter()
        .any(|entry| entry["shortName"] == "feature-sidebar" && entry["kind"] == "branch"));
    assert!(refs_body["items"]
        .as_array()
        .expect("refs should be an array")
        .iter()
        .any(|entry| entry["shortName"] == "v1.0.0" && entry["kind"] == "tag"));

    let (finder_status, finder_body) = send_json(
        app,
        &format!(
            "/api/repos/{}/{}/file-finder?ref=main&q=main",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(finder_status, StatusCode::OK);
    assert!(finder_body["items"]
        .as_array()
        .expect("finder items should be an array")
        .iter()
        .any(|entry| entry["path"] == "src/main.rs"
            && entry["href"]
                .as_str()
                .expect("finder href")
                .ends_with("/blob/main/src/main.rs")));
}

#[tokio::test]
async fn repository_code_overview_preserves_private_access_boundary() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private repository code overview scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-code-private-owner").await;
    let outsider = create_user(&pool, "repo-code-private-reader").await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-code-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app,
        &format!("/api/repos/{}/{}", repository.owner_login, repository.name),
        Some(&outsider_cookie),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");
}

#[tokio::test]
async fn repository_tree_blob_and_history_routes_resolve_nested_paths() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository path navigation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-path-owner").await;
    let cookie = cookie_header(&pool, &config, &owner).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("path-nav-{}", Uuid::new_v4().simple()),
            description: Some("Path navigation repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (tree_status, tree_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/contents/src?ref=main",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(tree_status, StatusCode::OK);
    assert_eq!(tree_body["path"], "src");
    assert!(tree_body["breadcrumbs"]
        .as_array()
        .expect("breadcrumbs should be an array")
        .iter()
        .any(|breadcrumb| breadcrumb["name"] == "src"));
    assert!(tree_body["entries"]
        .as_array()
        .expect("entries should be an array")
        .iter()
        .any(|entry| entry["kind"] == "file" && entry["name"] == "main.rs"));

    let (blob_status, blob_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/blobs/src/main.rs?ref=main",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(blob_status, StatusCode::OK);
    assert_eq!(blob_body["path"], "src/main.rs");
    assert_eq!(blob_body["language"], "Rust");
    assert!(blob_body["file"]["content"]
        .as_str()
        .expect("blob content should be present")
        .contains("tokio::main"));
    assert_eq!(
        blob_body["historyHref"],
        format!(
            "/{}/{}/commits/main/src/main.rs",
            repository.owner_login, repository.name
        )
    );

    let (history_status, history_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/commits?ref=main&path=src/main.rs",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(history_status, StatusCode::OK);
    assert_eq!(history_body["total"], 1);
    assert_eq!(history_body["items"][0]["message"], "Initial commit");

    let (missing_status, missing_body) = send_json(
        app,
        &format!(
            "/api/repos/{}/{}/contents/src/missing?ref=main",
            repository.owner_login, repository.name
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "path_not_found");
}

#[tokio::test]
async fn repository_header_actions_toggle_social_state_and_create_fork() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository social action scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "repo-social-owner").await;
    let actor = create_user(&pool, "repo-social-actor").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("social-source-{}", Uuid::new_v4().simple()),
            description: Some("Social action source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let star_uri = format!(
        "/api/repos/{}/{}/star",
        repository.owner_login, repository.name
    );
    let watch_uri = format!(
        "/api/repos/{}/{}/watch",
        repository.owner_login, repository.name
    );
    let fork_uri = format!(
        "/api/repos/{}/{}/forks",
        repository.owner_login, repository.name
    );

    let (anonymous_status, anonymous_body) =
        send_json_with_method(app.clone(), Method::PUT, &star_uri, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (star_status, star_body) =
        send_json_with_method(app.clone(), Method::PUT, &star_uri, Some(&actor_cookie)).await;
    assert_eq!(star_status, StatusCode::OK);
    assert_eq!(star_body["starred"], true);
    assert_eq!(star_body["starsCount"], 1);

    let (star_again_status, star_again_body) =
        send_json_with_method(app.clone(), Method::PUT, &star_uri, Some(&actor_cookie)).await;
    assert_eq!(star_again_status, StatusCode::OK);
    assert_eq!(star_again_body["starsCount"], 1);

    let (watch_status, watch_body) =
        send_json_with_method(app.clone(), Method::PUT, &watch_uri, Some(&actor_cookie)).await;
    assert_eq!(watch_status, StatusCode::OK);
    assert_eq!(watch_body["watching"], true);
    assert_eq!(watch_body["watchLevel"], "participating");
    assert_eq!(watch_body["watchLabel"], "Participating and @mentions");
    assert_eq!(watch_body["watchersCount"], 1);

    let (read_watch_status, read_watch_body) =
        send_json(app.clone(), &watch_uri, Some(&actor_cookie)).await;
    assert_eq!(read_watch_status, StatusCode::OK);
    assert_eq!(read_watch_body["level"], "participating");
    assert_eq!(read_watch_body["watching"], true);
    assert_eq!(
        read_watch_body["availableEvents"].as_array().unwrap().len(),
        7
    );

    let (custom_watch_status, custom_watch_body) = send_json_body(
        app.clone(),
        Method::PATCH,
        &watch_uri,
        Some(&actor_cookie),
        json!({
            "level": "custom",
            "customEvents": ["pull_requests", "issues", "issues"]
        }),
    )
    .await;
    assert_eq!(custom_watch_status, StatusCode::OK);
    assert_eq!(custom_watch_body["level"], "custom");
    assert_eq!(
        custom_watch_body["customEvents"],
        json!(["issues", "pull_requests"])
    );
    assert_eq!(custom_watch_body["watchersCount"], 1);

    let (ignore_status, ignore_body) = send_json_body(
        app.clone(),
        Method::PATCH,
        &watch_uri,
        Some(&actor_cookie),
        json!({ "level": "ignore" }),
    )
    .await;
    assert_eq!(ignore_status, StatusCode::OK);
    assert_eq!(ignore_body["level"], "ignore");
    assert_eq!(ignore_body["watching"], false);
    assert_eq!(ignore_body["watchersCount"], 0);
    assert!(ignore_body["ignoreWarning"]
        .as_str()
        .unwrap()
        .contains("suppresses"));

    let (invalid_custom_status, invalid_custom_body) = send_json_body(
        app.clone(),
        Method::PATCH,
        &watch_uri,
        Some(&actor_cookie),
        json!({ "level": "custom", "customEvents": [] }),
    )
    .await;
    assert_eq!(invalid_custom_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_custom_body["error"]["code"], "validation_failed");

    let (watch_again_status, watch_again_body) =
        send_json_with_method(app.clone(), Method::PUT, &watch_uri, Some(&actor_cookie)).await;
    assert_eq!(watch_again_status, StatusCode::OK);
    assert_eq!(watch_again_body["watching"], true);
    assert_eq!(watch_again_body["watchersCount"], 1);

    let (fork_status, fork_body) =
        send_json_with_method(app.clone(), Method::POST, &fork_uri, Some(&actor_cookie)).await;
    assert_eq!(fork_status, StatusCode::CREATED);
    assert!(fork_body["forkHref"]
        .as_str()
        .expect("fork href should exist")
        .ends_with(&format!("/{}", repository.name)));
    assert_eq!(fork_body["social"]["forksCount"], 1);
    assert!(fork_body["social"]["forkedRepositoryHref"].is_string());

    let (duplicate_status, duplicate_body) =
        send_json_with_method(app.clone(), Method::POST, &fork_uri, Some(&actor_cookie)).await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let (overview_status, overview_body) = send_json(
        app.clone(),
        &format!("/api/repos/{}/{}", repository.owner_login, repository.name),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(overview_status, StatusCode::OK);
    assert_eq!(overview_body["viewerState"]["starred"], true);
    assert_eq!(overview_body["viewerState"]["watching"], true);
    assert_eq!(overview_body["viewerState"]["watchLevel"], "participating");
    assert_eq!(overview_body["sidebar"]["starsCount"], 1);
    assert_eq!(overview_body["sidebar"]["watchersCount"], 1);
    assert_eq!(overview_body["sidebar"]["forksCount"], 1);

    let (unstar_status, unstar_body) =
        send_json_with_method(app.clone(), Method::DELETE, &star_uri, Some(&actor_cookie)).await;
    assert_eq!(unstar_status, StatusCode::OK);
    assert_eq!(unstar_body["starred"], false);
    assert_eq!(unstar_body["starsCount"], 0);

    let (unwatch_status, unwatch_body) =
        send_json_with_method(app, Method::DELETE, &watch_uri, Some(&actor_cookie)).await;
    assert_eq!(unwatch_status, StatusCode::OK);
    assert_eq!(unwatch_body["watching"], false);
    assert_eq!(unwatch_body["watchersCount"], 0);

    let feed_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM feed_events WHERE repository_id = $1 AND event_type IN ('star', 'fork')",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("feed count should query");
    assert!(feed_count >= 2);
}
