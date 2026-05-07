use chrono::Utc;
use opengithub_api::domain::{
    actions::{
        trigger_workflows_for_push, trigger_workflows_for_schedule, TriggerWorkflowsForPush,
        TriggerWorkflowsForSchedule,
    },
    identity::{upsert_user_by_email, User},
    pulls::{create_pull_request, CreatePullRequest},
    repositories::{create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility},
};
use serde_json::Value;
use sqlx::{PgPool, Row};
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

async fn create_public_repository(pool: &PgPool, owner: &User, label: &str) -> Uuid {
    create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{label}-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create")
    .id
}

async fn insert_commit_with_files(
    pool: &PgPool,
    repository_id: Uuid,
    actor_id: Uuid,
    oid: &str,
    branch: &str,
    files: &[(&str, &str)],
) -> Uuid {
    let commit_id = sqlx::query(
        r#"
        INSERT INTO commits (
            repository_id, oid, author_user_id, committer_user_id, message,
            tree_oid, committed_at
        )
        VALUES ($1, $2, $3, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(oid)
    .bind(actor_id)
    .bind(format!("Push {oid}"))
    .bind(format!("tree-{oid}"))
    .bind(Utc::now())
    .fetch_one(pool)
    .await
    .expect("commit should insert")
    .get::<Uuid, _>("id");

    for (path, content) in files {
        let blob_oid = format!("blob-{}-{}", oid, path.replace('/', "-"));
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(&blob_oid)
        .bind(content.len() as i64)
        .execute(pool)
        .await
        .expect("blob should insert");
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(commit_id)
        .bind(path)
        .bind(content)
        .bind(&blob_oid)
        .bind(content.len() as i64)
        .execute(pool)
        .await
        .expect("file should insert");
    }

    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{branch}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("ref should upsert");

    commit_id
}

#[tokio::test]
async fn push_trigger_loads_workflow_yaml_expands_matrix_and_enqueues_runs() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions push trigger scenario; set TEST_DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "actions-push-owner").await;
    let repository_id = create_public_repository(&pool, &owner, "actions-push").await;
    let workflow_yaml = r#"
name: CI
"on":
  push:
    branches: [main]
    paths:
      - "src/**"
  workflow_dispatch:
    inputs:
      environment:
        description: Target environment
        required: true
        type: choice
        options: [staging, production]
concurrency:
  group: ci-main
jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        node: [20, 22]
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Test
        run: cargo test
"#;
    insert_commit_with_files(
        &pool,
        repository_id,
        owner.id,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "main",
        &[
            (".github/workflows/ci.yml", workflow_yaml),
            ("src/lib.rs", "pub fn meaning() -> i32 { 42 }"),
        ],
    )
    .await;

    let result = trigger_workflows_for_push(
        &pool,
        TriggerWorkflowsForPush {
            repository_id,
            actor_user_id: owner.id,
            ref_name: "refs/heads/main".to_owned(),
            head_sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        },
    )
    .await
    .expect("push should trigger workflow");

    assert_eq!(result.scanned_workflows, 1);
    assert_eq!(result.skipped_workflows.len(), 0);
    assert_eq!(result.triggered_runs.len(), 1);
    let run = &result.triggered_runs[0];
    assert_eq!(run.event, "push");
    assert_eq!(run.head_branch, "main");
    assert_eq!(
        run.head_sha.as_deref(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );

    let workflow_row = sqlx::query(
        r#"
        SELECT name, path, trigger_events, source_sha, source_branch,
               yaml_parse_error, dispatch_enabled, dispatch_inputs
        FROM actions_workflows
        WHERE id = $1
        "#,
    )
    .bind(run.workflow_id)
    .fetch_one(&pool)
    .await
    .expect("workflow should persist");
    assert_eq!(workflow_row.get::<String, _>("name"), "CI");
    assert_eq!(
        workflow_row.get::<String, _>("path"),
        ".github/workflows/ci.yml"
    );
    assert_eq!(
        workflow_row.get::<Vec<String>, _>("trigger_events"),
        vec!["push".to_owned(), "workflow_dispatch".to_owned()]
    );
    assert_eq!(
        workflow_row
            .get::<Option<String>, _>("source_sha")
            .as_deref(),
        Some("blob-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-.github-workflows-ci.yml")
    );
    assert_eq!(
        workflow_row
            .get::<Option<String>, _>("source_branch")
            .as_deref(),
        Some("main")
    );
    assert!(workflow_row
        .get::<Option<String>, _>("yaml_parse_error")
        .is_none());
    assert!(workflow_row.get::<bool, _>("dispatch_enabled"));
    assert_eq!(
        workflow_row
            .get::<Value, _>("dispatch_inputs")
            .as_array()
            .expect("dispatch inputs should be array")
            .len(),
        1
    );

    let run_row = sqlx::query(
        r#"
        SELECT display_title, event_payload, concurrency_group, workflow_matrix
        FROM workflow_runs
        WHERE id = $1
        "#,
    )
    .bind(run.id)
    .fetch_one(&pool)
    .await
    .expect("run should persist");
    assert_eq!(
        run_row
            .get::<Option<String>, _>("concurrency_group")
            .as_deref(),
        Some("ci-main")
    );
    assert!(run_row
        .get::<Option<String>, _>("display_title")
        .expect("display title should persist")
        .ends_with(" pushed to main"));
    assert_eq!(
        run_row.get::<Value, _>("event_payload")["workflowPath"],
        ".github/workflows/ci.yml"
    );
    assert_eq!(
        run_row.get::<Value, _>("event_payload")["changedPaths"][1],
        "src/lib.rs"
    );
    assert_eq!(run_row.get::<Value, _>("workflow_matrix")["jobCount"], 4);

    let job_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM workflow_jobs WHERE run_id = $1 AND group_name = 'ci-main'",
    )
    .bind(run.id)
    .fetch_one(&pool)
    .await
    .expect("jobs should count");
    assert_eq!(job_count, 4);

    let step_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM workflow_steps
        JOIN workflow_jobs ON workflow_jobs.id = workflow_steps.job_id
        WHERE workflow_jobs.run_id = $1
        "#,
    )
    .bind(run.id)
    .fetch_one(&pool)
    .await
    .expect("steps should count");
    assert_eq!(step_count, 8);

    let lease = sqlx::query("SELECT queue, payload FROM job_leases WHERE lease_key = $1")
        .bind(format!("workflow-push:{}:{}", run.workflow_id, run.id))
        .fetch_one(&pool)
        .await
        .expect("job lease should enqueue");
    assert_eq!(lease.get::<String, _>("queue"), "actions.workflow_push");
    assert_eq!(
        lease.get::<Value, _>("payload")["concurrencyGroup"],
        "ci-main"
    );
}

