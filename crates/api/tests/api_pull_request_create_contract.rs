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
        issues::{create_issue, ensure_default_labels, CreateIssue},
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder
        .body(Body::from(body.to_string()))
        .expect("request should build");
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

async fn get_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
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

async fn create_repo(pool: &PgPool, owner: &User, name: &str) -> Repository {
    create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: name.to_owned(),
            description: None,
            visibility: RepositoryVisibility::Public,
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
async fn pull_request_create_contract_persists_metadata_snapshots_and_guardrails() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping pull create contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "pull-create-owner").await;
    let reviewer = create_user(&pool, "pull-create-reviewer").await;
    let repo_name = format!("pull-create-{}", Uuid::new_v4().simple());
    let repository = create_repo(&pool, &owner, &repo_name).await;
    let labels = ensure_default_labels(&pool, repository.id)
        .await
        .expect("labels should exist");
    let label = labels
        .iter()
        .find(|label| label.name == "bug")
        .expect("bug");
    let milestone_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO milestones (repository_id, title, created_by_user_id)
        VALUES ($1, 'Create contract', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_templates (repository_id, slug, name, body)
        VALUES ($1, 'default', 'Default', 'Template body closes #1')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("template should create");

    let linked_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Linked bug".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("linked issue should create");
    assert_eq!(linked_issue.number, 1);

    let base_oid = format!("base{}", Uuid::new_v4().simple());
    let head_oid = format!("head{}", Uuid::new_v4().simple());
    let base_commit =
        insert_commit(&pool, &repository, &owner, &base_oid, "Base", Vec::new()).await;
    let head_commit = insert_commit(
        &pool,
        &repository,
        &owner,
        &head_oid,
        "Feature",
        vec![base_oid.clone()],
    )
    .await;
    insert_file(
        &pool,
        &repository,
        base_commit,
        "README.md",
        "base\n",
        "blob-base",
    )
    .await;
    insert_file(
        &pool,
        &repository,
        head_commit,
        "README.md",
        "base\nhead\n",
        "blob-head",
    )
    .await;
    upsert_ref(&pool, &repository, "main", base_commit).await;
    upsert_ref(&pool, &repository, "feature/create", head_commit).await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let uri = format!("/api/repos/{}/{}/pulls", owner.email, repo_name);
    let (status, body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({
            "title": "Create metadata-rich PR",
            "headRef": "feature/create",
            "baseRef": "main",
            "isDraft": true,
            "templateSlug": "default",
            "labelIds": [label.id],
            "milestoneId": milestone_id,
            "assigneeUserIds": [reviewer.id],
            "reviewerUserIds": [reviewer.id]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["pull_request"]["is_draft"], true);
    assert_eq!(body["issue"]["body"], "Template body closes #1");
    assert_eq!(
        body["href"],
        format!(
            "/{}/{}/pull/{}",
            owner.username.as_deref().unwrap_or(&owner.email),
            repo_name,
            body["pull_request"]["number"]
        )
    );
    let pull_id: Uuid = serde_json::from_value(body["pull_request"]["id"].clone()).unwrap();
    let issue_id: Uuid = serde_json::from_value(body["issue"]["id"].clone()).unwrap();

    let snapshot_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pull_request_commits WHERE pull_request_id = $1",
    )
    .bind(pull_id)
    .fetch_one(&pool)
    .await
    .expect("snapshot count");
    assert_eq!(snapshot_count, 1);
    let file_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pull_request_files WHERE pull_request_id = $1",
    )
    .bind(pull_id)
    .fetch_one(&pool)
    .await
    .expect("file count");
    assert_eq!(file_count, 1);
    let review_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pull_request_review_requests WHERE pull_request_id = $1 AND requested_user_id = $2",
    )
    .bind(pull_id)
    .bind(reviewer.id)
    .fetch_one(&pool)
    .await
    .expect("review count");
    assert_eq!(review_count, 1);
    let metadata_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM issue_labels WHERE issue_id = $1 AND label_id = $2",
    )
    .bind(issue_id)
    .bind(label.id)
    .fetch_one(&pool)
    .await
    .expect("label count");
    assert_eq!(metadata_count, 1);
    let cross_reference_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM issue_cross_references WHERE source_issue_id = $1 AND target_issue_id = $2",
    )
    .bind(issue_id)
    .bind(linked_issue.id)
    .fetch_one(&pool)
    .await
    .expect("cross reference count");
    assert_eq!(cross_reference_count, 1);
    let notification_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM notifications WHERE subject_id = $1")
            .bind(pull_id)
            .fetch_one(&pool)
            .await
            .expect("notification count");
    assert_eq!(notification_count, 1);
    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit_events WHERE event_type = 'pull_request.created' AND target_id = $1",
    )
    .bind(pull_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert_eq!(audit_count, 1);

    let (duplicate_status, duplicate_body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({
            "title": "Duplicate PR",
            "headRef": "feature/create",
            "baseRef": "main"
        }),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(duplicate_body["error"]["code"], "validation_failed");

    let (same_ref_status, same_ref_body) = post_json(
        app,
        &uri,
        Some(&owner_cookie),
        json!({
            "title": "Same ref PR",
            "headRef": "main",
            "baseRef": "main"
        }),
    )
    .await;
    assert_eq!(same_ref_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(same_ref_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn pull_request_create_contract_supports_public_fork_heads_with_base_metadata_guardrails() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping fork pull create contract scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "pull-fork-owner").await;
    let fork_owner = create_user(&pool, "pull-fork-contributor").await;
    let repo_name = format!("pull-fork-{}", Uuid::new_v4().simple());
    let repository = create_repo(&pool, &owner, &repo_name).await;
    let fork_repository = create_repo(&pool, &fork_owner, &repo_name).await;
    sqlx::query(
        r#"
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(repository.id)
    .bind(fork_repository.id)
    .bind(fork_owner.id)
    .execute(&pool)
    .await
    .expect("fork relationship should create");

    let labels = ensure_default_labels(&pool, repository.id)
        .await
        .expect("labels should exist");
    let label = labels
        .iter()
        .find(|label| label.name == "bug")
        .expect("bug label");

    let base_oid = format!("base{}", Uuid::new_v4().simple());
    let head_oid = format!("fork{}", Uuid::new_v4().simple());
    let base_commit =
        insert_commit(&pool, &repository, &owner, &base_oid, "Base", Vec::new()).await;
    let fork_head_commit = insert_commit(
        &pool,
        &fork_repository,
        &fork_owner,
        &head_oid,
        "Fork feature",
        vec![base_oid.clone()],
    )
    .await;
    insert_file(
        &pool,
        &repository,
        base_commit,
        "README.md",
        "base\n",
        "base-blob",
    )
    .await;
    insert_file(
        &pool,
        &fork_repository,
        fork_head_commit,
        "README.md",
        "base\nfork\n",
        "fork-blob",
    )
    .await;
    upsert_ref(&pool, &repository, "main", base_commit).await;
    upsert_ref(&pool, &fork_repository, "feature/fork", fork_head_commit).await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let fork_cookie = cookie_header(&pool, &config, &fork_owner).await;
    let compare_uri = format!(
        "/api/repos/{}/{}/compare/main...feature%2Ffork?headOwner={}&headRepo={}",
        owner.email, repo_name, fork_owner.email, repo_name
    );
    let (compare_status, compare_body) =
        get_json(app.clone(), &compare_uri, Some(&fork_cookie)).await;
    assert_eq!(compare_status, StatusCode::OK, "{compare_body}");
    assert_eq!(
        compare_body["head"]["repository"]["id"],
        fork_repository.id.to_string()
    );
    assert_eq!(compare_body["createOptions"]["canCreate"], true);
    assert_eq!(compare_body["createOptions"]["labels"], json!([]));
    assert!(compare_body["createOptions"]["forkRepositories"]
        .as_array()
        .expect("fork options")
        .iter()
        .any(|option| option["id"] == fork_repository.id.to_string()
            && option["isSelectedHead"] == true));

    let uri = format!("/api/repos/{}/{}/pulls", owner.email, repo_name);
    let (metadata_status, metadata_body) = post_json(
        app.clone(),
        &uri,
        Some(&fork_cookie),
        json!({
            "title": "Fork metadata should fail",
            "headRef": "feature/fork",
            "baseRef": "main",
            "headRepositoryId": fork_repository.id,
            "labelIds": [label.id]
        }),
    )
    .await;
    assert_eq!(metadata_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(metadata_body["error"]["code"], "validation_failed");

    let (create_status, create_body) = post_json(
        app.clone(),
        &uri,
        Some(&fork_cookie),
        json!({
            "title": "Fork contribution PR",
            "body": "Compare from a readable fork",
            "headRef": "feature/fork",
            "baseRef": "main",
            "headRepositoryId": fork_repository.id
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{create_body}");
    assert_eq!(
        create_body["pull_request"]["head_repository_id"],
        fork_repository.id.to_string()
    );
    assert_eq!(
        create_body["pull_request"]["base_repository_id"],
        repository.id.to_string()
    );

    let pull_id: Uuid = serde_json::from_value(create_body["pull_request"]["id"].clone()).unwrap();
    let snapshot_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pull_request_files WHERE pull_request_id = $1",
    )
    .bind(pull_id)
    .fetch_one(&pool)
    .await
    .expect("file snapshot count");
    assert_eq!(snapshot_count, 1);

    let duplicate_status = post_json(
        app,
        &uri,
        Some(&fork_cookie),
        json!({
            "title": "Duplicate fork PR",
            "headRef": "feature/fork",
            "baseRef": "main",
            "headRepositoryId": fork_repository.id
        }),
    )
    .await
    .0;
    assert_eq!(duplicate_status, StatusCode::UNPROCESSABLE_ENTITY);
}
