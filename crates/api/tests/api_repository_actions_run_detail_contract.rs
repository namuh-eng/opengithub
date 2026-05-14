use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use flate2::read::GzDecoder;
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        actions::{
            create_workflow, create_workflow_job, create_workflow_run, create_workflow_step,
            transition_workflow_run, CreateWorkflow, CreateWorkflowJob, CreateWorkflowRun,
            CreateWorkflowStep, RunConclusion, RunStatus, TransitionRun,
        },
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
    },
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::io::Read;
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
    reset_rate_limit_subject(&pool, "ip", "unknown").await;
    Some(pool)
}

async fn reset_rate_limit_subject(pool: &PgPool, subject_type: &str, subject_key: &str) {
    sqlx::query(
        r#"
        DELETE FROM rate_limit_buckets
        WHERE subject_type = $1
          AND subject_key = $2
        "#,
    )
    .bind(subject_type)
    .bind(subject_key)
    .execute(pool)
    .await
    .expect("test rate limit bucket should reset");
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

async fn get_text(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder.body(Body::empty()).expect("request should build");
    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let is_gzip = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value == "application/gzip")
        .unwrap_or(false);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body_bytes = if is_gzip {
        let mut decoder = GzDecoder::new(bytes.as_ref());
        let mut decoded = String::new();
        decoder
            .read_to_string(&mut decoded)
            .expect("gzip response should decode as utf8");
        return (status, decoded);
    } else {
        bytes.to_vec()
    };
    (
        status,
        String::from_utf8(body_bytes).expect("response should be utf8"),
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

async fn delete_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(Method::DELETE).uri(uri);
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
async fn run_detail_returns_attempts_jobs_annotations_artifacts_and_action_state() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions run detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-run-owner").await;
    let repo_name = format!("actions-run-detail-{}", Uuid::new_v4().simple());
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

    let commit_id = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, 'abcdef0123456789', $2, $2, 'Add run detail fixture')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("commit should create")
    .get::<Uuid, _>("id");

    let workflow = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "CI".to_owned(),
            path: ".github/workflows/ci.yml".to_owned(),
            trigger_events: vec!["push".to_owned(), "pull_request".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    sqlx::query("UPDATE actions_workflows SET source_branch = 'main', source_sha = 'workflow-sha' WHERE id = $1")
        .bind(workflow.id)
        .execute(&pool)
        .await
        .expect("workflow source metadata should update");

    let run = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "feature/actions".to_owned(),
            head_sha: Some("abcdef0123456789".to_owned()),
            event: "pull_request".to_owned(),
        },
    )
    .await
    .expect("run should create");
    transition_workflow_run(
        &pool,
        run.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Failure),
        },
    )
    .await
    .expect("run should complete");
    sqlx::query(
        "UPDATE workflow_runs SET display_title = 'Validate run detail', commit_id = $2 WHERE id = $1",
    )
    .bind(run.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("run metadata should update");
    sqlx::query(
        r#"
        INSERT INTO workflow_run_attempts (
            run_id, attempt_number, status, conclusion, triggered_by_user_id, trigger_kind,
            started_at, completed_at
        )
        VALUES
            ($1, 1, 'completed', 'failure', $2, 'initial', now() - interval '10 minutes', now() - interval '8 minutes'),
            ($1, 2, 'completed', 'failure', $2, 'rerun_failed', now() - interval '4 minutes', now() - interval '1 minute')
        "#,
    )
    .bind(run.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("attempts should create");

    let job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "unit / web".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'completed',
            conclusion = 'failure',
            group_name = 'Checks',
            attempt_number = 2,
            log_storage_key = 'actions/logs/unit-web.txt',
            started_at = now() - interval '4 minutes',
            completed_at = now() - interval '1 minute'
        WHERE id = $1
        "#,
    )
    .bind(job.id)
    .execute(&pool)
    .await
    .expect("job should update");
    sqlx::query(
        r#"
        INSERT INTO workflow_job_log_lines (job_id, line_number, timestamp, content)
        VALUES
            ($1, 1, now() - interval '3 minutes', 'Installing dependencies'),
            ($1, 2, now() - interval '2 minutes', 'Running unit tests'),
            ($1, 3, now() - interval '1 minute', 'error: expected string, found number')
        "#,
    )
    .bind(job.id)
    .execute(&pool)
    .await
    .expect("job logs should create");
    let step = create_workflow_step(
        &pool,
        CreateWorkflowStep {
            job_id: job.id,
            number: 1,
            name: "Run tests".to_owned(),
        },
    )
    .await
    .expect("step should create");
    sqlx::query(
        r#"
        UPDATE workflow_steps
        SET status = 'completed',
            conclusion = 'failure',
            started_at = now() - interval '3 minutes',
            completed_at = now() - interval '1 minute'
        WHERE id = $1
        "#,
    )
    .bind(step.id)
    .execute(&pool)
    .await
    .expect("step should update");
    sqlx::query(
        r#"
        INSERT INTO workflow_annotations (
            run_id, job_id, step_id, annotation_level, path, start_line, end_line, title, message, raw_details
        )
        VALUES ($1, $2, $3, 'failure', 'web/src/app/page.tsx', 42, 42, 'Type error', 'Expected string, found number', 'tsc failed')
        "#,
    )
    .bind(run.id)
    .bind(job.id)
    .bind(step.id)
    .execute(&pool)
    .await
    .expect("annotation should create");
    sqlx::query(
        r#"
        INSERT INTO workflow_artifacts (run_id, name, digest, size_bytes, storage_key, expired_at)
        VALUES ($1, 'playwright-report', 'sha256:abc123', 2048, 'actions/artifacts/report.zip', now() + interval '1 day')
        "#,
    )
    .bind(run.id)
    .execute(&pool)
    .await
    .expect("artifact should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/detail",
        owner.email, repo_name, run.id
    );
    let (public_status, public_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(public_status, StatusCode::OK, "{public_body:?}");
    assert_eq!(public_body["repository"]["name"], repo_name);
    assert_eq!(public_body["viewerPermission"], "read");
    assert_eq!(public_body["workflow"]["name"], "CI");
    assert_eq!(
        public_body["workflow"]["sourceHref"],
        format!(
            "/{}/{}/blob/main/.github/workflows/ci.yml",
            owner.username.as_deref().expect("owner username"),
            repo_name
        )
    );
    assert_eq!(public_body["run"]["displayTitle"], "Validate run detail");
    assert_eq!(public_body["run"]["statusCategory"], "failure");
    assert_eq!(public_body["run"]["shortSha"], "abcdef0");
    assert_eq!(public_body["run"]["jobSummary"]["failure"], 1);
    assert_eq!(
        public_body["attempts"].as_array().expect("attempts").len(),
        2
    );
    assert_eq!(public_body["attempts"][1]["triggerKind"], "rerun_failed");
    assert_eq!(public_body["jobs"][0]["groupName"], "Checks");
    assert_eq!(public_body["jobs"][0]["attemptNumber"], 2);
    assert_eq!(public_body["jobs"][0]["logAvailable"], true);
    assert_eq!(public_body["jobs"][0]["steps"][0]["name"], "Run tests");
    assert_eq!(public_body["annotations"][0]["level"], "failure");
    assert_eq!(
        public_body["annotations"][0]["message"],
        "Expected string, found number"
    );
    assert_eq!(public_body["artifacts"][0]["name"], "playwright-report");
    assert_eq!(public_body["artifacts"][0]["downloadAvailable"], true);
    assert_eq!(public_body["actionState"]["canRerun"], false);
    assert_eq!(public_body["actionState"]["canRerunFailed"], false);
    assert_eq!(public_body["actionState"]["canDeleteLogs"], false);

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let (owner_status, owner_body) = get_json(app.clone(), &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewerPermission"], "owner");
    assert_eq!(owner_body["actionState"]["canRerun"], true);
    assert_eq!(owner_body["actionState"]["canRerunFailed"], true);
    assert_eq!(owner_body["actionState"]["canDeleteLogs"], true);
    assert_eq!(owner_body["actionState"]["canCancel"], false);

    let logs_uri = format!(
        "/api/repos/{}/{}/actions/jobs/{}/logs?q=error",
        owner.email, repo_name, job.id
    );
    let (logs_status, logs_body) = get_json(app.clone(), &logs_uri, None).await;
    assert_eq!(logs_status, StatusCode::OK);
    assert_eq!(logs_body["total"], 1);
    assert_eq!(logs_body["lines"][0]["lineNumber"], 3);
    assert_eq!(logs_body["lines"][0]["anchor"], "L3");

    let download_uri = format!(
        "/api/repos/{}/{}/actions/jobs/{}/logs/download",
        owner.email, repo_name, job.id
    );
    let (download_status, download_body) = get_text(app.clone(), &download_uri, None).await;
    assert_eq!(download_status, StatusCode::OK);
    assert!(download_body.contains("Running unit tests"));

    let artifact_id = owner_body["artifacts"][0]["id"]
        .as_str()
        .expect("artifact id");
    let artifact_uri = format!(
        "/api/repos/{}/{}/actions/artifacts/{}/download",
        owner.email, repo_name, artifact_id
    );
    let (artifact_status, artifact_body) = get_json(app.clone(), &artifact_uri, None).await;
    assert_eq!(artifact_status, StatusCode::OK);
    assert_eq!(artifact_body["filename"], "playwright-report.zip");
    assert!(artifact_body["downloadUrl"]
        .as_str()
        .expect("download url")
        .contains(artifact_id));
    assert!(
        artifact_body.get("storageKey").is_none(),
        "artifact download response must not expose internal storage keys: {artifact_body:?}"
    );

    let artifact_list_uri = format!("/_apis/pipelines/workflows/{}/artifacts", run.id);
    let (artifact_list_unauth_status, artifact_list_unauth_body) =
        get_json(app.clone(), &artifact_list_uri, None).await;
    assert_eq!(artifact_list_unauth_status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        artifact_list_unauth_body["error"]["code"],
        "not_authenticated"
    );
    let (artifact_list_status, artifact_list_body) =
        get_json(app.clone(), &artifact_list_uri, Some(&owner_cookie)).await;
    assert_eq!(
        artifact_list_status,
        StatusCode::OK,
        "{artifact_list_body:?}"
    );
    assert_eq!(artifact_list_body["count"], 1);
    assert_eq!(
        artifact_list_body["artifacts"][0]["name"],
        "playwright-report"
    );
    assert!(
        artifact_list_body["artifacts"][0]
            .get("storageKey")
            .is_none(),
        "artifact list response must not expose internal storage keys: {artifact_list_body:?}"
    );

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let upload_uri = format!("/_apis/pipelines/workflows/{}/artifacts", run.id);
    let (upload_status, upload_body) = post_json(
        app.clone(),
        &upload_uri,
        Some(&owner_cookie),
        json!({
            "name": "coverage-report",
            "sizeBytes": 4096,
            "digest": "sha256:def456",
            "retentionDays": 14
        }),
    )
    .await;
    assert_eq!(upload_status, StatusCode::OK);
    assert_eq!(upload_body["name"], "coverage-report");
    assert_eq!(upload_body["retentionDays"], 14);
    assert_eq!(upload_body["downloadAvailable"], true);

    let delete_artifact_uri = format!(
        "/api/repos/{}/{}/actions/artifacts/{}",
        owner.email,
        repo_name,
        upload_body["id"].as_str().expect("uploaded artifact id")
    );
    let (delete_artifact_status, delete_artifact_body) =
        delete_json(app.clone(), &delete_artifact_uri, Some(&owner_cookie)).await;
    assert_eq!(delete_artifact_status, StatusCode::OK);
    assert_eq!(delete_artifact_body["downloadAvailable"], false);

    let cache_uri = format!("/_apis/artifactcache/cache/{}/{}", owner.email, repo_name);
    let (cache_status, cache_body) = post_json(
        app.clone(),
        &cache_uri,
        Some(&owner_cookie),
        json!({
            "key": "node-linux-lock",
            "version": "v1-main",
            "sizeBytes": 2097152,
            "scope": "refs/heads/main"
        }),
    )
    .await;
    assert_eq!(cache_status, StatusCode::OK);
    assert_eq!(cache_body["key"], "node-linux-lock");
    assert_eq!(cache_body["sizeBytes"], 2_097_152);
    assert!(
        cache_body.get("storageKey").is_none(),
        "cache reserve response must not expose internal storage keys: {cache_body:?}"
    );

    let list_cache_uri = format!("/api/repos/{}/{}/actions/caches", owner.email, repo_name);
    let (list_cache_status, list_cache_body) = get_json(app.clone(), &list_cache_uri, None).await;
    assert_eq!(list_cache_status, StatusCode::OK, "{list_cache_body:?}");
    assert_eq!(list_cache_body["caches"]["total"], 1);
    assert_eq!(list_cache_body["totalSizeBytes"], 2_097_152);
    assert_eq!(list_cache_body["limitBytes"], 10_737_418_240_i64);
    assert_eq!(list_cache_body["canDelete"], false);

    let delete_cache_uri = format!(
        "/api/repos/{}/{}/actions/caches/{}",
        owner.email,
        repo_name,
        cache_body["id"].as_str().expect("cache id")
    );
    let (delete_cache_status, delete_cache_body) =
        delete_json(app, &delete_cache_uri, Some(&owner_cookie)).await;
    assert_eq!(delete_cache_status, StatusCode::OK);
    assert_eq!(delete_cache_body["key"], "node-linux-lock");
}

#[tokio::test]
async fn run_detail_preserves_private_repository_permissions_and_missing_run_errors() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions run detail authz scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-run-private-owner").await;
    let outsider = create_user(&pool, "actions-run-private-outsider").await;
    let repo_name = format!("actions-run-private-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
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
            name: "Private CI".to_owned(),
            path: ".github/workflows/private.yml".to_owned(),
            trigger_events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    let run = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("private-sha".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("run should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/detail",
        owner.email, repo_name, run.id
    );
    let (anon_status, anon_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(anon_status, StatusCode::FORBIDDEN);
    assert_eq!(anon_body["error"]["code"], "forbidden");

    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let (outsider_status, outsider_body) =
        get_json(app.clone(), &uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_body["error"]["code"], "forbidden");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let missing_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/detail",
        owner.email,
        repo_name,
        Uuid::new_v4()
    );
    let (missing_status, missing_body) = get_json(app, &missing_uri, Some(&owner_cookie)).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "not_found");
}

#[tokio::test]
async fn run_detail_mutations_rerun_cancel_and_delete_logs_are_stateful() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions run mutation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-run-mutate-owner").await;
    let outsider = create_user(&pool, "actions-run-mutate-outsider").await;
    let repo_name = format!("actions-run-mutate-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
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
            name: "Mutation CI".to_owned(),
            path: ".github/workflows/mutate.yml".to_owned(),
            trigger_events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    let run = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("mutate-sha".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("run should create");
    transition_workflow_run(
        &pool,
        run.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Failure),
        },
    )
    .await
    .expect("run should complete");
    sqlx::query(
        r#"
        INSERT INTO workflow_run_attempts (
            run_id, attempt_number, status, conclusion, triggered_by_user_id, trigger_kind
        )
        VALUES ($1, 1, 'completed', 'failure', $2, 'initial')
        "#,
    )
    .bind(run.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("attempt should create");
    let failed_job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "unit".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    let passing_job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "lint".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'completed',
            conclusion = CASE WHEN id = $2 THEN 'failure' ELSE 'success' END,
            group_name = 'Checks',
            log_storage_key = 'actions/logs/mutate.txt'
        WHERE run_id = $1
        "#,
    )
    .bind(run.id)
    .bind(failed_job.id)
    .execute(&pool)
    .await
    .expect("jobs should update");
    create_workflow_step(
        &pool,
        CreateWorkflowStep {
            job_id: failed_job.id,
            number: 1,
            name: "cargo test".to_owned(),
        },
    )
    .await
    .expect("step should create");
    sqlx::query(
        "INSERT INTO workflow_job_log_lines (job_id, line_number, content) VALUES ($1, 1, 'failure log'), ($2, 1, 'passing log')",
    )
    .bind(failed_job.id)
    .bind(passing_job.id)
    .execute(&pool)
    .await
    .expect("logs should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let rerun_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/rerun",
        owner.email, repo_name, run.id
    );

    let (forbidden_status, forbidden_body) =
        post_json(app.clone(), &rerun_uri, Some(&outsider_cookie), json!({})).await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden_body["error"]["code"], "forbidden");

    let (rerun_status, rerun_body) = post_json(
        app.clone(),
        &rerun_uri,
        Some(&owner_cookie),
        json!({ "mode": "failed" }),
    )
    .await;
    assert_eq!(rerun_status, StatusCode::OK);
    assert_eq!(rerun_body["run"]["status"], "queued");
    assert_eq!(rerun_body["attempts"][1]["attemptNumber"], 2);
    assert_eq!(rerun_body["attempts"][1]["triggerKind"], "rerun_failed");
    assert_eq!(
        rerun_body["jobs"]
            .as_array()
            .expect("jobs")
            .iter()
            .filter(|job| job["attemptNumber"] == 2)
            .count(),
        1
    );

    let lease_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM job_leases WHERE queue = 'actions.workflow_rerun' AND lease_key = $1",
    )
    .bind(format!("workflow-rerun:{}:2", run.id))
    .fetch_one(&pool)
    .await
    .expect("lease count should load");
    assert_eq!(lease_count, 1);

    let cancel_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/cancel",
        owner.email, repo_name, run.id
    );
    let (cancel_status, cancel_body) =
        post_json(app.clone(), &cancel_uri, Some(&owner_cookie), json!({})).await;
    assert_eq!(cancel_status, StatusCode::OK);
    assert_eq!(cancel_body["run"]["status"], "cancelled");
    assert_eq!(cancel_body["run"]["conclusion"], "cancelled");
    assert_eq!(cancel_body["actionState"]["canDeleteLogs"], true);

    let logs_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/logs",
        owner.email, repo_name, run.id
    );
    let (delete_status, delete_body) =
        delete_json(app.clone(), &logs_uri, Some(&owner_cookie)).await;
    assert_eq!(delete_status, StatusCode::OK);
    assert!(delete_body["jobs"]
        .as_array()
        .expect("jobs")
        .iter()
        .all(|job| job["logAvailable"] == false));
    let remaining_lines = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM workflow_job_log_lines WHERE job_id IN (SELECT id FROM workflow_jobs WHERE run_id = $1)",
    )
    .bind(run.id)
    .fetch_one(&pool)
    .await
    .expect("line count should load");
    assert_eq!(remaining_lines, 0);

    let audit_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM audit_events
        WHERE target_id = $1
          AND event_type IN ('workflow_run.rerun', 'workflow_run.cancelled', 'workflow_run.logs_deleted')
        "#,
    )
    .bind(run.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 3);
}
