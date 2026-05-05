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
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
        },
        repository_security::{
            repository_dependabot_alert_detail_for_actor_by_owner_name,
            repository_dependabot_alerts_for_actor_by_owner_name, DependabotAlertsQuery,
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
            eprintln!("skipping dependabot alerts scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_dependabot_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.dependabot_alerts') IS NOT NULL
               AND to_regclass('public.security_alert_events') IS NOT NULL
               AND to_regclass('public.dependency_advisories') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_dependabot_tables {
            eprintln!("skipping dependabot alerts scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing dependabot alerts scenario with pre-applied schema after migration warning: {error}"
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
async fn dependabot_alerts_derive_filter_detail_and_protect_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping dependabot alerts scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "dependabot-owner").await;
    let reader = create_user(&pool, "dependabot-reader").await;
    let outsider = create_user(&pool, "dependabot-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("dependabot-{}", Uuid::new_v4().simple()),
            description: Some("Dependabot alerts repository".to_owned()),
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
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    sqlx::query(
        r#"
        INSERT INTO repository_security_feature_settings (
            repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        VALUES ($1, 'dependabot', 'enabled', 'Dependency alerts are monitored.', 0, 0, '/settings/security_analysis')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("dependabot setting should insert");

    let package_name = format!("ansi-regex-{}", Uuid::new_v4().simple());
    let package_href = format!("https://www.npmjs.com/package/{package_name}");
    let manifest_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO dependency_manifests (
            repository_id, path, ecosystem, lockfile_path, dependency_count
        )
        VALUES ($1, 'package.json', 'npm', 'package-lock.json', 2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("manifest should insert");
    let package_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dependency_packages (ecosystem, name, package_href) VALUES ('npm', $1, $2) RETURNING id",
    )
    .bind(&package_name)
    .bind(&package_href)
    .fetch_one(&pool)
    .await
    .expect("package should insert");
    let dependency_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO repository_dependencies (
            repository_id, manifest_id, package_id, package_version, relationship, license, lockfile_path
        )
        VALUES ($1, $2, $3, '5.0.0', 'direct', 'MIT', 'package-lock.json')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(manifest_id)
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("dependency should insert");
    let advisory_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO dependency_advisories (
            package_id, advisory_identifier, severity, title, advisory_href, published_at
        )
        VALUES ($1, 'GHSA-dependabot-demo', 'high', 'Inefficient regular expression complexity', '/advisories/GHSA-dependabot-demo', now() - interval '1 day')
        RETURNING id
        "#,
    )
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("advisory should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!(
        "/api/repos/{owner_login}/{}/security/dependabot",
        repository.name
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert!(!outsider_body.to_string().contains(&package_name));

    let (reader_status, reader_body) = get_json(app.clone(), &base, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["availability"]["enabled"], true);
    assert_eq!(reader_body["viewer"]["canWrite"], false);
    assert_eq!(reader_body["counts"]["open"], 1);
    assert_eq!(
        reader_body["alerts"][0]["package"]["name"],
        package_name.as_str()
    );
    assert_eq!(reader_body["alerts"][0]["advisory"]["severity"], "high");
    assert_eq!(
        reader_body["alerts"][0]["manifestHref"],
        format!("/{owner_login}/{}/blob/main/package.json", repository.name)
    );
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let alert_number = reader_body["alerts"][0]["number"]
        .as_i64()
        .expect("alert number");
    let alert_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM dependabot_alerts WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository.id)
    .bind(alert_number)
    .fetch_one(&pool)
    .await
    .expect("alert should materialize");
    sqlx::query("INSERT INTO dependabot_alert_assignees (alert_id, user_id) VALUES ($1, $2)")
        .bind(alert_id)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("assignee should insert");
    sqlx::query(
        r#"
        INSERT INTO security_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, 'assigned', 'Assigned repository owner.', '{"redacted": true}'::jsonb)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("timeline event should insert");

    let (detail_status, detail_body) = get_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["alert"]["id"], alert_id.to_string());
    assert_eq!(
        detail_body["advisory"]["identifier"],
        "GHSA-dependabot-demo"
    );
    assert_eq!(
        detail_body["dependency"]["package"]["name"],
        package_name.as_str()
    );
    assert_eq!(detail_body["timeline"][0]["eventType"], "assigned");
    assert_eq!(detail_body["assigneeOptions"][0]["kind"], "user");
    assert_eq!(detail_body["securityUpdate"]["supported"], true);
    assert!(detail_body["securityUpdate"]["href"]
        .as_str()
        .expect("security update href")
        .contains("/security/dependabot/"));

    let package_filter = format!("npm:{package_name}");
    let filtered = repository_dependabot_alerts_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        DependabotAlertsQuery {
            state: Some("open"),
            query: Some("regular expression"),
            package: Some(&package_filter),
            ecosystem: Some("npm"),
            manifest: Some("package.json"),
            scope: Some("production"),
            severity: Some("high"),
            sort: Some("most_important"),
        },
    )
    .await
    .expect("direct alert list should load")
    .expect("repository should exist");
    assert_eq!(filtered.alerts.len(), 1);
    assert_eq!(filtered.packages[0].package.id, package_id);
    assert_eq!(filtered.manifests[0].path, "package.json");

    let direct_detail = repository_dependabot_alert_detail_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        alert_number,
    )
    .await
    .expect("direct alert detail should load")
    .expect("alert should exist");
    assert_eq!(direct_detail.alert.assignees[0].id, owner.id);
    assert_eq!(direct_detail.alert.advisory.id, advisory_id);

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}?severity=urgent"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let disabled_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("dependabot-disabled-{}", Uuid::new_v4().simple()),
            description: Some("Disabled Dependabot repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("disabled repository should create");
    sqlx::query(
        r#"
        INSERT INTO repository_security_feature_settings (
            repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        VALUES ($1, 'dependabot', 'disabled', 'Dependabot alerts are disabled by repository policy.', 0, 0, '/settings/security_analysis')
        "#,
    )
    .bind(disabled_repository.id)
    .execute(&pool)
    .await
    .expect("disabled setting should insert");
    let disabled_base = format!(
        "/api/repos/{owner_login}/{}/security/dependabot",
        disabled_repository.name
    );
    let (disabled_status, disabled_body) =
        get_json(app.clone(), &disabled_base, Some(&owner_cookie)).await;
    assert_eq!(disabled_status, StatusCode::OK, "{disabled_body}");
    assert_eq!(disabled_body["availability"]["enabled"], false);
    assert_eq!(disabled_body["alerts"].as_array().expect("alerts").len(), 0);
    assert!(!disabled_body.to_string().contains("test-session-secret"));

    let dependency_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM dependabot_alerts WHERE repository_dependency_id = $1",
    )
    .bind(dependency_id)
    .fetch_one(&pool)
    .await
    .expect("dependabot alert count should read");
    assert_eq!(dependency_count, 1);
}
