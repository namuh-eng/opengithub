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
        issues::{create_issue, CreateIssue},
        onboarding::dismiss_dashboard_hint,
        permissions::RepositoryRole,
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
    let mut user = upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert");
    user.username = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(pool)
        .await
        .expect("user username should reload");
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

#[tokio::test]
async fn dashboard_summary_rejects_anonymous_requests() {
    let app = opengithub_api::build_app_with_config(None, app_config());

    let (status, body) = send_json(app, "/api/dashboard", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "not_authenticated");
}

#[tokio::test]
async fn dashboard_summary_reports_database_unavailable_after_valid_cookie() {
    let config = app_config();
    let cookie = session::set_cookie_header(
        &config,
        "dashboard-summary-without-database",
        Utc::now() + Duration::minutes(5),
    )
    .expect("signed cookie should be created");
    let cookie_value =
        session::cookie_value_from_set_cookie(&cookie).expect("cookie value should exist");
    let app = opengithub_api::build_app_with_config(None, config.clone());

    let (status, body) = send_json(
        app,
        "/api/dashboard",
        Some(&format!("{}={cookie_value}", config.session_cookie_name)),
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "database_unavailable");
}

#[tokio::test]
async fn dashboard_summary_returns_empty_state_contract_for_new_user() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard summary scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "dashboard-empty").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);

    let (status, body) = send_json(app, "/api/dashboard", Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["id"], user.id.to_string());
    assert_eq!(body["repositories"]["total"], 0);
    assert_eq!(body["repositories"]["page"], 1);
    assert_eq!(body["repositories"]["pageSize"], 10);
    assert_eq!(body["topRepositories"]["total"], 0);
    assert_eq!(body["topRepositories"]["page"], 1);
    assert_eq!(body["topRepositories"]["pageSize"], 10);
    assert!(body["topRepositories"]["items"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(body["hasRepositories"], false);
    assert_eq!(body["recentActivity"].as_array().unwrap().len(), 0);
    assert_eq!(body["feedEvents"].as_array().unwrap().len(), 0);
    assert_eq!(body["assignedIssues"].as_array().unwrap().len(), 0);
    assert_eq!(body["reviewRequests"].as_array().unwrap().len(), 0);
    assert_eq!(body["dismissedHints"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn dashboard_summary_includes_repositories_and_dismissed_hints() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard summary scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "dashboard-repos").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let first_repo_name = format!("alpha-{}", Uuid::new_v4().simple());
    let second_repo_name = format!("beta-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: first_repo_name.clone(),
            description: Some("First dashboard repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("first repository should create");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: second_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("second repository should create");
    dismiss_dashboard_hint(&pool, user.id, "create-repository")
        .await
        .expect("hint should dismiss");
    let app = opengithub_api::build_app_with_config(Some(pool), config);

    let (status, body) = send_json(app, "/api/dashboard?pageSize=1", Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["hasRepositories"], true);
    assert_eq!(body["repositories"]["total"], 2);
    assert_eq!(body["repositories"]["pageSize"], 1);
    assert_eq!(
        body["repositories"]["items"]
            .as_array()
            .expect("repositories should be an array")
            .len(),
        1
    );
    let repo_name = body["repositories"]["items"][0]["name"]
        .as_str()
        .expect("repository name should be a string");
    assert!(repo_name == first_repo_name || repo_name == second_repo_name);
    assert_eq!(body["dismissedHints"][0]["hintKey"], "create-repository");
}

#[tokio::test]
async fn dashboard_summary_returns_ranked_sidebar_repository_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard summary scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "dashboard-top-repos").await;
    let other_user = create_user(&pool, "dashboard-other").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let alpha_name = format!("alpha-{}", Uuid::new_v4().simple());
    let beta_name = format!("beta-{}", Uuid::new_v4().simple());
    let private_name = format!("private-{}", Uuid::new_v4().simple());
    let inaccessible_name = format!("hidden-{}", Uuid::new_v4().simple());

    let alpha = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: alpha_name.clone(),
            description: Some("Rust service".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("alpha repository should create");
    let beta = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: beta_name.clone(),
            description: Some("TypeScript app".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("beta repository should create");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: private_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("private repository should create");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: other_user.id },
            name: inaccessible_name,
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: other_user.id,
        },
    )
    .await
    .expect("inaccessible repository should create");

    sqlx::query(
        r#"
        UPDATE repositories
        SET updated_at = CASE id
            WHEN $1 THEN '2026-04-30T11:00:00Z'::timestamptz
            WHEN $2 THEN '2026-04-30T09:00:00Z'::timestamptz
            ELSE '2026-04-30T08:00:00Z'::timestamptz
        END
        WHERE id IN ($1, $2)
           OR owner_user_id = $3
        "#,
    )
    .bind(alpha.id)
    .bind(beta.id)
    .bind(user.id)
    .execute(&pool)
    .await
    .expect("repository timestamps should update");

    sqlx::query(
        r#"
        INSERT INTO repository_languages (repository_id, language, color, byte_count)
        VALUES
            ($1, 'Rust', '#dea584', 800),
            ($1, 'Shell', '#89e051', 100),
            ($2, 'JavaScript', '#f1e05a', 100),
            ($2, 'TypeScript', '#3178c6', 900)
        "#,
    )
    .bind(alpha.id)
    .bind(beta.id)
    .execute(&pool)
    .await
    .expect("repository languages should insert");

    sqlx::query(
        r#"
        INSERT INTO recent_repository_visits (user_id, repository_id, visited_at)
        VALUES ($1, $2, '2026-04-30T12:00:00Z'::timestamptz)
        "#,
    )
    .bind(user.id)
    .bind(beta.id)
    .execute(&pool)
    .await
    .expect("recent visit should insert");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(app, "/api/dashboard?pageSize=2", Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["topRepositories"]["total"], 3);
    assert_eq!(body["topRepositories"]["page"], 1);
    assert_eq!(body["topRepositories"]["pageSize"], 2);
    let items = body["topRepositories"]["items"]
        .as_array()
        .expect("top repositories should be an array");
    assert_eq!(items.len(), 2);
    let owner_login = user.username.as_deref().expect("user has username");
    assert_eq!(items[0]["ownerLogin"], owner_login);
    assert_eq!(items[0]["name"], beta_name);
    assert_eq!(items[0]["visibility"], "private");
    assert_eq!(items[0]["primaryLanguage"], "TypeScript");
    assert_eq!(items[0]["primaryLanguageColor"], "#3178c6");
    assert_eq!(items[0]["lastVisitedAt"], "2026-04-30T12:00:00Z");
    assert_eq!(items[0]["href"], format!("/{owner_login}/{beta_name}"));
    assert_eq!(items[1]["name"], alpha_name);
    assert_eq!(items[1]["primaryLanguage"], "Rust");
    assert_eq!(body["topRepositories"]["total"], 3);
}

#[tokio::test]
async fn dashboard_summary_filters_top_repositories_without_leaking_private_repos() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard summary scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "dashboard-filter").await;
    let other_user = create_user(&pool, "dashboard-filter-other").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let visible_name = format!("visible-match-{}", Uuid::new_v4().simple());
    let hidden_name = format!("hidden-match-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: visible_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("visible repository should create");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: other_user.id },
            name: hidden_name,
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: other_user.id,
        },
    )
    .await
    .expect("hidden repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app,
        "/api/dashboard?pageSize=30&repositoryFilter=match",
        Some(&cookie),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["topRepositories"]["total"], 1);
    assert_eq!(body["topRepositories"]["items"][0]["name"], visible_name);
    assert!(!body.to_string().contains("hidden-match"));
}

