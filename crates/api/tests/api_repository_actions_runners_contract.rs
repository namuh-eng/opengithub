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
            create_workflow, create_workflow_job, create_workflow_run, CreateWorkflow,
            CreateWorkflowJob, CreateWorkflowRun,
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
    upsert_session(pool, &session_id, Some(user.id), json!({}), expires_at)
        .await
        .expect("session should persist");
    let set_cookie =
        session::set_cookie_header(config, &session_id, expires_at).expect("cookie should sign");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn json_request(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header(
        "x-forwarded-for",
        format!("198.51.100.{}", Uuid::new_v4().as_u128() % 250 + 1),
    );
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should build");
    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn actions_runners_register_heartbeat_and_schedule_matching_jobs() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions runners scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let config = app_config();
    let owner = create_user(&pool, "actions-runner-owner").await;
    let repo_name = format!("actions-runners-{}", Uuid::new_v4().simple());
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
            name: "CI".to_owned(),
            path: ".github/workflows/ci.yml".to_owned(),
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
            head_sha: Some("abcdef1234567890".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("run should create");
    let job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "build".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");

    let cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let uri = format!(
        "/api/repos/{}/{}/settings/actions/runners",
        owner.email, repo_name
    );
    let (create_status, create_body) = json_request(
        app.clone(),
        Method::POST,
        &uri,
        Some(&cookie),
        json!({ "name": "linux-build-1", "labels": ["self-hosted", "ubuntu-latest"] }),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK);
    assert_eq!(create_body["runners"][0]["name"], "linux-build-1");
    assert!(!create_body["setup"]["registrationToken"]
        .as_str()
        .expect("token")
        .is_empty());
    let runner_token = create_body["setup"]["registrationToken"]
        .as_str()
        .expect("token")
        .to_owned();
    assert_eq!(
        create_body["workflowPermissions"]["githubTokenPermission"],
        "read"
    );
    let runner_id = Uuid::parse_str(create_body["runners"][0]["id"].as_str().expect("runner id"))
        .expect("runner uuid");

    let (settings_status, settings_body) = json_request(
        app.clone(),
        Method::PATCH,
        &uri,
        Some(&cookie),
        json!({
            "concurrencyLimit": 8,
            "cancelInProgress": true,
            "githubTokenPermission": "write",
            "allowPullRequestApproval": true
        }),
    )
    .await;
    assert_eq!(settings_status, StatusCode::OK);
    assert_eq!(settings_body["queue"]["concurrencyLimit"], 8);
    assert_eq!(
        settings_body["workflowPermissions"]["githubTokenPermission"],
        "write"
    );
    assert_eq!(
        settings_body["workflowPermissions"]["allowPullRequestApproval"],
        true
    );
    assert!(settings_body["workflowPermissions"]["githubTokenScopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .any(|scope| scope == "pull-requests:approve"));

    let heartbeat_uri = format!("{uri}/heartbeat");
    let (heartbeat_status, heartbeat_body) = json_request(
        app.clone(),
        Method::POST,
        &heartbeat_uri,
        None,
        json!({ "runnerId": runner_id, "runnerToken": runner_token, "status": "online" }),
    )
    .await;
    assert_eq!(heartbeat_status, StatusCode::OK);
    assert_eq!(heartbeat_body["status"], "online");

    let (schedule_status, schedule_body) = json_request(
        app.clone(),
        Method::POST,
        &format!("{uri}/schedule"),
        Some(&cookie),
        json!({}),
    )
    .await;
    assert_eq!(schedule_status, StatusCode::OK);
    assert_eq!(schedule_body["assigned"][0]["jobId"], job.id.to_string());
    assert_eq!(schedule_body["queuedJobs"], 0);

    let row = sqlx::query(
        "SELECT workflow_jobs.status, actions_runners.status AS runner_status
         FROM workflow_jobs JOIN actions_runners ON actions_runners.id = workflow_jobs.runner_id
         WHERE workflow_jobs.id = $1",
    )
    .bind(job.id)
    .fetch_one(&pool)
    .await
    .expect("assigned job should query");
    assert_eq!(row.get::<String, _>("status"), "in_progress");
    assert_eq!(row.get::<String, _>("runner_status"), "busy");

    let (unauth_status, unauth_body) =
        json_request(app.clone(), Method::GET, &uri, None, json!({})).await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unauth_body["error"]["code"], "not_authenticated");

    sqlx::query(
        "UPDATE actions_runners
         SET last_heartbeat = now() - interval '10 minutes', busy_since = now() - interval '10 minutes'
         WHERE id = $1",
    )
    .bind(runner_id)
    .execute(&pool)
    .await
    .expect("runner should age");
    sqlx::query(
        "UPDATE workflow_jobs
         SET assigned_at = now() - interval '10 minutes'
         WHERE id = $1",
    )
    .bind(job.id)
    .execute(&pool)
    .await
    .expect("job assignment should age");
    sqlx::query(
        "UPDATE workflow_job_assignments
         SET started_at = now() - interval '10 minutes'
         WHERE job_id = $1",
    )
    .bind(job.id)
    .execute(&pool)
    .await
    .expect("assignment should age");

    let (timeout_status, timeout_body) =
        json_request(app.clone(), Method::GET, &uri, Some(&cookie), json!({})).await;
    assert_eq!(timeout_status, StatusCode::OK);
    assert_eq!(timeout_body["runners"][0]["status"], "offline");
    let timed_out = sqlx::query(
        "SELECT workflow_runs.status AS run_status, workflow_runs.conclusion AS run_conclusion,
                workflow_jobs.status AS job_status, workflow_jobs.conclusion AS job_conclusion,
                workflow_job_assignments.status AS assignment_status
         FROM workflow_runs
         JOIN workflow_jobs ON workflow_jobs.run_id = workflow_runs.id
         JOIN workflow_job_assignments ON workflow_job_assignments.job_id = workflow_jobs.id
         WHERE workflow_jobs.id = $1",
    )
    .bind(job.id)
    .fetch_one(&pool)
    .await
    .expect("timed out run should query");
    assert_eq!(timed_out.get::<String, _>("run_status"), "completed");
    assert_eq!(timed_out.get::<String, _>("run_conclusion"), "failure");
    assert_eq!(timed_out.get::<String, _>("job_status"), "completed");
    assert_eq!(timed_out.get::<String, _>("job_conclusion"), "failure");
    assert_eq!(timed_out.get::<String, _>("assignment_status"), "timed_out");

    let (online_status, _) = json_request(
        app.clone(),
        Method::POST,
        &heartbeat_uri,
        None,
        json!({ "runnerId": runner_id, "runnerToken": runner_token, "status": "online" }),
    )
    .await;
    assert_eq!(online_status, StatusCode::OK);

    let first_run = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("1111111111111111".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("first concurrent run should create");
    let first_job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: first_run.id,
            name: "build".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("first concurrent job should create");
    sqlx::query("UPDATE workflow_runs SET concurrency_group = 'ci-main' WHERE id = $1")
        .bind(first_run.id)
        .execute(&pool)
        .await
        .expect("first run concurrency group should update");
    sqlx::query("UPDATE workflow_jobs SET group_name = 'ci-main' WHERE id = $1")
        .bind(first_job.id)
        .execute(&pool)
        .await
        .expect("first job group should update");

    let (first_schedule_status, first_schedule_body) = json_request(
        app.clone(),
        Method::POST,
        &format!("{uri}/schedule"),
        Some(&cookie),
        json!({}),
    )
    .await;
    assert_eq!(first_schedule_status, StatusCode::OK);
    assert_eq!(
        first_schedule_body["assigned"][0]["jobId"],
        first_job.id.to_string()
    );

    let second_run = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("2222222222222222".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("second concurrent run should create");
    let second_job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: second_run.id,
            name: "build".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("second concurrent job should create");
    sqlx::query("UPDATE workflow_runs SET concurrency_group = 'ci-main' WHERE id = $1")
        .bind(second_run.id)
        .execute(&pool)
        .await
        .expect("second run concurrency group should update");
    sqlx::query("UPDATE workflow_jobs SET group_name = 'ci-main' WHERE id = $1")
        .bind(second_job.id)
        .execute(&pool)
        .await
        .expect("second job group should update");

    let (cancel_schedule_status, cancel_schedule_body) = json_request(
        app.clone(),
        Method::POST,
        &format!("{uri}/schedule"),
        Some(&cookie),
        json!({}),
    )
    .await;
    assert_eq!(cancel_schedule_status, StatusCode::OK);
    assert_eq!(
        cancel_schedule_body["assigned"][0]["jobId"],
        second_job.id.to_string()
    );
    let cancelled = sqlx::query(
        "SELECT first_run.status AS first_run_status, first_run.conclusion AS first_run_conclusion,
                first_job.status AS first_job_status, first_job.conclusion AS first_job_conclusion,
                second_job.status AS second_job_status
         FROM workflow_runs first_run
         JOIN workflow_jobs first_job ON first_job.run_id = first_run.id
         CROSS JOIN workflow_jobs second_job
         WHERE first_run.id = $1 AND second_job.id = $2",
    )
    .bind(first_run.id)
    .bind(second_job.id)
    .fetch_one(&pool)
    .await
    .expect("cancel-in-progress rows should query");
    assert_eq!(cancelled.get::<String, _>("first_run_status"), "cancelled");
    assert_eq!(
        cancelled.get::<String, _>("first_run_conclusion"),
        "cancelled"
    );
    assert_eq!(cancelled.get::<String, _>("first_job_status"), "completed");
    assert_eq!(
        cancelled.get::<String, _>("first_job_conclusion"),
        "cancelled"
    );
    assert_eq!(
        cancelled.get::<String, _>("second_job_status"),
        "in_progress"
    );
}
