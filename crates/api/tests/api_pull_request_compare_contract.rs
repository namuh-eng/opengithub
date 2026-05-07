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
            create_repository, CreateRepository, Repository, RepositoryOwner, RepositoryVisibility,
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
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
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
    let request = builder.body(Body::empty()).expect("request should build");
    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response should be json")
    };
    (status, value)
}

async fn create_repo(
    pool: &PgPool,
    owner: &User,
    name: &str,
    visibility: RepositoryVisibility,
) -> Repository {
    create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: name.to_owned(),
            description: None,
            visibility,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create")
}

async fn insert_commit(
    pool: &PgPool,
    repository: &Repository,
    author: &User,
    oid: &str,
    message: &str,
    parents: Vec<String>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, parent_oids, committed_at)
        VALUES ($1, $2, $3, $3, $4, $5, now())
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(oid)
    .bind(author.id)
    .bind(message)
    .bind(parents)
    .fetch_one(pool)
    .await
    .expect("commit should insert")
}

async fn insert_file(
    pool: &PgPool,
    repository: &Repository,
    commit_id: Uuid,
    path: &str,
    content: &str,
    oid: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .bind(path)
    .bind(content)
    .bind(oid)
    .bind(content.len() as i64)
    .execute(pool)
    .await
    .expect("file should insert");
}

async fn upsert_ref(pool: &PgPool, repository: &Repository, name: &str, commit_id: Uuid) {
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository.id)
    .bind(name)
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("ref should upsert");
}

#[tokio::test]
async fn pull_request_compare_contract_resolves_refs_files_and_privacy() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping pull compare contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "compare-owner").await;
    let repo_name = format!("compare-contract-{}", Uuid::new_v4().simple());
    let repository = create_repo(&pool, &owner, &repo_name, RepositoryVisibility::Public).await;
    let base_oid = format!("base{}", Uuid::new_v4().simple());
    let head_oid = format!("head{}", Uuid::new_v4().simple());
    let base_commit = insert_commit(
        &pool,
        &repository,
        &owner,
        &base_oid,
        "Initial compare base",
        Vec::new(),
    )
    .await;
    let head_commit = insert_commit(
        &pool,
        &repository,
        &owner,
        &head_oid,
        "Add compare feature",
        vec![base_oid.clone()],
    )
    .await;
    insert_file(
        &pool,
        &repository,
        base_commit,
        "README.md",
        "hello\n",
        "blob-base-readme",
    )
    .await;
    insert_file(
        &pool,
        &repository,
        head_commit,
        "README.md",
        "hello\ncompare\n",
        "blob-head-readme",
    )
    .await;
    insert_file(
        &pool,
        &repository,
        head_commit,
        "src/lib.rs",
        "pub fn compare() {}\n",
        "blob-head-lib",
    )
    .await;
    upsert_ref(&pool, &repository, "main", base_commit).await;
    upsert_ref(&pool, &repository, "feature/compare", head_commit).await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let compare_uri = format!(
        "/api/repos/{}/{}/compare/main...{}",
        owner.email,
        repo_name,
        url::form_urlencoded::byte_serialize("feature/compare".as_bytes()).collect::<String>()
    );
    let (status, body) = send_json(app.clone(), &compare_uri, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "ahead");
    assert_eq!(body["base"]["shortName"], "main");
    assert_eq!(body["head"]["shortName"], "feature/compare");
    assert_eq!(body["aheadBy"], 1);
    assert_eq!(body["behindBy"], 0);
    assert_eq!(body["commits"][0]["message"], "Add compare feature");
    assert_eq!(body["files"].as_array().expect("files").len(), 2);
    assert!(body["files"]
        .as_array()
        .expect("files")
        .iter()
        .any(|file| file["path"] == "src/lib.rs" && file["status"] == "added"));
    assert_eq!(
        body["pullListHref"],
        format!(
            "/{}/{}/pulls",
            owner.username.as_deref().unwrap_or(&owner.email),
            repo_name
        )
    );

    let same_uri = format!(
        "/api/repos/{}/{}/compare/main...main",
        owner.email, repo_name
    );
    let (same_status, same_body) = send_json(app.clone(), &same_uri, None).await;
    assert_eq!(same_status, StatusCode::OK, "{same_body}");
    assert_eq!(same_body["status"], "same_ref");
    assert_eq!(same_body["totalFiles"], 0);

    let invalid_uri = format!(
        "/api/repos/{}/{}/compare/main...missing",
        owner.email, repo_name
    );
    let (invalid_status, invalid_body) = send_json(app.clone(), &invalid_uri, None).await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let private_repo_name = format!("compare-private-{}", Uuid::new_v4().simple());
    let private_repo = create_repo(
        &pool,
        &owner,
        &private_repo_name,
        RepositoryVisibility::Private,
    )
    .await;
    upsert_ref(&pool, &private_repo, "main", base_commit).await;
    let private_uri = format!(
        "/api/repos/{}/{}/compare/main...main",
        owner.email, private_repo_name
    );
    let (private_anon_status, private_anon_body) = send_json(app.clone(), &private_uri, None).await;
    assert_eq!(private_anon_status, StatusCode::FORBIDDEN);
    assert_eq!(private_anon_body["error"]["code"], "forbidden");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let (private_owner_status, private_owner_body) =
        send_json(app, &private_uri, Some(&owner_cookie)).await;
    assert_eq!(private_owner_status, StatusCode::OK, "{private_owner_body}");
    assert_eq!(private_owner_body["status"], "same_ref");
}
