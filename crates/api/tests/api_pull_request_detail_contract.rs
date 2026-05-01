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
        issues::{create_issue, ensure_default_labels, CreateComment, CreateIssue},
        pulls::{add_pull_request_comment, create_pull_request, CreatePullRequest},
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
async fn pull_request_detail_contract_returns_screen_ready_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping pull request detail scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "pull-detail-owner").await;
    let reviewer = create_user(&pool, "pull-detail-reviewer").await;
    let outsider = create_user(&pool, "pull-detail-outsider").await;
    let repo_name = format!("pull-detail-{}", Uuid::new_v4().simple());
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
    let labels = ensure_default_labels(&pool, repository.id)
        .await
        .expect("labels should exist");
    let bug = labels
        .iter()
        .find(|label| label.name == "bug")
        .expect("bug label should exist");
    let milestone_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO milestones (repository_id, title, description, created_by_user_id)
        VALUES ($1, 'Review queue', 'Pull detail milestone', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should create");
    let linked_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Linked detail issue".to_owned(),
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
    let pull = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Render pull request detail".to_owned(),
            body: Some("Closes linked issue and **renders** markdown.".to_owned()),
            head_ref: "feature/detail".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: true,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await
    .expect("pull should create");
    sqlx::query("UPDATE issues SET milestone_id = $2 WHERE id = $1")
        .bind(pull.issue.id)
        .bind(milestone_id)
        .execute(&pool)
        .await
        .expect("pull issue should get milestone");
    sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
        .bind(pull.issue.id)
        .bind(bug.id)
        .execute(&pool)
        .await
        .expect("pull issue should get label");
    sqlx::query(
        r#"
        INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(pull.issue.id)
    .bind(reviewer.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pull issue should get assignee");
    sqlx::query(
        r#"
        INSERT INTO issue_cross_references (source_issue_id, target_issue_id, created_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(pull.issue.id)
    .bind(linked_issue.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("linked issue reference should create");
    add_pull_request_comment(
        &pool,
        pull.pull_request.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Looks ready".to_owned(),
        },
    )
    .await
    .expect("owner pull comment should create");
    add_pull_request_comment(
        &pool,
        pull.pull_request.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Needs final pass".to_owned(),
        },
    )
    .await
    .expect("reviewer pull comment should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_reviews (pull_request_id, reviewer_user_id, state, body)
        VALUES ($1, $2, 'approved', 'Ready')
        "#,
    )
    .bind(pull.pull_request.id)
    .bind(reviewer.id)
    .execute(&pool)
    .await
    .expect("review should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_review_requests (
            pull_request_id, requested_user_id, requested_by_user_id
        )
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(pull.pull_request.id)
    .bind(reviewer.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("review request should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_checks_summary (
            pull_request_id, status, conclusion, total_count, completed_count, failed_count
        )
        VALUES ($1, 'completed', 'success', 4, 4, 0)
        "#,
    )
    .bind(pull.pull_request.id)
    .execute(&pool)
    .await
    .expect("check summary should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_files (pull_request_id, path, status, additions, deletions, byte_size)
        VALUES
            ($1, 'src/lib.rs', 'modified', 80, 12, 1024),
            ($1, 'src/routes.rs', 'added', 40, 20, 2048)
        "#,
    )
    .bind(pull.pull_request.id)
    .execute(&pool)
    .await
    .expect("file snapshots should create");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let uri = format!(
        "/api/repos/{}/{}/pulls/{}",
        owner.email, repo_name, pull.pull_request.number
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(anonymous_status, StatusCode::OK);
    assert_eq!(anonymous_body["viewerPermission"], "read");
    assert_eq!(anonymous_body["title"], "Render pull request detail");
    assert_eq!(anonymous_body["isDraft"], true);
    assert!(anonymous_body["bodyHtml"]
        .as_str()
        .expect("body html should be a string")
        .contains("<strong>renders</strong>"));
    assert_eq!(anonymous_body["author"]["login"], owner.email);
    assert_eq!(anonymous_body["labels"][0]["name"], "bug");
    assert_eq!(anonymous_body["milestone"]["title"], "Review queue");
    assert_eq!(anonymous_body["assignees"][0]["login"], reviewer.email);
    assert_eq!(
        anonymous_body["requestedReviewers"][0]["login"],
        reviewer.email
    );
    assert_eq!(anonymous_body["latestReviews"][0]["state"], "approved");
    assert_eq!(
        anonymous_body["linkedIssues"][0]["number"],
        linked_issue.number
    );
    assert_eq!(anonymous_body["checks"]["totalCount"], 4);
    assert_eq!(anonymous_body["mergeability"]["state"], "blocked");
    assert_eq!(anonymous_body["mergeability"]["canMerge"], false);
    assert!(anonymous_body["mergeability"]["blockers"]
        .as_array()
        .expect("merge blockers should be an array")
        .iter()
        .any(|item| item["code"] == "missing_write_permission"));
    assert!(anonymous_body["mergeability"]["blockers"]
        .as_array()
        .expect("merge blockers should be an array")
        .iter()
        .any(|item| item["code"] == "draft"));
    assert_eq!(anonymous_body["stats"]["files"], 2);
    assert_eq!(anonymous_body["stats"]["additions"], 120);
    assert_eq!(anonymous_body["stats"]["deletions"], 32);
    assert_eq!(anonymous_body["stats"]["comments"], 2);
    assert_eq!(
        anonymous_body["filesHref"],
        format!(
            "/{}/{}/pull/{}/files",
            owner.email, repo_name, pull.pull_request.number
        )
    );
    assert_eq!(anonymous_body["subscription"]["subscribed"], false);

    let timeline_uri = format!("{uri}/timeline");
    let (anonymous_timeline_status, anonymous_timeline_body) =
        get_json(app.clone(), &timeline_uri, None).await;
    assert_eq!(
        anonymous_timeline_status,
        StatusCode::OK,
        "timeline response: {anonymous_timeline_body}"
    );
    let timeline_items = anonymous_timeline_body
        .as_array()
        .expect("timeline should be an array");
    assert!(
        timeline_items
            .iter()
            .any(|item| item["eventType"] == "opened"),
        "timeline should include opened event"
    );
    let rendered_comment = timeline_items
        .iter()
        .find(|item| item["comment"].is_object())
        .expect("timeline should include rendered comment");
    assert_eq!(rendered_comment["eventType"], "commented");
    assert!(rendered_comment["comment"]["bodyHtml"]
        .as_str()
        .expect("comment html should be a string")
        .contains("Looks ready"));

    let (owner_status, owner_body) = get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewerPermission"], "owner");
    assert_eq!(owner_body["subscription"]["subscribed"], true);

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::OK);
    assert_eq!(outsider_body["viewerPermission"], "read");

    let (blank_comment_status, blank_comment_body) = post_json(
        app.clone(),
        &format!("{uri}/comments"),
        Some(&owner_cookie),
        json!({ "body": "   " }),
    )
    .await;
    assert_eq!(blank_comment_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(blank_comment_body["error"]["code"], "validation_failed");

    let (comment_status, comment_body) = post_json(
        app.clone(),
        &format!("{uri}/comments"),
        Some(&owner_cookie),
        json!({ "body": "Phase 2 **comment** persists." }),
    )
    .await;
    assert_eq!(comment_status, StatusCode::CREATED);
    assert_eq!(comment_body["eventType"], "commented");
    assert_eq!(comment_body["actor"]["login"], owner.email);
    assert!(comment_body["comment"]["bodyHtml"]
        .as_str()
        .expect("created comment html should be a string")
        .contains("<strong>comment</strong>"));

    let (review_request_status, review_request_body) = patch_json(
        app.clone(),
        &format!("{uri}/review-requests"),
        Some(&owner_cookie),
        json!({ "reviewerUserIds": [outsider.id] }),
    )
    .await;
    assert_eq!(review_request_status, StatusCode::OK);
    assert_eq!(
        review_request_body["requestedReviewers"][0]["login"],
        outsider.email
    );
    assert!(review_request_body["requestedReviewers"]
        .as_array()
        .expect("reviewers should be an array")
        .iter()
        .all(|item| item["login"] != reviewer.email));

    let (metadata_status, metadata_body) = patch_json(
        app.clone(),
        &format!("{uri}/metadata"),
        Some(&owner_cookie),
        json!({
            "labelIds": [bug.id],
            "assigneeUserIds": [outsider.id],
            "milestoneId": null
        }),
    )
    .await;
    assert_eq!(metadata_status, StatusCode::OK);
    assert_eq!(metadata_body["labels"][0]["name"], "bug");
    assert_eq!(metadata_body["assignees"][0]["login"], outsider.email);
    assert_eq!(metadata_body["milestone"], Value::Null);

    let (draft_status, draft_body) = patch_json(
        app.clone(),
        &format!("{uri}/draft"),
        Some(&owner_cookie),
        json!({ "isDraft": false }),
    )
    .await;
    assert_eq!(draft_status, StatusCode::OK);
    assert_eq!(draft_body["isDraft"], false);
    assert_eq!(draft_body["mergeability"]["state"], "ready");
    assert_eq!(draft_body["mergeability"]["canMerge"], true);

    let (close_status, close_body) = patch_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "state": "closed" }),
    )
    .await;
    assert_eq!(close_status, StatusCode::OK);
    assert_eq!(close_body["state"], "closed");
    assert_eq!(close_body["mergeability"]["canReopen"], true);
    assert_eq!(close_body["mergeability"]["canMerge"], false);

    let (reopen_status, reopen_body) = patch_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "state": "open" }),
    )
    .await;
    assert_eq!(reopen_status, StatusCode::OK);
    assert_eq!(reopen_body["state"], "open");
    assert_eq!(reopen_body["mergeability"]["canMerge"], true);

    let (merge_status, merge_body) = post_json(
        app.clone(),
        &format!("{uri}/merge"),
        Some(&owner_cookie),
        json!({ "method": "squash" }),
    )
    .await;
    assert_eq!(merge_status, StatusCode::OK);
    assert_eq!(merge_body["state"], "merged");
    assert_eq!(merge_body["mergeability"]["state"], "merged");
    assert_eq!(merge_body["mergeability"]["canMerge"], false);

    let (merge_again_status, merge_again_body) = post_json(
        app.clone(),
        &format!("{uri}/merge"),
        Some(&owner_cookie),
        json!({ "method": "squash" }),
    )
    .await;
    assert_eq!(merge_again_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(merge_again_body["error"]["code"], "validation_failed");

    let (unsubscribe_status, unsubscribe_body) = patch_json(
        app.clone(),
        &format!("{uri}/subscription"),
        Some(&owner_cookie),
        json!({ "subscribed": false }),
    )
    .await;
    assert_eq!(unsubscribe_status, StatusCode::OK);
    assert_eq!(unsubscribe_body["subscribed"], false);
    assert_eq!(unsubscribe_body["reason"], "ignored");

    let (owner_after_unsubscribe_status, owner_after_unsubscribe_body) =
        get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_after_unsubscribe_status, StatusCode::OK);
    assert_eq!(
        owner_after_unsubscribe_body["subscription"]["subscribed"],
        false
    );

    let (timeline_after_status, timeline_after_body) =
        get_json(app.clone(), &timeline_uri, Some(&owner_cookie)).await;
    assert_eq!(timeline_after_status, StatusCode::OK);
    assert!(
        timeline_after_body
            .as_array()
            .expect("timeline after comment should be an array")
            .iter()
            .any(|item| item["comment"]["body"] == "Phase 2 **comment** persists."),
        "created comment should reload through the timeline"
    );
    assert!(
        timeline_after_body
            .as_array()
            .expect("timeline after interactions should be an array")
            .iter()
            .any(|item| item["eventType"] == "review_requested"),
        "review request changes should append a timeline event"
    );

    let private_repo_name = format!("private-detail-{}", Uuid::new_v4().simple());
    let private_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: private_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let private_pull = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: private_repository.id,
            actor_user_id: owner.id,
            title: "Private pull detail".to_owned(),
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
    .expect("private pull should create");
    let private_uri = format!(
        "/api/repos/{}/{}/pulls/{}",
        owner.email, private_repo_name, private_pull.pull_request.number
    );
    let (anonymous_private_status, anonymous_private_body) =
        get_json(app.clone(), &private_uri, None).await;
    assert_eq!(anonymous_private_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_private_body["error"]["code"], "forbidden");

    let (anonymous_private_timeline_status, anonymous_private_timeline_body) =
        get_json(app.clone(), &format!("{private_uri}/timeline"), None).await;
    assert_eq!(anonymous_private_timeline_status, StatusCode::FORBIDDEN);
    assert_eq!(
        anonymous_private_timeline_body["error"]["code"],
        "forbidden"
    );
}
