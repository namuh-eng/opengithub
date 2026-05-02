use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
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

async fn create_profile_user(pool: &PgPool, username: &str, private: bool) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{username}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{username} display")),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1, profile_visibility = $2
        WHERE id = $3
        "#,
    )
    .bind(username)
    .bind(if private { "private" } else { "public" })
    .bind(user.id)
    .execute(pool)
    .await
    .expect("profile columns should update");
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

async fn get_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
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

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn profile_repositories_filter_sort_and_redact_private_rows() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile repositories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("profrepo{}", Uuid::new_v4().simple());
    let owner = create_profile_user(&pool, &marker, false).await;
    let collaborator = create_profile_user(&pool, &format!("{marker}-collab"), false).await;
    let outsider = create_profile_user(&pool, &format!("{marker}-outsider"), false).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let collaborator_cookie = cookie_header(&pool, &config, &collaborator).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let source = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: outsider.id },
            name: format!("{marker}-upstream"),
            description: Some("Upstream Rust source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: outsider.id,
        },
    )
    .await
    .expect("source repo should create");
    let alpha = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-alpha"),
            description: Some("Rust API source repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("alpha repo should create");
    let beta_private = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-private"),
            description: Some("TypeScript private repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");
    let fork = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-fork"),
            description: Some("Forked template mirror".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("fork repo should create");
    sqlx::query(
        r#"
        UPDATE repositories
        SET is_template = true,
            is_mirror = true,
            can_be_sponsored = true,
            license_template_slug = 'mit',
            updated_at = now() - INTERVAL '1 hour'
        WHERE id = $1
        "#,
    )
    .bind(alpha.id)
    .execute(&pool)
    .await
    .expect("alpha metadata should update");
    sqlx::query(
        "UPDATE repositories SET is_archived = true, updated_at = now() - INTERVAL '2 hours' WHERE id = $1",
    )
    .bind(beta_private.id)
    .execute(&pool)
    .await
    .expect("private metadata should update");
    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '3 hours' WHERE id = $1")
        .bind(fork.id)
        .execute(&pool)
        .await
        .expect("fork metadata should update");
    sqlx::query(
        "INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(source.id)
    .bind(fork.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("fork edge should insert");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', '#b7410e', 900), ($2, 'TypeScript', '#8c5a3c', 500), ($3, 'Go', '#6f8f72', 200)",
    )
    .bind(alpha.id)
    .bind(beta_private.id)
    .bind(fork.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query(
        "INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2), ($3, $2), ($1, $4)",
    )
    .bind(collaborator.id)
    .bind(alpha.id)
    .bind(outsider.id)
    .bind(fork.id)
    .execute(&pool)
    .await
    .expect("stars should insert");
    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role, source) VALUES ($1, $2, 'read', 'direct')",
    )
    .bind(beta_private.id)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("private read grant should insert");
    let issue_id: Uuid = sqlx::query_scalar(
        "INSERT INTO issues (repository_id, number, title, author_user_id) VALUES ($1, 1, 'Open issue', $2) RETURNING id",
    )
    .bind(alpha.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    sqlx::query(
        "INSERT INTO pull_requests (repository_id, issue_id, number, title, author_user_id, head_ref, base_ref, head_repository_id, base_repository_id) VALUES ($1, $2, 2, 'Open PR', $3, 'feature', 'main', $1, $1)",
    )
    .bind(alpha.id)
    .bind(issue_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pull request should insert");

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/repositories?sort=stars&page=0&pageSize=500"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["total"], 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 100);
    assert_eq!(body["items"][0]["name"], alpha.name);
    assert_eq!(body["items"][0]["starsCount"], 2);
    assert_eq!(body["items"][0]["openIssuesCount"], 1);
    assert_eq!(body["items"][0]["openPullRequestsCount"], 1);
    assert_eq!(body["items"][0]["license"]["slug"], "mit");
    assert_eq!(body["items"][0]["isTemplate"], true);
    assert_eq!(body["items"][0]["isMirror"], true);
    assert_eq!(body["items"][0]["canBeSponsored"], true);
    assert_eq!(body["items"][0]["primaryLanguage"]["language"], "Rust");
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["visibility"] == "public"));
    assert_eq!(body["tabCounts"]["repositories"], 2);
    assert_eq!(body["availableTypes"][0]["value"], "all");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/repositories?type=forks"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], fork.name);
    assert_eq!(body["items"][0]["isFork"], true);
    assert_eq!(body["items"][0]["forkSource"]["name"], source.name);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/repositories?q=typescript&language=TypeScript"),
        Some(&collaborator_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], beta_private.name);
    assert_eq!(body["items"][0]["visibility"], "private");
    assert_eq!(body["tabCounts"]["repositories"], 3);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/repositories?type=archived"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["isArchived"], true);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/repositories?sort=name&page=1&pageSize=1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2);
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["filters"]["sort"], "name");
    assert_eq!(body["filters"]["pageSize"], 1);

    let (status, _, body) = get_json(
        app,
        &format!("/api/users/{marker}/repositories?type=unsupported"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn private_profile_repository_tab_returns_empty_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private profile repositories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("privprofrepo{}", Uuid::new_v4().simple());
    let owner = create_profile_user(&pool, &marker, true).await;
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-hidden"),
            description: Some("Hidden profile repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("hidden repo should create");
    let app = opengithub_api::build_app_with_config(Some(pool), config);

    let (status, _, body) = get_json(app, &format!("/api/users/{marker}/repositories"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["tabCounts"]["repositories"], 0);
}
