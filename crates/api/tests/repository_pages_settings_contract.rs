use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        permissions::RepositoryRole,
        repositories::{
            create_organization, create_repository, grant_repository_permission,
            CreateOrganization, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
    },
    jobs::pages::run_pages_build_deployment_once,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
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

#[tokio::test]
async fn organization_pages_policy_locks_member_publishing() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization Pages policy scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pagespolicy{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Pages Policy".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("member should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_policy_settings (organization_id, pages_private_publishing)
        VALUES ($1, false)
        ON CONFLICT (organization_id)
        DO UPDATE SET pages_private_publishing = false
        "#,
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("policy should update");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-repo"),
            description: Some("Pages policy contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Admin, "direct")
        .await
        .expect("member admin grant should persist");
    let commit_id = seed_commit_and_branch(&pool, repo.id, "main").await;
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, 'docs/index.html', '<h1>Policy</h1>', $3, 15)
        "#,
    )
    .bind(repo.id)
    .bind(commit_id)
    .bind(format!("{}-docs", Uuid::new_v4().simple()))
    .execute(&pool)
    .await
    .expect("docs file should persist");

    let uri = format!("/api/repos/{marker}/{}/settings/pages", repo.name);
    let (status, _, body) =
        send_json(app.clone(), Method::GET, &uri, Some(&member_cookie), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["canEdit"], true);
    assert_eq!(body["policyLock"]["field"], "pagesPrivatePublishing");
    assert_eq!(
        body["policyLock"]["settingsHref"],
        format!("/organizations/{marker}/settings/member_privileges")
    );

    let (blocked_status, _, blocked_body) = send_json(
        app,
        Method::PATCH,
        &format!("{uri}/source"),
        Some(&member_cookie),
        Some(json!({ "kind": "branch", "branch": "main", "folder": "/docs" })),
    )
    .await;
    assert_eq!(blocked_status, StatusCode::FORBIDDEN);
    assert_eq!(blocked_body["error"]["code"], "policy_locked");
    assert_eq!(blocked_body["details"]["field"], "pagesPrivatePublishing");
    assert!(!blocked_body.to_string().contains(&owner.email));
}

