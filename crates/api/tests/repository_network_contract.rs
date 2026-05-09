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
            repository_forks_for_actor_by_owner_name, repository_network_for_actor_by_owner_name,
            save_repository_fork_defaults_by_owner_name, upsert_git_ref, CreateCommit,
            CreateRepository, RepositoryForksQuery, RepositoryOwner, RepositoryVisibility,
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
            eprintln!("skipping repository network scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_network_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_network_forks') IS NOT NULL
               AND to_regclass('public.saved_fork_filter_defaults') IS NOT NULL
               AND to_regclass('public.repository_insight_snapshots') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_network_tables {
            eprintln!("skipping repository network scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository network scenario with pre-applied schema after migration warning: {error}"
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
    let mut builder = Request::builder()
        .uri(uri)
        .header("x-forwarded-for", format!("198.51.100.{}", 10 + uri.len() % 100));
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

async fn put_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    payload: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(payload.to_string()))
                .expect("request should build"),
        )
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

async fn seed_fork(
    pool: &PgPool,
    source_repository_id: Uuid,
    owner: &User,
    name: &str,
    visibility: RepositoryVisibility,
    pushed_at: chrono::DateTime<Utc>,
) -> opengithub_api::domain::repositories::Repository {
    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{name}-{}", Uuid::new_v4().simple()),
            description: Some(format!("{name} fork")),
            visibility,
            default_branch: Some("release/main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("fork repository should create");
    sqlx::query(
        r#"
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(source_repository_id)
    .bind(repository.id)
    .bind(owner.id)
    .bind(pushed_at)
    .execute(pool)
    .await
    .expect("fork edge should insert");
    let commit = insert_commit(
        pool,
        repository.id,
        CreateCommit {
            oid: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: format!("{name} push"),
            tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
            parent_oids: Vec::new(),
            committed_at: pushed_at,
        },
    )
    .await
    .expect("fork commit should insert");
    upsert_git_ref(
        pool,
        repository.id,
        "release/main",
        "branch",
        Some(commit.id),
    )
    .await
    .expect("fork branch should upsert");
    repository
}

#[tokio::test]
async fn repository_network_and_forks_return_readable_projection_filters_and_defaults() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository network scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "network-owner").await;
    let actor = create_user(&pool, "network-actor").await;
    let private_owner = create_user(&pool, "network-private").await;
    let outsider = create_user(&pool, "network-outsider").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("network-{}", Uuid::new_v4().simple()),
            description: Some("Network source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("release/main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("source repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        actor.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("actor should read source");

    let now = Utc::now();
    let popular = seed_fork(
        &pool,
        repository.id,
        &actor,
        "popular",
        RepositoryVisibility::Public,
        now - Duration::days(1),
    )
    .await;
    let stale = seed_fork(
        &pool,
        repository.id,
        &owner,
        "stale",
        RepositoryVisibility::Public,
        now - Duration::days(18),
    )
    .await;
    let private = seed_fork(
        &pool,
        repository.id,
        &private_owner,
        "hidden",
        RepositoryVisibility::Private,
        now - Duration::hours(3),
    )
    .await;
    let child = seed_fork(
        &pool,
        popular.id,
        &owner,
        "child",
        RepositoryVisibility::Public,
        now - Duration::hours(2),
    )
    .await;
    let _ = child;
    let archived = seed_fork(
        &pool,
        repository.id,
        &owner,
        "archived",
        RepositoryVisibility::Public,
        now - Duration::days(2),
    )
    .await;
    sqlx::query("UPDATE repositories SET is_archived = true, default_branch = 'feature/special' WHERE id = $1")
        .bind(archived.id)
        .execute(&pool)
        .await
        .expect("archived fork should update");

    for _ in 0..3 {
        sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
            .bind(create_user(&pool, "stargazer").await.id)
            .bind(popular.id)
            .execute(&pool)
            .await
            .expect("star should insert");
    }
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(actor.id)
        .bind(stale.id)
        .execute(&pool)
        .await
        .expect("actor star should insert");
    let issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, author_user_id)
        VALUES ($1, 1, 'Open issue', $2)
        RETURNING id
        "#,
    )
    .bind(popular.id)
    .bind(actor.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let pull_issue_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issues (repository_id, number, title, author_user_id)
        VALUES ($1, 2, 'Open pull', $2)
        RETURNING id
        "#,
    )
    .bind(popular.id)
    .bind(actor.id)
    .fetch_one(&pool)
    .await
    .expect("pull issue should insert");
    sqlx::query(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, author_user_id, head_ref, base_ref,
            head_repository_id, base_repository_id
        )
        VALUES ($1, $2, 1, 'Open pull', $3, 'feature', 'release/main', $1, $1)
        "#,
    )
    .bind(popular.id)
    .bind(pull_issue_id)
    .bind(actor.id)
    .execute(&pool)
    .await
    .expect("pull request should insert");
    assert_ne!(issue_id, pull_issue_id);
    sqlx::query(
        r#"
        INSERT INTO saved_fork_filter_defaults (repository_id, user_id, period_key, repository_type, sort_key)
        VALUES ($1, $2, 'all', 'starred', 'recently_pushed')
        "#,
    )
    .bind(repository.id)
    .bind(actor.id)
    .execute(&pool)
    .await
    .expect("saved defaults should insert");

    let network = repository_network_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
    )
    .await
    .expect("network should load")
    .expect("network should exist");
    assert_eq!(network.summary.projected_forks, 3);
    assert_eq!(network.summary.hidden_private_forks, 1);
    assert!(network
        .forks
        .iter()
        .all(|fork| fork.repository_id != private.id));
    assert!(network
        .forks
        .iter()
        .any(|fork| fork.repository_id == archived.id && fork.is_archived));
    let popular_node = network
        .forks
        .iter()
        .find(|fork| fork.repository_id == popular.id)
        .expect("popular fork should be projected");
    assert_eq!(popular_node.stars_count, 3);
    assert_eq!(popular_node.forks_count, 1);
    assert_eq!(popular_node.open_issues_count, 1);
    assert_eq!(popular_node.open_pull_requests_count, 1);
    assert!(popular_node.tree_href.contains("/tree/release%2Fmain"));

    let forks = repository_forks_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryForksQuery {
            period: Some("all"),
            repository_type: Some("starred"),
            sort: Some("recently_pushed"),
        },
    )
    .await
    .expect("forks should load")
    .expect("forks should exist");
    assert_eq!(forks.total, 1);
    assert_eq!(forks.forks[0].node.repository_id, stale.id);
    assert!(forks.defaults.saved);
    assert!(forks.defaults.matches_current);
    assert_eq!(forks.hidden_private_forks, 1);

    let inactive = repository_forks_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryForksQuery {
            period: Some("1w"),
            repository_type: Some("inactive"),
            sort: Some("name"),
        },
    )
    .await
    .expect("inactive forks should load")
    .expect("inactive forks repository should exist");
    assert_eq!(inactive.total, 1);
    assert_eq!(inactive.forks[0].node.repository_id, stale.id);
    assert!(inactive.forks[0].badges.contains(&"inactive".to_owned()));

    let archived_forks = repository_forks_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryForksQuery {
            period: Some("all"),
            repository_type: Some("archived"),
            sort: Some("recently_pushed"),
        },
    )
    .await
    .expect("archived forks should load")
    .expect("archived forks repository should exist");
    assert_eq!(archived_forks.total, 1);
    assert_eq!(archived_forks.forks[0].node.repository_id, archived.id);
    assert!(archived_forks.forks[0]
        .badges
        .contains(&"archived".to_owned()));
    assert!(archived_forks.forks[0]
        .node
        .tree_href
        .contains("/tree/feature%2Fspecial"));

    let owner_forks_without_defaults = repository_forks_for_actor_by_owner_name(
        &pool,
        owner.id,
        &repository.owner_login,
        &repository.name,
        RepositoryForksQuery {
            period: Some("1m"),
            repository_type: Some("all"),
            sort: Some("most_starred"),
        },
    )
    .await
    .expect("forks should load without saved defaults")
    .expect("forks repository should exist");
    assert!(!owner_forks_without_defaults.defaults.saved);
    assert!(owner_forks_without_defaults.defaults.matches_current);

    let empty_public = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("network-empty-{}", Uuid::new_v4().simple()),
            description: Some("Empty network source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("empty public source should create");
    let empty_network = repository_network_for_actor_by_owner_name(
        &pool,
        actor.id,
        &empty_public.owner_login,
        &empty_public.name,
    )
    .await
    .expect("empty network should load")
    .expect("empty network should exist");
    assert_eq!(empty_network.summary.total_readable_forks, 0);
    assert!(empty_network.forks.is_empty());

    let saved = save_repository_fork_defaults_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryForksQuery {
            period: Some("24h"),
            repository_type: Some("active"),
            sort: Some("recently_created"),
        },
    )
    .await
    .expect("fork defaults should save")
    .expect("fork defaults repository should exist");
    assert!(saved.defaults.saved);
    assert!(saved.defaults.matches_current);
    assert_eq!(saved.defaults.period_key, "24h");
    assert_eq!(saved.defaults.repository_type, "active");
    assert_eq!(saved.defaults.sort_key, "recently_created");

    let route = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, body) = get_json(
        route.clone(),
        &format!(
            "/api/repos/{}/{}/network",
            repository.owner_login, repository.name
        ),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["hiddenPrivateForks"], 1);
    assert!(!body.to_string().contains(private.name.as_str()));

    let (status, body) = get_json(
        route.clone(),
        &format!(
            "/api/repos/{}/{}/forks?period=banana",
            repository.owner_login, repository.name
        ),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(!body.to_string().contains("test-session-secret"));

    let (status, body) = put_json(
        route.clone(),
        &format!(
            "/api/repos/{}/{}/forks/defaults",
            repository.owner_login, repository.name
        ),
        Some(&actor_cookie),
        json!({
            "period": "all",
            "repositoryType": "starred",
            "sort": "recently_pushed"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["defaults"]["matchesCurrent"]
        .as_bool()
        .unwrap_or(false));
    assert_eq!(body["defaults"]["periodKey"], "all");

    let (status, body) = put_json(
        route.clone(),
        &format!(
            "/api/repos/{}/{}/forks/defaults",
            repository.owner_login, repository.name
        ),
        Some(&actor_cookie),
        json!({
            "period": "all",
            "repositoryType": "starred",
            "sort": "surprise"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, body) = get_json(
        route.clone(),
        &format!(
            "/api/repos/{}/{}/network",
            repository.owner_login, repository.name
        ),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, body) = get_json(
        route,
        &format!(
            "/api/repos/{}/{}/network",
            repository.owner_login, repository.name
        ),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");

    let projected_rows = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::bigint FROM repository_network_forks WHERE source_repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("projection rows should count");
    assert!(projected_rows >= 2);
}
