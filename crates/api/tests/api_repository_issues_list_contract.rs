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
        issues::{
            add_issue_comment, create_issue, ensure_default_labels, update_issue_state,
            CreateComment, CreateIssue, IssueState, UpdateIssueState,
        },
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

#[tokio::test]
async fn issue_list_contract_returns_screen_ready_rows_counts_and_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue list contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-list-owner").await;
    let reader = create_user(&pool, "issue-list-reader").await;
    let repo_name = format!("issues-contract-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
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
        VALUES ($1, 'MVP', 'First issue list milestone', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should create");

    let open_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Issue list keeps search filters".to_owned(),
            body: Some("Labels and milestones should survive pagination.".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: Some(milestone_id),
            label_ids: vec![bug.id],
            assignee_user_ids: vec![owner.id],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("open issue should create");
    add_issue_comment(
        &pool,
        open_issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "First reproduction".to_owned(),
        },
    )
    .await
    .expect("first comment should create");
    add_issue_comment(
        &pool,
        open_issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Second reproduction".to_owned(),
        },
    )
    .await
    .expect("second comment should create");

    let closed_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Closed issue hidden by default".to_owned(),
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
    .expect("closed issue should create");
    update_issue_state(
        &pool,
        closed_issue.id,
        UpdateIssueState {
            actor_user_id: owner.id,
            state: IssueState::Closed,
        },
    )
    .await
    .expect("issue should close");

    let linked_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Fix issue list filters".to_owned(),
            body: Some("Closes the filter bug.".to_owned()),
            head_ref: "feature/issues".to_owned(),
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
    .expect("pull request should create");
    sqlx::query(
        r#"
        INSERT INTO issue_cross_references (source_issue_id, target_issue_id, created_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(linked_pr.issue.id)
    .bind(open_issue.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("linked PR reference should create");

    let cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let uri = format!(
        "/api/repos/{}/{}/issues?q=is%3Aissue%20state%3Aopen%20filters&labels=bug&milestone=MVP&assignee={}&page=0&pageSize=1000",
        owner.email,
        repo_name,
        owner.email.replace('@', "%40")
    );

    let (status, body) = send_json(app, &uri, Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 100);
    assert_eq!(body["total"], 1);
    assert_eq!(body["openCount"], 1);
    assert_eq!(body["closedCount"], 0);
    assert_eq!(body["counts"]["open"], 1);
    assert_eq!(body["filters"]["state"], "open");
    assert_eq!(body["filters"]["author"], Value::Null);
    assert_eq!(body["filters"]["excludedAuthor"], Value::Null);
    assert_eq!(body["filters"]["labels"][0], "bug");
    assert_eq!(body["filters"]["excludedLabels"], json!([]));
    assert_eq!(body["filters"]["noLabels"], false);
    assert_eq!(body["filters"]["noMilestone"], false);
    assert_eq!(body["filters"]["noAssignee"], false);
    assert_eq!(body["filters"]["project"], Value::Null);
    assert_eq!(body["filters"]["issueType"], Value::Null);
    assert!(body["filterOptions"]["labels"]
        .as_array()
        .expect("label options should be an array")
        .iter()
        .any(|label| label["name"] == "bug"));
    assert!(body["filterOptions"]["users"]
        .as_array()
        .expect("user options should be an array")
        .iter()
        .any(|user| user["login"] == owner.email));
    assert!(body["filterOptions"]["milestones"]
        .as_array()
        .expect("milestone options should be an array")
        .iter()
        .any(|milestone| milestone["title"] == "MVP"));
    assert_eq!(body["filterOptions"]["projects"], json!([]));
    assert_eq!(body["filterOptions"]["issueTypes"], json!([]));
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["viewerPermission"], "read");
    assert_eq!(body["preferences"]["dismissedContributorBanner"], false);
    assert_eq!(
        body["preferences"]["dismissedContributorBannerAt"],
        Value::Null
    );
    let item = &body["items"][0];
    assert_eq!(item["number"], open_issue.number);
    assert_eq!(item["title"], "Issue list keeps search filters");
    assert_eq!(item["author"]["login"], owner.email);
    assert_eq!(item["labels"][0]["name"], "bug");
    assert_eq!(item["milestone"]["title"], "MVP");
    assert_eq!(item["assignees"][0]["login"], owner.email);
    assert_eq!(item["commentCount"], 2);
    assert_eq!(
        item["linkedPullRequest"]["number"],
        linked_pr.pull_request.number
    );
    assert_eq!(
        item["href"],
        format!(
            "/{}/{}/issues/{}",
            owner.email, repo_name, open_issue.number
        )
    );
}

#[tokio::test]
async fn issue_detail_contract_returns_public_read_model_and_redacts_private_access() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue detail contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-detail-owner").await;
    let repo_name = format!("issue-detail-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: Some("Issue detail repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
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
    let documentation = labels
        .iter()
        .find(|label| label.name == "documentation")
        .expect("documentation label should exist");
    let milestone_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO milestones (repository_id, title, description, created_by_user_id)
        VALUES ($1, 'Phase 1', 'Detail read model', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should create");
    let issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Render issue detail read model".to_owned(),
            body: Some("Tracks `body` rendering and **metadata**.".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: Some(milestone_id),
            label_ids: vec![bug.id],
            assignee_user_ids: vec![owner.id],
            attachments: vec![],
        },
    )
    .await
    .expect("issue should create");
    add_issue_comment(
        &pool,
        issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Participant comment".to_owned(),
        },
    )
    .await
    .expect("comment should create");
    sqlx::query(
        r#"
        INSERT INTO issue_attachments (
            issue_id, uploader_user_id, file_name, byte_size, content_type, storage_status
        )
        VALUES ($1, $2, 'trace.txt', 42, 'text/plain', 'metadata_only')
        "#,
    )
    .bind(issue.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("attachment metadata should create");
    let linked_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Close issue detail gap".to_owned(),
            body: Some("References detail read model.".to_owned()),
            head_ref: "feature/detail".to_owned(),
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
    .expect("pull request should create");
    sqlx::query(
        r#"
        INSERT INTO issue_cross_references (source_issue_id, target_issue_id, created_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(linked_pr.issue.id)
    .bind(issue.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("linked PR reference should create");

    let private_repo_name = format!("private-detail-{}", Uuid::new_v4().simple());
    let private_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: private_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let private_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: private_repository.id,
            actor_user_id: owner.id,
            title: "Private detail should be hidden".to_owned(),
            body: Some("secret body".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![],
            attachments: vec![],
        },
    )
    .await
    .expect("private issue should create");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let uri = format!(
        "/api/repos/{}/{}/issues/{}",
        owner.email, repo_name, issue.number
    );
    let (status, body) = send_json(app.clone(), &uri, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["number"], issue.number);
    assert_eq!(body["title"], "Render issue detail read model");
    assert_eq!(body["state"], "open");
    assert_eq!(body["author"]["login"], owner.email);
    assert_eq!(body["labels"][0]["name"], "bug");
    assert_eq!(body["milestone"]["title"], "Phase 1");
    assert_eq!(body["assignees"][0]["login"], owner.email);
    assert_eq!(body["participants"][0]["login"], owner.email);
    assert_eq!(body["attachments"][0]["fileName"], "trace.txt");
    assert_eq!(body["attachments"][0]["byteSize"], 42);
    assert_eq!(body["commentCount"], 1);
    assert_eq!(
        body["linkedPullRequest"]["number"],
        linked_pr.pull_request.number
    );
    assert_eq!(body["viewerPermission"], "read");
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["subscription"]["subscribed"], false);
    assert_eq!(body["reactions"], json!([]));
    assert!(body["metadataOptions"]["labels"]
        .as_array()
        .expect("label options should be an array")
        .iter()
        .any(|label| label["name"] == "documentation"));
    assert!(body["metadataOptions"]["assignees"]
        .as_array()
        .expect("assignee options should be an array")
        .iter()
        .any(|user| user["login"] == owner.email));
    assert!(body["metadataOptions"]["milestones"]
        .as_array()
        .expect("milestone options should be an array")
        .iter()
        .any(|milestone| milestone["title"] == "Phase 1"));
    assert!(body["bodyHtml"]
        .as_str()
        .expect("body html should be a string")
        .contains("<strong>metadata</strong>"));

    let (anonymous_comment_status, anonymous_comment_body) = post_json(
        app.clone(),
        &format!("{uri}/comments"),
        None,
        json!({ "body": "anonymous write should be rejected" }),
    )
    .await;
    assert_eq!(anonymous_comment_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_comment_body["error"]["code"], "not_authenticated");

    let (anonymous_state_status, anonymous_state_body) =
        patch_json(app.clone(), &uri, None, json!({ "state": "closed" })).await;
    assert_eq!(anonymous_state_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_state_body["error"]["code"], "not_authenticated");

    let (anonymous_metadata_status, anonymous_metadata_body) = patch_json(
        app.clone(),
        &format!("{uri}/metadata"),
        None,
        json!({
            "labelIds": [documentation.id],
            "assigneeUserIds": [],
            "milestoneId": null
        }),
    )
    .await;
    assert_eq!(anonymous_metadata_status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous_metadata_body["error"]["code"],
        "not_authenticated"
    );

    let (subscribe_status, subscribe_body) = patch_json(
        app.clone(),
        &format!("{uri}/subscription"),
        Some(&owner_cookie),
        json!({ "subscribed": true }),
    )
    .await;
    assert_eq!(subscribe_status, StatusCode::OK);
    assert_eq!(subscribe_body["subscribed"], true);
    assert_eq!(subscribe_body["reason"], "subscribed");
    assert_eq!(subscribe_body["customEvents"], json!([]));
    assert_eq!(subscribe_body["canCustomize"], true);

    let (customize_status, customize_body) = patch_json(
        app.clone(),
        &format!("{uri}/subscription"),
        Some(&owner_cookie),
        json!({ "subscribed": true, "customEvents": ["closed", "reopened"] }),
    )
    .await;
    assert_eq!(customize_status, StatusCode::OK);
    assert_eq!(customize_body["subscribed"], true);
    assert_eq!(
        customize_body["customEvents"],
        json!(["closed", "reopened"])
    );

    let (invalid_custom_status, invalid_custom_body) = patch_json(
        app.clone(),
        &format!("{uri}/subscription"),
        Some(&owner_cookie),
        json!({ "subscribed": true, "customEvents": ["assigned"] }),
    )
    .await;
    assert_eq!(invalid_custom_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_custom_body["error"]["code"], "validation_failed");

    let (react_status, react_body) = post_json(
        app.clone(),
        &format!("{uri}/reactions"),
        Some(&owner_cookie),
        json!({ "content": "thumbs_up" }),
    )
    .await;
    assert_eq!(react_status, StatusCode::CREATED);
    assert_eq!(react_body["user_id"], owner.id.to_string());
    assert_eq!(react_body["summaries"][0]["content"], "thumbs_up");
    assert_eq!(react_body["summaries"][0]["count"], 1);
    assert_eq!(react_body["summaries"][0]["viewerReacted"], true);

    let (closed_status, closed_body) = patch_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "state": "closed" }),
    )
    .await;
    assert_eq!(closed_status, StatusCode::OK);
    assert_eq!(closed_body["state"], "closed");
    assert_eq!(closed_body["subscription"]["subscribed"], true);
    assert_eq!(closed_body["reactions"][0]["content"], "thumbs_up");

    let (metadata_status, metadata_body) = patch_json(
        app.clone(),
        &format!("{uri}/metadata"),
        Some(&owner_cookie),
        json!({
            "labelIds": [documentation.id],
            "assigneeUserIds": [],
            "milestoneId": null
        }),
    )
    .await;
    assert_eq!(metadata_status, StatusCode::OK);
    assert_eq!(metadata_body["labels"][0]["name"], "documentation");
    assert_eq!(metadata_body["assignees"], json!([]));
    assert_eq!(metadata_body["milestone"], Value::Null);

    let timeline_uri = format!("{uri}/timeline");
    let (timeline_status, timeline_body) = send_json(app.clone(), &timeline_uri, None).await;
    assert_eq!(timeline_status, StatusCode::OK);
    let timeline_items = timeline_body
        .as_array()
        .expect("timeline should be an array");
    assert!(timeline_items
        .iter()
        .any(|item| item["eventType"] == "opened"));
    assert!(timeline_items
        .iter()
        .any(|item| item["eventType"] == "metadata_changed"));
    let comment_item = timeline_items
        .iter()
        .find(|item| item["eventType"] == "commented")
        .expect("comment timeline item should exist");
    assert_eq!(comment_item["actor"]["login"], owner.email);
    assert!(comment_item["comment"]["bodyHtml"]
        .as_str()
        .expect("comment body html should be a string")
        .contains("Participant comment"));

    let (invalid_comment_status, invalid_comment_body) = post_json(
        app.clone(),
        &format!("{uri}/comments"),
        Some(&owner_cookie),
        json!({ "body": "   " }),
    )
    .await;
    assert_eq!(invalid_comment_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_comment_body["error"]["code"], "validation_failed");

    let (comment_status, comment_body) = post_json(
        app.clone(),
        &format!("{uri}/comments"),
        Some(&owner_cookie),
        json!({ "body": "New **timeline** comment" }),
    )
    .await;
    assert_eq!(comment_status, StatusCode::CREATED);
    assert_eq!(comment_body["eventType"], "commented");
    assert_eq!(comment_body["actor"]["login"], owner.email);
    assert!(comment_body["comment"]["bodyHtml"]
        .as_str()
        .expect("created comment body html should be a string")
        .contains("<strong>timeline</strong>"));

    let (final_timeline_status, final_timeline_body) =
        send_json(app.clone(), &timeline_uri, Some(&owner_cookie)).await;
    assert_eq!(final_timeline_status, StatusCode::OK);
    let final_timeline_items = final_timeline_body
        .as_array()
        .expect("final timeline should be an array");
    let mut previous_timestamp = String::new();
    for item in final_timeline_items {
        let created_at = item["createdAt"]
            .as_str()
            .expect("timeline item should expose createdAt");
        assert!(
            previous_timestamp.is_empty() || previous_timestamp.as_str() <= created_at,
            "timeline items should be ordered oldest-first: {previous_timestamp} then {created_at}"
        );
        previous_timestamp = created_at.to_owned();
    }
    assert!(final_timeline_items.iter().any(|item| {
        item["eventType"] == "commented"
            && item["comment"]["bodyHtml"]
                .as_str()
                .is_some_and(|html| html.contains("<strong>timeline</strong>"))
    }));

    let private_uri = format!(
        "/api/repos/{}/{}/issues/{}",
        owner.email, private_repo_name, private_issue.number
    );
    let (private_status, private_body) = send_json(app, &private_uri, None).await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");
    assert!(!private_body.to_string().contains("secret body"));
}