#[tokio::test]
async fn dashboard_summary_populates_activity_assignments_and_review_requests() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard summary scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let user = create_user(&pool, "dashboard-feed").await;
    let reviewer = create_user(&pool, "dashboard-review-author").await;
    let hidden_user = create_user(&pool, "dashboard-feed-hidden").await;
    let cookie = cookie_header(&pool, &config, &user).await;
    let repo_name = format!("feed-{}", Uuid::new_v4().simple());
    let hidden_repo_name = format!("hidden-feed-{}", Uuid::new_v4().simple());

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user.id },
            name: repo_name.clone(),
            description: Some("Dashboard activity source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: user.id,
        },
    )
    .await
    .expect("repository should create");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: hidden_user.id },
            name: hidden_repo_name.clone(),
            description: Some("Hidden dashboard activity".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: hidden_user.id,
        },
    )
    .await
    .expect("hidden repository should create");
    sqlx::query(
        r#"
        INSERT INTO repository_permissions (repository_id, user_id, role, source)
        VALUES ($1, $2, $3, 'direct')
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(repository.id)
    .bind(reviewer.id)
    .bind(RepositoryRole::Write.as_str())
    .execute(&pool)
    .await
    .expect("review author should receive repository write access");

    sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, committed_at)
        VALUES ($1, 'abcdef1234567890', $2, $2, $3, '2026-04-30T12:00:00Z'::timestamptz)
        "#,
    )
    .bind(repository.id)
    .bind(user.id)
    .bind("Wire dashboard feed\n\nBody")
    .execute(&pool)
    .await
    .expect("commit should insert");

    let assigned_issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: user.id,
            title: "Fix failing setup workflow".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![user.id],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("assigned issue should create");
    let review_request = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: reviewer.id,
            title: "Add dashboard activity feed".to_owned(),
            body: None,
            head_ref: "feed-layout".to_owned(),
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
        UPDATE issues
        SET updated_at = CASE
            WHEN id = $1 THEN '2026-04-30T11:00:00Z'::timestamptz
            WHEN id = $2 THEN '2026-04-30T10:30:00Z'::timestamptz
            ELSE updated_at
        END
        WHERE id IN ($1, $2)
        "#,
    )
    .bind(assigned_issue.id)
    .bind(review_request.issue.id)
    .execute(&pool)
    .await
    .expect("issue timestamps should update");
    sqlx::query(
        "UPDATE pull_requests SET updated_at = '2026-04-30T10:30:00Z'::timestamptz WHERE id = $1",
    )
    .bind(review_request.pull_request.id)
    .execute(&pool)
    .await
    .expect("pull request timestamp should update");

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(app, "/api/dashboard", Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    let recent_activity = body["recentActivity"]
        .as_array()
        .expect("activity should be an array");
    assert!(!recent_activity.is_empty());
    assert!(recent_activity.len() <= 4);
    assert!(recent_activity
        .iter()
        .all(|item| item["kind"] == "issue" || item["kind"] == "pull_request"));
    let owner_login = user.username.as_deref().expect("user has username");
    assert_eq!(
        recent_activity[0]["repositoryName"],
        format!("{owner_login}/{repo_name}")
    );
    assert!(recent_activity.iter().any(|item| {
        item["kind"] == "issue"
            && item["title"] == "Fix failing setup workflow"
            && item["number"] == assigned_issue.number
            && item["state"] == "open"
            && item["href"]
                == format!("/{owner_login}/{repo_name}/issues/{}", assigned_issue.number)
            && item["actorLogin"] == owner_login
    }));
    assert!(recent_activity.iter().any(|item| {
        item["kind"] == "pull_request"
            && item["title"] == "Add dashboard activity feed"
            && item["number"] == review_request.pull_request.number
            && item["state"] == "open"
            && item["href"]
                == format!(
                    "/{owner_login}/{repo_name}/pull/{}",
                    review_request.pull_request.number
                )
            && item["actorLogin"] == reviewer
                .username
                .as_deref()
                .expect("review author has username")
    }));
    assert_eq!(
        body["assignedIssues"][0]["title"],
        "Fix failing setup workflow"
    );
    assert_eq!(body["assignedIssues"][0]["number"], assigned_issue.number);
    assert_eq!(
        body["assignedIssues"][0]["href"],
        format!("/{owner_login}/{repo_name}/issues/{}", assigned_issue.number)
    );
    assert_eq!(
        body["reviewRequests"][0]["title"],
        "Add dashboard activity feed"
    );
    assert_eq!(
        body["reviewRequests"][0]["number"],
        review_request.pull_request.number
    );
    assert_eq!(
        body["reviewRequests"][0]["href"],
        format!(
            "/{owner_login}/{repo_name}/pull/{}",
            review_request.pull_request.number
        )
    );
    assert!(!body.to_string().contains(&hidden_repo_name));
}

