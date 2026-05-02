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
        None,
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1, profile_visibility = $2 WHERE id = $3")
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
async fn starred_repositories_filter_sort_and_redact_private_rows() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile starred repositories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("profstars{}", Uuid::new_v4().simple());
    let profile_owner = create_profile_user(&pool, &marker, false).await;
    let repo_owner = create_profile_user(&pool, &format!("{marker}-owner"), false).await;
    let collaborator = create_profile_user(&pool, &format!("{marker}-collab"), false).await;
    let collaborator_cookie = cookie_header(&pool, &config, &collaborator).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let alpha = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: repo_owner.id },
            name: format!("{marker}-alpha"),
            description: Some("Rust starred collaboration repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: repo_owner.id,
        },
    )
    .await
    .expect("alpha repo should create");
    let beta = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: repo_owner.id },
            name: format!("{marker}-beta"),
            description: Some("TypeScript active repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: repo_owner.id,
        },
    )
    .await
    .expect("beta repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: repo_owner.id },
            name: format!("{marker}-private"),
            description: Some("Private starred repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: repo_owner.id,
        },
    )
    .await
    .expect("private repo should create");

    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '3 hours' WHERE id = $1")
        .bind(alpha.id)
        .execute(&pool)
        .await
        .expect("alpha updated_at should update");
    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '1 hour' WHERE id = $1")
        .bind(beta.id)
        .execute(&pool)
        .await
        .expect("beta updated_at should update");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', '#b7410e', 900), ($2, 'TypeScript', '#8c5a3c', 500), ($3, 'Rust', '#b7410e', 200)",
    )
    .bind(alpha.id)
    .bind(beta.id)
    .bind(private_repo.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_stars (user_id, repository_id, created_at)
        VALUES
          ($1, $2, now() - INTERVAL '1 day'),
          ($1, $3, now() - INTERVAL '2 days'),
          ($1, $4, now() - INTERVAL '3 days'),
          ($5, $2, now() - INTERVAL '1 hour')
        "#,
    )
    .bind(profile_owner.id)
    .bind(alpha.id)
    .bind(beta.id)
    .bind(private_repo.id)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("stars should insert");
    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role, source) VALUES ($1, $2, 'read', 'direct')",
    )
    .bind(private_repo.id)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("private grant should insert");

    let (status, headers, body) =
        get_json(app.clone(), &format!("/api/users/{marker}/stars"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["mode"], "stars");
    assert_eq!(body["total"], 2);
    assert_eq!(body["filters"]["sort"], "recently-starred");
    assert_eq!(body["items"][0]["name"], alpha.name);
    assert!(body["items"][0]["starredAt"].is_string());
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["visibility"] == "public"));
    assert_eq!(body["tabCounts"]["stars"], 2);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/stars?sort=recently-active"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["name"], beta.name);
    assert_eq!(body["filters"]["sort"], "recently-active");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/stars?sort=most-stars"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["name"], alpha.name);
    assert_eq!(body["items"][0]["starsCount"], 2);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/stars?q=typescript&language=TypeScript"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], beta.name);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/stars?language=Rust"),
        Some(&collaborator_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2);
    assert_eq!(body["tabCounts"]["stars"], 3);
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["name"] == private_repo.name));

    let (status, _, body) = get_json(
        app,
        &format!("/api/users/{marker}/stars?sort=unsupported"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn private_profile_stars_tab_returns_empty_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private profile stars scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("privstars{}", Uuid::new_v4().simple());
    let owner = create_profile_user(&pool, &marker, true).await;
    let repo_owner = create_profile_user(&pool, &format!("{marker}-owner"), false).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: repo_owner.id },
            name: format!("{marker}-visible"),
            description: Some("Visible repository starred by private profile".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: repo_owner.id,
        },
    )
    .await
    .expect("repository should create");
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(owner.id)
        .bind(repository.id)
        .execute(&pool)
        .await
        .expect("star should insert");
    let app = opengithub_api::build_app_with_config(Some(pool), config);

    let (status, _, body) = get_json(app, &format!("/api/users/{marker}/stars"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mode"], "stars");
    assert_eq!(body["total"], 0);
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["tabCounts"]["stars"], 0);
}
