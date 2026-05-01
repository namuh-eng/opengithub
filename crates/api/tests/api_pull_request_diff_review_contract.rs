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
        pulls::{create_pull_request, CreatePullRequest},
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

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(Method::PATCH)
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

#[tokio::test]
async fn pull_request_diff_review_contract_returns_files_hunks_and_viewer_state() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping pull request diff review scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "diff-owner").await;
    let reviewer = create_user(&pool, "diff-reviewer").await;
    let repo_name = format!("diff-review-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let pull = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Review diff contract".to_owned(),
            body: Some("Adds diff review data.".to_owned()),
            head_ref: "feature/diff-review".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await
    .expect("pull should create");
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, message, parent_oids)
        VALUES ($1, 'abcdef1234567890', $2, 'Add diff review screen', ARRAY[]::text[])
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("commit should insert");
    sqlx::query(
        "INSERT INTO pull_request_commits (pull_request_id, commit_id, position) VALUES ($1, $2, 0)",
    )
    .bind(pull.pull_request.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("commit snapshot should insert");
    let file_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO pull_request_files (
            pull_request_id, path, status, additions, deletions, blob_oid, byte_size
        )
        VALUES ($1, 'src/review.rs', 'modified', 2, 1, 'blob-new', 320)
        RETURNING id
        "#,
    )
    .bind(pull.pull_request.id)
    .fetch_one(&pool)
    .await
    .expect("file snapshot should insert");
    let second_file_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO pull_request_files (
            pull_request_id, path, status, additions, deletions, blob_oid, byte_size
        )
        VALUES ($1, 'docs/review.md', 'added', 5, 0, 'docs-blob', 180)
        RETURNING id
        "#,
    )
    .bind(pull.pull_request.id)
    .fetch_one(&pool)
    .await
    .expect("second file snapshot should insert");
    let hunk_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO pull_request_file_hunks (
            pull_request_file_id, old_start, old_lines, new_start, new_lines, header, display_order
        )
        VALUES ($1, 10, 3, 10, 4, '@@ -10,3 +10,4 @@ fn review()', 0)
        RETURNING id
        "#,
    )
    .bind(file_id)
    .fetch_one(&pool)
    .await
    .expect("hunk should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_request_hunk_lines (hunk_id, kind, old_line, new_line, content, position)
        VALUES
            ($1, 'context', 10, 10, 'fn review() {', 0),
            ($1, 'removed', 11, NULL, '    old_review();', 1),
            ($1, 'added', NULL, 11, '    new_review();', 2),
            ($1, 'added', NULL, 12, '    record_state();', 3)
        "#,
    )
    .bind(hunk_id)
    .execute(&pool)
    .await
    .expect("hunk lines should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_request_viewed_files (pull_request_file_id, user_id, version_key, viewed)
        VALUES
            ($1, $2, 'blob-new:2:1', true),
            ($3, $2, 'stale-version', true)
        "#,
    )
    .bind(file_id)
    .bind(owner.id)
    .bind(second_file_id)
    .execute(&pool)
    .await
    .expect("viewed rows should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_request_review_drafts (
            pull_request_id, author_user_id, summary_body, review_state
        )
        VALUES ($1, $2, 'Pending summary', 'commented')
        "#,
    )
    .bind(pull.pull_request.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("draft should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_request_review_comments (
            pull_request_id, pull_request_file_id, author_user_id, body, body_html,
            path, side, new_line, position, state
        )
        VALUES
            ($1, $2, $3, 'Published note', '<p>Published note</p>', 'src/review.rs', 'right', 11, 2, 'published'),
            ($1, $2, $4, 'Pending note', '<p>Pending note</p>', 'src/review.rs', 'right', 12, 3, 'pending')
        "#,
    )
    .bind(pull.pull_request.id)
    .bind(file_id)
    .bind(reviewer.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("review comments should insert");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let uri = format!(
        "/api/repos/{}/{}/pulls/{}/files?view=split&whitespace=hide",
        owner.email, repo_name, pull.pull_request.number
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(
        anonymous_status,
        StatusCode::OK,
        "anonymous response: {anonymous_body}"
    );
    assert_eq!(
        anonymous_body["pullRequest"]["title"],
        "Review diff contract"
    );
    assert_eq!(anonymous_body["pullRequest"]["viewerPermission"], "read");
    assert_eq!(anonymous_body["settings"]["view"], "split");
    assert_eq!(anonymous_body["settings"]["whitespace"], "hide");
    assert_eq!(anonymous_body["totalFiles"], 2);
    assert_eq!(anonymous_body["fileTree"].as_array().unwrap().len(), 2);
    assert_eq!(anonymous_body["files"][0]["path"], "docs/review.md");
    assert_eq!(anonymous_body["files"][1]["path"], "src/review.rs");
    assert_eq!(anonymous_body["files"][1]["language"], "Rust");
    assert_eq!(
        anonymous_body["files"][1]["hunks"][0]["lines"][2]["content"],
        "    new_review();"
    );
    assert_eq!(
        anonymous_body["files"][1]["comments"][0]["body"],
        "Published note"
    );
    assert_eq!(
        anonymous_body["files"][1]["comments"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(anonymous_body["files"][1]["viewed"], false);
    assert_eq!(anonymous_body["pendingReview"]["commentCount"], 0);
    assert_eq!(anonymous_body["commits"][0]["shortOid"], "abcdef1");

    let (owner_status, owner_body) = get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["pullRequest"]["viewerPermission"], "owner");
    assert_eq!(owner_body["files"][0]["viewed"], false);
    assert_eq!(owner_body["files"][1]["viewed"], true);
    assert_eq!(owner_body["pendingReview"]["commentCount"], 1);
    assert_eq!(
        owner_body["pendingReview"]["summaryBody"],
        "Pending summary"
    );

    let viewed_uri = format!(
        "/api/repos/{}/{}/pulls/{}/files/viewed",
        owner.email, repo_name, pull.pull_request.number
    );
    let (anonymous_viewed_status, anonymous_viewed_body) = patch_json(
        app.clone(),
        &viewed_uri,
        None,
        json!({
            "fileId": second_file_id,
            "versionKey": "docs-blob:5:0",
            "viewed": true
        }),
    )
    .await;
    assert_eq!(anonymous_viewed_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_viewed_body["error"]["code"], "not_authenticated");

    let (viewed_status, viewed_body) = patch_json(
        app.clone(),
        &viewed_uri,
        Some(&owner_cookie),
        json!({
            "fileId": second_file_id,
            "versionKey": "docs-blob:5:0",
            "viewed": true
        }),
    )
    .await;
    assert_eq!(viewed_status, StatusCode::OK, "viewed body: {viewed_body}");
    assert_eq!(viewed_body["fileId"], second_file_id.to_string());
    assert_eq!(viewed_body["path"], "docs/review.md");
    assert_eq!(viewed_body["viewed"], true);

    let (updated_owner_status, updated_owner_body) =
        get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(updated_owner_status, StatusCode::OK);
    assert_eq!(updated_owner_body["files"][0]["viewed"], true);

    let (stale_status, stale_body) = patch_json(
        app.clone(),
        &viewed_uri,
        Some(&owner_cookie),
        json!({
            "fileId": second_file_id,
            "versionKey": "stale-version",
            "viewed": false
        }),
    )
    .await;
    assert_eq!(stale_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(stale_body["error"]["code"], "validation_failed");

    let filter_uri = format!(
        "/api/repos/{}/{}/pulls/{}/files?filter=src&pageSize=1",
        owner.email, repo_name, pull.pull_request.number
    );
    let (filter_status, filter_body) =
        get_json(app.clone(), &filter_uri, Some(&owner_cookie)).await;
    assert_eq!(filter_status, StatusCode::OK);
    assert_eq!(filter_body["totalFiles"], 1);
    assert_eq!(filter_body["files"][0]["path"], "src/review.rs");
    assert_eq!(filter_body["settings"]["filter"], "src");

    let invalid_uri = format!(
        "/api/repos/{}/{}/pulls/{}/files?view=sideways",
        owner.email, repo_name, pull.pull_request.number
    );
    let (invalid_status, invalid_body) =
        get_json(app.clone(), &invalid_uri, Some(&owner_cookie)).await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn pull_request_diff_review_contract_denies_private_anonymous_reads() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private pull request diff review scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "private-diff-owner").await;
    let repo_name = format!("private-diff-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let pull = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Private diff".to_owned(),
            body: None,
            head_ref: "feature/private".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await
    .expect("pull should create");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let uri = format!(
        "/api/repos/{}/{}/pulls/{}/files",
        owner.email, repo_name, pull.pull_request.number
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(anonymous_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_body["error"]["code"], "forbidden");

    let (owner_status, owner_body) = get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["pullRequest"]["viewerPermission"], "owner");
}
