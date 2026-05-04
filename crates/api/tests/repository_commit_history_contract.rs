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
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, insert_commit, upsert_git_ref,
            CreateCommit, CreateRepository, RepositoryOwner, RepositoryVisibility,
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
                "skipping repository commit history scenario; database connect failed: {error}"
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
            eprintln!("skipping repository commit history scenario; migration failed: {error}");
            return None;
        }
        eprintln!("continuing repository commit history scenario with pre-applied schema after migration warning: {error}");
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
        Some(&format!("https://avatars.opengithub.local/{label}.png")),
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

async fn insert_file(pool: &PgPool, repository_id: Uuid, commit_id: Uuid, path: &str) {
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(path)
    .bind(format!("content for {path}\n"))
    .bind(format!(
        "oid-{}-{}",
        commit_id.simple(),
        path.replace('/', "-")
    ))
    .bind(24_i64)
    .execute(pool)
    .await
    .expect("file should insert");
}

#[tokio::test]
async fn repository_commit_history_returns_grouped_ref_filtered_screen_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository commit history scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "commit-owner").await;
    let teammate = create_user(&pool, "commit-author").await;
    let outsider = create_user(&pool, "commit-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("commit-history-{}", Uuid::new_v4().simple()),
            description: Some("Commit history repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        teammate.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("teammate permission should grant");

    let base_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("base{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Initial commit".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(2),
        },
    )
    .await
    .expect("base commit should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/main",
        "branch",
        Some(base_commit.id),
    )
    .await
    .expect("main branch should upsert");
    let docs_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("docs{}", Uuid::new_v4().simple()),
            author_user_id: Some(teammate.id),
            committer_user_id: Some(teammate.id),
            message: "Document commit history\n\nAdds grouped commits.".to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit.oid.clone()],
            committed_at: Utc::now() - Duration::days(1),
        },
    )
    .await
    .expect("docs commit should insert");
    let app_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("app{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Wire commit history page".to_owned(),
            tree_oid: None,
            parent_oids: vec![docs_commit.oid.clone()],
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("app commit should insert");
    insert_file(&pool, repository.id, docs_commit.id, "docs/history.md").await;
    insert_file(&pool, repository.id, app_commit.id, "src/history.rs").await;
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/history/contract",
        "branch",
        Some(app_commit.id),
    )
    .await
    .expect("history branch should upsert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/tags/history-v1",
        "tag",
        Some(docs_commit.id),
    )
    .await
    .expect("history tag should upsert");

    let issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id)
        VALUES ($1, 41, 'Track commit history', 'Body', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let pull_request_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, author_user_id, head_ref, base_ref,
            head_repository_id, base_repository_id
        )
        VALUES ($1, $2, 41, 'Add commit history', $3, 'history/contract', 'main', $1, $1)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(issue_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("pull request should insert");
    sqlx::query(
        "INSERT INTO pull_request_commits (pull_request_id, commit_id, position) VALUES ($1, $2, 1)",
    )
    .bind(pull_request_id)
    .bind(app_commit.id)
    .execute(&pool)
    .await
    .expect("pull request commit should link");
    sqlx::query(
        r#"
        INSERT INTO repository_commit_status_summaries
            (commit_id, status, conclusion, total_count, completed_count, failed_count)
        VALUES ($1, 'completed', 'success', 3, 3, 0)
        "#,
    )
    .bind(app_commit.id)
    .execute(&pool)
    .await
    .expect("commit status should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let encoded_ref = "history%2Fcontract";
    let (anonymous_status, anonymous_body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref={encoded_ref}"),
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref={encoded_ref}"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");

    let (status, body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref={encoded_ref}&page=0&pageSize=1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["repository"]["name"], repository.name);
    assert_eq!(body["resolvedRef"]["shortName"], "history/contract");
    assert_eq!(body["resolvedRef"]["kind"], "branch");
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 1);
    assert_eq!(body["total"], 3);
    assert_eq!(body["hasNextPage"], true);
    assert_eq!(
        body["groups"][0]["commits"][0]["subject"],
        "Wire commit history page"
    );
    assert_eq!(
        body["groups"][0]["commits"][0]["pullRequests"][0]["number"],
        41
    );
    assert_eq!(
        body["groups"][0]["commits"][0]["status"]["conclusion"],
        "success"
    );
    assert_eq!(
        body["groups"][0]["commits"][0]["verification"]["verified"],
        false
    );
    assert!(body["authorOptions"]
        .as_array()
        .expect("author options")
        .iter()
        .any(|option| option["login"]
            .as_str()
            .expect("login")
            .starts_with("commit-author")));
    assert!(!body.to_string().to_lowercase().contains("token"));

    let author_login = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
        .bind(teammate.id)
        .fetch_one(&pool)
        .await
        .expect("teammate username should exist");
    let (filtered_status, filtered_body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref={encoded_ref}&path=docs&author={author_login}&pageSize=20"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(filtered_status, StatusCode::OK);
    assert_eq!(filtered_body["filters"]["path"], "docs");
    assert_eq!(filtered_body["filters"]["author"], author_login);
    assert_eq!(filtered_body["total"], 1);
    assert_eq!(
        filtered_body["groups"][0]["commits"][0]["subject"],
        "Document commit history"
    );
    assert_eq!(
        filtered_body["groups"][0]["commits"][0]["body"],
        "Adds grouped commits."
    );

    let (tag_status, tag_body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref=history-v1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(tag_status, StatusCode::OK);
    assert_eq!(tag_body["resolvedRef"]["kind"], "tag");
    assert_eq!(tag_body["total"], 2);

    let (missing_ref_status, missing_ref_body) = send_json(
        app.clone(),
        &format!("{base}/commits?ref=missing-secret-ref"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_ref_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_ref_body["error"]["code"], "ref_not_found");
    assert!(!missing_ref_body.to_string().contains(&app_commit.oid));

    let (missing_path_status, missing_path_body) = send_json(
        app,
        &format!("{base}/commits?ref={encoded_ref}&path=does-not-exist"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_path_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_path_body["error"]["code"], "path_not_found");
}
