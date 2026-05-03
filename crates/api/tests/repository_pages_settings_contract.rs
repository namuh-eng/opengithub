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
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
        },
    },
};
use serde_json::{json, Value};
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

#[tokio::test]
async fn repository_pages_settings_validate_privacy_mutations_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository Pages settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pages{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outside")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Pages settings contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, reader.id, RepositoryRole::Read, "direct")
        .await
        .expect("reader grant should persist");
    let commit_id = seed_commit_and_branch(&pool, repo.id, "main").await;

    let uri = format!("/api/repos/{}/{}/settings/pages", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outside_status, _, outside_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&outsider_cookie), None).await;
    assert_eq!(outside_status, StatusCode::FORBIDDEN);
    assert!(!outside_body.to_string().contains("opengithub-pages"));

    let (initial_status, _, initial_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&owner_cookie), None).await;
    assert_eq!(initial_status, StatusCode::OK);
    assert_eq!(initial_body["site"]["source"]["kind"], "none");
    assert_eq!(initial_body["canEdit"], true);
    assert_eq!(initial_body["availableRefs"][0]["name"], "main");

    let (missing_docs_status, _, missing_docs_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/source"),
        Some(&owner_cookie),
        Some(json!({ "kind": "branch", "branch": "main", "folder": "/docs" })),
    )
    .await;
    assert_eq!(missing_docs_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing_docs_body["error"]["code"], "validation_failed");

    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, 'docs/index.html', '<h1>Pages</h1>', $3, 14)
        "#,
    )
    .bind(repo.id)
    .bind(commit_id)
    .bind(format!("{}-docs", Uuid::new_v4().simple()))
    .execute(&pool)
    .await
    .expect("docs file should persist");

    let (source_status, _, source_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/source"),
        Some(&owner_cookie),
        Some(json!({ "kind": "branch", "branch": "refs/heads/main", "folder": "/docs" })),
    )
    .await;
    assert_eq!(source_status, StatusCode::OK);
    assert_eq!(source_body["site"]["source"]["kind"], "branch");
    assert_eq!(source_body["site"]["source"]["branch"], "main");
    assert_eq!(source_body["site"]["source"]["folder"], "/docs");
    assert_eq!(source_body["deployments"][0]["status"], "queued");

    let deployment_id = source_body["deployments"][0]["id"]
        .as_str()
        .expect("deployment id should serialize");
    let queued = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM job_leases WHERE queue = 'pages-build-deploy' AND lease_key = $1)",
    )
    .bind(deployment_id)
    .fetch_one(&pool)
    .await
    .expect("job lookup should run");
    assert!(queued);

    let (invalid_domain_status, _, invalid_domain_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": "*.example.com" })),
    )
    .await;
    assert_eq!(invalid_domain_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_domain_body["error"]["code"], "validation_failed");

    let (domain_status, _, domain_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": "Pages.Example.COM." })),
    )
    .await;
    assert_eq!(domain_status, StatusCode::OK);
    assert_eq!(domain_body["site"]["customDomain"], "pages.example.com");
    assert_eq!(domain_body["site"]["domain"]["status"], "pending");
    assert!(domain_body["site"]["domain"]["challenge"]["value"]
        .as_str()
        .expect("challenge should be visible to admins")
        .starts_with("og-pages-"));

    let (reader_status, _, reader_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&reader_cookie), None).await;
    assert_eq!(reader_status, StatusCode::OK);
    assert_eq!(reader_body["canEdit"], false);
    assert_eq!(reader_body["site"]["customDomain"], "pages.example.com");
    assert!(reader_body["site"]["domain"]["challenge"].is_null());
    assert!(!reader_body.to_string().contains("og-pages-"));

    let (https_blocked_status, _, https_blocked_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/https"),
        Some(&owner_cookie),
        Some(json!({ "enforced": true })),
    )
    .await;
    assert_eq!(https_blocked_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(https_blocked_body["error"]["code"], "validation_failed");

    std::env::set_var("PAGES_DNS_VERIFICATION_MODE", "verified");
    let (recheck_status, _, recheck_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain/recheck"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(recheck_status, StatusCode::OK);
    assert_eq!(recheck_body["site"]["domain"]["status"], "verified");
    assert_eq!(recheck_body["site"]["certificateStatus"], "issued");

    let (https_status, _, https_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/https"),
        Some(&owner_cookie),
        Some(json!({ "enforced": true })),
    )
    .await;
    assert_eq!(https_status, StatusCode::OK);
    assert_eq!(https_body["site"]["httpsEnforced"], true);

    let second_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-other"),
            description: Some("Pages domain conflict".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("second repository should create");
    seed_commit_and_branch(&pool, second_repo.id, "main").await;
    let second_uri = format!(
        "/api/repos/{}/{}/settings/pages",
        owner.email, second_repo.name
    );
    let (conflict_status, _, conflict_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{second_uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": "pages.example.com" })),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
    assert_eq!(conflict_body["error"]["code"], "conflict");

    let (unpublish_status, _, unpublish_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/unpublish"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(unpublish_status, StatusCode::OK);
    assert_eq!(unpublish_body["site"]["source"]["kind"], "none");
    assert_eq!(unpublish_body["site"]["provisioningStatus"], "unpublished");

    let docs_file_still_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM repository_files WHERE repository_id = $1 AND path = 'docs/index.html')",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("source file check should run");
    assert!(docs_file_still_exists);

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.pages.%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 6);
}

async fn seed_commit_and_branch(pool: &PgPool, repository_id: Uuid, branch: &str) -> Uuid {
    let oid = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let row = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, message, tree_oid)
        VALUES ($1, $2, 'seed pages commit', $3)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(&oid)
    .bind(format!("tree-{oid}"))
    .fetch_one(pool)
    .await
    .expect("commit should persist");
    let commit_id: Uuid = row.get("id");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{branch}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("branch ref should persist");
    commit_id
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

async fn create_user(pool: &PgPool, login: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{login} display")),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(login)
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
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request_body = if let Some(value) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&value).expect("body should serialize"))
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
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
