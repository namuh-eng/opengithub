use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        actions::{
            create_workflow, create_workflow_job, create_workflow_run, transition_workflow_run,
            CreateWorkflow, CreateWorkflowJob, CreateWorkflowRun, RunConclusion, RunStatus,
            TransitionRun,
        },
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
    },
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
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert")
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
    let set_cookie =
        session::set_cookie_header(config, &session_id, expires_at).expect("cookie should sign");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn get_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
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
    let request = builder
        .body(Body::from(body.to_string()))
        .expect("request should build");
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

fn encoded_workflow_path(path: &str) -> String {
    path.replace('/', "%2F")
}

#[tokio::test]
async fn workflow_detail_returns_scoped_runs_dispatch_metadata_refs_and_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions workflow detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-workflow-owner").await;
    let repo_name = format!("actions-workflow-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    let commit = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, $2, $3, $3, 'Add workflow fixtures')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(format!("workflow-sha-{}", Uuid::new_v4().simple()))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("commit should create");
    let commit_id = commit.get::<Uuid, _>("id");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, 'refs/heads/main', 'branch', $2),
               ($1, 'refs/heads/release', 'branch', $2),
               ($1, 'refs/tags/v1.0.0', 'tag', $2)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("refs should create");
    let source_blob_id = sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, 'workflow-source-blob', 'blob', 256)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("source blob should create")
    .get::<Uuid, _>("id");

    let ci = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "CI".to_owned(),
            path: ".github/workflows/ci.yml".to_owned(),
            trigger_events: vec!["push".to_owned(), "workflow_dispatch".to_owned()],
        },
    )
    .await
    .expect("ci workflow should create");
    sqlx::query(
        r#"
        UPDATE actions_workflows
        SET source_blob_id = $2,
            source_sha = 'workflow-source-sha',
            source_branch = 'main',
            dispatch_enabled = true,
            dispatch_inputs = $3
        WHERE id = $1
        "#,
    )
    .bind(ci.id)
    .bind(source_blob_id)
    .bind(json!([
        {
            "name": "environment",
            "type": "choice",
            "label": "Environment",
            "description": "Deployment target",
            "required": true,
            "default": "staging",
            "options": ["staging", "production"]
        },
        {
            "name": "reason",
            "type": "string",
            "label": "Reason",
            "description": null,
            "required": true,
            "default": null,
            "options": []
        },
        {
            "name": "dryRun",
            "type": "boolean",
            "label": "Dry run",
            "description": null,
            "required": false,
            "default": "true",
            "options": []
        }
    ]))
    .execute(&pool)
    .await
    .expect("workflow source metadata should update");
    let lint = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "Lint".to_owned(),
            path: ".github/workflows/lint.yml".to_owned(),
            trigger_events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("lint workflow should create");

    let ci_success = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: ci.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("abcdef0123456789".to_owned()),
            event: "workflow_dispatch".to_owned(),
        },
    )
    .await
    .expect("ci run should create");
    transition_workflow_run(
        &pool,
        ci_success.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Success),
        },
    )
    .await
    .expect("ci run should complete");
    sqlx::query("UPDATE workflow_runs SET display_title = 'Run CI manually' WHERE id = $1")
        .bind(ci_success.id)
        .execute(&pool)
        .await
        .expect("display title should update");
    let job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: ci_success.id,
            name: "test".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    sqlx::query(
        "UPDATE workflow_jobs SET status = 'completed', conclusion = 'success' WHERE id = $1",
    )
    .bind(job.id)
    .execute(&pool)
    .await
    .expect("job should update");

    create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: lint.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("fedcba9876543210".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("other workflow run should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let uri = format!(
        "/api/repos/{}/{}/actions/workflows/{}/dashboard?status=success&page=1&pageSize=5",
        owner.email,
        repo_name,
        encoded_workflow_path(".github/workflows/ci.yml")
    );
    let (status, body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["viewerPermission"], "read");
    assert_eq!(body["workflow"]["id"], ci.id.to_string());
    assert_eq!(body["workflow"]["path"], ".github/workflows/ci.yml");
    assert_eq!(body["workflow"]["sourceBranch"], "main");
    assert_eq!(body["workflow"]["sourceSha"], "workflow-source-sha");
    assert_eq!(body["workflow"]["sourceBlobId"], source_blob_id.to_string());
    assert_eq!(
        body["workflow"]["sourceHref"],
        format!(
            "/{owner}/{repo_name}/blob/main/.github/workflows/ci.yml",
            owner = owner.email
        )
    );
    assert_eq!(body["workflow"]["dispatch"]["enabled"], true);
    assert_eq!(
        body["workflow"]["dispatch"]["inputs"][0]["name"],
        "environment"
    );
    assert_eq!(body["workflow"]["dispatch"]["inputs"][0]["type"], "choice");
    assert_eq!(body["workflow"]["valid"], true);
    assert_eq!(body["filters"]["workflow"], Value::Null);
    assert_eq!(
        body["filterOptions"]["workflows"]
            .as_array()
            .expect("workflow options")
            .len(),
        0
    );
    assert_eq!(body["runs"]["total"], 1);
    assert_eq!(body["runs"]["items"][0]["workflowId"], ci.id.to_string());
    assert_eq!(body["runs"]["items"][0]["displayTitle"], "Run CI manually");
    assert_eq!(body["runs"]["items"][0]["jobSummary"]["success"], 1);
    assert_eq!(
        body["refs"]
            .as_array()
            .expect("refs")
            .iter()
            .map(|item| item["shortName"].as_str().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["main", "release", "v1.0.0"]
    );

    let missing_uri = format!(
        "/api/repos/{}/{}/actions/workflows/{}/dashboard",
        owner.email,
        repo_name,
        encoded_workflow_path(".github/workflows/missing.yml")
    );
    let (missing_status, missing_body) = get_json(app, &missing_uri, None).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "not_found");
}