#[tokio::test]
async fn dashboard_feed_following_reads_followed_and_watched_activity_without_private_leaks() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard feed scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let viewer = create_user(&pool, "feed-viewer").await;
    let followed_actor = create_user(&pool, "feed-followed").await;
    let watched_actor = create_user(&pool, "feed-watched").await;
    let hidden_actor = create_user(&pool, "feed-hidden").await;
    let cookie = cookie_header(&pool, &config, &viewer).await;
    let followed_repo_name = format!("followed-{}", Uuid::new_v4().simple());
    let watched_repo_name = format!("watched-{}", Uuid::new_v4().simple());
    let hidden_repo_name = format!("hidden-{}", Uuid::new_v4().simple());

    let followed_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: followed_actor.id,
            },
            name: followed_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: followed_actor.id,
        },
    )
    .await
    .expect("followed public repo should create");
    let watched_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: watched_actor.id,
            },
            name: watched_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: watched_actor.id,
        },
    )
    .await
    .expect("watched public repo should create");
    let hidden_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: hidden_actor.id,
            },
            name: hidden_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: hidden_actor.id,
        },
    )
    .await
    .expect("hidden private repo should create");

    sqlx::query(
        r#"
        INSERT INTO user_follows (follower_user_id, followed_user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(viewer.id)
    .bind(followed_actor.id)
    .execute(&pool)
    .await
    .expect("follow should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_watches (user_id, repository_id)
        VALUES ($1, $2), ($1, $3)
        "#,
    )
    .bind(viewer.id)
    .bind(watched_repo.id)
    .bind(hidden_repo.id)
    .execute(&pool)
    .await
    .expect("watches should insert");

    insert_feed_event(
        &pool,
        followed_actor.id,
        followed_repo.id,
        "push",
        "Pushed dashboard feed changes",
        "2026-04-30T12:00:00Z",
    )
    .await;
    insert_feed_event(
        &pool,
        watched_actor.id,
        watched_repo.id,
        "release",
        "Published v1.0.0",
        "2026-04-30T11:00:00Z",
    )
    .await;
    insert_feed_event(
        &pool,
        hidden_actor.id,
        hidden_repo.id,
        "push",
        "Private roadmap update",
        "2026-04-30T13:00:00Z",
    )
    .await;

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(app, "/api/dashboard?feedTab=following", Some(&cookie)).await;

    assert_eq!(status, StatusCode::OK);
    let feed_events = body["feedEvents"]
        .as_array()
        .expect("feed events should be an array");
    assert_eq!(feed_events.len(), 2);
    assert_eq!(feed_events[0]["eventType"], "push");
    assert_eq!(feed_events[0]["title"], "Pushed dashboard feed changes");
    let followed_login = followed_actor
        .username
        .as_deref()
        .expect("followed actor has username");
    assert_eq!(
        feed_events[0]["repositoryName"],
        format!("{followed_login}/{followed_repo_name}")
    );
    assert_eq!(
        feed_events[0]["actionSummary"],
        format!("{followed_login} pushed to {followed_login}/{followed_repo_name}")
    );
    assert_eq!(feed_events[1]["eventType"], "release");
    assert_eq!(feed_events[1]["title"], "Published v1.0.0");
    assert!(!body.to_string().contains(&hidden_repo_name));
    assert!(!body.to_string().contains("Private roadmap update"));
}

#[tokio::test]
async fn dashboard_feed_for_you_filters_recommended_events_by_type() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres dashboard feed scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let viewer = create_user(&pool, "feed-recommend-viewer").await;
    let actor = create_user(&pool, "feed-recommend-actor").await;
    let cookie = cookie_header(&pool, &config, &viewer).await;
    let repo_name = format!("recommended-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: actor.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: actor.id,
        },
    )
    .await
    .expect("recommended repo should create");

    sqlx::query(
        r#"
        INSERT INTO repository_stars (user_id, repository_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(viewer.id)
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("star should insert");
    insert_feed_event(
        &pool,
        actor.id,
        repository.id,
        "fork",
        "Forked the dashboard feed repo",
        "2026-04-30T10:00:00Z",
    )
    .await;
    insert_feed_event(
        &pool,
        actor.id,
        repository.id,
        "release",
        "Published dashboard feed v2",
        "2026-04-30T09:00:00Z",
    )
    .await;

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let (status, body) = send_json(
        app,
        "/api/dashboard?feedTab=for_you&eventType=release,fork&eventType=release",
        Some(&cookie),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let feed_events = body["feedEvents"]
        .as_array()
        .expect("feed events should be an array");
    assert_eq!(feed_events.len(), 2);
    assert_eq!(feed_events[0]["eventType"], "fork");
    assert_eq!(feed_events[1]["eventType"], "release");

    let app = opengithub_api::build_app_with_config(
        Some(database_pool().await.expect("pool should reconnect")),
        app_config(),
    );
    let (status, body) = send_json(
        app,
        "/api/dashboard?feedTab=for_you&eventType=release",
        Some(&cookie),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let feed_events = body["feedEvents"]
        .as_array()
        .expect("feed events should be an array");
    assert_eq!(feed_events.len(), 1);
    assert_eq!(feed_events[0]["eventType"], "release");
    assert_eq!(feed_events[0]["title"], "Published dashboard feed v2");
}

#[tokio::test]
async fn dashboard_feed_rejects_unknown_tabs_and_event_types() {
    let config = app_config();
    let app = opengithub_api::build_app_with_config(None, config.clone());
    let cookie = session::set_cookie_header(
        &config,
        "dashboard-feed-invalid-query",
        Utc::now() + Duration::minutes(5),
    )
    .expect("signed cookie should be created");
    let cookie_value =
        session::cookie_value_from_set_cookie(&cookie).expect("cookie value should exist");
    let cookie_header = format!("{}={cookie_value}", config.session_cookie_name);

    let (status, body) = send_json(
        app.clone(),
        "/api/dashboard?feedTab=popular",
        Some(&cookie_header),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, body) = send_json(
        app,
        "/api/dashboard?eventType=unknown",
        Some(&cookie_header),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

async fn insert_feed_event(
    pool: &PgPool,
    actor_user_id: Uuid,
    repository_id: Uuid,
    event_type: &str,
    title: &str,
    occurred_at: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            occurred_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository_id)
    .bind(event_type)
    .bind(title)
    .bind(Some(format!("Excerpt for {title}")))
    .bind(format!("/feed/{event_type}/{}", Uuid::new_v4()))
    .bind(occurred_at)
    .execute(pool)
    .await
    .expect("feed event should insert");
}
