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
            create_repository, grant_repository_permission, insert_commit,
            repository_pulse_for_actor_by_owner_name, upsert_git_ref, CreateCommit,
            CreateRepository, RepositoryOwner, RepositoryPulseQuery, RepositoryVisibility,
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
            eprintln!("skipping repository pulse scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_pulse_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_insight_snapshots') IS NOT NULL
               AND to_regclass('public.recent_insight_views') IS NOT NULL
               AND to_regclass('public.commit_file_changes') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_pulse_tables {
            eprintln!("skipping repository pulse scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository pulse scenario with pre-applied schema after migration warning: {error}"
        );
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
async fn repository_pulse_returns_activity_aggregates_privacy_and_cache_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository pulse scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "pulse-owner").await;
    let committer = create_user(&pool, "pulse-committer").await;
    let outsider = create_user(&pool, "pulse-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("pulse-insights-{}", Uuid::new_v4().simple()),
            description: Some("Pulse insights repository".to_owned()),
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
        committer.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("committer permission should grant");

    let base_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("base{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Seed pulse history".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(8),
        },
    )
    .await
    .expect("base commit should insert");
    let pulse_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("pulse{}", Uuid::new_v4().simple()),
            author_user_id: Some(committer.id),
            committer_user_id: Some(committer.id),
            message: "Render Pulse aggregates".to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit.oid.clone()],
            committed_at: Utc::now() - Duration::hours(18),
        },
    )
    .await
    .expect("pulse commit should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/main",
        "branch",
        Some(pulse_commit.id),
    )
    .await
    .expect("main ref should insert");
    sqlx::query(
        r#"
        INSERT INTO commit_file_changes (commit_id, path, status, additions, deletions)
        VALUES
            ($1, 'src/pulse.rs', 'modified', 12, 3),
            ($1, 'docs/pulse.md', 'added', 8, 0)
        "#,
    )
    .bind(pulse_commit.id)
    .execute(&pool)
    .await
    .expect("file changes should insert");

    let pr_issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id, state)
        VALUES ($1, 41, 'Merge Pulse summary', 'PR issue', $2, 'closed')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(committer.id)
    .fetch_one(&pool)
    .await
    .expect("pr issue should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, author_user_id, head_ref, base_ref,
            head_repository_id, base_repository_id, state, merged_by_user_id, merged_at
        )
        VALUES ($1, $2, 41, 'Merge Pulse summary', $3, 'pulse', 'main', $1, $1, 'merged', $4, now() - interval '3 hours')
        "#,
    )
    .bind(repository.id)
    .bind(pr_issue_id)
    .bind(committer.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("merged pull request should insert");
    let issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id, state, closed_by_user_id, closed_at)
        VALUES ($1, 9, 'Close stale Pulse gap', 'Issue body', $2, 'closed', $3, now() - interval '2 hours')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .bind(committer.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    sqlx::query("UPDATE issues SET created_at = now() - interval '22 hours' WHERE id = $1")
        .bind(issue_id)
        .execute(&pool)
        .await
        .expect("issue created_at should update");
    sqlx::query(
        r#"
        INSERT INTO releases (repository_id, tag_name, name, body, author_user_id, published_at)
        VALUES ($1, 'v1.2.3', 'Pulse preview', 'Release body', $2, now() - interval '1 hour')
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("release should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let (anonymous_status, anonymous_body) =
        get_json(app.clone(), &format!("{base}/pulse"), None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = get_json(
        app.clone(),
        &format!("{base}/pulse"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");

    let direct_pulse = repository_pulse_for_actor_by_owner_name(
        &pool,
        owner.id,
        &repository.owner_login,
        &repository.name,
        RepositoryPulseQuery {
            period: Some("24h"),
        },
    )
    .await;
    assert!(direct_pulse.is_ok(), "direct pulse error: {direct_pulse:?}");

    let (status, body) = get_json(
        app.clone(),
        &format!("{base}/pulse?period=24h"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["repository"]["name"], repository.name);
    assert_eq!(body["repository"]["viewerPermission"], "owner");
    assert_eq!(body["period"]["key"], "24h");
    assert_eq!(body["summary"]["commits"], 1);
    assert_eq!(body["summary"]["filesChanged"], 2);
    assert_eq!(body["summary"]["additions"], 20);
    assert_eq!(body["summary"]["deletions"], 3);
    assert_eq!(body["summary"]["authors"], 1);
    assert_eq!(body["summary"]["mergedPullRequests"], 1);
    assert_eq!(body["summary"]["closedIssues"], 1);
    assert_eq!(body["summary"]["newIssues"], 1);
    assert_eq!(body["summary"]["releases"], 1);
    assert_eq!(body["metrics"][0]["key"], "merged_pull_requests");
    assert!(body["metrics"][0]["href"]
        .as_str()
        .expect("metric href")
        .contains("/pulls?state=merged"));
    let new_issues_href = body["metrics"][3]["href"]
        .as_str()
        .expect("new issues metric href");
    assert!(new_issues_href.contains("/issues?state=open"));
    assert!(new_issues_href.contains("sort=created-desc"));
    assert!(!new_issues_href.contains("state=created"));
    assert_eq!(
        body["topCommitters"][0]["login"],
        committer.username.as_deref().expect("username")
    );
    assert_eq!(body["topCommitters"][0]["commits"], 1);
    assert_eq!(body["topCommitters"][0]["additions"], 20);
    assert!(body["topCommitters"][0]["commitsHref"]
        .as_str()
        .expect("commits href")
        .contains("/commits/main"));
    assert_eq!(body["releases"][0]["title"], "Pulse preview");
    assert_eq!(
        body["releases"][0]["href"],
        format!(
            "/{}/{}/releases/tag/v1.2.3",
            repository.owner_login, repository.name
        )
    );
    assert_eq!(body["mergedPullRequests"][0]["number"], 41);
    assert_eq!(body["issueActivity"][0]["number"], 9);
    assert_eq!(body["snapshot"]["stale"], false);
    assert!(!body.to_string().contains("SESSION_SECRET"));

    let snapshot_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_insight_snapshots
        WHERE repository_id = $1 AND period_key = '24h'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("snapshot count should query");
    assert_eq!(snapshot_count, 1);
    let view_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM recent_insight_views
        WHERE repository_id = $1 AND user_id = $2 AND period_key = '24h'
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("view count should query");
    assert_eq!(view_count, 1);

    let (empty_status, empty_body) = get_json(
        app.clone(),
        &format!("{base}/pulse?period=3d"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(empty_body["period"]["key"], "3d");
    assert_eq!(empty_body["summary"]["commits"], 1);

    let (week_status, week_body) = get_json(
        app.clone(),
        &format!("{base}/pulse?period=1w"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(week_status, StatusCode::OK);
    assert_eq!(week_body["period"]["key"], "1w");
    assert_eq!(week_body["summary"]["commits"], 1);

    let (month_status, month_body) = get_json(
        app.clone(),
        &format!("{base}/pulse?period=1m"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(month_status, StatusCode::OK);
    assert_eq!(month_body["period"]["key"], "1m");
    assert_eq!(month_body["summary"]["commits"], 2);

    let (invalid_status, invalid_body) = get_json(
        app,
        &format!("{base}/pulse?period=forever"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert!(!invalid_body.to_string().contains("test-session-secret"));
}
