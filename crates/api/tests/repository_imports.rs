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
        repositories::{create_organization, CreateOrganization},
        repository_imports::{repository_import_credential_metadata, validate_import_source_url},
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

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("request should build")
    } else {
        builder.body(Body::empty()).expect("request should build")
    };

    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

#[test]
fn import_source_url_validation_blocks_unsafe_or_non_git_sources() {
    let valid = validate_import_source_url("https://github.com/octocat/Hello-World.git#main")
        .expect("GitHub URL should validate");
    assert_eq!(valid.host, "github.com");
    assert_eq!(valid.path, "octocat/Hello-World.git");
    assert_eq!(valid.url, "https://github.com/octocat/Hello-World.git");

    assert!(validate_import_source_url("ssh://github.com/octocat/Hello-World.git").is_err());
    assert!(validate_import_source_url("https://localhost/octocat/repo.git").is_err());
    assert!(validate_import_source_url("https://127.0.0.1/octocat/repo.git").is_err());
    assert!(validate_import_source_url("https://10.0.0.4/octocat/repo.git").is_err());
    assert!(validate_import_source_url("https://example.com/").is_err());
}

#[tokio::test]
async fn repository_import_create_queues_job_and_redacts_credentials() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository import scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "importer").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo_name = format!("imported-{}", Uuid::new_v4().simple());

    let (status, body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(json!({
            "sourceUrl": "https://github.com/octocat/Hello-World.git",
            "sourceUsername": "octocat",
            "sourceToken": "super-secret-token",
            "ownerType": "user",
            "ownerId": user.id,
            "name": repo_name,
            "description": " Imported through the API contract ",
            "visibility": "private"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "queued");
    assert_eq!(body["source"]["host"], "github.com");
    assert_eq!(body["source"]["path"], "octocat/Hello-World.git");
    assert_eq!(body["requestedByUserId"], user.id.to_string());
    assert!(body["statusHref"]
        .as_str()
        .expect("status href")
        .starts_with("/new/import/"));
    assert!(body["repositoryHref"]
        .as_str()
        .expect("repository href")
        .contains(&repo_name));
    assert!(!body.to_string().contains("super-secret-token"));

    let import_id =
        Uuid::parse_str(body["id"].as_str().expect("import id")).expect("import id should parse");
    let repository_id = Uuid::parse_str(body["repositoryId"].as_str().expect("repository id"))
        .expect("repository id should parse");

    let credential = repository_import_credential_metadata(&pool, import_id)
        .await
        .expect("credential metadata should query")
        .expect("credential metadata should exist");
    assert_eq!(credential.credential_kind, "basic");
    assert_eq!(credential.username.as_deref(), Some("octocat"));
    let secret_ref = credential.secret_ref.expect("secret ref should exist");
    assert!(secret_ref.starts_with("repo-import-secret-ref:"));
    assert_ne!(secret_ref, "super-secret-token");

    let job_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM job_leases WHERE queue = 'repository_import' AND lease_key = $1 AND payload->>'repositoryId' = $2",
    )
    .bind(import_id.to_string())
    .bind(repository_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("job should count");
    assert_eq!(job_count, 1);

    let (read_status, read_body) = send_json(
        app,
        Method::GET,
        &format!("/api/repos/imports/{import_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(read_status, StatusCode::OK);
    assert_eq!(read_body["id"], import_id.to_string());
    assert_eq!(read_body["status"], "queued");
}

#[tokio::test]
async fn repository_import_rejects_anonymous_invalid_duplicate_and_forbidden_requests() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository import rejection scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "import-actor").await;
    let other = create_user(&pool, "import-other").await;
    let org_owner = create_user(&pool, "import-org-owner").await;
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("import-org-{}", Uuid::new_v4().simple()),
            display_name: "Import Org".to_owned(),
            description: None,
            owner_user_id: org_owner.id,
        },
    )
    .await
    .expect("org should create");
    let cookie = cookie_header(&pool, &config, &actor).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo_name = format!("import-conflict-{}", Uuid::new_v4().simple());

    let request_body = json!({
        "sourceUrl": "https://github.com/octocat/Hello-World.git",
        "ownerType": "user",
        "ownerId": actor.id,
        "name": repo_name,
        "visibility": "public"
    });
    let (anonymous_status, anonymous_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        None,
        Some(request_body.clone()),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (missing_fields_status, missing_fields_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(json!({})),
    )
    .await;
    assert_eq!(missing_fields_status, StatusCode::BAD_REQUEST);
    assert_eq!(missing_fields_body["error"]["code"], "invalid_json");
    assert_eq!(missing_fields_body["status"], 400);

    let (invalid_status, invalid_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(json!({
            "sourceUrl": "https://127.0.0.1/private/repo.git",
            "ownerType": "user",
            "ownerId": actor.id,
            "name": format!("blocked-{}", Uuid::new_v4().simple()),
            "visibility": "public"
        })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (forbidden_user_status, forbidden_user_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(json!({
            "sourceUrl": "https://github.com/octocat/Hello-World.git",
            "ownerType": "user",
            "ownerId": other.id,
            "name": format!("forbidden-{}", Uuid::new_v4().simple()),
            "visibility": "public"
        })),
    )
    .await;
    assert_eq!(forbidden_user_status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden_user_body["error"]["code"], "forbidden");

    let (forbidden_org_status, forbidden_org_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(json!({
            "sourceUrl": "https://github.com/octocat/Hello-World.git",
            "ownerType": "organization",
            "ownerId": org.id,
            "name": format!("forbidden-org-{}", Uuid::new_v4().simple()),
            "visibility": "private"
        })),
    )
    .await;
    assert_eq!(forbidden_org_status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden_org_body["error"]["code"], "forbidden");

    let (created_status, created_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(request_body.clone()),
    )
    .await;
    assert_eq!(created_status, StatusCode::CREATED);

    let (duplicate_status, duplicate_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos/imports",
        Some(&cookie),
        Some(request_body),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let import_id = Uuid::parse_str(created_body["id"].as_str().expect("import id"))
        .expect("import id should parse");
    let other_cookie = cookie_header(&pool, &app_config(), &other).await;
    let (read_status, read_body) = send_json(
        app,
        Method::GET,
        &format!("/api/repos/imports/{import_id}"),
        Some(&other_cookie),
        None,
    )
    .await;
    assert_eq!(read_status, StatusCode::FORBIDDEN);
    assert_eq!(read_body["error"]["code"], "forbidden");
}