#[tokio::test]
async fn issue_label_filters_support_include_exclude_and_no_label_queries() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue label filter scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-label-menu-owner").await;
    let repo_name = format!("issue-label-menu-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
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
    let docs = labels
        .iter()
        .find(|label| label.name == "documentation")
        .expect("documentation label should exist");
    let bug_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Bug only issue".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![bug.id],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("bug issue should create");
    let docs_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Documentation issue".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![docs.id],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("documentation issue should create");
    let unlabeled_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Unlabeled issue".to_owned(),
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
    .expect("unlabeled issue should create");

    let cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let owner_path = owner.email.replace('@', "%40");
    let base = format!("/api/repos/{owner_path}/{repo_name}/issues");

    let (include_status, include_body) = send_json(
        app.clone(),
        &format!("{base}?q=is%3Aissue%20state%3Aopen%20label%3Abug"),
        Some(&cookie),
    )
    .await;
    assert_eq!(include_status, StatusCode::OK);
    assert_eq!(include_body["total"], 1);
    assert_eq!(include_body["items"][0]["number"], bug_issue.number);
    assert_eq!(include_body["filters"]["labels"], json!(["bug"]));

    let (exclude_status, exclude_body) = send_json(
        app.clone(),
        &format!("{base}?q=is%3Aissue%20state%3Aopen%20-label%3Abug"),
        Some(&cookie),
    )
    .await;
    assert_eq!(exclude_status, StatusCode::OK);
    assert_eq!(exclude_body["total"], 2);
    assert_eq!(exclude_body["filters"]["excludedLabels"], json!(["bug"]));
    let exclude_numbers = exclude_body["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .map(|item| item["number"].as_i64().expect("number"))
        .collect::<Vec<_>>();
    assert!(exclude_numbers.contains(&docs_issue.number));
    assert!(exclude_numbers.contains(&unlabeled_issue.number));
    assert!(!exclude_numbers.contains(&bug_issue.number));

    let (no_label_status, no_label_body) = send_json(
        app.clone(),
        &format!("{base}?q=is%3Aissue%20state%3Aopen%20no%3Alabel"),
        Some(&cookie),
    )
    .await;
    assert_eq!(no_label_status, StatusCode::OK);
    assert_eq!(no_label_body["total"], 1);
    assert_eq!(no_label_body["items"][0]["number"], unlabeled_issue.number);
    assert_eq!(no_label_body["filters"]["noLabels"], true);

    let (bad_label_status, bad_label_body) = send_json(
        app,
        &format!("{base}?q=is%3Aissue%20state%3Aopen%20label%3A"),
        Some(&cookie),
    )
    .await;
    assert_eq!(bad_label_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_label_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn issue_preferences_persist_contributor_banner_dismissal_per_viewer_repository() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue preferences scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-preferences-owner").await;
    let reader = create_user(&pool, "issue-preferences-reader").await;
    let other_reader = create_user(&pool, "issue-preferences-other-reader").await;
    let repo_name = format!("issue-preferences-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Contributor banner preference".to_owned(),
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
    .expect("issue should create");

    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let other_cookie = cookie_header(&pool, &config, &other_reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let list_uri = format!("/api/repos/{}/{}/issues", owner.email, repo_name);
    let preferences_uri = format!(
        "/api/repos/{}/{}/issues/preferences",
        owner.email, repo_name
    );

    let (anonymous_status, anonymous_body) = patch_json(
        app.clone(),
        &preferences_uri,
        None,
        json!({ "dismissedContributorBanner": true }),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (initial_status, initial_body) =
        send_json(app.clone(), &list_uri, Some(&reader_cookie)).await;
    assert_eq!(initial_status, StatusCode::OK);
    assert_eq!(
        initial_body["preferences"]["dismissedContributorBanner"],
        false
    );

    let (patch_status, patch_body) = patch_json(
        app.clone(),
        &preferences_uri,
        Some(&reader_cookie),
        json!({ "dismissedContributorBanner": true }),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["dismissedContributorBanner"], true);
    assert!(patch_body["dismissedContributorBannerAt"].is_string());

    let (persisted_status, persisted_body) =
        send_json(app.clone(), &list_uri, Some(&reader_cookie)).await;
    assert_eq!(persisted_status, StatusCode::OK);
    assert_eq!(
        persisted_body["preferences"]["dismissedContributorBanner"],
        true
    );
    assert!(persisted_body["preferences"]["dismissedContributorBannerAt"].is_string());

    let (other_status, other_body) = send_json(app, &list_uri, Some(&other_cookie)).await;
    assert_eq!(other_status, StatusCode::OK);
    assert_eq!(
        other_body["preferences"]["dismissedContributorBanner"],
        false
    );
}

#[tokio::test]
async fn private_issue_lists_require_repository_permission_and_redact_errors() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue list private scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-private-owner").await;
    let outsider = create_user(&pool, "issue-private-outsider").await;
    let repo_name = format!("private-issues-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Private issue".to_owned(),
            body: Some("must not leak".to_owned()),
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
    .expect("issue should create");

    let cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app,
        &format!("/api/repos/{}/{}/issues", owner.email, repo_name),
        Some(&cookie),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");
    let serialized = body.to_string();
    assert!(!serialized.contains("Private issue"));
    assert!(!serialized.contains("__Host-session"));
    assert!(!serialized.contains("DATABASE_URL"));
}

#[tokio::test]
async fn anonymous_issue_lists_read_public_repositories_but_not_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping anonymous issue list scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-anonymous-owner").await;
    let public_repo_name = format!("public-issues-{}", Uuid::new_v4().simple());
    let private_repo_name = format!("hidden-issues-{}", Uuid::new_v4().simple());
    let public_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: public_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repository should create");
    let private_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: private_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let public_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: public_repository.id,
            actor_user_id: owner.id,
            title: "Anonymous users can read public issues".to_owned(),
            body: Some("Public repository issue list content.".to_owned()),
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
    .expect("public issue should create");
    create_issue(
        &pool,
        CreateIssue {
            repository_id: private_repository.id,
            actor_user_id: owner.id,
            title: "Private issue should stay hidden".to_owned(),
            body: Some("sensitive private issue body".to_owned()),
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
    .expect("private issue should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let owner_path = owner.email.replace('@', "%40");
    let (public_status, public_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{public_repo_name}/issues"),
        None,
    )
    .await;
    assert_eq!(public_status, StatusCode::OK);
    assert_eq!(public_body["viewerPermission"], "read");
    assert_eq!(
        public_body["preferences"]["dismissedContributorBanner"],
        false
    );
    assert_eq!(public_body["items"][0]["number"], public_issue.number);
    assert_eq!(
        public_body["items"][0]["title"],
        "Anonymous users can read public issues"
    );

    let (private_status, private_body) = send_json(
        app,
        &format!("/api/repos/{owner_path}/{private_repo_name}/issues"),
        None,
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");
    let serialized = private_body.to_string();
    assert!(!serialized.contains("Private issue should stay hidden"));
    assert!(!serialized.contains("sensitive private issue body"));
}

#[tokio::test]
async fn issue_list_filters_round_trip_urls_and_validate_bad_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping issue list filter scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "issue-filter-owner").await;
    let repo_name = format!("issue-filters-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
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
    let enhancement = labels
        .iter()
        .find(|label| label.name == "enhancement")
        .expect("enhancement label should exist");
    let milestone_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO milestones (repository_id, title, description, created_by_user_id)
        VALUES ($1, 'Phase 3', 'Filter milestone', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should create");

    let matched = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Filtered issue smoke target".to_owned(),
            body: Some("plain text search should match this body".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: Some(milestone_id),
            label_ids: vec![bug.id],
            assignee_user_ids: vec![owner.id],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("matched issue should create");
    let other = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Enhancement backlog item".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![enhancement.id],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("other issue should create");
    update_issue_state(
        &pool,
        other.id,
        UpdateIssueState {
            actor_user_id: owner.id,
            state: IssueState::Closed,
        },
    )
    .await
    .expect("other issue should close");
    let quiet_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Quiet ordering target".to_owned(),
            body: Some("Sorting coverage without comments".to_owned()),
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
    .expect("quiet issue should create");
    let busy_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Busy ordering target".to_owned(),
            body: Some("Sorting coverage with discussion".to_owned()),
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
    .expect("busy issue should create");
    add_issue_comment(
        &pool,
        busy_issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "First sort comment".to_owned(),
        },
    )
    .await
    .expect("first sort comment should create");
    add_issue_comment(
        &pool,
        busy_issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Second sort comment".to_owned(),
        },
    )
    .await
    .expect("second sort comment should create");

    let cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let owner_path = owner.email.replace('@', "%40");
    let uri = format!(
        "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20plain%20text%20label%3Abug%20milestone%3A%22Phase%203%22%20assignee%3A%40me&sort=created-asc"
    );

    let (status, body) = send_json(app.clone(), &uri, Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["number"], matched.number);
    assert_eq!(body["filters"]["labels"][0], "bug");
    assert_eq!(body["filters"]["milestone"], "Phase 3");
    assert_eq!(body["filters"]["assignee"], owner.email);
    assert_eq!(body["filters"]["sort"], "created-asc");

    let (author_status, author_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20author%3A%40me"
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(author_status, StatusCode::OK);
    assert_eq!(author_body["total"], 3);
    assert_eq!(author_body["filters"]["author"], owner.email);

    let (exclude_author_status, exclude_author_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20-author%3A%40me"
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(exclude_author_status, StatusCode::OK);
    assert_eq!(exclude_author_body["total"], 0);
    assert_eq!(
        exclude_author_body["filters"]["excludedAuthor"],
        owner.email
    );

    let (no_assignee_status, no_assignee_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20no%3Aassignee"),
        Some(&cookie),
    )
    .await;
    assert_eq!(no_assignee_status, StatusCode::OK);
    assert_eq!(no_assignee_body["total"], 2);
    assert_eq!(no_assignee_body["filters"]["noAssignee"], true);

    let (no_milestone_status, no_milestone_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20no%3Amilestone"),
        Some(&cookie),
    )
    .await;
    assert_eq!(no_milestone_status, StatusCode::OK);
    assert_eq!(no_milestone_body["total"], 2);
    assert_eq!(no_milestone_body["filters"]["noMilestone"], true);

    let (cross_filter_status, cross_filter_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20-label%3Abug%20no%3Aassignee&sort=comments-desc"
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(cross_filter_status, StatusCode::OK);
    assert_eq!(cross_filter_body["total"], 2);
    assert_eq!(
        cross_filter_body["filters"]["excludedLabels"],
        json!(["bug"])
    );
    assert_eq!(cross_filter_body["filters"]["noAssignee"], true);
    assert_eq!(cross_filter_body["filters"]["sort"], "comments-desc");
    assert_eq!(cross_filter_body["items"][0]["number"], busy_issue.number);
    assert!(cross_filter_body["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .all(|item| item["number"] != matched.number));

    let (no_label_status, no_label_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20no%3Alabel&sort=comments-asc"
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(no_label_status, StatusCode::OK);
    assert_eq!(no_label_body["total"], 2);
    assert_eq!(no_label_body["filters"]["noLabels"], true);
    assert_eq!(no_label_body["filters"]["sort"], "comments-asc");
    assert_eq!(no_label_body["items"][0]["number"], quiet_issue.number);

    let (project_status, project_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20project%3ARoadmap"),
        Some(&cookie),
    )
    .await;
    assert_eq!(project_status, StatusCode::OK);
    assert_eq!(project_body["total"], 0);
    assert_eq!(project_body["filters"]["project"], "Roadmap");

    let (comments_desc_status, comments_desc_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?sort=comments-desc"),
        Some(&cookie),
    )
    .await;
    assert_eq!(comments_desc_status, StatusCode::OK);
    assert_eq!(comments_desc_body["filters"]["sort"], "comments-desc");
    assert_eq!(comments_desc_body["items"][0]["number"], busy_issue.number);

    let (comments_asc_status, comments_asc_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?sort=comments&order=asc"),
        Some(&cookie),
    )
    .await;
    assert_eq!(comments_asc_status, StatusCode::OK);
    assert_eq!(comments_asc_body["filters"]["sort"], "comments-asc");
    assert_eq!(comments_asc_body["items"][0]["number"], quiet_issue.number);

    let (best_match_status, best_match_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Aopen%20Busy&sort=best-match"
        ),
        Some(&cookie),
    )
    .await;
    assert_eq!(best_match_status, StatusCode::OK);
    assert_eq!(best_match_body["filters"]["sort"], "best-match");
    assert_eq!(best_match_body["items"][0]["number"], busy_issue.number);

    let (bad_sort_status, bad_sort_body) = send_json(
        app.clone(),
        &format!("/api/repos/{owner_path}/{repo_name}/issues?sort=random"),
        Some(&cookie),
    )
    .await;
    assert_eq!(bad_sort_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_sort_body["error"]["code"], "validation_failed");
    assert_eq!(bad_sort_body["details"]["field"], "q");
    assert!(bad_sort_body["details"]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("sort must be one of"));

    let (bad_state_status, bad_state_body) = send_json(
        app,
        &format!("/api/repos/{owner_path}/{repo_name}/issues?q=is%3Aissue%20state%3Amerged"),
        Some(&cookie),
    )
    .await;
    assert_eq!(bad_state_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_state_body["error"]["code"], "validation_failed");
    assert_eq!(bad_state_body["details"]["field"], "q");
    assert!(bad_state_body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("state filter must be open or closed"));
}
