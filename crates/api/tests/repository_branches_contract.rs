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
            eprintln!("skipping repository branches scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_branch_directory_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_branch_directory_recent_visits') IS NOT NULL
               AND to_regclass('public.repository_commit_status_summaries') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_branch_directory_tables {
            eprintln!("skipping repository branches scenario; migration failed: {error}");
            return None;
        }
        eprintln!("continuing repository branches scenario with pre-applied schema after migration warning: {error}");
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

#[tokio::test]
async fn repository_branches_returns_screen_ready_metadata_privacy_and_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository branches scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "branch-owner").await;
    let teammate = create_user(&pool, "branch-author").await;
    let outsider = create_user(&pool, "branch-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("branch-directory-{}", Uuid::new_v4().simple()),
            description: Some("Branch directory repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
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
            message: "Initial branch directory seed".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(5),
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
    .expect("main ref should insert");
    let feature_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("feat{}", Uuid::new_v4().simple()),
            author_user_id: Some(teammate.id),
            committer_user_id: Some(teammate.id),
            message: "Add protected branch metadata\n\nScreen-ready rows.".to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit.oid.clone()],
            committed_at: Utc::now() - Duration::days(1),
        },
    )
    .await
    .expect("feature commit should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/feature/policy",
        "branch",
        Some(feature_commit.id),
    )
    .await
    .expect("feature ref should insert");
    let stale_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("old{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Old release branch".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(140),
        },
    )
    .await
    .expect("stale commit should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/release/old",
        "branch",
        Some(stale_commit.id),
    )
    .await
    .expect("stale ref should insert");

    sqlx::query(
        r#"
        INSERT INTO repository_commit_status_summaries
            (commit_id, status, conclusion, total_count, completed_count, failed_count)
        VALUES ($1, 'completed', 'success', 4, 4, 0)
        "#,
    )
    .bind(feature_commit.id)
    .execute(&pool)
    .await
    .expect("commit status should insert");
    let issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id)
        VALUES ($1, 17, 'Track feature branch', 'Body', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, author_user_id, head_ref, base_ref,
            head_repository_id, base_repository_id, is_draft
        )
        VALUES ($1, $2, 17, 'Feature branch policy', $3, 'feature/policy', 'main', $1, $1, true)
        "#,
    )
    .bind(repository.id)
    .bind(issue_id)
    .bind(teammate.id)
    .execute(&pool)
    .await
    .expect("pull request should insert");
    let rule_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repository_branch_protection_rules (
            repository_id, pattern, description, enforcement, required_approving_review_count,
            requires_up_to_date_branch
        )
        VALUES ($1, 'main', 'Protect main', 'active', 1, true)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("branch rule should insert");
    sqlx::query(
        "INSERT INTO repository_required_status_checks (branch_protection_rule_id, context) VALUES ($1, 'ci/test')",
    )
    .bind(rule_id)
    .execute(&pool)
    .await
    .expect("required check should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_rulesets (repository_id, name, enforcement, patterns, required_status_checks)
        VALUES ($1, 'Feature branches', 'active', ARRAY['feature/*'], ARRAY['security/review'])
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("ruleset should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let (anonymous_status, anonymous_body) =
        send_json(app.clone(), &format!("{base}/branches"), None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = send_json(
        app.clone(),
        &format!("{base}/branches"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");

    let (status, body) = send_json(
        app.clone(),
        &format!("{base}/branches"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repository"]["defaultBranch"], "main");
    assert_eq!(body["tabs"]["default"], 1);
    assert_eq!(body["tabs"]["active"], 1);
    assert_eq!(body["tabs"]["stale"], 1);
    assert_eq!(body["tabs"]["all"], 3);
    assert_eq!(body["defaultBranch"]["name"], "main");
    assert_eq!(body["defaultBranch"]["protection"]["protected"], true);
    assert_eq!(
        body["defaultBranch"]["protection"]["requiredStatusChecks"][0],
        "ci/test"
    );
    assert_eq!(body["branches"][0]["name"], "feature/policy");
    assert_eq!(body["branches"][0]["classification"], "active");
    assert_eq!(body["branches"][0]["checks"]["totalCount"], 4);
    assert_eq!(body["branches"][0]["pullRequest"]["number"], 17);
    assert_eq!(body["branches"][0]["pullRequest"]["draft"], true);
    assert_eq!(body["branches"][0]["protection"]["matchingRulesetCount"], 1);
    assert_eq!(
        body["branches"][0]["protection"]["requiredStatusChecks"][0],
        "security/review"
    );
    assert_eq!(body["branches"][0]["ahead"], 1);
    assert_eq!(body["branches"][0]["behind"], 0);
    assert_eq!(body["branches"][0]["capabilities"]["canDelete"], false);
    assert!(body["branches"][0]["href"]
        .as_str()
        .expect("href")
        .contains("/tree/feature%2Fpolicy"));

    let (activity_status, activity_body) = send_json(
        app.clone(),
        &format!("{base}/branches/activity?branch=feature%2Fpolicy"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(activity_status, StatusCode::OK);
    assert_eq!(activity_body["branch"]["name"], "feature/policy");
    assert_eq!(
        activity_body["branch"]["protection"]["matchingRulesetCount"],
        1
    );
    assert_eq!(
        activity_body["recentCommits"][0]["subject"],
        "Add protected branch metadata"
    );
    assert_eq!(activity_body["recentPullRequests"][0]["number"], 17);
    assert_eq!(
        activity_body["protectionEvents"][0]["name"],
        "Feature branches"
    );
    assert_eq!(
        activity_body["protectionEvents"][0]["requiredStatusChecks"][0],
        "security/review"
    );
    assert!(activity_body["links"]["treeHref"]
        .as_str()
        .expect("tree href")
        .contains("/tree/feature%2Fpolicy"));
    assert!(!activity_body.to_string().contains("test-session-secret"));

    let (missing_activity_status, missing_activity_body) = send_json(
        app.clone(),
        &format!("{base}/branches/activity?branch=missing"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_activity_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_activity_body["error"]["code"], "ref_not_found");
    assert_eq!(
        missing_activity_body["details"]["recoveryHref"],
        format!("/{}/{}/branches", repository.owner_login, repository.name)
    );

    let (private_activity_status, private_activity_body) = send_json(
        app.clone(),
        &format!("{base}/branches/activity?branch=feature%2Fpolicy"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_activity_status, StatusCode::NOT_FOUND);
    assert_eq!(private_activity_body["error"]["code"], "not_found");

    let (stale_status, stale_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=stale"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(stale_status, StatusCode::OK);
    assert_eq!(stale_body["branches"][0]["name"], "release/old");
    assert_eq!(stale_body["branches"][0]["classification"], "stale");

    let (active_status, active_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=active&pageSize=1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(active_status, StatusCode::OK);
    assert_eq!(active_body["total"], 1);
    assert_eq!(active_body["branches"][0]["name"], "feature/policy");
    assert_eq!(active_body["hasNextPage"], false);
    assert_eq!(active_body["hasPreviousPage"], false);
    assert!(active_body["branches"]
        .as_array()
        .expect("branches")
        .iter()
        .all(|branch| branch["classification"] == "active"));

    let (all_page_status, all_page_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=all&pageSize=1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(all_page_status, StatusCode::OK);
    assert_eq!(all_page_body["total"], 3);
    assert_eq!(all_page_body["branches"][0]["name"], "main");
    assert_eq!(all_page_body["hasNextPage"], true);
    assert_eq!(all_page_body["hasPreviousPage"], false);

    let (all_second_status, all_second_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=all&page=2&pageSize=1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(all_second_status, StatusCode::OK);
    assert_eq!(all_second_body["branches"][0]["name"], "feature/policy");
    assert_eq!(all_second_body["hasNextPage"], true);
    assert_eq!(all_second_body["hasPreviousPage"], true);

    let (search_status, search_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=all&q=POLICY&pageSize=1"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(search_body["total"], 1);
    assert_eq!(search_body["branches"][0]["name"], "feature/policy");
    assert_eq!(search_body["pageSize"], 1);

    let (empty_status, empty_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=stale&q=POLICY"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(empty_body["total"], 0);
    assert_eq!(
        empty_body["emptyState"]["title"],
        "No branches matched this search"
    );
    assert_eq!(
        empty_body["emptyState"]["resetHref"],
        format!("/{}/{}/branches", repository.owner_login, repository.name)
    );

    let visit_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_branch_directory_recent_visits
        WHERE repository_id = $1 AND user_id = $2 AND tab = 'all' AND query = 'POLICY'
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("branch directory telemetry should query");
    assert_eq!(visit_count, 1);

    let (invalid_status, invalid_body) = send_json(
        app.clone(),
        &format!("{base}/branches?tab=secrets"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert!(!invalid_body.to_string().contains("SESSION_SECRET"));

    let long_query = "x".repeat(121);
    let (long_query_status, long_query_body) = send_json(
        app,
        &format!("{base}/branches?q={long_query}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(long_query_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(long_query_body["error"]["code"], "validation_failed");
    assert!(!long_query_body.to_string().contains("test-session-secret"));
}
