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
            create_repository, grant_repository_permission, replace_repository_snapshot,
            CreateCommit, CreateRepository, RepositoryOwner, RepositorySnapshot,
            RepositorySnapshotFile, RepositoryVisibility,
        },
        repository_security::{
            repository_secret_scanning_alert_detail_for_actor_by_owner_name,
            repository_secret_scanning_alerts_for_actor_by_owner_name,
            update_repository_secret_scanning_alert_for_actor_by_owner_name,
            SecretScanningAlertMutation, SecretScanningAlertsQuery,
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
            eprintln!("skipping secret scanning alerts scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_secret_scanning_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.secret_scanning_alerts') IS NOT NULL
               AND to_regclass('public.secret_scanning_patterns') IS NOT NULL
               AND to_regclass('public.push_protection_bypasses') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_secret_scanning_tables {
            eprintln!("skipping secret scanning alerts scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing secret scanning alerts scenario with pre-applied schema after migration warning: {error}"
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

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("PATCH")
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
async fn secret_scanning_alerts_redact_filter_and_protect_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping secret scanning alerts scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "secret-scan-owner").await;
    let reader = create_user(&pool, "secret-scan-reader").await;
    let outsider = create_user(&pool, "secret-scan-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("secret-scanning-{}", Uuid::new_v4().simple()),
            description: Some("Secret scanning alerts repository".to_owned()),
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

    let plaintext_secret = "ghp_plaintext_secret_value_that_must_never_leak";
    replace_repository_snapshot(
        &pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: format!("commit-{}", Uuid::new_v4().simple()),
                author_user_id: Some(owner.id),
                committer_user_id: Some(owner.id),
                message: "Seed redacted credential evidence".to_owned(),
                tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
                parent_oids: Vec::new(),
                committed_at: Utc::now(),
            },
            branch_name: "main".to_owned(),
            files: vec![RepositorySnapshotFile {
                path: "src/config.rs".to_owned(),
                content: format!("TOKEN={plaintext_secret}\n"),
                oid: format!("blob-{}", Uuid::new_v4().simple()),
                byte_size: plaintext_secret.len() as i64 + 7,
            }],
        },
    )
    .await
    .expect("default branch files should seed");

    sqlx::query(
        r#"
        INSERT INTO repository_security_feature_settings (
            repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        VALUES ($1, 'secret_scanning', 'enabled', 'Secret scanning is monitoring committed content.', 1, 1, '/settings/security_analysis')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("secret scanning setting should insert");

    let pattern_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO secret_scanning_patterns (
            slug, provider, secret_type, display_name, result_kind, push_protection_enabled
        )
        VALUES ($1, 'GitHub', 'github_pat', 'GitHub personal access token', 'provider', true)
        ON CONFLICT (lower(slug)) DO UPDATE
        SET provider = EXCLUDED.provider
        RETURNING id
        "#,
    )
    .bind(format!("github-pat-{}", Uuid::new_v4().simple()))
    .fetch_one(&pool)
    .await
    .expect("pattern should insert");

    let alert_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO secret_scanning_alerts (
            repository_id, pattern_id, number, state, fingerprint, secret_hash,
            redacted_secret, redacted_context, result_kind, validity_state
        )
        VALUES ($1, $2, 1, 'open', 'fingerprint-secret-demo', 'sha256:redacted-demo',
                'ghp_****1234', 'TOKEN=ghp_****1234', 'provider', 'active')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(pattern_id)
    .fetch_one(&pool)
    .await
    .expect("secret scanning alert should insert");
    let file_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM repository_files WHERE repository_id = $1 AND path = 'src/config.rs' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("repository file should exist");
    let commit_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM commits WHERE repository_id = $1 ORDER BY committed_at DESC LIMIT 1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("commit should exist");
    sqlx::query(
        r#"
        INSERT INTO secret_scanning_alert_locations (
            alert_id, repository_file_id, commit_id, ref_name, branch_name, path, start_line, redacted_snippet
        )
        VALUES ($1, $2, $3, 'refs/heads/main', 'main', 'src/config.rs', 1, 'TOKEN=ghp_****1234')
        "#,
    )
    .bind(alert_id)
    .bind(file_id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("location should insert");
    sqlx::query("INSERT INTO secret_scanning_alert_assignees (alert_id, user_id) VALUES ($1, $2)")
        .bind(alert_id)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("assignee should insert");
    sqlx::query(
        r#"
        INSERT INTO secret_scanning_validity_checks (alert_id, provider, status, message)
        VALUES ($1, 'GitHub', 'active', 'Provider reported this credential is active.')
        "#,
    )
    .bind(alert_id)
    .execute(&pool)
    .await
    .expect("validity check should insert");
    sqlx::query(
        r#"
        INSERT INTO push_protection_bypasses (
            repository_id, alert_id, actor_user_id, ref_name, commit_oid, path, reason, status, redacted_snippet
        )
        VALUES ($1, $2, $3, 'refs/heads/main', 'commit-secret-demo', 'src/config.rs',
                'false_positive', 'accepted', 'TOKEN=ghp_****1234')
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("bypass should insert");
    sqlx::query(
        r#"
        INSERT INTO secret_scanning_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, 'created', 'Secret scanning opened this alert with redacted evidence.', '{"redacted": true}'::jsonb)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("timeline event should insert");

    let disabled_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("secret-scanning-disabled-{}", Uuid::new_v4().simple()),
            description: Some("Disabled secret scanning repository".to_owned()),
            visibility: RepositoryVisibility::Private,
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
        VALUES ($1, 'secret_scanning', 'disabled', 'Secret scanning is disabled by repository policy.', 0, 0, '/settings/security_analysis')
        "#,
    )
    .bind(disabled_repository.id)
    .execute(&pool)
    .await
    .expect("disabled setting should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!(
        "/api/repos/{owner_login}/{}/security/secret-scanning",
        repository.name
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert!(!outsider_body.to_string().contains("ghp_"));

    let (reader_status, reader_body) = get_json(app.clone(), &base, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["availability"]["enabled"], true);
    assert_eq!(reader_body["viewer"]["canWrite"], false);
    assert_eq!(reader_body["counts"]["open"], 1);
    assert_eq!(reader_body["counts"]["provider"], 1);
    assert_eq!(reader_body["counts"]["bypassed"], 1);
    assert_eq!(reader_body["alerts"][0]["redactedSecret"], "ghp_****1234");
    assert_eq!(reader_body["alerts"][0]["validity"]["status"], "active");
    assert_eq!(reader_body["alerts"][0]["bypassed"], true);
    assert_eq!(
        reader_body["alerts"][0]["primaryLocation"]["pathHref"],
        format!(
            "/{owner_login}/{}/blob/refs%2Fheads%2Fmain/src/config.rs#L1",
            repository.name
        )
    );
    assert_eq!(reader_body["pushProtection"]["enabled"], true);
    let reader_text = reader_body.to_string();
    assert!(!reader_text.contains(plaintext_secret));
    assert!(!reader_text.contains("test-session-secret"));

    let (detail_status, detail_body) =
        get_json(app.clone(), &format!("{base}/1"), Some(&owner_cookie)).await;
    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["alert"]["id"], alert_id.to_string());
    assert_eq!(detail_body["pattern"]["secretType"], "github_pat");
    assert_eq!(detail_body["locations"][0]["path"], "src/config.rs");
    assert_eq!(detail_body["validity"]["status"], "active");
    assert_eq!(detail_body["bypasses"][0]["reason"], "false_positive");
    assert_eq!(detail_body["timeline"][0]["eventType"], "created");
    assert_eq!(detail_body["assigneeOptions"][0]["kind"], "user");
    assert!(!detail_body.to_string().contains(plaintext_secret));

    let (reader_patch_status, reader_patch_body) = patch_json(
        app.clone(),
        &format!("{base}/1"),
        Some(&reader_cookie),
        json!({ "action": "resolve", "resolution": "revoked" }),
    )
    .await;
    assert_eq!(reader_patch_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_patch_body["error"]["code"], "forbidden");

    let (resolve_status, resolve_body) = patch_json(
        app.clone(),
        &format!("{base}/1"),
        Some(&owner_cookie),
        json!({
            "action": "resolve",
            "resolution": "revoked",
            "resolutionComment": "Rotated outside opengithub."
        }),
    )
    .await;
    assert_eq!(resolve_status, StatusCode::OK, "{resolve_body}");
    assert_eq!(resolve_body["alert"]["state"], "resolved");
    assert_eq!(resolve_body["alert"]["resolution"], "revoked");
    assert_eq!(
        resolve_body["timeline"]
            .as_array()
            .expect("timeline")
            .last()
            .expect("latest event")["eventType"],
        "resolved"
    );
    assert!(!resolve_body.to_string().contains(plaintext_secret));

    let (reopen_status, reopen_body) = patch_json(
        app.clone(),
        &format!("{base}/1"),
        Some(&owner_cookie),
        json!({ "action": "reopen" }),
    )
    .await;
    assert_eq!(reopen_status, StatusCode::OK, "{reopen_body}");
    assert_eq!(reopen_body["alert"]["state"], "open");
    assert_eq!(reopen_body["alert"]["resolution"], Value::Null);

    let (assign_status, assign_body) = patch_json(
        app.clone(),
        &format!("{base}/1"),
        Some(&owner_cookie),
        json!({ "action": "assign", "assigneeIds": [reader.id] }),
    )
    .await;
    assert_eq!(assign_status, StatusCode::OK, "{assign_body}");
    assert_eq!(
        assign_body["alert"]["assignees"][0]["login"],
        reader.username.as_deref().expect("reader username")
    );

    let (validity_status, validity_body) = patch_json(
        app.clone(),
        &format!("{base}/1"),
        Some(&owner_cookie),
        json!({ "action": "validity", "validity": "inactive" }),
    )
    .await;
    assert_eq!(validity_status, StatusCode::OK, "{validity_body}");
    assert_eq!(validity_body["validity"]["status"], "inactive");

    let audit_and_events: String = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT jsonb_agg(payload)::text
        FROM (
            SELECT metadata AS payload FROM secret_scanning_alert_events WHERE alert_id = $1
            UNION ALL
            SELECT metadata AS payload FROM security_audit_events
            WHERE event_type = 'repository.secret_scanning_alert.update'
              AND target_id = $2::text
        ) events
        "#,
    )
    .bind(alert_id)
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("audit payloads should load")
    .unwrap_or_default();
    assert!(audit_and_events.contains("revoked"));
    assert!(!audit_and_events.contains(plaintext_secret));

    let notification_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications WHERE repository_id = $1 AND subject_type = 'secret_scanning_alert'",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("notifications should count");
    assert!(notification_count >= 1);

    let (filtered_status, filtered_body) = get_json(
        app.clone(),
        &format!("{base}?state=open&q=github&provider=GitHub&secret_type=github_pat&validity=inactive&bypassed=true&topic=provider&sort=provider"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(filtered_status, StatusCode::OK, "{filtered_body}");
    assert_eq!(filtered_body["alerts"].as_array().expect("alerts").len(), 1);
    assert_eq!(filtered_body["providers"][0]["provider"], "GitHub");
    assert_eq!(filtered_body["secretTypes"][0]["secretType"], "github_pat");

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}?validity=plaintext"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let disabled_base = format!(
        "/api/repos/{owner_login}/{}/security/secret-scanning",
        disabled_repository.name
    );
    let (disabled_status, disabled_body) =
        get_json(app.clone(), &disabled_base, Some(&owner_cookie)).await;
    assert_eq!(disabled_status, StatusCode::OK, "{disabled_body}");
    assert_eq!(disabled_body["availability"]["enabled"], false);
    assert_eq!(disabled_body["alerts"].as_array().expect("alerts").len(), 0);

    let direct = repository_secret_scanning_alerts_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        SecretScanningAlertsQuery {
            state: Some("open"),
            query: Some("github"),
            provider: Some("GitHub"),
            secret_type: Some("github_pat"),
            validity: Some("inactive"),
            resolution: None,
            bypassed: Some("true"),
            team: None,
            topic: Some("provider"),
            sort: Some("recently_detected"),
        },
    )
    .await
    .expect("direct list should load")
    .expect("repository should exist");
    assert_eq!(direct.alerts.len(), 1);
    assert_eq!(direct.alerts[0].pattern.secret_type, "github_pat");
    assert!(direct.alerts[0].redacted_secret.contains("****"));

    let direct_detail = repository_secret_scanning_alert_detail_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        1,
    )
    .await
    .expect("direct detail should load")
    .expect("alert should exist");
    assert_eq!(direct_detail.locations[0].path, "src/config.rs");
    assert_eq!(direct_detail.bypasses[0].status, "accepted");
    assert!(!serde_json::to_string(&direct_detail)
        .expect("detail should serialize")
        .contains(plaintext_secret));

    let direct_resolved = update_repository_secret_scanning_alert_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        1,
        SecretScanningAlertMutation {
            action: "resolve".to_owned(),
            resolution: Some("false_positive".to_owned()),
            resolution_comment: None,
            validity: None,
            assignee_ids: None,
        },
    )
    .await
    .expect("direct update should succeed")
    .expect("alert should exist");
    assert_eq!(direct_resolved.alert.state, "resolved");
    assert_eq!(
        direct_resolved.alert.resolution.as_deref(),
        Some("false_positive")
    );
}
