use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, insert_commit, upsert_git_ref, CreateCommit, CreateRepository,
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
    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!(
                "skipping repository commit detail scenario; database connect failed: {error}"
            );
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_commit_history_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_commit_status_summaries') IS NOT NULL
               AND to_regclass('public.repository_commit_recent_visits') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_commit_history_tables {
            eprintln!("skipping repository commit detail scenario; migration failed: {error}");
            return None;
        }
        eprintln!("continuing repository commit detail scenario with pre-applied schema after migration warning: {error}");
    }
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

async fn get_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
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

#[tokio::test]
async fn repository_commit_detail_returns_summary_contract_without_leaking_private_data() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository commit detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "detail-owner").await;
    let outsider = create_user(&pool, "detail-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("commit-detail-{}", Uuid::new_v4().simple()),
            description: Some("Commit detail repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let base_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("base{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Initial detail commit".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(1),
        },
    )
    .await
    .expect("base commit should insert");
    let detail_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("detail{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Render commit detail\n\nShows summary metadata before the diff slice."
                .to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit.oid.clone()],
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("detail commit should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/main",
        "branch",
        Some(detail_commit.id),
    )
    .await
    .expect("main branch should upsert");
    for (commit_id, path, content, oid) in [
        (
            base_commit.id,
            "src/main.rs",
            "fn main() {\n    println!(\"old\");\n}\n",
            "blob-base-main",
        ),
        (
            detail_commit.id,
            "src/main.rs",
            "fn main() {\n    println!(\"new\");\n}\n",
            "blob-detail-main",
        ),
        (
            detail_commit.id,
            "docs/commit-detail.md",
            "# Commit detail\n",
            "blob-detail-docs",
        ),
    ] {
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
        .execute(&pool)
        .await
        .expect("repository file should insert");
    }
    sqlx::query(
        r#"
        INSERT INTO repository_commit_status_summaries
            (commit_id, status, conclusion, total_count, completed_count, failed_count)
        VALUES ($1, 'completed', 'success', 2, 2, 0)
        "#,
    )
    .bind(detail_commit.id)
    .execute(&pool)
    .await
    .expect("commit status should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let (anonymous_status, anonymous_body) = get_json(
        app.clone(),
        &format!("{base}/commits/{}", detail_commit.oid),
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = get_json(
        app.clone(),
        &format!("{base}/commits/{}", detail_commit.oid),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");

    let abbreviated = &detail_commit.oid[..12];
    let (status, body) = get_json(
        app.clone(),
        &format!("{base}/commits/{abbreviated}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["repository"]["name"], repository.name);
    assert_eq!(body["commit"]["oid"], detail_commit.oid);
    assert_eq!(body["commit"]["subject"], "Render commit detail");
    assert_eq!(
        body["commit"]["body"],
        "Shows summary metadata before the diff slice."
    );
    assert_eq!(body["parents"][0]["oid"], base_commit.oid);
    assert_eq!(body["branches"][0]["name"], "main");
    assert_eq!(body["status"]["conclusion"], "success");
    assert_eq!(body["verification"]["verified"], false);
    assert_eq!(body["diffPlaceholder"]["state"], "ready");
    assert_eq!(body["diffSummary"]["totalFiles"], 2);
    assert_eq!(body["diffSummary"]["additions"], 2);
    assert_eq!(body["diffSummary"]["deletions"], 1);
    assert_eq!(body["fileTree"][0]["path"], "docs/commit-detail.md");
    assert_eq!(body["files"][0]["status"], "added");
    assert_eq!(body["files"][0]["hunks"][0]["lines"][0]["kind"], "added");
    assert_eq!(body["files"][1]["path"], "src/main.rs");
    assert_eq!(body["files"][1]["hunks"][0]["lines"][1]["kind"], "removed");
    assert_eq!(body["files"][1]["hunks"][0]["lines"][2]["kind"], "added");
    assert_eq!(
        body["files"][1]["rawHref"],
        format!(
            "/{}/{}/raw/{}/src/main.rs",
            repository.owner_login, repository.name, detail_commit.oid
        )
    );
    assert_eq!(
        body["commit"]["browseHref"],
        format!(
            "/{}/{}/tree/{}",
            repository.owner_login, repository.name, detail_commit.oid
        )
    );
    assert!(!body.to_string().to_lowercase().contains("token"));

    let context_uri = format!(
        "{base}/commits/{}/context?path=src/main.rs&hunkId=diff-src-main-rs-hunk-1&contextLines=80",
        detail_commit.oid
    );
    let (context_status, context_body) =
        get_json(app.clone(), &context_uri, Some(&owner_cookie)).await;
    assert_eq!(context_status, StatusCode::OK, "body: {context_body}");
    assert_eq!(context_body["path"], "src/main.rs");
    assert_eq!(context_body["hunkId"], "diff-src-main-rs-hunk-1");
    assert_eq!(context_body["expanded"], true);
    assert_eq!(context_body["lines"][1]["kind"], "removed");
    assert_eq!(
        context_body["lines"][2]["content"],
        "    println!(\"new\");"
    );

    let (invalid_context_status, invalid_context_body) = get_json(
        app.clone(),
        &format!(
            "{base}/commits/{}/context?path=src/main.rs&hunkId=missing-hunk",
            detail_commit.oid
        ),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_context_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_context_body["error"]["code"], "validation_failed");
    assert!(!invalid_context_body
        .to_string()
        .contains("test-session-secret"));

    let visit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_commit_recent_visits WHERE repository_id = $1 AND user_id = $2 AND ref_name = $3",
    )
    .bind(repository.id)
    .bind(owner.id)
    .bind(&detail_commit.oid)
    .fetch_one(&pool)
    .await
    .expect("visit count should load");
    assert_eq!(visit_count, 1);

    let (missing_status, missing_body) = get_json(
        app,
        &format!("{base}/commits/not-a-real-sha"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "not_found");
    assert!(!missing_body.to_string().contains(&detail_commit.oid));
}
