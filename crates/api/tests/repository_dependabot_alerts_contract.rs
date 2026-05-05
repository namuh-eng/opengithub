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
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, replace_repository_snapshot,
            CreateCommit, CreateRepository, RepositoryOwner, RepositorySnapshot,
            RepositorySnapshotFile, RepositoryVisibility,
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
    sqlx::query(
        r#"
        ALTER TABLE dependabot_alerts
        ADD COLUMN IF NOT EXISTS security_update_pull_request_id uuid REFERENCES pull_requests(id) ON DELETE SET NULL
        "#,
    )
    .execute(&pool)
    .await
    .expect("dependabot security update column should exist");
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
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
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
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
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
    let package_json =
        format!("{{\n  \"dependencies\": {{\n    \"{package_name}\": \"5.0.0\"\n  }}\n}}\n");
    replace_repository_snapshot(
        &pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: format!("commit-{}", Uuid::new_v4().simple()),
                author_user_id: Some(owner.id),
                committer_user_id: Some(owner.id),
                message: "Seed vulnerable package manifest".to_owned(),
                tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
                parent_oids: Vec::new(),
                committed_at: Utc::now(),
            },
            branch_name: "main".to_owned(),
            files: vec![
                RepositorySnapshotFile {
                    path: "package.json".to_owned(),
                    content: package_json.clone(),
                    oid: format!("blob-{}", Uuid::new_v4().simple()),
                    byte_size: package_json.len() as i64,
                },
                RepositorySnapshotFile {
                    path: "package-lock.json".to_owned(),
                    content: "{}\n".to_owned(),
                    oid: format!("blob-{}", Uuid::new_v4().simple()),
                    byte_size: 3,
                },
            ],
        },
    )
    .await
    .expect("default branch files should seed");
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

    let (reader_patch_status, reader_patch_body) = patch_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&reader_cookie),
        json!({
            "action": "dismiss",
            "dismissalReason": "not_used"
        }),
    )
    .await;
    assert_eq!(reader_patch_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_patch_body["error"]["code"], "forbidden");

    let (invalid_patch_status, invalid_patch_body) = patch_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&owner_cookie),
        json!({
            "action": "dismiss",
            "dismissalReason": "unsupported"
        }),
    )
    .await;
    assert_eq!(invalid_patch_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_patch_body["error"]["code"], "validation_failed");

    let (assign_status, assign_body) = patch_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&owner_cookie),
        json!({
            "action": "assign",
            "assigneeIds": [reader.id.to_string()]
        }),
    )
    .await;
    assert_eq!(assign_status, StatusCode::OK, "{assign_body}");
    assert_eq!(
        assign_body["alert"]["assignees"][0]["id"],
        reader.id.to_string()
    );
    assert_eq!(
        assign_body["timeline"]
            .as_array()
            .expect("timeline")
            .last()
            .expect("event")["eventType"],
        "assigned"
    );

    let (dismiss_status, dismiss_body) = patch_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&owner_cookie),
        json!({
            "action": "dismiss",
            "dismissalReason": "not_used",
            "dismissalComment": "Only a development fixture uses this dependency."
        }),
    )
    .await;
    assert_eq!(dismiss_status, StatusCode::OK, "{dismiss_body}");
    assert_eq!(dismiss_body["alert"]["state"], "dismissed");
    assert_eq!(
        dismiss_body["timeline"]
            .as_array()
            .expect("timeline")
            .last()
            .expect("event")["eventType"],
        "dismissed"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM security_audit_events WHERE actor_user_id = $1 AND event_type = 'repository.dependabot_alert.update' AND target_id::text = $2",
    )
    .bind(owner.id)
    .bind(repository.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("security audit count should read");
    assert!(audit_count >= 2);

    let notification_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications WHERE user_id = $1 AND repository_id = $2 AND subject_type = 'dependabot_alert'",
    )
    .bind(reader.id)
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("notification count should read");
    assert!(notification_count >= 1);

    let (reopen_status, reopen_body) = patch_json(
        app.clone(),
        &format!("{base}/{alert_number}"),
        Some(&owner_cookie),
        json!({ "action": "reopen" }),
    )
    .await;
    assert_eq!(reopen_status, StatusCode::OK, "{reopen_body}");
    assert_eq!(reopen_body["alert"]["state"], "open");

    let (bulk_dismiss_status, bulk_dismiss_body) = post_json(
        app.clone(),
        &format!("{base}/bulk"),
        Some(&owner_cookie),
        json!({
            "action": "dismiss",
            "alertIds": [alert_id.to_string()],
            "dismissalReason": "tolerable_risk",
            "dismissalComment": "Bulk triage accepts this risk for now."
        }),
    )
    .await;
    assert_eq!(bulk_dismiss_status, StatusCode::OK, "{bulk_dismiss_body}");
    assert_eq!(bulk_dismiss_body["updatedCount"], 1);
    assert_eq!(bulk_dismiss_body["results"][0]["state"], "dismissed");

    let (bulk_reopen_status, bulk_reopen_body) = post_json(
        app.clone(),
        &format!("{base}/bulk"),
        Some(&owner_cookie),
        json!({
            "action": "reopen",
            "alertIds": [alert_id.to_string()]
        }),
    )
    .await;
    assert_eq!(bulk_reopen_status, StatusCode::OK, "{bulk_reopen_body}");
    assert_eq!(bulk_reopen_body["updatedCount"], 1);
    assert_eq!(bulk_reopen_body["results"][0]["state"], "open");

    sqlx::query("UPDATE dependabot_alerts SET fixed_version = '6.0.0' WHERE id = $1")
        .bind(alert_id)
        .execute(&pool)
        .await
        .expect("alert fixed version should update");

    let (security_update_status, security_update_body) = post_json(
        app.clone(),
        &format!("{base}/{alert_number}/security-update"),
        Some(&owner_cookie),
        json!({}),
    )
    .await;
    assert_eq!(
        security_update_status,
        StatusCode::CREATED,
        "{security_update_body}"
    );
    assert_eq!(security_update_body["status"], "created");
    assert!(security_update_body["pullRequestHref"]
        .as_str()
        .expect("security update pull href")
        .contains("/pull/"));
    assert!(security_update_body["branch"]
        .as_str()
        .expect("security update branch")
        .starts_with("dependabot/npm/"));

    let linked_pull_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM dependabot_alerts JOIN pull_requests ON pull_requests.id = dependabot_alerts.security_update_pull_request_id WHERE dependabot_alerts.id = $1",
    )
    .bind(alert_id)
    .fetch_one(&pool)
    .await
    .expect("linked pull request count should read");
    assert_eq!(linked_pull_count, 1);

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
