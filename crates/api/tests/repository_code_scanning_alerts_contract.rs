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
            repository_code_scanning_alert_detail_for_actor_by_owner_name,
            repository_code_scanning_alerts_for_actor_by_owner_name, CodeScanningAlertsQuery,
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
            eprintln!("skipping code scanning alerts scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_code_scanning_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.code_scanning_alerts') IS NOT NULL
               AND to_regclass('public.code_scanning_runs') IS NOT NULL
               AND to_regclass('public.code_scanning_alert_events') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_code_scanning_tables {
            eprintln!("skipping code scanning alerts scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing code scanning alerts scenario with pre-applied schema after migration warning: {error}"
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

async fn request_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let response = app
        .oneshot(
            builder
                .body(match body {
                    Some(body) => Body::from(body.to_string()),
                    None => Body::empty(),
                })
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
async fn code_scanning_alerts_filter_detail_and_protect_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping code scanning alerts scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "code-scan-owner").await;
    let reader = create_user(&pool, "code-scan-reader").await;
    let outsider = create_user(&pool, "code-scan-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("code-scanning-{}", Uuid::new_v4().simple()),
            description: Some("Code scanning alerts repository".to_owned()),
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
        VALUES ($1, 'code_scanning', 'enabled', 'CodeQL analysis is monitored.', 0, 0, '/settings/security_analysis')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("code scanning setting should insert");

    let source = "pub fn run(input: &str) { println!(\"{}\", input); }\n";
    replace_repository_snapshot(
        &pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: format!("commit-{}", Uuid::new_v4().simple()),
                author_user_id: Some(owner.id),
                committer_user_id: Some(owner.id),
                message: "Seed code scanning source".to_owned(),
                tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
                parent_oids: Vec::new(),
                committed_at: Utc::now(),
            },
            branch_name: "main".to_owned(),
            files: vec![RepositorySnapshotFile {
                path: "src/lib.rs".to_owned(),
                content: source.to_owned(),
                oid: format!("blob-{}", Uuid::new_v4().simple()),
                byte_size: source.len() as i64,
            }],
        },
    )
    .await
    .expect("default branch files should seed");

    let run_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO code_scanning_runs (
            repository_id, tool_name, tool_version, category, ref_name, commit_oid, status, source, completed_at
        )
        VALUES ($1, 'CodeQL', '2.17.0', 'rust', 'refs/heads/main', 'commit-codeql-demo', 'completed', 'sarif', now())
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("code scanning run should insert");
    let alert_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO code_scanning_alerts (
            repository_id, run_id, number, state, rule_id, rule_name, rule_description, message,
            severity, security_severity, tool_name, ref_name, branch_name, path, start_line,
            fingerprint, code_snippet, help_markdown, help_uri
        )
        VALUES (
            $1, $2, 1, 'open', 'rust/log-injection', 'Log injection', 'Untrusted data reaches a log sink.',
            'User-controlled data is written to logs.', 'warning', 'high', 'CodeQL', 'refs/heads/main',
            'main', 'src/lib.rs', 1, 'fingerprint-codeql-demo', $3,
            'Validate or encode untrusted values before logging.', 'https://codeql.github.com'
        )
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(run_id)
    .bind(source)
    .fetch_one(&pool)
    .await
    .expect("code scanning alert should insert");
    sqlx::query(
        r#"
        INSERT INTO code_scanning_alert_instances (
            alert_id, run_id, ref_name, commit_oid, path, start_line, message
        )
        VALUES ($1, $2, 'refs/heads/main', 'commit-codeql-demo', 'src/lib.rs', 1, 'User-controlled data is written to logs.')
        "#,
    )
    .bind(alert_id)
    .bind(run_id)
    .execute(&pool)
    .await
    .expect("code scanning instance should insert");
    sqlx::query("INSERT INTO code_scanning_alert_assignees (alert_id, user_id) VALUES ($1, $2)")
        .bind(alert_id)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("assignee should insert");
    sqlx::query(
        r#"
        INSERT INTO code_scanning_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, 'created', 'CodeQL opened this alert from SARIF analysis.', '{"redacted": true}'::jsonb)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("timeline event should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!(
        "/api/repos/{owner_login}/{}/security/code-scanning",
        repository.name
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert!(!outsider_body.to_string().contains("Log injection"));

    let (reader_status, reader_body) = get_json(app.clone(), &base, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["availability"]["enabled"], true);
    assert_eq!(reader_body["viewer"]["canWrite"], false);
    assert_eq!(reader_body["counts"]["open"], 1);
    assert_eq!(reader_body["alerts"][0]["ruleName"], "Log injection");
    assert_eq!(reader_body["alerts"][0]["securitySeverity"], "high");
    assert_eq!(
        reader_body["alerts"][0]["pathHref"],
        format!(
            "/{owner_login}/{}/blob/refs%2Fheads%2Fmain/src/lib.rs#L1",
            repository.name
        )
    );
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let (detail_status, detail_body) =
        get_json(app.clone(), &format!("{base}/1"), Some(&owner_cookie)).await;
    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["alert"]["id"], alert_id.to_string());
    assert_eq!(detail_body["location"]["path"], "src/lib.rs");
    assert_eq!(detail_body["rule"]["id"], "rust/log-injection");
    assert_eq!(detail_body["timeline"][0]["eventType"], "created");
    assert_eq!(detail_body["assigneeOptions"][0]["kind"], "user");
    assert_eq!(detail_body["linkedIssue"]["canLink"], true);

    let (reader_patch_status, reader_patch_body) = request_json(
        app.clone(),
        "PATCH",
        &format!("{base}/1"),
        Some(&reader_cookie),
        Some(json!({ "action": "dismiss", "dismissalReason": "false_positive" })),
    )
    .await;
    assert_eq!(reader_patch_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_patch_body["error"]["code"], "forbidden");

    let (invalid_dismiss_status, invalid_dismiss_body) = request_json(
        app.clone(),
        "PATCH",
        &format!("{base}/1"),
        Some(&owner_cookie),
        Some(json!({ "action": "dismiss" })),
    )
    .await;
    assert_eq!(invalid_dismiss_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_dismiss_body["error"]["code"], "validation_failed");

    let (dismiss_status, dismiss_body) = request_json(
        app.clone(),
        "PATCH",
        &format!("{base}/1"),
        Some(&owner_cookie),
        Some(json!({
            "action": "dismiss",
            "dismissalReason": "false_positive",
            "dismissalComment": "Confirmed by triage"
        })),
    )
    .await;
    assert_eq!(dismiss_status, StatusCode::OK, "{dismiss_body}");
    assert_eq!(dismiss_body["alert"]["state"], "dismissed");
    assert!(dismiss_body["timeline"]
        .as_array()
        .expect("timeline")
        .iter()
        .any(|event| event["eventType"] == "dismissed"));

    let (reopen_status, reopen_body) = request_json(
        app.clone(),
        "PATCH",
        &format!("{base}/1"),
        Some(&owner_cookie),
        Some(json!({ "action": "reopen" })),
    )
    .await;
    assert_eq!(reopen_status, StatusCode::OK, "{reopen_body}");
    assert_eq!(reopen_body["alert"]["state"], "open");

    let (assign_status, assign_body) = request_json(
        app.clone(),
        "PATCH",
        &format!("{base}/1"),
        Some(&owner_cookie),
        Some(json!({ "action": "assign", "assigneeIds": [reader.id] })),
    )
    .await;
    assert_eq!(assign_status, StatusCode::OK, "{assign_body}");
    assert_eq!(
        assign_body["alert"]["assignees"][0]["id"],
        reader.id.to_string()
    );

    let (issue_status, issue_body) = request_json(
        app.clone(),
        "POST",
        &format!("{base}/1/issue"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(issue_status, StatusCode::CREATED, "{issue_body}");
    assert_eq!(issue_body["linkedIssue"]["issue"]["number"], 1);
    assert_eq!(
        issue_body["linkedIssue"]["issue"]["href"],
        format!("/{owner_login}/{}/issues/1", repository.name)
    );

    let notification_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notifications WHERE repository_id = $1 AND subject_type IN ('code_scanning_alert', 'issue')",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("notification count should load");
    assert!(notification_count >= 1);
    let audit_payloads = sqlx::query_scalar::<_, Value>(
        "SELECT metadata FROM security_audit_events WHERE event_type = 'repository.code_scanning_alert.update' AND target_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(repository.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit event should persist");
    assert!(!audit_payloads.to_string().contains("test-session-secret"));

    let filtered = repository_code_scanning_alerts_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        CodeScanningAlertsQuery {
            state: Some("open"),
            query: Some("log"),
            severity: Some("warning"),
            security_severity: Some("high"),
            tool: Some("CodeQL"),
            branch: Some("main"),
            ref_name: Some("refs/heads/main"),
            tag: None,
            application_code: Some("true"),
            sort: Some("most_important"),
        },
    )
    .await
    .expect("direct alert list should load")
    .expect("repository should exist");
    assert_eq!(filtered.alerts.len(), 1);
    assert_eq!(filtered.tools[0].name, "CodeQL");
    assert_eq!(filtered.branches[0].name, "main");

    let direct_detail = repository_code_scanning_alert_detail_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        1,
    )
    .await
    .expect("direct alert detail should load")
    .expect("alert should exist");
    assert_eq!(direct_detail.alert.assignees[0].id, reader.id);
    assert_eq!(direct_detail.rule.name, "Log injection");

    let sarif_upload = json!({
        "ref": "main",
        "commitSha": "commit-sarif-upload",
        "sarif": {
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "CodeQL",
                        "version": "2.18.0",
                        "rules": [{
                            "id": "rust/sql-injection",
                            "name": "SQL injection",
                            "shortDescription": { "text": "Untrusted data reaches a query sink." },
                            "help": { "markdown": "Use parameterized queries." },
                            "helpUri": "https://codeql.github.com/codeql-query-help/rust/"
                        }]
                    }
                },
                "results": [{
                    "ruleId": "rust/sql-injection",
                    "level": "error",
                    "message": { "text": "Untrusted input is used in a database query." },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": "src/lib.rs" },
                            "region": { "startLine": 1, "endLine": 1 }
                        }
                    }],
                    "partialFingerprints": { "primaryLocationLineHash": "sarif-sql-fingerprint" },
                    "properties": { "security-severity": "9.1" }
                }]
            }]
        }
    });
    let upload_endpoint = format!(
        "/api/repos/{owner_login}/{}/code-scanning/sarifs",
        repository.name
    );
    let (reader_upload_status, reader_upload_body) = request_json(
        app.clone(),
        "POST",
        &upload_endpoint,
        Some(&reader_cookie),
        Some(sarif_upload.clone()),
    )
    .await;
    assert_eq!(reader_upload_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_upload_body["error"]["code"], "forbidden");

    let (malformed_upload_status, malformed_upload_body) = request_json(
        app.clone(),
        "POST",
        &upload_endpoint,
        Some(&owner_cookie),
        Some(json!({ "sarif": { "version": "2.1.0", "runs": [] } })),
    )
    .await;
    assert_eq!(malformed_upload_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(malformed_upload_body["error"]["code"], "validation_failed");
    assert!(!malformed_upload_body
        .to_string()
        .contains("test-session-secret"));

    let (upload_status, upload_body) = request_json(
        app.clone(),
        "POST",
        &upload_endpoint,
        Some(&owner_cookie),
        Some(sarif_upload),
    )
    .await;
    assert_eq!(upload_status, StatusCode::ACCEPTED, "{upload_body}");
    assert_eq!(upload_body["status"], "processed");
    assert_eq!(upload_body["processedAlerts"], 1);
    assert_eq!(upload_body["fixedAlerts"], 1);
    assert_eq!(upload_body["toolVersion"], "2.18.0");
    assert!(!upload_body["artifactStorageKey"]
        .as_str()
        .expect("storage key")
        .contains("test-session-secret"));

    let sarif_list = repository_code_scanning_alerts_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        CodeScanningAlertsQuery {
            state: Some("all"),
            query: None,
            severity: None,
            security_severity: None,
            tool: Some("CodeQL"),
            branch: Some("main"),
            ref_name: Some("refs/heads/main"),
            tag: None,
            application_code: Some("true"),
            sort: Some("recently_detected"),
        },
    )
    .await
    .expect("SARIF alert list should load")
    .expect("repository should exist");
    assert_eq!(sarif_list.counts.total, 2);
    assert!(sarif_list
        .alerts
        .iter()
        .any(|alert| alert.rule_id == "rust/sql-injection" && alert.state == "open"));
    assert!(sarif_list
        .alerts
        .iter()
        .any(|alert| alert.rule_id == "rust/log-injection" && alert.state == "fixed"));
    assert_eq!(sarif_list.tools[0].version.as_deref(), Some("2.18.0"));

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}?severity=critical"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    sqlx::query(
        "UPDATE repository_security_feature_settings SET status = 'disabled', summary = 'Code scanning is not enabled.' WHERE repository_id = $1 AND feature_key = 'code_scanning'",
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("setting should disable");
    let (disabled_status, disabled_body) = get_json(app, &base, Some(&owner_cookie)).await;
    assert_eq!(disabled_status, StatusCode::OK, "{disabled_body}");
    assert_eq!(disabled_body["availability"]["enabled"], false);
    assert_eq!(disabled_body["counts"]["open"], 0);
    assert_eq!(disabled_body["alerts"].as_array().expect("alerts").len(), 0);
}
