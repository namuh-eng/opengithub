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

#[tokio::test]
async fn actions_dashboard_returns_workflows_runs_filters_and_summaries() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions dashboard scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-owner").await;
    let repo_name = format!("actions-dashboard-{}", Uuid::new_v4().simple());
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

    let ci = create_workflow(
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
    .expect("ci workflow should create");
    let deploy = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "Deploy".to_owned(),
            path: ".github/workflows/deploy.yml".to_owned(),
            trigger_events: vec!["workflow_dispatch".to_owned()],
        },
    )
    .await
    .expect("deploy workflow should create");
    sqlx::query("UPDATE actions_workflows SET pinned_order = 1 WHERE id = $1")
        .bind(ci.id)
        .execute(&pool)
        .await
        .expect("workflow pinned order should update");
    sqlx::query("UPDATE actions_workflows SET state = 'disabled' WHERE id = $1")
        .bind(deploy.id)
        .execute(&pool)
        .await
        .expect("workflow state should update");

    let ci_success = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: ci.id,
            actor_user_id: Some(owner.id),
            head_branch: "main".to_owned(),
            head_sha: Some("abcdef0123456789".to_owned()),
            event: "push".to_owned(),
        },
    )
    .await
    .expect("success run should create");
    transition_workflow_run(
        &pool,
        ci_success.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Success),
        },
    )
    .await
    .expect("success run should complete");
    sqlx::query(
        "UPDATE workflow_runs SET display_title = 'Add Actions dashboard contract' WHERE id = $1",
    )
    .bind(ci_success.id)
    .execute(&pool)
    .await
    .expect("display title should update");
    let success_job = create_workflow_job(
        &pool,
        CreateWorkflowJob {
            run_id: ci_success.id,
            name: "unit".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await
    .expect("job should create");
    sqlx::query(
        "UPDATE workflow_jobs SET status = 'completed', conclusion = 'success' WHERE id = $1",
    )
    .bind(success_job.id)
    .execute(&pool)
    .await
    .expect("job should update");

    let ci_live = create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: ci.id,
            actor_user_id: Some(owner.id),
            head_branch: "feature/actions".to_owned(),
            head_sha: Some("1234567890abcdef".to_owned()),
            event: "pull_request".to_owned(),
        },
    )
    .await
    .expect("live run should create");
    transition_workflow_run(
        &pool,
        ci_live.id,
        TransitionRun {
            status: RunStatus::InProgress,
            conclusion: None,
        },
    )
    .await
    .expect("live run should start");

    create_workflow_run(
        &pool,
        CreateWorkflowRun {
            workflow_id: deploy.id,
            actor_user_id: Some(owner.id),
            head_branch: "release".to_owned(),
            head_sha: Some("fedcba9876543210".to_owned()),
            event: "workflow_dispatch".to_owned(),
        },
    )
    .await
    .expect("deploy run should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let dashboard_uri = format!(
        "/api/repos/{}/{}/actions/dashboard?page=1&pageSize=2",
        owner.email, repo_name
    );
    let (status, body) = get_json(app.clone(), &dashboard_uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repository"]["name"], repo_name);
    assert_eq!(body["viewerPermission"], "read");
    assert_eq!(body["workflows"].as_array().expect("workflows").len(), 2);
    assert_eq!(body["workflows"][0]["name"], "CI");
    assert_eq!(body["workflows"][0]["pinned"], true);
    assert_eq!(body["workflows"][0]["runCount"], 2);
    assert_eq!(body["runs"]["total"], 3);
    assert_eq!(body["runs"]["pageSize"], 2);
    assert_eq!(
        body["filterOptions"]["statuses"]
            .as_array()
            .expect("statuses")
            .iter()
            .map(|option| option["value"].as_str().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "action_required",
            "cancelled",
            "completed",
            "failure",
            "in_progress",
            "neutral",
            "queued",
            "skipped",
            "stale",
            "success",
            "timed_out",
            "waiting",
        ]
    );

    let filtered_uri = format!(
        "/api/repos/{}/{}/actions/dashboard?q=dashboard&workflow={}&status=success",
        owner.email, repo_name, ci.id
    );
    let (filtered_status, filtered_body) =
        get_json(app.clone(), &filtered_uri, Some(&owner_cookie)).await;
    assert_eq!(filtered_status, StatusCode::OK);
    assert_eq!(filtered_body["filters"]["q"], "dashboard");
    assert_eq!(filtered_body["filters"]["workflow"], ci.id.to_string());
    assert_eq!(filtered_body["runs"]["total"], 1);
    assert_eq!(
        filtered_body["runs"]["items"][0]["displayTitle"],
        "Add Actions dashboard contract"
    );
    assert_eq!(
        filtered_body["runs"]["items"][0]["statusCategory"],
        "success"
    );
    assert_eq!(
        filtered_body["runs"]["items"][0]["jobSummary"]["success"],
        1
    );
    assert_eq!(filtered_body["runs"]["items"][0]["shortSha"], "abcdef0");

    let branch_uri = format!(
        "/api/repos/{}/{}/actions/dashboard?branch=feature/actions&event=pull_request&actor={}",
        owner.email, repo_name, owner.id
    );
    let (branch_status, branch_body) = get_json(app.clone(), &branch_uri, None).await;
    assert_eq!(branch_status, StatusCode::OK);
    assert_eq!(branch_body["runs"]["total"], 1);
    assert_eq!(branch_body["runs"]["items"][0]["isLive"], true);
    assert_eq!(
        branch_body["runs"]["items"][0]["headBranch"],
        "feature/actions"
    );

    let invalid_status_uri = format!(
        "/api/repos/{}/{}/actions/dashboard?status=not-real",
        owner.email, repo_name
    );
    let (invalid_status, invalid_body) = get_json(app, &invalid_status_uri, None).await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn actions_recent_view_persists_signed_in_filter_telemetry() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions recent view scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-recent-owner").await;
    let repo_name = format!("actions-recent-{}", Uuid::new_v4().simple());
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

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/actions/recent-view",
        owner.email, repo_name
    );
    let anonymous = post_json(
        app.clone(),
        &uri,
        None,
        json!({ "workflow": workflow.id, "status": "in progress" }),
    )
    .await;
    assert_eq!(anonymous.0, StatusCode::UNAUTHORIZED);

    let cookie = cookie_header(&pool, &config, &owner).await;
    let (status, body) = post_json(
        app.clone(),
        &uri,
        Some(&cookie),
        json!({
            "q": "deploy",
            "workflow": workflow.id.to_string(),
            "event": "push",
            "status": "in progress",
            "branch": "main",
            "actor": owner.id.to_string()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["workflowId"], workflow.id.to_string());
    assert_eq!(body["filters"]["status"], "in_progress");

    let saved = sqlx::query(
        "SELECT workflow_id, filters FROM actions_recent_views WHERE repository_id = $1 AND user_id = $2",
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("recent view should persist");
    assert_eq!(
        saved.get::<Option<Uuid>, _>("workflow_id"),
        Some(workflow.id)
    );
    assert_eq!(
        saved.get::<Value, _>("filters")["q"],
        Value::String("deploy".to_owned())
    );

    let invalid = post_json(
        app,
        &uri,
        Some(&cookie),
        json!({ "workflow": Uuid::new_v4().to_string() }),
    )
    .await;
    assert_eq!(invalid.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid.1["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn actions_dashboard_filters_supported_status_matrix_and_clamps_pagination() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions status matrix scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-status-owner").await;
    let repo_name = format!("actions-status-{}", Uuid::new_v4().simple());
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
    let workflow = create_workflow(
        &pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: owner.id,
            name: "Matrix".to_owned(),
            path: ".github/workflows/matrix.yml".to_owned(),
            trigger_events: vec!["push".to_owned(), "workflow_dispatch".to_owned()],
        },
    )
    .await
    .expect("workflow should create");

    let scenarios = [
        ("Queued", RunStatus::Queued, None, "queued"),
        ("In progress", RunStatus::InProgress, None, "in_progress"),
        (
            "Success",
            RunStatus::Completed,
            Some(RunConclusion::Success),
            "success",
        ),
        (
            "Failure",
            RunStatus::Completed,
            Some(RunConclusion::Failure),
            "failure",
        ),
        (
            "Skipped",
            RunStatus::Completed,
            Some(RunConclusion::Skipped),
            "skipped",
        ),
        (
            "Timed out",
            RunStatus::Completed,
            Some(RunConclusion::TimedOut),
            "timed_out",
        ),
        (
            "Cancelled",
            RunStatus::Cancelled,
            Some(RunConclusion::Cancelled),
            "cancelled",
        ),
    ];

    for (title, status, conclusion, branch) in scenarios {
        let run = create_workflow_run(
            &pool,
            CreateWorkflowRun {
                workflow_id: workflow.id,
                actor_user_id: Some(owner.id),
                head_branch: branch.to_owned(),
                head_sha: Some(format!("{:016x}", branch.len())),
                event: "push".to_owned(),
            },
        )
        .await
        .expect("matrix run should create");
        transition_workflow_run(&pool, run.id, TransitionRun { status, conclusion })
            .await
            .expect("matrix run should transition");
        sqlx::query("UPDATE workflow_runs SET display_title = $2 WHERE id = $1")
            .bind(run.id)
            .bind(title)
            .execute(&pool)
            .await
            .expect("display title should update");
    }

    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let clamped_uri = format!(
        "/api/repos/{}/{}/actions/dashboard?page=0&pageSize=500",
        owner.email, repo_name
    );
    let (clamped_status, clamped_body) = get_json(app.clone(), &clamped_uri, None).await;
    assert_eq!(clamped_status, StatusCode::OK);
    assert_eq!(clamped_body["runs"]["page"], 1);
    assert_eq!(clamped_body["runs"]["pageSize"], 100);
    assert_eq!(clamped_body["runs"]["total"], 7);

    for (status_filter, expected_normalized, expected_total, expected_category) in [
        ("queued", "queued", 1, "queued"),
        ("in%20progress", "in_progress", 1, "in_progress"),
        ("completed", "completed", 4, "success"),
        ("success", "success", 1, "success"),
        ("failure", "failure", 1, "failure"),
        ("skipped", "skipped", 1, "skipped"),
        ("timed-out", "timed_out", 1, "timed_out"),
        ("cancelled", "cancelled", 1, "cancelled"),
    ] {
        let uri = format!(
            "/api/repos/{}/{}/actions/dashboard?status={status_filter}",
            owner.email, repo_name
        );
        let (status, body) = get_json(app.clone(), &uri, None).await;
        assert_eq!(status, StatusCode::OK, "status filter {status_filter}");
        assert_eq!(body["filters"]["status"], expected_normalized);
        assert_eq!(
            body["runs"]["total"], expected_total,
            "total for {status_filter}"
        );
        if status_filter == "completed" {
            let mut categories = body["runs"]["items"]
                .as_array()
                .expect("completed items")
                .iter()
                .map(|item| item["statusCategory"].as_str().unwrap_or_default())
                .collect::<Vec<_>>();
            categories.sort_unstable();
            assert_eq!(
                categories,
                vec!["failure", "skipped", "success", "timed_out"]
            );
        } else {
            assert_eq!(
                body["runs"]["items"][0]["statusCategory"], expected_category,
                "category for {status_filter}"
            );
        }
    }
}

#[tokio::test]
async fn actions_dashboard_preserves_empty_state_and_private_permissions() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping actions dashboard authz scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "actions-private-owner").await;
    let outsider = create_user(&pool, "actions-private-outsider").await;
    let public_repo_name = format!("actions-empty-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: public_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public empty repository should create");
    let private_repo_name = format!("actions-private-{}", Uuid::new_v4().simple());
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: private_repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let empty_uri = format!(
        "/api/repos/{}/{}/actions/dashboard",
        owner.email, public_repo_name
    );
    let (empty_status, empty_body) = get_json(app.clone(), &empty_uri, None).await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(empty_body["emptyState"]["hasWorkflows"], false);
    assert_eq!(empty_body["emptyState"]["hasRuns"], false);
    assert!(empty_body["emptyState"]["newWorkflowHref"]
        .as_str()
        .expect("new workflow href")
        .contains("/.github/workflows"));

    let private_uri = format!(
        "/api/repos/{}/{}/actions/dashboard",
        owner.email, private_repo_name
    );
    let (anonymous_status, anonymous_body) = get_json(app.clone(), &private_uri, None).await;
    assert_eq!(anonymous_status, StatusCode::FORBIDDEN);
    assert_eq!(anonymous_body["error"]["code"], "forbidden");
    assert!(
        !anonymous_body.to_string().contains(&private_repo_name),
        "private repository metadata must not leak in forbidden responses"
    );

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &private_uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_body["error"]["code"], "forbidden");

    let (owner_status, owner_body) = get_json(app, &private_uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["repository"]["name"], private_repo_name);
    assert_eq!(owner_body["viewerPermission"], "owner");
}
