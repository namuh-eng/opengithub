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
            repository_contributors_for_actor_by_owner_name, upsert_git_ref, CreateCommit,
            CreateRepository, RepositoryContributorsQuery, RepositoryOwner, RepositoryVisibility,
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
                "skipping repository contributors scenario; database connect failed: {error}"
            );
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_contributors_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_insight_snapshots') IS NOT NULL
               AND to_regclass('public.recent_insight_views') IS NOT NULL
               AND to_regclass('public.commit_file_changes') IS NOT NULL
               AND to_regclass('public.repository_contributors_weekly') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_contributors_tables {
            eprintln!("skipping repository contributors scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository contributors scenario with pre-applied schema after migration warning: {error}"
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

async fn add_file_change(
    pool: &PgPool,
    commit_id: Uuid,
    path: &str,
    additions: i64,
    deletions: i64,
) {
    sqlx::query(
        r#"
        INSERT INTO commit_file_changes (commit_id, path, status, additions, deletions)
        VALUES ($1, $2, 'modified', $3, $4)
        "#,
    )
    .bind(commit_id)
    .bind(path)
    .bind(additions)
    .bind(deletions)
    .execute(pool)
    .await
    .expect("file change should insert");
}

#[tokio::test]
async fn repository_contributors_returns_default_branch_weekly_analytics_privacy_and_cache() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository contributors scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "contributors-owner").await;
    let contributor = create_user(&pool, "contributors-author").await;
    let bot = create_user(&pool, "contributors-bot").await;
    let outsider = create_user(&pool, "contributors-outsider").await;
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("contributors-bot-{}[bot]", Uuid::new_v4().simple()))
        .bind(bot.id)
        .execute(&pool)
        .await
        .expect("bot username should update");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("contributors-{}", Uuid::new_v4().simple()),
            description: Some("Contributor analytics repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("release/main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        contributor.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("contributor permission should grant");

    let base_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("base{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Seed contributor history".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(8),
        },
    )
    .await
    .expect("base commit should insert");
    add_file_change(&pool, base_commit.id, "README.md", 1, 0).await;

    let contributor_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("author{}", Uuid::new_v4().simple()),
            author_user_id: Some(contributor.id),
            committer_user_id: Some(contributor.id),
            message: "Render contributor analytics".to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit.oid.clone()],
            committed_at: Utc::now() - Duration::hours(22),
        },
    )
    .await
    .expect("contributor commit should insert");
    add_file_change(&pool, contributor_commit.id, "src/contributors.rs", 14, 2).await;

    let bot_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("bot{}", Uuid::new_v4().simple()),
            author_user_id: Some(bot.id),
            committer_user_id: Some(bot.id),
            message: "Automate contributors cache".to_owned(),
            tree_oid: None,
            parent_oids: vec![contributor_commit.oid.clone()],
            committed_at: Utc::now() - Duration::hours(12),
        },
    )
    .await
    .expect("bot commit should insert");
    add_file_change(&pool, bot_commit.id, "src/cache.rs", 3, 1).await;

    let merge_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("merge{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Merge branch for contributors".to_owned(),
            tree_oid: None,
            parent_oids: vec![bot_commit.oid.clone(), contributor_commit.oid.clone()],
            committed_at: Utc::now() - Duration::hours(6),
        },
    )
    .await
    .expect("merge commit should insert");
    add_file_change(&pool, merge_commit.id, "src/merge.rs", 99, 99).await;

    let detached_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("detached{}", Uuid::new_v4().simple()),
            author_user_id: None,
            committer_user_id: None,
            message: "Import unmatched contributor".to_owned(),
            tree_oid: None,
            parent_oids: vec![merge_commit.oid.clone()],
            committed_at: Utc::now() - Duration::hours(2),
        },
    )
    .await
    .expect("detached commit should insert");
    add_file_change(&pool, detached_commit.id, "src/import.rs", 5, 4).await;

    let side_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("side{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Side branch only".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::hours(1),
        },
    )
    .await
    .expect("side commit should insert");
    add_file_change(&pool, side_commit.id, "side.rs", 50, 50).await;
    let empty_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("empty{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Empty default branch commit".to_owned(),
            tree_oid: None,
            parent_oids: vec![detached_commit.oid.clone()],
            committed_at: Utc::now() - Duration::minutes(30),
        },
    )
    .await
    .expect("empty commit should insert");

    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/release/main",
        "branch",
        Some(empty_commit.id),
    )
    .await
    .expect("default ref should insert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/side",
        "branch",
        Some(side_commit.id),
    )
    .await
    .expect("side ref should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let (anonymous_status, anonymous_body) =
        get_json(app.clone(), &format!("{base}/graphs/contributors"), None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = get_json(
        app.clone(),
        &format!("{base}/graphs/contributors"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");

    let direct = repository_contributors_for_actor_by_owner_name(
        &pool,
        owner.id,
        &repository.owner_login,
        &repository.name,
        RepositoryContributorsQuery {
            period: Some("24h"),
            start: None,
            end: None,
        },
    )
    .await;
    assert!(direct.is_ok(), "direct contributors error: {direct:?}");

    let (status, body) = get_json(
        app.clone(),
        &format!("{base}/graphs/contributors?period=24h"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["repository"]["defaultBranch"], "release/main");
    assert_eq!(body["repository"]["viewerPermission"], "owner");
    assert_eq!(body["period"]["key"], "24h");
    assert_eq!(body["totals"]["commits"], 3);
    assert_eq!(body["totals"]["authors"], 3);
    assert_eq!(body["totals"]["additions"], 22);
    assert_eq!(body["totals"]["deletions"], 7);
    assert_eq!(body["threshold"]["lineCountsOmitted"], false);
    let weekly_commits: i64 = body["weeks"]
        .as_array()
        .expect("weeks should be an array")
        .iter()
        .map(|week| week["commits"].as_i64().expect("week commits"))
        .sum();
    assert_eq!(weekly_commits, 3);
    assert!(body["weeks"][0]["additions"].is_number());

    let contributors = body["contributors"]
        .as_array()
        .expect("contributors should be an array");
    assert_eq!(contributors.len(), 3);
    assert_eq!(
        contributors[0]["login"],
        contributor.username.as_deref().expect("username")
    );
    assert_eq!(contributors[0]["totalCommits"], 1);
    assert_eq!(contributors[0]["totalAdditions"], 14);
    assert!(contributors[0]["profileHref"]
        .as_str()
        .expect("profile href")
        .starts_with('/'));
    assert!(contributors[0]["commitsHref"]
        .as_str()
        .expect("commits href")
        .contains("/commits/release%2Fmain"));
    assert!(contributors[0]["commitsHref"]
        .as_str()
        .expect("commits href")
        .contains("author="));
    assert!(contributors
        .iter()
        .any(|item| item["isBot"] == true && item["authorStatus"] == "bot"));
    assert!(contributors.iter().any(|item| {
        item["login"] == "Unmatched author" && item["authorStatus"] == "unmatched"
    }));
    assert_eq!(body["snapshot"]["stale"], false);
    assert!(body["snapshot"]["cacheKey"]
        .as_str()
        .expect("cache key")
        .starts_with("contributors:"));
    assert!(!body.to_string().contains("SESSION_SECRET"));
    assert!(!body.to_string().contains("side.rs"));

    let public_empty = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("contributors-empty-{}", Uuid::new_v4().simple()),
            description: Some("Empty public contributor analytics repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("trunk/empty".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("empty repository should create");
    let empty_base = format!(
        "/api/repos/{}/{}",
        public_empty.owner_login, public_empty.name
    );
    let (empty_status, empty_body) = get_json(
        app.clone(),
        &format!("{empty_base}/graphs/contributors?period=1w"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK, "body: {empty_body}");
    assert_eq!(empty_body["repository"]["defaultBranch"], "trunk/empty");
    assert_eq!(empty_body["repository"]["viewerPermission"], "read");
    assert_eq!(empty_body["totals"]["commits"], 0);
    assert_eq!(empty_body["totals"]["authors"], 0);
    assert_eq!(
        empty_body["weeks"].as_array().expect("empty weeks").len(),
        0
    );
    assert_eq!(
        empty_body["contributors"]
            .as_array()
            .expect("empty contributors")
            .len(),
        0
    );

    let rollup_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_contributors_weekly
        WHERE repository_id = $1 AND period_key = '24h'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("rollup count should query");
    assert_eq!(rollup_count, 3);
    let snapshot_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_insight_snapshots
        WHERE repository_id = $1 AND period_key = '24h' AND cache_key LIKE 'contributors:%'
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

    let (month_status, month_body) = get_json(
        app.clone(),
        &format!("{base}/graphs/contributors?period=1m"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(month_status, StatusCode::OK);
    assert_eq!(month_body["period"]["key"], "1m");
    assert_eq!(month_body["totals"]["commits"], 4);

    let week_start = month_body["weeks"][0]["weekStart"]
        .as_str()
        .expect("week start should be serialized");
    let week_end = month_body["weeks"][0]["weekEnd"]
        .as_str()
        .expect("week end should be serialized");
    let mut ranged_url = Url::parse(&format!("http://localhost{base}/graphs/contributors"))
        .expect("ranged URL should parse");
    ranged_url
        .query_pairs_mut()
        .append_pair("period", "1m")
        .append_pair("start", week_start)
        .append_pair("end", week_end);
    let (range_status, range_body) = get_json(
        app.clone(),
        &format!(
            "{}?{}",
            ranged_url.path(),
            ranged_url.query().expect("range query should exist")
        ),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(range_status, StatusCode::OK, "body: {range_body}");
    assert_eq!(range_body["period"]["key"], "1m");
    assert_eq!(range_body["period"]["startedAt"], week_start);
    assert_eq!(range_body["period"]["endedAt"], week_end);
    assert!(range_body["snapshot"]["cacheKey"]
        .as_str()
        .expect("range cache key")
        .contains("contributors:"));

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}/graphs/contributors?period=forever"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert!(!invalid_body.to_string().contains("test-session-secret"));

    let (invalid_range_status, invalid_range_body) = get_json(
        app,
        &format!("{base}/graphs/contributors?period=1w&start=2026-05-08&end=2026-05-01"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_range_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_range_body["error"]["code"], "validation_failed");
}