#[tokio::test]
async fn repository_pages_settings_validate_privacy_mutations_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository Pages settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pages{}", Uuid::new_v4().simple());
    let custom_domain = format!("{marker}.pages.example.com");
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outside")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Pages settings contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, reader.id, RepositoryRole::Read, "direct")
        .await
        .expect("reader grant should persist");
    let commit_id = seed_commit_and_branch(&pool, repo.id, "main").await;

    let uri = format!("/api/repos/{}/{}/settings/pages", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outside_status, _, outside_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&outsider_cookie), None).await;
    assert_eq!(outside_status, StatusCode::FORBIDDEN);
    assert!(!outside_body.to_string().contains("opengithub-pages"));

    let (initial_status, _, initial_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&owner_cookie), None).await;
    assert_eq!(initial_status, StatusCode::OK);
    assert_eq!(initial_body["site"]["source"]["kind"], "none");
    assert_eq!(initial_body["canEdit"], true);
    assert_eq!(initial_body["availableRefs"][0]["name"], "main");

    let (missing_docs_status, _, missing_docs_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/source"),
        Some(&owner_cookie),
        Some(json!({ "kind": "branch", "branch": "main", "folder": "/docs" })),
    )
    .await;
    assert_eq!(missing_docs_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing_docs_body["error"]["code"], "validation_failed");

    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, 'docs/index.html', '<h1>Pages</h1>', $3, 14)
        "#,
    )
    .bind(repo.id)
    .bind(commit_id)
    .bind(format!("{}-docs", Uuid::new_v4().simple()))
    .execute(&pool)
    .await
    .expect("docs file should persist");

    let (source_status, _, source_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/source"),
        Some(&owner_cookie),
        Some(json!({ "kind": "branch", "branch": "refs/heads/main", "folder": "/docs" })),
    )
    .await;
    assert_eq!(source_status, StatusCode::OK);
    assert_eq!(source_body["site"]["source"]["kind"], "branch");
    assert_eq!(source_body["site"]["source"]["branch"], "main");
    assert_eq!(source_body["site"]["source"]["folder"], "/docs");
    assert_eq!(source_body["deployments"][0]["status"], "queued");

    let deployment_id = source_body["deployments"][0]["id"]
        .as_str()
        .expect("deployment id should serialize");
    let queued = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM job_leases WHERE queue = 'pages-build-deploy' AND lease_key = $1)",
    )
    .bind(deployment_id)
    .fetch_one(&pool)
    .await
    .expect("job lookup should run");
    assert!(queued);
    sqlx::query(
        r#"
        INSERT INTO webhooks (
            repository_id, url, events, event_selection, created_by_user_id
        )
        VALUES ($1, 'https://hooks.example.com/pages', ARRAY['page_build']::text[], 'selected', $2)
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("Pages webhook should persist");
    let pages_result = run_pages_build_deployment_once(
        &pool,
        Uuid::parse_str(deployment_id).expect("deployment id should parse"),
        "pages-worker-test",
    )
    .await
    .expect("Pages worker should run")
    .expect("Pages worker should return result");
    assert_eq!(pages_result.status, "deployed");
    assert_eq!(pages_result.artifact_count, 1);
    let published = sqlx::query(
        r#"
        SELECT pages_deployments.status,
               pages_deployments.conclusion,
               pages_deployments.artifact_storage_key,
               pages_deployments.artifact_manifest,
               pages_deployments.build_log_excerpt,
               pages_sites.provisioning_status
        FROM pages_deployments
        JOIN pages_sites ON pages_sites.id = pages_deployments.site_id
        WHERE pages_deployments.id = $1
        "#,
    )
    .bind(Uuid::parse_str(deployment_id).expect("deployment id should parse"))
    .fetch_one(&pool)
    .await
    .expect("published deployment should load");
    assert_eq!(published.get::<String, _>("status"), "deployed");
    assert_eq!(
        published.get::<Option<String>, _>("conclusion").as_deref(),
        Some("success")
    );
    assert_eq!(published.get::<String, _>("provisioning_status"), "ready");
    assert!(published
        .get::<Option<String>, _>("artifact_storage_key")
        .expect("artifact storage key should persist")
        .starts_with("pages/"));
    let manifest = published.get::<Value, _>("artifact_manifest");
    assert_eq!(manifest["artifactCount"], 1);
    assert_eq!(manifest["files"][0]["path"], "index.html");
    assert!(published
        .get::<Option<String>, _>("build_log_excerpt")
        .expect("build log should persist")
        .contains("Published 1 Pages artifact"));
    let artifact_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pages_build_artifacts WHERE deployment_id = $1 AND path = 'index.html'",
    )
    .bind(Uuid::parse_str(deployment_id).expect("deployment id should parse"))
    .fetch_one(&pool)
    .await
    .expect("artifact count should load");
    assert_eq!(artifact_count, 1);
    let page_build_delivery = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM webhook_deliveries
        JOIN webhooks ON webhooks.id = webhook_deliveries.webhook_id
        WHERE webhooks.repository_id = $1
          AND webhook_deliveries.event = 'page_build'
          AND webhook_deliveries.payload->'payload'->>'status' = 'deployed'
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("page_build delivery count should load");
    assert_eq!(page_build_delivery, 1);

    let (invalid_domain_status, _, invalid_domain_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": "*.example.com" })),
    )
    .await;
    assert_eq!(invalid_domain_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_domain_body["error"]["code"], "validation_failed");

    let (domain_status, _, domain_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": format!("{marker}.Pages.Example.COM.") })),
    )
    .await;
    assert_eq!(domain_status, StatusCode::OK);
    assert_eq!(domain_body["site"]["customDomain"], custom_domain);
    assert_eq!(domain_body["site"]["domain"]["status"], "pending");
    assert!(domain_body["site"]["domain"]["challenge"]["value"]
        .as_str()
        .expect("challenge should be visible to admins")
        .starts_with("og-pages-"));

    let (reader_status, _, reader_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&reader_cookie), None).await;
    assert_eq!(reader_status, StatusCode::OK);
    assert_eq!(reader_body["canEdit"], false);
    assert_eq!(reader_body["site"]["customDomain"], custom_domain);
    assert!(reader_body["site"]["domain"]["challenge"].is_null());
    assert!(
        reader_body["deployments"][0]["artifactStorageKey"].is_null(),
        "reader body should redact deployment metadata: {reader_body:?}"
    );
    assert!(reader_body["deployments"][0]["workflowArtifactId"].is_null());
    assert_eq!(reader_body["deployments"][0]["artifactManifest"], json!({}));
    assert!(reader_body["deployments"][0]["buildLogExcerpt"].is_null());
    assert!(reader_body["deployments"][0]["failureReason"].is_null());
    assert!(!reader_body.to_string().contains("og-pages-"));
    assert!(!reader_body.to_string().contains("pages/"));
    assert!(!reader_body
        .to_string()
        .contains("Published 1 Pages artifact"));

    let (https_blocked_status, _, https_blocked_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/https"),
        Some(&owner_cookie),
        Some(json!({ "enforced": true })),
    )
    .await;
    assert_eq!(https_blocked_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(https_blocked_body["error"]["code"], "validation_failed");

    std::env::set_var("PAGES_DNS_VERIFICATION_MODE", "verified");
    let (recheck_status, _, recheck_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/domain/recheck"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(recheck_status, StatusCode::OK);
    assert_eq!(recheck_body["site"]["domain"]["status"], "verified");
    assert_eq!(recheck_body["site"]["certificateStatus"], "issued");

    let (https_status, _, https_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/https"),
        Some(&owner_cookie),
        Some(json!({ "enforced": true })),
    )
    .await;
    assert_eq!(https_status, StatusCode::OK);
    assert_eq!(https_body["site"]["httpsEnforced"], true);

    let second_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-other"),
            description: Some("Pages domain conflict".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("second repository should create");
    seed_commit_and_branch(&pool, second_repo.id, "main").await;
    let second_uri = format!(
        "/api/repos/{}/{}/settings/pages",
        owner.email, second_repo.name
    );
    let (conflict_status, _, conflict_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{second_uri}/domain"),
        Some(&owner_cookie),
        Some(json!({ "domain": custom_domain })),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
    assert_eq!(conflict_body["error"]["code"], "conflict");

    let workflow_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO actions_workflows (repository_id, name, path, trigger_events)
        VALUES ($1, 'Pages deploy', '.github/workflows/pages.yml', ARRAY['workflow_dispatch']::text[])
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("workflow should persist");
    let run_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, status,
            conclusion, head_branch, event, completed_at
        )
        VALUES ($1, $2, $3, 7, 'completed', 'success', 'main', 'workflow_dispatch', now())
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(workflow_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("workflow run should persist");
    let artifact_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO workflow_artifacts (run_id, name, digest, size_bytes, storage_key)
        VALUES ($1, 'github-pages', 'sha256:pages-artifact', 2048, 'actions/artifacts/github-pages.zip')
        RETURNING id
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("workflow artifact should persist");
    let (actions_deploy_status, _, actions_deploy_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/actions-deployments"),
        Some(&owner_cookie),
        Some(json!({ "workflowRunId": run_id, "workflowArtifactId": artifact_id })),
    )
    .await;
    assert_eq!(
        actions_deploy_status,
        StatusCode::OK,
        "actions deployment response body: {actions_deploy_body:?}"
    );
    let actions_deployment_id = actions_deploy_body["deployment"]["id"]
        .as_str()
        .expect("Actions deployment id should serialize");
    let actions_result = run_pages_build_deployment_once(
        &pool,
        Uuid::parse_str(actions_deployment_id).expect("Actions deployment id should parse"),
        "pages-worker-test",
    )
    .await
    .expect("Actions Pages worker should run")
    .expect("Actions Pages worker should return result");
    assert_eq!(actions_result.status, "deployed");
    let actions_artifact_path = sqlx::query_scalar::<_, String>(
        "SELECT path FROM pages_build_artifacts WHERE deployment_id = $1",
    )
    .bind(Uuid::parse_str(actions_deployment_id).expect("Actions deployment id should parse"))
    .fetch_one(&pool)
    .await
    .expect("Actions Pages artifact should persist");
    assert_eq!(actions_artifact_path, "github-pages.zip");

    let (unpublish_status, _, unpublish_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/unpublish"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(unpublish_status, StatusCode::OK);
    assert_eq!(unpublish_body["site"]["source"]["kind"], "none");
    assert_eq!(unpublish_body["site"]["provisioningStatus"], "unpublished");

    let docs_file_still_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM repository_files WHERE repository_id = $1 AND path = 'docs/index.html')",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("source file check should run");
    assert!(docs_file_still_exists);

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.pages.%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 6);
}

async fn seed_commit_and_branch(pool: &PgPool, repository_id: Uuid, branch: &str) -> Uuid {
    let oid = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let row = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, message, tree_oid)
        VALUES ($1, $2, 'seed pages commit', $3)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(&oid)
    .bind(format!("tree-{oid}"))
    .fetch_one(pool)
    .await
    .expect("commit should persist");
    let commit_id: Uuid = row.get("id");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{branch}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("branch ref should persist");
    commit_id
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

async fn create_user(pool: &PgPool, login: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{login} display")),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(login)
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

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request_body = if let Some(value) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&value).expect("body should serialize"))
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, headers, value)
}
