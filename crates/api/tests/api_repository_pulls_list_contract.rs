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
        pulls::{
            create_pull_request, update_pull_request_state, CreatePullRequest, PullRequestState,
            UpdatePullRequestState,
        },
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

#[tokio::test]
async fn pull_list_contract_returns_screen_ready_rows_counts_and_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping pull list contract scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "pull-list-owner").await;
    let reviewer = create_user(&pool, "pull-list-reviewer").await;
    let outsider = create_user(&pool, "pull-list-outsider").await;
    let repo_name = format!("pulls-contract-{}", Uuid::new_v4().simple());
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
        VALUES ($1, 'Review queue', 'Pull list milestone', $2)
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
            title: "Linked issue for pull list".to_owned(),
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
    let open_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Fix pull list filters".to_owned(),
            body: Some("Closes linked issue and exercises review metadata.".to_owned()),
            head_ref: "feature/pulls-list".to_owned(),
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
    .expect("open pull should create");
    sqlx::query("UPDATE issues SET milestone_id = $2 WHERE id = $1")
        .bind(open_pr.issue.id)
        .bind(milestone_id)
        .execute(&pool)
        .await
        .expect("pull issue should get milestone");
    sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
        .bind(open_pr.issue.id)
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
    .bind(open_pr.issue.id)
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
    .bind(open_pr.issue.id)
    .bind(linked_issue.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("linked issue reference should create");
    sqlx::query(
        r#"
        INSERT INTO comments (repository_id, pull_request_id, author_user_id, body)
        VALUES ($1, $2, $3, 'Looks ready'), ($1, $2, $3, 'Needs final pass')
        "#,
    )
    .bind(repository.id)
    .bind(open_pr.pull_request.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pull comments should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_reviews (pull_request_id, reviewer_user_id, state, body)
        VALUES ($1, $2, 'approved', 'Ready')
        "#,
    )
    .bind(open_pr.pull_request.id)
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
    .bind(open_pr.pull_request.id)
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
    .bind(open_pr.pull_request.id)
    .execute(&pool)
    .await
    .expect("check summary should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_task_progress (pull_request_id, completed_count, total_count)
        VALUES ($1, 3, 5)
        "#,
    )
    .bind(open_pr.pull_request.id)
    .execute(&pool)
    .await
    .expect("task progress should create");

    let closed_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Closed pull hidden by default".to_owned(),
            body: None,
            head_ref: "feature/closed".to_owned(),
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
    .expect("closed pull should create");
    update_pull_request_state(
        &pool,
        closed_pr.pull_request.id,
        UpdatePullRequestState {
            actor_user_id: owner.id,
            state: PullRequestState::Closed,
            merge_commit_id: None,
            method: None,
        },
    )
    .await
    .expect("pull should close");
    let merged_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Merged pull counted separately".to_owned(),
            body: None,
            head_ref: "feature/merged".to_owned(),
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
    .expect("merged pull should create");
    sqlx::query(
        r#"
        INSERT INTO pull_request_files (pull_request_id, path, status, additions, deletions, byte_size)
        VALUES ($1, 'src/merged.rs', 'added', 12, 0, 256)
        "#,
    )
    .bind(merged_pr.pull_request.id)
    .execute(&pool)
    .await
    .expect("merged pull should have a diff snapshot");
    update_pull_request_state(
        &pool,
        merged_pr.pull_request.id,
        UpdatePullRequestState {
            actor_user_id: owner.id,
            state: PullRequestState::Merged,
            merge_commit_id: None,
            method: None,
        },
    )
    .await
    .expect("pull should merge");

    let private_repo_name = format!("private-pulls-{}", Uuid::new_v4().simple());
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
    let _private_pr = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: private_repository.id,
            actor_user_id: owner.id,
            title: "Private pull hidden".to_owned(),
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
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reviewer_cookie = cookie_header(&pool, &config, &reviewer).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let default_uri = format!("/api/repos/{}/{}/pulls", owner.email, repo_name);
    let (default_status, default_body) = send_json(app.clone(), &default_uri, None).await;
    assert_eq!(default_status, StatusCode::OK);
    assert_eq!(default_body["total"], 1);
    assert_eq!(default_body["openCount"], 1);
    assert_eq!(default_body["closedCount"], 1);
    assert_eq!(default_body["mergedCount"], 1);
    assert_eq!(default_body["filters"]["query"], "is:pr is:open");

    let uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20filters&author={}&labels=bug&milestone=Review%20queue&assignee={}&review=approved&checks=success&page=0&pageSize=1000",
        owner.email, repo_name, owner.email, reviewer.email
    );

    let (status, body) = send_json(app.clone(), &uri, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 100);
    assert_eq!(body["total"], 1);
    assert_eq!(body["openCount"], 1);
    assert_eq!(body["closedCount"], 0);
    assert_eq!(body["mergedCount"], 0);
    assert_eq!(body["counts"]["open"], 1);
    assert_eq!(body["counts"]["closed"], 0);
    assert_eq!(body["counts"]["merged"], 0);
    assert_eq!(body["filters"]["state"], "open");
    assert_eq!(body["filters"]["author"], owner.email);
    assert_eq!(body["filters"]["labels"][0], "bug");
    assert_eq!(body["filters"]["milestone"], "Review queue");
    assert_eq!(body["filters"]["noMilestone"], false);
    assert_eq!(body["filters"]["assignee"], reviewer.email);
    assert_eq!(body["filters"]["noAssignee"], false);
    assert_eq!(body["filters"]["project"], Value::Null);
    assert_eq!(body["filters"]["review"], "approved");
    assert_eq!(body["filters"]["checks"], "success");
    assert_eq!(body["filters"]["sort"], "updated-desc");
    assert!(body["filterOptions"]["labels"]
        .as_array()
        .expect("label options should be an array")
        .iter()
        .any(|label| label["name"] == "bug"));
    assert!(body["filterOptions"]["users"]
        .as_array()
        .expect("user options should be an array")
        .iter()
        .any(|user| user["login"] == reviewer.email));
    assert!(body["filterOptions"]["milestones"]
        .as_array()
        .expect("milestone options should be an array")
        .iter()
        .any(|milestone| milestone["title"] == "Review queue"));
    assert!(body["filterOptions"]["projects"]
        .as_array()
        .expect("project options should be an array")
        .iter()
        .any(|project| {
            project["name"] == "No repository projects"
                && project["disabledReason"]
                    == "Project filters will activate when project links exist."
        }));
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["repository"]["defaultBranch"], "main");
    assert_eq!(body["viewerPermission"], "read");
    assert_eq!(body["preferences"]["dismissedContributorBanner"], false);

    let no_assignee_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20no%3Aassignee&noAssignee=true",
        owner.email, repo_name
    );
    let (no_assignee_status, no_assignee_body) =
        send_json(app.clone(), &no_assignee_uri, None).await;
    assert_eq!(no_assignee_status, StatusCode::OK);
    assert_eq!(no_assignee_body["filters"]["noAssignee"], true);
    assert_eq!(no_assignee_body["total"], 0);

    let no_milestone_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20state%3Aclosed%20no%3Amilestone&noMilestone=true&state=closed",
        owner.email, repo_name
    );
    let (no_milestone_status, no_milestone_body) =
        send_json(app.clone(), &no_milestone_uri, None).await;
    assert_eq!(no_milestone_status, StatusCode::OK);
    assert_eq!(no_milestone_body["filters"]["noMilestone"], true);
    assert_eq!(no_milestone_body["filters"]["milestone"], Value::Null);
    assert_eq!(no_milestone_body["total"], 1);
    assert_eq!(
        no_milestone_body["items"][0]["number"],
        closed_pr.pull_request.number
    );

    let preference_uri = format!("/api/repos/{}/{}/pulls/preferences", owner.email, repo_name);
    let (anonymous_preference_status, anonymous_preference_body) = patch_json(
        app.clone(),
        &preference_uri,
        None,
        json!({ "dismissedContributorBanner": true }),
    )
    .await;
    assert_eq!(anonymous_preference_status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous_preference_body["error"]["code"],
        "not_authenticated"
    );

    let (preference_status, preference_body) = patch_json(
        app.clone(),
        &preference_uri,
        Some(&owner_cookie),
        json!({ "dismissedContributorBanner": true }),
    )
    .await;
    assert_eq!(preference_status, StatusCode::OK);
    assert_eq!(preference_body["dismissedContributorBanner"], true);
    assert!(preference_body["dismissedContributorBannerAt"].is_string());

    let (dismissed_status, dismissed_body) =
        send_json(app.clone(), &default_uri, Some(&owner_cookie)).await;
    assert_eq!(dismissed_status, StatusCode::OK);
    assert_eq!(
        dismissed_body["preferences"]["dismissedContributorBanner"],
        true
    );

    let (restore_status, restore_body) = patch_json(
        app.clone(),
        &preference_uri,
        Some(&owner_cookie),
        json!({ "dismissedContributorBanner": false }),
    )
    .await;
    assert_eq!(restore_status, StatusCode::OK);
    assert_eq!(restore_body["dismissedContributorBanner"], false);
    assert_eq!(restore_body["dismissedContributorBannerAt"], Value::Null);

    let item = &body["items"][0];
    assert_eq!(item["number"], open_pr.pull_request.number);
    assert_eq!(item["title"], "Fix pull list filters");
    assert_eq!(item["state"], "open");
    assert_eq!(item["author"]["login"], owner.email);
    assert_eq!(item["authorRole"], "owner");
    assert_eq!(item["labels"][0]["name"], "bug");
    assert_eq!(item["milestone"]["title"], "Review queue");
    assert_eq!(item["commentCount"], 2);
    assert_eq!(item["linkedIssues"][0]["number"], linked_issue.number);
    assert_eq!(item["review"]["state"], "approved");
    assert_eq!(item["review"]["required"], true);
    assert_eq!(item["review"]["reviewerCount"], 1);
    assert_eq!(
        item["review"]["requestedReviewers"][0]["login"],
        reviewer.email
    );
    assert_eq!(item["checks"]["status"], "completed");
    assert_eq!(item["checks"]["conclusion"], "success");
    assert_eq!(item["checks"]["totalCount"], 4);
    assert_eq!(item["taskProgress"]["completed"], 3);
    assert_eq!(item["taskProgress"]["total"], 5);
    assert_eq!(item["headRef"], "feature/pulls-list");
    assert_eq!(item["baseRef"], "main");
    assert_eq!(
        item["href"],
        format!(
            "/{}/{}/pull/{}",
            owner.email, repo_name, open_pr.pull_request.number
        )
    );
    assert_eq!(
        item["checksHref"],
        format!(
            "/{}/{}/pull/{}/checks",
            owner.email, repo_name, open_pr.pull_request.number
        )
    );
    assert_eq!(
        item["reviewsHref"],
        format!(
            "/{}/{}/pull/{}#reviews",
            owner.email, repo_name, open_pr.pull_request.number
        )
    );
    assert_eq!(
        item["commentsHref"],
        format!(
            "/{}/{}/pull/{}#comments",
            owner.email, repo_name, open_pr.pull_request.number
        )
    );
    assert_eq!(
        item["linkedIssuesHref"],
        format!(
            "/{}/{}/pull/{}#linked-issues",
            owner.email, repo_name, open_pr.pull_request.number
        )
    );

    let typed_filter_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20state%3Aopen%20author%3A{}%20label%3Abug%20milestone%3A%22Review%20queue%22%20assignee%3A{}%20review%3Aapproved%20checks%3Asuccess%20filters&sort=comments-desc",
        owner.email, repo_name, owner.email, reviewer.email
    );
    let (typed_filter_status, typed_filter_body) =
        send_json(app.clone(), &typed_filter_uri, None).await;
    assert_eq!(typed_filter_status, StatusCode::OK);
    assert_eq!(typed_filter_body["total"], 1);
    assert_eq!(typed_filter_body["filters"]["author"], owner.email);
    assert_eq!(typed_filter_body["filters"]["labels"][0], "bug");
    assert_eq!(typed_filter_body["filters"]["milestone"], "Review queue");
    assert_eq!(typed_filter_body["filters"]["noMilestone"], false);
    assert_eq!(typed_filter_body["filters"]["assignee"], reviewer.email);
    assert_eq!(typed_filter_body["filters"]["review"], "approved");
    assert_eq!(typed_filter_body["filters"]["checks"], "success");
    assert_eq!(typed_filter_body["filters"]["sort"], "comments-desc");
    assert_eq!(
        typed_filter_body["items"][0]["number"],
        open_pr.pull_request.number
    );

    let sort_competitor = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Rocket sort best match candidate".to_owned(),
            body: Some("Rocket search text for best match ordering.".to_owned()),
            head_ref: "feature/rocket-sort".to_owned(),
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
    .expect("sort competitor pull should create");
    sqlx::query(
        r#"
        INSERT INTO comments (repository_id, pull_request_id, author_user_id, body)
        VALUES ($1, $2, $3, 'Sort comment A'), ($1, $2, $3, 'Sort comment B'), ($1, $2, $3, 'Sort comment C')
        "#,
    )
    .bind(repository.id)
    .bind(sort_competitor.pull_request.id)
    .bind(reviewer.id)
    .execute(&pool)
    .await
    .expect("sort comments should create");
    sqlx::query(
        r#"
        INSERT INTO reactions (repository_id, pull_request_id, user_id, content)
        VALUES
            ($1, $2, $3, 'rocket'),
            ($1, $2, $4, 'rocket'),
            ($1, $2, $3, 'heart'),
            ($1, $5, $3, 'thumbs_up')
        "#,
    )
    .bind(repository.id)
    .bind(sort_competitor.pull_request.id)
    .bind(owner.id)
    .bind(reviewer.id)
    .bind(open_pr.pull_request.id)
    .execute(&pool)
    .await
    .expect("sort reactions should create");
    sqlx::query(
        r#"
        UPDATE pull_requests
        SET created_at = CASE id
                WHEN $1 THEN now() - interval '3 days'
                WHEN $2 THEN now() - interval '1 day'
                ELSE created_at
            END,
            updated_at = CASE id
                WHEN $1 THEN now() - interval '1 day'
                WHEN $2 THEN now() - interval '3 days'
                ELSE updated_at
            END
        WHERE id IN ($1, $2)
        "#,
    )
    .bind(open_pr.pull_request.id)
    .bind(sort_competitor.pull_request.id)
    .execute(&pool)
    .await
    .expect("sort timestamps should update");

    let comments_sort_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&sort=comments-desc",
        owner.email, repo_name
    );
    let (comments_sort_status, comments_sort_body) =
        send_json(app.clone(), &comments_sort_uri, None).await;
    assert_eq!(comments_sort_status, StatusCode::OK);
    assert_eq!(comments_sort_body["filters"]["sort"], "comments-desc");
    assert_eq!(
        comments_sort_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let least_updated_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&sort=least-recently-updated",
        owner.email, repo_name
    );
    let (least_updated_status, least_updated_body) =
        send_json(app.clone(), &least_updated_uri, None).await;
    assert_eq!(least_updated_status, StatusCode::OK);
    assert_eq!(least_updated_body["filters"]["sort"], "updated-asc");
    assert_eq!(
        least_updated_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let reaction_sort_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&sort=reactions-rocket-desc",
        owner.email, repo_name
    );
    let (reaction_sort_status, reaction_sort_body) =
        send_json(app.clone(), &reaction_sort_uri, None).await;
    assert_eq!(reaction_sort_status, StatusCode::OK);
    assert_eq!(
        reaction_sort_body["filters"]["sort"],
        "reactions-rocket-desc"
    );
    assert_eq!(
        reaction_sort_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let advanced_matrix_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20no%3Aassignee%20no%3Amilestone%20review%3Anone&noAssignee=true&noMilestone=true&review=none&sort=reactions-desc&page=1&pageSize=1",
        owner.email, repo_name
    );
    let (advanced_matrix_status, advanced_matrix_body) =
        send_json(app.clone(), &advanced_matrix_uri, None).await;
    assert_eq!(advanced_matrix_status, StatusCode::OK);
    assert_eq!(advanced_matrix_body["filters"]["noAssignee"], true);
    assert_eq!(advanced_matrix_body["filters"]["noMilestone"], true);
    assert_eq!(advanced_matrix_body["filters"]["review"], "none");
    assert_eq!(advanced_matrix_body["filters"]["sort"], "reactions-desc");
    assert_eq!(advanced_matrix_body["page"], 1);
    assert_eq!(advanced_matrix_body["pageSize"], 1);
    assert_eq!(advanced_matrix_body["total"], 1);
    assert_eq!(
        advanced_matrix_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let best_match_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20Rocket&sort=best-match",
        owner.email, repo_name
    );
    let (best_match_status, best_match_body) = send_json(app.clone(), &best_match_uri, None).await;
    assert_eq!(best_match_status, StatusCode::OK);
    assert_eq!(best_match_body["filters"]["sort"], "best-match");
    assert_eq!(
        best_match_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let best_match_without_text_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&sort=best-match",
        owner.email, repo_name
    );
    let (best_match_without_text_status, best_match_without_text_body) =
        send_json(app.clone(), &best_match_without_text_uri, None).await;
    assert_eq!(
        best_match_without_text_status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        best_match_without_text_body["details"]["reason"],
        "invalid issue filter: best match sort requires a search term"
    );

    let invalid_order_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&sort=updated&order=sideways",
        owner.email, repo_name
    );
    let (invalid_order_status, invalid_order_body) =
        send_json(app.clone(), &invalid_order_uri, None).await;
    assert_eq!(invalid_order_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_order_body["error"]["code"], "validation_failed");
    assert_eq!(
        invalid_order_body["details"]["reason"],
        "invalid issue filter: order must be asc or desc"
    );

    let review_required_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20review%3Arequired",
        owner.email, repo_name
    );
    let (review_required_status, review_required_body) =
        send_json(app.clone(), &review_required_uri, None).await;
    assert_eq!(review_required_status, StatusCode::OK);
    assert_eq!(review_required_body["filters"]["review"], "required");
    assert_eq!(review_required_body["total"], 1);

    let no_reviews_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20review%3Anone",
        owner.email, repo_name
    );
    let (no_reviews_status, no_reviews_body) = send_json(app.clone(), &no_reviews_uri, None).await;
    assert_eq!(no_reviews_status, StatusCode::OK);
    assert_eq!(no_reviews_body["filters"]["review"], "none");
    assert_eq!(no_reviews_body["total"], 1);
    assert_eq!(
        no_reviews_body["items"][0]["number"],
        sort_competitor.pull_request.number
    );

    let reviewed_by_me_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20review%3Areviewed_by_me",
        owner.email, repo_name
    );
    let (reviewed_by_me_status, reviewed_by_me_body) =
        send_json(app.clone(), &reviewed_by_me_uri, Some(&reviewer_cookie)).await;
    assert_eq!(reviewed_by_me_status, StatusCode::OK);
    assert_eq!(reviewed_by_me_body["filters"]["review"], "reviewed_by_me");
    assert_eq!(reviewed_by_me_body["total"], 1);

    let requested_me_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20review-requested%3A%40me",
        owner.email, repo_name
    );
    let (requested_me_status, requested_me_body) =
        send_json(app.clone(), &requested_me_uri, Some(&reviewer_cookie)).await;
    assert_eq!(requested_me_status, StatusCode::OK);
    assert_eq!(requested_me_body["filters"]["review"], "review_requested");
    assert_eq!(requested_me_body["total"], 1);

    let team_requested_uri = format!(
        "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&review=team_review_requested",
        owner.email, repo_name
    );
    let (team_requested_status, team_requested_body) =
        send_json(app.clone(), &team_requested_uri, Some(&reviewer_cookie)).await;
    assert_eq!(team_requested_status, StatusCode::OK);
    assert_eq!(
        team_requested_body["filters"]["review"],
        "team_review_requested"
    );
    assert_eq!(team_requested_body["total"], 1);

    let (anonymous_viewer_filter_status, anonymous_viewer_filter_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&review=review_requested",
            owner.email, repo_name
        ),
        None,
    )
    .await;
    assert_eq!(
        anonymous_viewer_filter_status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        anonymous_viewer_filter_body["details"]["reason"],
        "invalid issue filter: viewer-relative review filters require a signed-in session"
    );

    let (invalid_filter_status, invalid_filter_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen&review=stale",
            owner.email, repo_name
        ),
        None,
    )
    .await;
    assert_eq!(invalid_filter_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_filter_body["error"]["code"], "validation_failed");

    let (invalid_project_status, invalid_project_body) = send_json(
        app.clone(),
        &format!(
            "/api/repos/{}/{}/pulls?q=is%3Apr%20is%3Aopen%20project%3Aroadmap",
            owner.email, repo_name
        ),
        None,
    )
    .await;
    assert_eq!(invalid_project_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_project_body["error"]["code"], "validation_failed");
    assert_eq!(
        invalid_project_body["details"]["reason"],
        "invalid issue filter: project filters are not available until repository project links are modeled"
    );

    let private_uri = format!(
        "/api/repos/{owner}/{private_repo_name}/pulls",
        owner = owner.email
    );
    let (anonymous_private_status, anonymous_private_body) =
        send_json(app.clone(), &private_uri, None).await;
    assert_eq!(anonymous_private_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_private_body["error"]["code"], "forbidden");
    assert!(
        !anonymous_private_body
            .to_string()
            .contains("Private pull hidden"),
        "private pull titles must not leak in forbidden responses"
    );
    let (outsider_private_status, outsider_private_body) =
        send_json(app.clone(), &private_uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_private_status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_private_body["error"]["code"], "forbidden");
    assert!(
        !outsider_private_body
            .to_string()
            .contains("Private pull hidden"),
        "private pull titles must not leak to non-members"
    );
    let (owner_private_status, owner_private_body) =
        send_json(app, &private_uri, Some(&owner_cookie)).await;
    assert_eq!(owner_private_status, StatusCode::OK);
    assert_eq!(owner_private_body["total"], 1);
}