#[tokio::test]
async fn push_trigger_respects_filters_and_records_invalid_yaml_without_runs() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions push filter scenario; set TEST_DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "actions-push-filter-owner").await;
    let repository_id = create_public_repository(&pool, &owner, "actions-push-filter").await;
    insert_commit_with_files(
        &pool,
        repository_id,
        owner.id,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "main",
        &[
            (
                ".github/workflows/release.yml",
                r#"
name: Release
"on":
  push:
    branches: [release/**]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - run: echo release
"#,
            ),
            (
                ".github/workflows/docs.yml",
                r#"
name: Docs
"on":
  push:
    paths:
      - "docs/**"
    paths-ignore:
      - ".github/workflows/**"
jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - run: echo docs
"#,
            ),
            (".github/workflows/broken.yml", "name: [broken"),
            ("src/lib.rs", "pub fn main_change() {}"),
        ],
    )
    .await;

    let result = trigger_workflows_for_push(
        &pool,
        TriggerWorkflowsForPush {
            repository_id,
            actor_user_id: owner.id,
            ref_name: "refs/heads/main".to_owned(),
            head_sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        },
    )
    .await
    .expect("push should scan workflows");

    assert_eq!(result.scanned_workflows, 3);
    assert!(result.triggered_runs.is_empty());
    assert!(result
        .skipped_workflows
        .iter()
        .any(|skip| skip.path.ends_with("release.yml") && skip.reason == "ref_filter"));
    assert!(result
        .skipped_workflows
        .iter()
        .any(|skip| skip.path.ends_with("docs.yml") && skip.reason == "path_filter"));
    assert!(result
        .skipped_workflows
        .iter()
        .any(|skip| skip.path.ends_with("broken.yml") && skip.reason == "invalid_yaml"));

    let broken = sqlx::query(
        r#"
        SELECT yaml_parse_error
        FROM actions_workflows
        WHERE repository_id = $1 AND path = '.github/workflows/broken.yml'
        "#,
    )
    .bind(repository_id)
    .fetch_one(&pool)
    .await
    .expect("broken workflow should persist");
    let parse_error = broken
        .get::<Option<String>, _>("yaml_parse_error")
        .expect("parse error should be recorded");
    assert!(!parse_error.contains("panicked"));

    let run_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM workflow_runs WHERE repository_id = $1")
            .bind(repository_id)
            .fetch_one(&pool)
            .await
            .expect("runs should count");
    assert_eq!(run_count, 0);
}

#[tokio::test]
async fn pull_request_creation_dispatches_matching_workflow_runs() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions pull_request trigger scenario; set TEST_DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "actions-pr-owner").await;
    let repository_id = create_public_repository(&pool, &owner, "actions-pr").await;
    let workflow_yaml = r#"
name: PR checks
"on":
  pull_request:
    branches: [main]
    paths:
      - "src/**"
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test
"#;
    insert_commit_with_files(
        &pool,
        repository_id,
        owner.id,
        "cccccccccccccccccccccccccccccccccccccccc",
        "main",
        &[
            (".github/workflows/pr.yml", workflow_yaml),
            ("README.md", "base"),
        ],
    )
    .await;
    insert_commit_with_files(
        &pool,
        repository_id,
        owner.id,
        "dddddddddddddddddddddddddddddddddddddddd",
        "feature/pr-checks",
        &[("src/lib.rs", "pub fn changed() {}")],
    )
    .await;

    let detail = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id,
            actor_user_id: owner.id,
            title: "Add PR checks".to_owned(),
            body: None,
            head_ref: "feature/pr-checks".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: Vec::new(),
            milestone_id: None,
            assignee_user_ids: Vec::new(),
            reviewer_user_ids: Vec::new(),
            template_slug: None,
        },
    )
    .await
    .expect("pull request should create and dispatch actions");

    let run = sqlx::query(
        r#"
        SELECT workflow_runs.event, workflow_runs.head_branch, workflow_runs.head_sha,
               workflow_runs.event_payload, workflow_runs.pull_request_id,
               job_leases.queue
        FROM workflow_runs
        JOIN job_leases ON (job_leases.payload->>'runId')::uuid = workflow_runs.id
        WHERE workflow_runs.repository_id = $1
          AND workflow_runs.pull_request_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(detail.pull_request.id)
    .fetch_one(&pool)
    .await
    .expect("pull_request workflow run should persist");

    assert_eq!(run.get::<String, _>("event"), "pull_request");
    assert_eq!(run.get::<String, _>("head_branch"), "feature/pr-checks");
    assert_eq!(
        run.get::<Option<String>, _>("head_sha").as_deref(),
        Some("dddddddddddddddddddddddddddddddddddddddd")
    );
    assert_eq!(
        run.get::<String, _>("queue"),
        "actions.workflow_pull_request"
    );
    assert_eq!(run.get::<Value, _>("event_payload")["baseRef"], "main");
    assert_eq!(
        run.get::<Value, _>("event_payload")["changedPaths"][0],
        "src/lib.rs"
    );
}

#[tokio::test]
async fn schedule_trigger_dispatches_default_branch_workflows() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping actions schedule trigger scenario; set TEST_DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "actions-schedule-owner").await;
    let repository_id = create_public_repository(&pool, &owner, "actions-schedule").await;
    let workflow_yaml = r#"
name: Nightly
"on":
  schedule:
    - cron: "0 0 * * *"
jobs:
  nightly:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test
"#;
    insert_commit_with_files(
        &pool,
        repository_id,
        owner.id,
        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "main",
        &[(".github/workflows/nightly.yml", workflow_yaml)],
    )
    .await;

    let result = trigger_workflows_for_schedule(
        &pool,
        TriggerWorkflowsForSchedule {
            repository_id,
            schedule: Some("0 0 * * *".to_owned()),
        },
    )
    .await
    .expect("schedule should trigger workflow");

    assert_eq!(result.scanned_workflows, 1);
    assert_eq!(result.triggered_runs.len(), 1);
    let run = &result.triggered_runs[0];
    assert_eq!(run.event, "schedule");
    assert_eq!(run.head_branch, "main");

    let lease = sqlx::query("SELECT queue, payload FROM job_leases WHERE payload->>'runId' = $1")
        .bind(run.id.to_string())
        .fetch_one(&pool)
        .await
        .expect("schedule lease should enqueue");
    assert_eq!(lease.get::<String, _>("queue"), "actions.workflow_schedule");
    assert_eq!(lease.get::<Value, _>("payload")["event"], "schedule");
}
