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
            create_repository, grant_repository_permission, insert_commit, upsert_git_ref,
            CreateCommit, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        repository_security::{
            repository_security_overview_for_actor_by_owner_name,
            repository_security_policy_for_actor_by_owner_name,
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
            eprintln!("skipping repository security scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_security_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_security_feature_settings') IS NOT NULL
               AND to_regclass('public.repository_security_policies') IS NOT NULL
               AND to_regclass('public.repository_security_advisories') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_security_tables {
            eprintln!("skipping repository security scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository security scenario with pre-applied schema after migration warning: {error}"
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
        // Keep anonymous auth expectations independent from rate-limit buckets
        // accumulated by earlier integration tests in the shared test DB.
        .header(
            "x-forwarded-for",
            format!("security-policy-contract-{}", Uuid::new_v4()),
        );
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

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(
            "x-forwarded-for",
            format!("security-policy-contract-{}", Uuid::new_v4()),
        )
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
async fn repository_security_overview_returns_policy_advisories_privacy_and_sanitized_markdown() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository security scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "security-owner").await;
    let reader = create_user(&pool, "security-reader").await;
    let outsider = create_user(&pool, "security-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("security-{}", Uuid::new_v4().simple()),
            description: Some("Security policy repository".to_owned()),
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

    let commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("policy{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Publish security policy".to_owned(),
            tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
            parent_oids: Vec::new(),
            committed_at: Utc::now() - Duration::hours(2),
        },
    )
    .await
    .expect("commit should insert");
    upsert_git_ref(&pool, repository.id, "main", "branch", Some(commit.id))
        .await
        .expect("main ref should upsert");
    let policy_markdown = "# Security policy\n\nPlease email [security](mailto:security@example.com).\n\n## Supported versions\n\nSee [the guide](docs/security-guide.md).\n\n<script>alert('x')</script>";
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, 'SECURITY.md', $3, $4, $5)
        "#,
    )
    .bind(repository.id)
    .bind(commit.id)
    .bind(policy_markdown)
    .bind(format!("blob-{}", Uuid::new_v4().simple()))
    .bind(policy_markdown.len() as i64)
    .execute(&pool)
    .await
    .expect("security policy file should insert");

    sqlx::query(
        r#"
        INSERT INTO repository_security_feature_settings (
            repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        VALUES
            ($1, 'dependabot', 'enabled', 'Dependency alerts are monitored.', 7, 2, '/settings/security_analysis'),
            ($1, 'code_scanning', 'needs_setup', 'No code scanning workflow is configured.', 3, 1, '/security/code-scanning/setup')
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("feature settings should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisories (
            repository_id, advisory_identifier, severity, status, title, summary,
            package_name, vulnerable_range, advisory_href, published_at
        )
        VALUES
            ($1, 'GHSA-visible-demo', 'high', 'published', 'Visible advisory', 'Patch the affected dependency.', 'demo-package', '< 1.2.3', '/advisories/GHSA-visible-demo', now() - interval '1 hour'),
            ($1, 'GHSA-draft-demo', 'critical', 'draft', 'Draft advisory', 'This should stay hidden.', 'secret-package', '< 9.9.9', '/advisories/GHSA-draft-demo', NULL)
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("advisories should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!("/api/repos/{owner_login}/{}/security", repository.name);

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert!(!outsider_body.to_string().contains("Dependency alerts"));

    let (reader_status, reader_body) = get_json(app.clone(), &base, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["viewer"]["canWrite"], false);
    assert_eq!(reader_body["policy"]["exists"], true);
    assert!(reader_body["policy"]["html"]
        .as_str()
        .expect("policy html")
        .contains("mailto:security@example.com"));
    assert!(!reader_body["policy"]["html"]
        .as_str()
        .expect("policy html")
        .contains("<script"));
    assert_eq!(reader_body["features"][0]["alertCount"], Value::Null);
    assert_eq!(reader_body["features"][0]["privateCount"], Value::Null);
    assert_eq!(
        reader_body["advisories"]
            .as_array()
            .expect("advisories")
            .len(),
        1
    );
    assert_eq!(
        reader_body["advisories"][0]["identifier"],
        "GHSA-visible-demo"
    );
    assert!(!reader_body.to_string().contains("GHSA-draft-demo"));
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let direct = repository_security_overview_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
    )
    .await
    .expect("direct security overview should load")
    .expect("repository should exist");
    assert!(direct.viewer.can_edit_policy);
    assert_eq!(
        direct
            .features
            .iter()
            .find(|feature| feature.key == "dependabot")
            .expect("dependabot card")
            .alert_count,
        Some(7)
    );

    let (owner_status, owner_body) = get_json(app.clone(), &base, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewer"]["canViewPrivateAlertCounts"], true);
    assert_eq!(owner_body["features"][0]["alertCount"], 7);
    assert!(owner_body["policy"]["editHref"]
        .as_str()
        .expect("edit href")
        .contains("/security/policy/edit"));

    let policy_base = format!("{base}/policy");
    let (policy_status, policy_body) =
        get_json(app.clone(), &policy_base, Some(&reader_cookie)).await;
    assert_eq!(policy_status, StatusCode::OK, "{policy_body}");
    assert_eq!(policy_body["policy"]["exists"], true);
    assert_eq!(policy_body["policy"]["path"], "SECURITY.md");
    assert_eq!(
        policy_body["policy"]["latestCommit"]["message"],
        "Publish security policy"
    );
    assert!(policy_body["policy"]["latestCommit"]["href"]
        .as_str()
        .expect("commit href")
        .contains("/commit/policy"));
    assert_eq!(
        policy_body["policy"]["outline"][0]["href"],
        "#security-policy"
    );
    assert_eq!(
        policy_body["policy"]["outline"][1]["text"],
        "Supported versions"
    );
    assert!(policy_body["policy"]["html"]
        .as_str()
        .expect("policy html")
        .contains("mailto:security@example.com"));
    assert!(policy_body["policy"]["html"]
        .as_str()
        .expect("policy html")
        .contains("/blob/main/docs/security-guide.md"));
    assert!(!policy_body["policy"]["html"]
        .as_str()
        .expect("policy html")
        .contains("<script"));
    assert_eq!(policy_body["policy"]["editHref"], Value::Null);
    assert!(!policy_body.to_string().contains("test-session-secret"));

    let direct_policy = repository_security_policy_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
    )
    .await
    .expect("direct security policy should load")
    .expect("repository should exist");
    assert!(direct_policy.viewer.can_edit_policy);
    assert_eq!(direct_policy.policy.outline.len(), 2);
    assert!(direct_policy
        .policy
        .edit_href
        .expect("owner edit href")
        .contains("/security/policy/edit"));

    let updated_markdown = "# Security policy\n\nEmail [triage](mailto:triage@example.com).\n\n## Scope\n\nDefault branch only.";
    let (reader_update_status, reader_update_body) = send_json(
        app.clone(),
        Method::PATCH,
        &policy_base,
        Some(&reader_cookie),
        json!({
            "markdown": updated_markdown,
            "commitMessage": "Update security policy",
            "expectedContentSha": policy_body["policy"]["contentSha"],
        }),
    )
    .await;
    assert_eq!(reader_update_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_update_body["error"]["code"], "forbidden");

    let (stale_status, stale_body) = send_json(
        app.clone(),
        Method::PATCH,
        &policy_base,
        Some(&owner_cookie),
        json!({
            "markdown": updated_markdown,
            "commitMessage": "Update security policy",
            "expectedContentSha": "stale-content-sha",
        }),
    )
    .await;
    assert_eq!(stale_status, StatusCode::CONFLICT);
    assert_eq!(stale_body["error"]["code"], "conflict");

    let (invalid_status, invalid_body) = send_json(
        app.clone(),
        Method::PATCH,
        &policy_base,
        Some(&owner_cookie),
        json!({
            "markdown": "",
            "commitMessage": "Update security policy",
            "expectedContentSha": policy_body["policy"]["contentSha"],
        }),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (update_status, update_body) = send_json(
        app.clone(),
        Method::PATCH,
        &policy_base,
        Some(&owner_cookie),
        json!({
            "markdown": updated_markdown,
            "commitMessage": "Update security policy",
            "expectedContentSha": policy_body["policy"]["contentSha"],
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{update_body}");
    assert_eq!(update_body["policy"]["markdown"], updated_markdown);
    assert_eq!(
        update_body["policy"]["latestCommit"]["message"],
        "Update security policy"
    );
    assert!(update_body["policy"]["html"]
        .as_str()
        .expect("updated html")
        .contains("mailto:triage@example.com"));

    let updated_commit_id: Uuid = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT source_commit_id
        FROM repository_security_policies
        WHERE repository_id = $1 AND lower(path) = 'security.md'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("policy source commit should persist")
    .expect("source commit should not be null");
    let reflected_file: String = sqlx::query_scalar(
        "SELECT content FROM repository_files WHERE repository_id = $1 AND commit_id = $2 AND path = 'SECURITY.md'",
    )
    .bind(repository.id)
    .bind(updated_commit_id)
    .fetch_one(&pool)
    .await
    .expect("updated repository file should reflect policy");
    assert_eq!(reflected_file, updated_markdown);
    let ref_points_to_policy_commit: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_git_refs
            WHERE repository_id = $1
              AND name = 'refs/heads/main'
              AND target_commit_id = $2
        )
        "#,
    )
    .bind(repository.id)
    .bind(updated_commit_id)
    .fetch_one(&pool)
    .await
    .expect("ref should read");
    assert!(ref_points_to_policy_commit);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM security_audit_events WHERE actor_user_id = $1 AND target_id = $2 AND event_type = 'repository.security_policy.upsert'",
    )
    .bind(owner.id)
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should read");
    assert_eq!(audit_count, 1);
    assert!(!update_body.to_string().contains("test-session-secret"));

    let create_repository_target = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("security-create-{}", Uuid::new_v4().simple()),
            description: Some("Policy create repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("release/2026".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("create target repository should create");
    let create_policy_base = format!(
        "/api/repos/{owner_login}/{}/security/policy",
        create_repository_target.name
    );
    let created_markdown = "# Security policy\n\nReport issues privately.";
    let (create_status, create_body) = send_json(
        app.clone(),
        Method::POST,
        &create_policy_base,
        Some(&owner_cookie),
        json!({
            "markdown": created_markdown,
            "commitMessage": "Create security policy",
            "path": "SECURITY.md",
            "ref": "release/2026",
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{create_body}");
    assert_eq!(create_body["policy"]["exists"], true);
    assert_eq!(create_body["policy"]["ref"], "release/2026");
    assert_eq!(create_body["policy"]["markdown"], created_markdown);
    assert!(create_body["policy"]["sourceHref"]
        .as_str()
        .expect("created source href")
        .contains("/blob/release%2F2026/SECURITY.md"));

    sqlx::query("UPDATE repositories SET is_archived = true WHERE id = $1")
        .bind(repository.id)
        .execute(&pool)
        .await
        .expect("repository should archive");
    let (archived_status, archived_body) = send_json(
        app.clone(),
        Method::PATCH,
        &policy_base,
        Some(&owner_cookie),
        json!({
            "markdown": updated_markdown,
            "commitMessage": "Update security policy",
            "expectedContentSha": update_body["policy"]["contentSha"],
        }),
    )
    .await;
    assert_eq!(archived_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(archived_body["error"]["code"], "validation_failed");

    let (missing_policy_status, missing_policy_body) = get_json(
        app.clone(),
        "/api/repos/missing/repo/security/policy",
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_policy_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_policy_body["error"]["code"], "not_found");
}
