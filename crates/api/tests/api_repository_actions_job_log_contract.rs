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

async fn get_text(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
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
    (
        status,
        String::from_utf8(bytes.to_vec()).expect("body should be utf8"),
    )
}

async fn seed_run_with_job_logs(
    pool: &PgPool,
    owner: &User,
    visibility: RepositoryVisibility,
    label: &str,
) -> (String, Uuid, Uuid, Uuid) {
    let repo_name = format!("{label}-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let workflow = create_workflow(
        pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "CI".to_owned(),
            path: ".github/workflows/ci.yml".to_owned(),
            trigger_events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("workflow should create");
    let run = create_workflow_run(
        pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("abcdef0123456789".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("run should create");
    transition_workflow_run(
        pool,
        run.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Failure),
        },
    )
    .await
    .expect("run should complete");
    sqlx::query("UPDATE workflow_runs SET display_title = 'Inspect job logs' WHERE id = $1")
        .bind(run.id)
        .execute(pool)
        .await
        .expect("run title should update");

    let job = create_workflow_job(
        pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "unit / web".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    let sibling = create_workflow_job(
        pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "lint".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("sibling job should create");
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'completed',
            conclusion = CASE WHEN id = $1 THEN 'failure' ELSE 'success' END,
            group_name = 'Checks',
            log_storage_key = 'actions/logs/job-detail.txt',
            started_at = now() - interval '4 minutes',
            completed_at = now() - interval '1 minute'
        WHERE id IN ($1, $2)
        "#,
    )
    .bind(job.id)
    .bind(sibling.id)
    .execute(pool)
    .await
    .expect("jobs should update");
    let setup = create_workflow_step(
        pool,
        CreateWorkflowStep {
            job_id: job.id,
            number: 1,
            name: "Install dependencies".to_owned(),
        },
    )
    .await
    .expect("setup step should create");
    let test = create_workflow_step(
        pool,
        CreateWorkflowStep {
            job_id: job.id,
            number: 2,
            name: "Run tests".to_owned(),
        },
    )
    .await
    .expect("test step should create");
    sqlx::query(
        r#"
        UPDATE workflow_steps
        SET status = 'completed',
            conclusion = CASE WHEN id = $2 THEN 'failure' ELSE 'success' END,
            started_at = now() - interval '3 minutes',
            completed_at = now() - interval '1 minute'
        WHERE id IN ($1, $2)
        "#,
    )
    .bind(setup.id)
    .bind(test.id)
    .execute(pool)
    .await
    .expect("steps should update");
    sqlx::query(
        r#"
        INSERT INTO workflow_job_log_lines (job_id, step_id, line_number, timestamp, content)
        VALUES
            ($1, $2, 1, now() - interval '3 minutes', 'Installing dependencies'),
            ($1, $2, 2, now() - interval '2 minutes', 'Dependencies restored'),
            ($1, $3, 3, now() - interval '1 minute', 'error: expected string, found number'),
            ($1, NULL, 4, now(), 'Post job cleanup complete')
        "#,
    )
    .bind(job.id)
    .bind(setup.id)
    .bind(test.id)
    .execute(pool)
    .await
    .expect("log lines should create");
    sqlx::query(
        r#"
        INSERT INTO workflow_annotations (
            run_id, job_id, step_id, annotation_level, path, start_line, end_line, title, message
        )
        VALUES ($1, $2, $3, 'failure', 'web/src/app/page.tsx', 42, 42, 'Type error', 'Expected string, found number')
        "#,
    )
    .bind(run.id)
    .bind(job.id)
    .bind(test.id)
    .execute(pool)
    .await
    .expect("annotation should create");

    (repo_name, repository.id, run.id, job.id)
}

#[tokio::test]
async fn job_log_detail_groups_steps_searches_lines_and_reads_options() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions job log scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-job-log-owner").await;
    let (repo_name, repository_id, run_id, job_id) = seed_run_with_job_logs(
        &pool,
        &owner,
        RepositoryVisibility::Public,
        "actions-job-log",
    )
    .await;
    sqlx::query(
        r#"
        INSERT INTO actions_log_preferences (repository_id, user_id, show_timestamps, raw_logs, wrap_lines)
        VALUES ($1, $2, false, true, true)
        "#,
    )
    .bind(repository_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("preferences should persist");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let detail_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/jobs/{}/detail",
        owner.email, repo_name, run_id, job_id
    );
    let (status, body) = get_json(app.clone(), &detail_uri, Some(&owner_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["viewerPermission"], "owner");
    assert_eq!(body["run"]["displayTitle"], "Inspect job logs");
    assert_eq!(body["job"]["name"], "unit / web");
    assert_eq!(body["jobs"].as_array().expect("jobs").len(), 2);
    assert_eq!(body["logState"]["available"], true);
    assert_eq!(body["logState"]["status"], 200);
    assert_eq!(body["logState"]["nextCursor"], 4);
    assert_eq!(body["options"]["showTimestamps"], false);
    assert_eq!(body["options"]["rawLogs"], true);
    assert_eq!(body["options"]["wrapLines"], true);
    assert_eq!(body["steps"][0]["name"], "Job log");
    assert_eq!(body["steps"][0]["lines"]["items"][0]["lineNumber"], 4);
    assert_eq!(body["steps"][1]["name"], "Install dependencies");
    assert_eq!(body["steps"][1]["lines"]["total"], 2);
    assert_eq!(body["steps"][2]["name"], "Run tests");
    assert_eq!(
        body["annotations"][0]["message"],
        "Expected string, found number"
    );
    assert!(body["downloadHref"]
        .as_str()
        .expect("download href")
        .ends_with("/logs/download"));
    assert!(body["runArchiveHref"]
        .as_str()
        .expect("archive href")
        .ends_with("/logs/archive"));

    let search_uri = format!("{detail_uri}?q=error&match=1&timestamps=true&raw=false");
    let (search_status, search_body) =
        get_json(app.clone(), &search_uri, Some(&owner_cookie)).await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(search_body["search"]["query"], "error");
    assert_eq!(search_body["search"]["totalMatches"], 1);
    assert_eq!(search_body["search"]["selectedMatch"], 1);
    assert_eq!(search_body["search"]["matches"][0]["lineNumber"], 3);
    assert_eq!(search_body["search"]["matches"][0]["stepNumber"], 2);
    assert_eq!(search_body["steps"][1]["matchCount"], 1);
    assert_eq!(search_body["steps"][1]["lines"]["items"][0]["anchor"], "L3");
    assert_eq!(search_body["options"]["showTimestamps"], true);
    assert_eq!(search_body["options"]["rawLogs"], false);

    let wrong_run_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/jobs/{}/detail",
        owner.email,
        repo_name,
        Uuid::new_v4(),
        job_id
    );
    let (wrong_run_status, wrong_run_body) =
        get_json(app.clone(), &wrong_run_uri, Some(&owner_cookie)).await;
    assert_eq!(wrong_run_status, StatusCode::NOT_FOUND);
    assert_eq!(wrong_run_body["error"]["code"], "not_found");

    let preferences_uri = format!(
        "/api/repos/{}/{}/actions/log-preferences",
        owner.email, repo_name
    );
    let (preferences_status, preferences_body) = patch_json(
        app.clone(),
        &preferences_uri,
        Some(&owner_cookie),
        json!({
            "showTimestamps": true,
            "rawLogs": false,
            "wrapLines": false
        }),
    )
    .await;
    assert_eq!(preferences_status, StatusCode::OK);
    assert_eq!(preferences_body["showTimestamps"], true);
    assert_eq!(preferences_body["rawLogs"], false);
    assert_eq!(preferences_body["wrapLines"], false);
    let (persisted_status, persisted_body) =
        get_json(app.clone(), &detail_uri, Some(&owner_cookie)).await;
    assert_eq!(persisted_status, StatusCode::OK);
    assert_eq!(persisted_body["options"]["showTimestamps"], true);
    assert_eq!(persisted_body["options"]["rawLogs"], false);
    assert_eq!(persisted_body["options"]["wrapLines"], false);

    let archive_uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/logs/archive",
        owner.email, repo_name, run_id
    );
    let (archive_status, archive_body) =
        get_text(app.clone(), &archive_uri, Some(&owner_cookie)).await;
    assert_eq!(archive_status, StatusCode::OK);
    assert!(archive_body.contains("opengithub workflow log archive"));
    assert!(archive_body.contains("unit / web"));
    assert!(archive_body.contains("expected string"));

    sqlx::query("UPDATE workflow_jobs SET log_deleted_at = now() WHERE run_id = $1")
        .bind(run_id)
        .execute(&pool)
        .await
        .expect("log should delete");
    let (deleted_archive_status, deleted_archive_body) =
        get_json(app.clone(), &archive_uri, Some(&owner_cookie)).await;
    assert_eq!(deleted_archive_status, StatusCode::GONE);
    assert_eq!(deleted_archive_body["error"]["code"], "gone");

    let (deleted_status, deleted_body) = get_json(app, &detail_uri, Some(&owner_cookie)).await;
    assert_eq!(deleted_status, StatusCode::OK);
    assert_eq!(deleted_body["logState"]["available"], false);
    assert_eq!(deleted_body["logState"]["status"], 410);
    assert_eq!(deleted_body["search"]["totalMatches"], 0);
}

#[tokio::test]
async fn job_log_detail_preserves_private_repository_permissions() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions job log private scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-job-log-private-owner").await;
    let outsider = create_user(&pool, "actions-job-log-private-outsider").await;
    let (repo_name, _repository_id, run_id, job_id) = seed_run_with_job_logs(
        &pool,
        &owner,
        RepositoryVisibility::Private,
        "actions-job-private",
    )
    .await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/runs/{}/jobs/{}/detail",
        owner.email, repo_name, run_id, job_id
    );
    let (anonymous_status, anonymous_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(anonymous_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_body["error"]["code"], "forbidden");

    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let (outsider_status, outsider_body) =
        get_json(app.clone(), &uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_body["error"]["code"], "forbidden");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let (owner_status, owner_body) = get_json(app, &uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewerPermission"], "owner");
    assert_eq!(owner_body["logState"]["available"], true);
}