#[tokio::test]
async fn workflow_dispatch_validates_inputs_permissions_refs_and_queues_run() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions workflow dispatch scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-dispatch-owner").await;
    let outsider = create_user(&pool, "actions-dispatch-outsider").await;
    let repo_name = format!("actions-dispatch-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let commit = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, $2, $3, $3, 'Dispatch workflow fixture')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(format!("dispatch-sha-{}", Uuid::new_v4().simple()))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("commit should create");
    let commit_id = commit.get::<Uuid, _>("id");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, 'refs/heads/main', 'branch', $2),
               ($1, 'refs/heads/release', 'branch', $2)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("refs should create");

    let workflow = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "Release".to_owned(),
            path: ".github/workflows/release.yml".to_owned(),
            trigger_events: vec!["workflow_dispatch".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    sqlx::query(
        r#"
        UPDATE actions_workflows
        SET source_branch = 'main',
            dispatch_enabled = true,
            dispatch_inputs = $2
        WHERE id = $1
        "#,
    )
    .bind(workflow.id)
    .bind(json!([
        {
            "name": "environment",
            "type": "choice",
            "label": "Environment",
            "description": "Deployment target",
            "required": true,
            "default": "staging",
            "options": ["staging", "production"]
        },
        {
            "name": "reason",
            "type": "string",
            "label": "Reason",
            "description": null,
            "required": true,
            "default": null,
            "options": []
        },
        {
            "name": "dryRun",
            "type": "boolean",
            "label": "Dry run",
            "description": null,
            "required": false,
            "default": "true",
            "options": []
        }
    ]))
    .execute(&pool)
    .await
    .expect("workflow dispatch metadata should update");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/workflows/{}/dispatches",
        owner.email,
        repo_name,
        encoded_workflow_path(".github/workflows/release.yml")
    );
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let (outsider_status, outsider_body) = post_json(
        app.clone(),
        &uri,
        Some(&outsider_cookie),
        json!({ "ref": "main", "inputs": { "environment": "staging" } }),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_body["error"]["code"], "forbidden");

    let (missing_input_status, missing_input_body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "ref": "main", "inputs": { "dryRun": true } }),
    )
    .await;
    assert_eq!(missing_input_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(missing_input_body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("reason"));

    let (choice_status, choice_body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "ref": "main", "inputs": { "reason": "ship", "environment": "qa" } }),
    )
    .await;
    assert_eq!(choice_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(choice_body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("staging, production"));

    let (bad_ref_status, bad_ref_body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({ "ref": "missing", "inputs": { "reason": "ship", "environment": "staging" } }),
    )
    .await;
    assert_eq!(bad_ref_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(bad_ref_body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("unknown ref"));

    let (created_status, created_body) = post_json(
        app.clone(),
        &uri,
        Some(&owner_cookie),
        json!({
            "ref": "release",
            "inputs": { "reason": "ship", "environment": "production", "dryRun": false }
        }),
    )
    .await;
    assert_eq!(created_status, StatusCode::CREATED);
    assert_eq!(created_body["workflowId"], workflow.id.to_string());
    assert_eq!(created_body["runNumber"], 1);
    assert_eq!(created_body["event"], "workflow_dispatch");
    assert_eq!(created_body["headBranch"], "release");
    assert_eq!(created_body["displayTitle"], "Run Release manually");
    assert_eq!(created_body["statusCategory"], "queued");
    assert_eq!(created_body["jobSummary"]["queued"], 1);

    let run_id =
        Uuid::parse_str(created_body["id"].as_str().expect("run id")).expect("valid run id");
    let job_payload = sqlx::query(
        r#"
        SELECT job_leases.payload, workflow_jobs.name AS job_name
        FROM job_leases
        JOIN workflow_runs ON workflow_runs.id = (job_leases.payload->>'runId')::uuid
        JOIN workflow_jobs ON workflow_jobs.run_id = workflow_runs.id
        WHERE job_leases.queue = 'actions.workflow_dispatch'
          AND workflow_runs.id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("dispatch job lease and workflow job should exist");
    let payload = job_payload.get::<Value, _>("payload");
    assert_eq!(payload["inputs"]["environment"], "production");
    assert_eq!(payload["inputs"]["reason"], "ship");
    assert_eq!(payload["inputs"]["dryRun"], false);
    assert_eq!(
        job_payload.get::<String, _>("job_name"),
        "workflow dispatch"
    );
}

#[tokio::test]
async fn workflow_detail_preserves_private_permissions_and_invalid_yaml_state() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions workflow detail authz scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-invalid-owner").await;
    let outsider = create_user(&pool, "actions-invalid-outsider").await;
    let repo_name = format!("actions-invalid-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let workflow = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "Broken".to_owned(),
            path: ".github/workflows/broken.yml".to_owned(),
            trigger_events: vec!["workflow_dispatch".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    sqlx::query(
        r#"
        UPDATE actions_workflows
        SET yaml_parse_error = 'mapping values are not allowed here
stack backtrace:
at crates/api/src/domain/actions.rs:42',
            dispatch_enabled = false,
            source_branch = 'trunk',
            updated_at = '2026-05-01T10:11:12Z'
        WHERE id = $1
        "#,
    )
    .bind(workflow.id)
    .execute(&pool)
    .await
    .expect("workflow should mark invalid");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/workflows/{}/dashboard",
        owner.email,
        repo_name,
        encoded_workflow_path(".github/workflows/broken.yml")
    );
    let (anonymous_status, anonymous_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(anonymous_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_body["error"]["code"], "forbidden");
    assert!(
        !anonymous_body.to_string().contains(&repo_name),
        "private repository metadata must not leak in forbidden responses"
    );

    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let (outsider_status, _) = get_json(app.clone(), &uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let (owner_status, owner_body) = get_json(app, &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewerPermission"], "owner");
    assert_eq!(owner_body["workflow"]["valid"], false);
    assert_eq!(
        owner_body["workflow"]["yamlParseError"],
        "mapping values are not allowed here"
    );
    assert!(
        owner_body["workflow"]["yamlParsedAt"]
            .as_str()
            .is_some_and(|value| value.ends_with('Z')),
        "workflow detail should expose a stable parse timestamp"
    );
    assert!(
        !owner_body["workflow"]["yamlParseError"]
            .to_string()
            .contains("stack backtrace"),
        "parse errors should not expose raw stack traces"
    );
    assert_eq!(owner_body["workflow"]["dispatch"]["enabled"], false);
    assert_eq!(owner_body["workflow"]["sourceBranch"], "trunk");
    assert_eq!(
        owner_body["emptyState"]["message"],
        "This workflow has not run yet."
    );
}
