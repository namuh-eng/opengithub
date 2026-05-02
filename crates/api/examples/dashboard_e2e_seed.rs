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
        identity::{upsert_session, upsert_user_by_email},
        issues::{create_issue, ensure_default_labels, CreateIssue},
        permissions::RepositoryRole,
        pulls::{create_pull_request, CreatePullRequest},
        repositories::{
            create_repository, create_repository_with_bootstrap, insert_commit,
            replace_repository_snapshot, upsert_git_ref, CreateCommit, CreateRepository,
            RepositoryBootstrapRequest, RepositoryOwner, RepositorySnapshot,
            RepositorySnapshotFile, RepositoryVisibility,
        },
        search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
    },
};
use serde::Serialize;
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedOutput {
    cookie_name: String,
    cookie_value: String,
    first_repository_href: String,
    second_repository_href: String,
    social_source_repository_href: String,
    tree_repository_href: String,
    fork_compare_href: String,
    pull_request_merge_href: String,
    actions_run_detail_href: String,
    actions_job_log_href: String,
}

fn seed_empty_dashboard() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_EMPTY").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_tree_repository() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_TREE_REFS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_fork_compare_repository() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_FORK_REFS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_blob_edge_files() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_BLOB_EDGE").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_pull_request_merge() -> bool {
    matches!(
        std::env::var("PULL_REQUEST_MERGE_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_actions_run_detail() -> bool {
    matches!(
        std::env::var("ACTIONS_RUN_DETAIL_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_issue_templates_enabled() -> bool {
    matches!(
        std::env::var("ISSUE_TEMPLATE_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn search_e2e_marker() -> Option<String> {
    std::env::var("SEARCH_E2E_MARKER")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("app URL"),
        api_url: Url::parse("http://localhost:3016").expect("api URL"),
        auth: Some(AuthConfig {
            google_client_id: "playwright-client-id.apps.googleusercontent.com".to_owned(),
            google_client_secret: "playwright-client-secret".to_owned(),
            session_secret: std::env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "playwright-session-secret-with-enough-entropy".to_owned()),
        }),
        session_cookie_name: std::env::var("SESSION_COOKIE_NAME")
            .unwrap_or_else(|_| "__Host-session".to_owned()),
        session_cookie_secure: false,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| anyhow::anyhow!("TEST_DATABASE_URL or DATABASE_URL is required"))?;
    let pool = opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await?;
    MIGRATOR.run(&pool).await?;

    let config = app_config();
    let suffix = Uuid::new_v4().simple().to_string();
    let username = format!("dash-{}", &suffix[..12]);
    let user = upsert_user_by_email(
        &pool,
        &format!("{username}@opengithub.local"),
        Some("Dashboard Tester"),
        None,
    )
    .await?;
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&username)
        .bind(user.id)
        .execute(&pool)
        .await?;
    let source_owner_username = format!("source-{}", &suffix[..12]);
    let source_owner = upsert_user_by_email(
        &pool,
        &format!("{source_owner_username}@opengithub.local"),
        Some("Repository Source"),
        None,
    )
    .await?;
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&source_owner_username)
        .bind(source_owner.id)
        .execute(&pool)
        .await?;
    let social_source_name = format!("social-source-{}", &suffix[..12]);
    let social_source_repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: source_owner.id,
            },
            name: social_source_name.clone(),
            description: Some("Repository social action source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: source_owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await?;
    let (tree_repository_href, fork_compare_href) = if seed_tree_repository() {
        let tree_repository_name = format!("tree-nav-{}", &suffix[..12]);
        let tree_repository = create_repository_with_bootstrap(
            &pool,
            CreateRepository {
                owner: RepositoryOwner::User { id: user.id },
                name: tree_repository_name.clone(),
                description: Some("Repository tree navigation seed".to_owned()),
                visibility: RepositoryVisibility::Public,
                default_branch: None,
                created_by_user_id: user.id,
            },
            RepositoryBootstrapRequest {
                initialize_readme: true,
                template_slug: Some("rust-axum".to_owned()),
                ..RepositoryBootstrapRequest::default()
            },
        )
        .await?;
        seed_tree_refs(&pool, user.id, tree_repository.id).await?;
        if seed_blob_edge_files() {
            seed_blob_edge_cases(&pool, tree_repository.id).await?;
        }
        let fork_compare_href = if seed_fork_compare_repository() {
            seed_fork_compare_refs(
                &pool,
                user.id,
                &username,
                source_owner.id,
                &source_owner_username,
                &suffix,
            )
            .await?
        } else {
            String::new()
        };
        (
            format!("/{username}/{tree_repository_name}"),
            fork_compare_href,
        )
    } else {
        (String::new(), String::new())
    };
    let mut pull_request_merge_href = String::new();
    let (first_repository_href, second_repository_href) = if seed_empty_dashboard() {
        (String::new(), String::new())
    } else {
        let reviewer = upsert_user_by_email(
            &pool,
            &format!("reviewer-{suffix}@opengithub.local"),
            Some("Review Author"),
            None,
        )
        .await?;

        let first_repository_name = format!("alpha-{}", &suffix[..12]);
        let second_repository_name = format!("infra-{}", &suffix[..12]);
        let first_repository = create_repository(
            &pool,
            CreateRepository {
                owner: RepositoryOwner::User { id: user.id },
                name: first_repository_name.clone(),
                description: Some("Repository collaboration workspace".to_owned()),
                visibility: RepositoryVisibility::Public,
                default_branch: None,
                created_by_user_id: user.id,
            },
        )
        .await?;
        let second_repository = create_repository(
            &pool,
            CreateRepository {
                owner: RepositoryOwner::User { id: user.id },
                name: second_repository_name.clone(),
                description: Some("Infrastructure automation".to_owned()),
                visibility: RepositoryVisibility::Private,
                default_branch: None,
                created_by_user_id: user.id,
            },
        )
        .await?;

        upsert_language(&pool, first_repository.id, "TypeScript", "#3178c6", 1200).await?;
        upsert_language(&pool, second_repository.id, "Rust", "#dea584", 900).await?;
        sqlx::query(
            r#"
            INSERT INTO recent_repository_visits (user_id, repository_id, visited_at)
            VALUES ($1, $2, now())
            ON CONFLICT (user_id, repository_id)
            DO UPDATE SET visited_at = EXCLUDED.visited_at
            "#,
        )
        .bind(user.id)
        .bind(second_repository.id)
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO repository_permissions (repository_id, user_id, role, source)
            VALUES ($1, $2, $3, 'direct')
            ON CONFLICT (repository_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            "#,
        )
        .bind(first_repository.id)
        .bind(reviewer.id)
        .bind(RepositoryRole::Write.as_str())
        .execute(&pool)
        .await?;
        if seed_issue_templates_enabled() {
            seed_issue_templates(&pool, first_repository.id, user.id).await?;
        }
        sqlx::query(
            r#"
            INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, committed_at)
            VALUES ($1, $2, $3, $3, 'Wire dashboard activity feed', now())
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(first_repository.id)
        .bind(format!("{}abcdef", &suffix[..16]))
        .bind(user.id)
        .execute(&pool)
        .await?;
        create_issue(
            &pool,
            CreateIssue {
                repository_id: first_repository.id,
                actor_user_id: user.id,
                title: "Fix dashboard setup workflow".to_owned(),
                body: None,
                template_id: None,
                template_slug: None,
                field_values: std::collections::HashMap::new(),
                milestone_id: None,
                label_ids: vec![],
                assignee_user_ids: vec![user.id],
                attachments: Vec::new(),
            },
        )
        .await?;
        create_pull_request(
            &pool,
            CreatePullRequest {
                repository_id: first_repository.id,
                actor_user_id: reviewer.id,
                title: "Add signed-in dashboard feed".to_owned(),
                body: None,
                head_ref: "dashboard-feed".to_owned(),
                base_ref: "main".to_owned(),
                head_repository_id: None,
                is_draft: false,
                label_ids: vec![],
                milestone_id: None,
                assignee_user_ids: vec![],
                reviewer_user_ids: vec![],
                template_slug: None,
            },
        )
        .await?;
        if seed_pull_request_merge() {
            pull_request_merge_href =
                seed_merge_ready_pull_request(&pool, user.id, &username, &suffix).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO user_follows (follower_user_id, followed_user_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user.id)
        .bind(reviewer.id)
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO repository_watches (user_id, repository_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user.id)
        .bind(first_repository.id)
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO repository_stars (user_id, repository_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user.id)
        .bind(second_repository.id)
        .execute(&pool)
        .await?;
        seed_feed_event(
            &pool,
            user.id,
            first_repository.id,
            "push",
            "Pushed dashboard activity feed",
            format!(
                "/{username}/{first_repository_name}/commit/{}",
                &suffix[..12]
            ),
        )
        .await?;
        seed_feed_event(
            &pool,
            reviewer.id,
            first_repository.id,
            "help_wanted_pull_request",
            "Asked for help reviewing dashboard feed",
            format!("/{username}/{first_repository_name}/pull/1"),
        )
        .await?;
        seed_feed_event(
            &pool,
            user.id,
            second_repository.id,
            "release",
            "Published infrastructure preview",
            format!("/{username}/{second_repository_name}/releases/tag/v0.1.0"),
        )
        .await?;

        (
            format!("/{username}/{first_repository_name}"),
            format!("/{username}/{second_repository_name}"),
        )
    };

    if let Some(marker) = search_e2e_marker() {
        seed_search_documents(&pool, user.id, &username, &marker).await?;
    }
    let (actions_run_detail_href, actions_job_log_href) = if seed_actions_run_detail() {
        seed_actions_run_detail_repository(&pool, user.id, &username, &suffix).await?
    } else {
        (String::new(), String::new())
    };

    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    upsert_session(
        &pool,
        &session_id,
        Some(user.id),
        serde_json::json!({ "provider": "google" }),
        expires_at,
    )
    .await?;
    let set_cookie = session::set_cookie_header(&config, &session_id, expires_at)?;
    let cookie_value = session::cookie_value_from_set_cookie(&set_cookie)
        .ok_or_else(|| anyhow::anyhow!("set-cookie did not include a value"))?;

    let output = SeedOutput {
        cookie_name: config.session_cookie_name,
        cookie_value: cookie_value.to_owned(),
        first_repository_href,
        second_repository_href,
        social_source_repository_href: format!(
            "/{}/{}",
            social_source_repository.owner_login, social_source_repository.name
        ),
        tree_repository_href,
        fork_compare_href,
        pull_request_merge_href,
        actions_run_detail_href,
        actions_job_log_href,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

async fn seed_search_documents(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    marker: &str,
) -> anyhow::Result<()> {
    let repository_name = format!("search-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user_id },
            name: repository_name.clone(),
            description: Some(format!("Repository result seeded for {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: user_id,
        },
    )
    .await?;

    upsert_search_document(
        pool,
        user_id,
        UpsertSearchDocument {
            repository_id: Some(repository.id),
            owner_user_id: Some(user_id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Repository,
            resource_id: format!("repo-{}", repository.id),
            title: format!("{repository_name} {marker}"),
            body: Some(format!("Repository result seeded for {marker}")),
            path: None,
            language: None,
            branch: None,
            visibility: RepositoryVisibility::Public,
            metadata: serde_json::json!({}),
        },
    )
    .await?;

    let commit_oid = format!("search-phase3-{}", Uuid::new_v4().simple());
    replace_repository_snapshot(
        pool,
        repository.id,
        RepositorySnapshot {
            branch_name: "main".to_owned(),
            commit: CreateCommit {
                oid: commit_oid,
                author_user_id: Some(user_id),
                committer_user_id: Some(user_id),
                message: format!("Add {marker} code search fixture\n\nCommit result for {marker}."),
                tree_oid: Some(format!("tree-{marker}")),
                parent_oids: vec![],
                committed_at: Utc::now(),
            },
            files: vec![RepositorySnapshotFile {
                path: "src/search_phase_three.rs".to_owned(),
                content: format!(
                    "pub fn {marker}() {{\n    println!(\"search phase three\");\n}}\n"
                ),
                oid: format!("blob-{marker}"),
                byte_size: 72,
            }],
        },
    )
    .await?;

    let labels = ensure_default_labels(pool, repository.id).await?;
    create_issue(
        pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: user_id,
            title: format!("Investigate {marker} issue search"),
            body: Some(format!("Issue result seeded for {marker}")),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: labels
                .first()
                .map(|label| vec![label.id])
                .unwrap_or_default(),
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await?;
    create_pull_request(
        pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: user_id,
            title: format!("Review {marker} pull search"),
            body: Some(format!("Pull request result seeded for {marker}")),
            head_ref: format!("feature/{marker}"),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await?;

    upsert_search_document(
        pool,
        user_id,
        UpsertSearchDocument {
            repository_id: None,
            owner_user_id: Some(user_id),
            owner_organization_id: None,
            kind: SearchDocumentKind::User,
            resource_id: username.to_owned(),
            title: format!("Dashboard Tester {marker}"),
            body: Some("Searchable profile result".to_owned()),
            path: None,
            language: None,
            branch: None,
            visibility: RepositoryVisibility::Public,
            metadata: serde_json::json!({}),
        },
    )
    .await?;

    let organization_slug = format!("search-org-{}", Uuid::new_v4().simple());
    let organization_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO organizations (slug, display_name, description, owner_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&organization_slug)
    .bind(format!("Search Organization {marker}"))
    .bind(Some(format!("Organization result seeded for {marker}")))
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    upsert_search_document(
        pool,
        user_id,
        UpsertSearchDocument {
            repository_id: None,
            owner_user_id: None,
            owner_organization_id: Some(organization_id),
            kind: SearchDocumentKind::Organization,
            resource_id: organization_slug,
            title: format!("Search Organization {marker}"),
            body: Some("Organization result seeded for people search".to_owned()),
            path: None,
            language: None,
            branch: None,
            visibility: RepositoryVisibility::Public,
            metadata: serde_json::json!({}),
        },
    )
    .await?;

    Ok(())
}

async fn seed_actions_run_detail_repository(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    suffix: &str,
) -> anyhow::Result<(String, String)> {
    let repository_name = format!("actions-run-{}", &suffix[..12]);
    let repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user_id },
            name: repository_name.clone(),
            description: Some("Workflow run detail smoke seed".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: user_id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await?;

    let commit = insert_commit(
        pool,
        repository.id,
        CreateCommit {
            oid: format!("{}abcdef012345", &suffix[..16]),
            author_user_id: Some(user_id),
            committer_user_id: Some(user_id),
            message: "Add workflow run detail fixture".to_owned(),
            tree_oid: Some(format!("tree-actions-{}", &suffix[..12])),
            parent_oids: vec![],
            committed_at: Utc::now(),
        },
    )
    .await?;
    let workflow = create_workflow(
        pool,
        CreateWorkflow {
            repository_id: repository.id,
            actor_user_id: user_id,
            name: "Editorial CI".to_owned(),
            path: ".github/workflows/editorial-ci.yml".to_owned(),
            trigger_events: vec!["push".to_owned(), "workflow_dispatch".to_owned()],
        },
    )
    .await?;
    sqlx::query(
        "UPDATE actions_workflows SET source_branch = 'main', source_sha = $2 WHERE id = $1",
    )
    .bind(workflow.id)
    .bind(format!("workflow-sha-{}", &suffix[..12]))
    .execute(pool)
    .await?;

    let run = create_workflow_run(
        pool,
        CreateWorkflowRun {
            workflow_id: workflow.id,
            actor_user_id: Some(user_id),
            head_branch: "main".to_owned(),
            head_sha: Some(format!("{}abcdef012345", &suffix[..16])),
            event: "workflow_dispatch".to_owned(),
        },
    )
    .await?;
    transition_workflow_run(
        pool,
        run.id,
        TransitionRun {
            status: RunStatus::Completed,
            conclusion: Some(RunConclusion::Failure),
        },
    )
    .await?;
    sqlx::query(
        "UPDATE workflow_runs SET display_title = 'Validate Editorial CI', commit_id = $2 WHERE id = $1",
    )
    .bind(run.id)
    .bind(commit.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_run_attempts (
            run_id, attempt_number, status, conclusion, triggered_by_user_id, trigger_kind,
            started_at, completed_at
        )
        VALUES
            ($1, 1, 'completed', 'failure', $2, 'initial', now() - interval '8 minutes', now() - interval '6 minutes'),
            ($1, 2, 'completed', 'failure', $2, 'rerun_failed', now() - interval '4 minutes', now() - interval '1 minute')
        "#,
    )
    .bind(run.id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let web_job = create_workflow_job(
        pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "unit / web".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await?;
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
    .bind(web_job.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_job_log_lines (job_id, line_number, timestamp, content)
        VALUES
            ($1, 1, now() - interval '3 minutes', 'Installing dependencies'),
            ($1, 2, now() - interval '2 minutes', 'Running unit tests after cache error'),
            ($1, 3, now() - interval '1 minute', 'error: Expected string, found number')
        "#,
    )
    .bind(web_job.id)
    .execute(pool)
    .await?;
    let step = create_workflow_step(
        pool,
        CreateWorkflowStep {
            job_id: web_job.id,
            number: 1,
            name: "Run tests".to_owned(),
        },
    )
    .await?;
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
    .execute(pool)
    .await?;

    let deploy_job = create_workflow_job(
        pool,
        CreateWorkflowJob {
            run_id: run.id,
            name: "deploy preview".to_owned(),
            runner_label: Some("ubuntu-latest".to_owned()),
        },
    )
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'completed',
            conclusion = 'success',
            group_name = 'Deploy',
            attempt_number = 2,
            log_storage_key = 'actions/logs/deploy-preview.txt',
            log_deleted_at = now(),
            started_at = now() - interval '4 minutes',
            completed_at = now() - interval '2 minutes'
        WHERE id = $1
        "#,
    )
    .bind(deploy_job.id)
    .execute(pool)
    .await?;
    create_workflow_step(
        pool,
        CreateWorkflowStep {
            job_id: deploy_job.id,
            number: 1,
            name: "Publish preview".to_owned(),
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO workflow_annotations (
            run_id, job_id, step_id, annotation_level, path, start_line, end_line, title, message, raw_details
        )
        VALUES ($1, $2, $3, 'failure', 'web/src/app/page.tsx', 42, 42, 'Type error', 'Expected string, found number', 'tsc failed')
        "#,
    )
    .bind(run.id)
    .bind(web_job.id)
    .bind(step.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_artifacts (run_id, name, digest, size_bytes, storage_key, expired_at)
        VALUES ($1, 'playwright-report', 'sha256:abc123', 2048, 'actions/artifacts/report.zip', now() + interval '1 day')
        "#,
    )
    .bind(run.id)
    .execute(pool)
    .await?;

    Ok((
        format!("/{username}/{repository_name}/actions/runs/{}", run.id),
        format!(
            "/{username}/{repository_name}/actions/runs/{}/jobs/{}",
            run.id, web_job.id
        ),
    ))
}

async fn seed_merge_ready_pull_request(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    suffix: &str,
) -> anyhow::Result<String> {
    let repository_name = format!("merge-ready-{}", &suffix[..12]);
    let repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user_id },
            name: repository_name.clone(),
            description: Some("Pull request merge confirmation seed".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: user_id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await?;

    let head_commit = insert_commit(
        pool,
        repository.id,
        CreateCommit {
            oid: format!("merge-head-{}", Uuid::new_v4().simple()),
            author_user_id: Some(user_id),
            committer_user_id: Some(user_id),
            message: "Prepare merge confirmation fixture".to_owned(),
            tree_oid: Some(format!("merge-tree-{}", Uuid::new_v4().simple())),
            parent_oids: Vec::new(),
            committed_at: Utc::now(),
        },
    )
    .await?;
    upsert_git_ref(
        pool,
        repository.id,
        "refs/heads/feature/merge-confirmation",
        "branch",
        Some(head_commit.id),
    )
    .await?;

    let pull_request = create_pull_request(
        pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: user_id,
            title: "Confirm merge workflow".to_owned(),
            body: Some("Exercises method selection, commit fields, and branch cleanup.".to_owned()),
            head_ref: "feature/merge-confirmation".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO pull_request_files (
            pull_request_id, path, status, additions, deletions, byte_size
        )
        VALUES ($1, 'web/src/components/MergeConfirmation.tsx', 'added', 42, 3, 2400)
        "#,
    )
    .bind(pull_request.pull_request.id)
    .execute(pool)
    .await?;

    Ok(format!(
        "/{username}/{repository_name}/pull/{}",
        pull_request.pull_request.number
    ))
}

async fn seed_issue_templates(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let labels = ensure_default_labels(pool, repository_id).await?;
    let bug_label_id = labels
        .iter()
        .find(|label| label.name == "bug")
        .map(|label| label.id);
    let template_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO issue_templates (
            repository_id, slug, name, description, title_prefill, body, issue_type, display_order
        )
        VALUES (
            $1,
            'bug-report',
            'Bug report',
            'Report a reproducible defect.',
            '[Bug]: ',
            '### Expected behavior

### Actual behavior
',
            'bug',
            1
        )
        ON CONFLICT (repository_id, lower(slug))
        DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            title_prefill = EXCLUDED.title_prefill,
            body = EXCLUDED.body,
            issue_type = EXCLUDED.issue_type
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    if let Some(label_id) = bug_label_id {
        sqlx::query(
            r#"
            INSERT INTO issue_template_default_labels (template_id, label_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(template_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }
    sqlx::query(
        r#"
        INSERT INTO issue_template_default_assignees (template_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(template_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO issue_form_fields (
            template_id, field_key, label, field_type, description, placeholder, value, required, display_order
        )
        VALUES
            (
                $1,
                'steps',
                'Reproduction steps',
                'markdown',
                'Describe the shortest path that reproduces the defect.',
                '1. Open...\n2. Click...',
                '',
                true,
                1
            ),
            (
                $1,
                'environment',
                'Environment',
                'input',
                'Browser, OS, or runtime where the issue appears.',
                'Chrome on macOS',
                '',
                false,
                2
            )
        ON CONFLICT (template_id, field_key)
        DO UPDATE SET
            label = EXCLUDED.label,
            field_type = EXCLUDED.field_type,
            description = EXCLUDED.description,
            placeholder = EXCLUDED.placeholder,
            value = EXCLUDED.value,
            required = EXCLUDED.required,
            display_order = EXCLUDED.display_order
        "#,
    )
    .bind(template_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_tree_refs(pool: &PgPool, user_id: Uuid, repository_id: Uuid) -> anyhow::Result<()> {
    let default_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = 'refs/heads/main'
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let default_commit_oid =
        sqlx::query_scalar::<_, String>("SELECT oid FROM commits WHERE id = $1")
            .bind(default_commit_id)
            .fetch_one(pool)
            .await?;
    let feature_commit = insert_commit(
        pool,
        repository_id,
        CreateCommit {
            oid: format!("tree-feature-{}", Uuid::new_v4().simple()),
            author_user_id: Some(user_id),
            committer_user_id: Some(user_id),
            message: "Add docs on tree feature branch".to_owned(),
            tree_oid: None,
            parent_oids: vec![default_commit_oid],
            committed_at: Utc::now(),
        },
    )
    .await?;
    for (path, content) in [
        ("README.md", "# Feature tree branch\n"),
        ("docs/guide.md", "# Feature guide\n"),
        ("docs/reference/api.md", "# API reference\n"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(feature_commit.id)
        .bind(path)
        .bind(content)
        .bind(format!("{}-{}", feature_commit.oid, path.replace('/', "-")))
        .bind(content.len() as i64)
        .execute(pool)
        .await?;
    }
    for index in 0..72 {
        let path = format!("docs/example-{index:03}.md");
        let content = format!("# Example {index}\n");
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(feature_commit.id)
        .bind(&path)
        .bind(&content)
        .bind(format!("{}-{}", feature_commit.oid, path.replace('/', "-")))
        .bind(content.len() as i64)
        .execute(pool)
        .await?;
    }
    upsert_git_ref(
        pool,
        repository_id,
        "refs/heads/feature/tree-nav",
        "branch",
        Some(feature_commit.id),
    )
    .await?;
    upsert_git_ref(
        pool,
        repository_id,
        "refs/tags/v1.0.0",
        "tag",
        Some(default_commit_id),
    )
    .await?;
    ensure_default_labels(pool, repository_id).await?;
    sqlx::query(
        r#"
        INSERT INTO pull_request_templates (repository_id, slug, name, body)
        VALUES ($1, 'default', 'Default', '## Summary

Describe the change.
')
        ON CONFLICT (repository_id, lower(slug))
        DO UPDATE SET name = EXCLUDED.name, body = EXCLUDED.body
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_fork_compare_refs(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    source_owner_id: Uuid,
    source_owner_username: &str,
    suffix: &str,
) -> anyhow::Result<String> {
    let repository_name = format!("fork-base-{}", &suffix[..12]);
    let fork_name = format!("fork-head-{}", &suffix[..12]);
    let base_repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: source_owner_id,
            },
            name: repository_name.clone(),
            description: Some("Fork comparison base repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: source_owner_id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await?;
    let fork_repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: user_id },
            name: fork_name.clone(),
            description: Some("Fork comparison head repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: user_id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await?;
    sqlx::query(
        r#"
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id)
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(base_repository.id)
    .bind(fork_repository.id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let base_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = 'refs/heads/main'
        "#,
    )
    .bind(base_repository.id)
    .fetch_one(pool)
    .await?;
    let base_commit_oid = sqlx::query_scalar::<_, String>("SELECT oid FROM commits WHERE id = $1")
        .bind(base_commit_id)
        .fetch_one(pool)
        .await?;
    let fork_commit = insert_commit(
        pool,
        fork_repository.id,
        CreateCommit {
            oid: format!("fork-feature-{}", Uuid::new_v4().simple()),
            author_user_id: Some(user_id),
            committer_user_id: Some(user_id),
            message: "Add public fork contribution".to_owned(),
            tree_oid: None,
            parent_oids: vec![base_commit_oid],
            committed_at: Utc::now(),
        },
    )
    .await?;
    for (path, content) in [
        ("README.md", "# Fork contribution\n"),
        ("docs/fork-guide.md", "# Fork guide\n"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(fork_repository.id)
        .bind(fork_commit.id)
        .bind(path)
        .bind(content)
        .bind(format!("{}-{}", fork_commit.oid, path.replace('/', "-")))
        .bind(content.len() as i64)
        .execute(pool)
        .await?;
    }
    upsert_git_ref(
        pool,
        fork_repository.id,
        "refs/heads/feature/fork-contribution",
        "branch",
        Some(fork_commit.id),
    )
    .await?;

    Ok(format!(
        "/{source_owner_username}/{repository_name}/compare/main...feature%2Ffork-contribution?headOwner={username}&headRepo={fork_name}"
    ))
}

async fn seed_blob_edge_cases(pool: &PgPool, repository_id: Uuid) -> anyhow::Result<()> {
    let default_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = 'refs/heads/main'
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;

    let files = vec![
        ("assets/app.bin", "\u{1}\u{2}\u{3}\u{4}".to_owned()),
        ("logs/large.txt", "large line\n".repeat(60_000)),
        (
            "docs/symbols.md",
            "# Symbols\n\n## Install\n\n## Usage\n".to_owned(),
        ),
    ];
    for (path, content) in files {
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(default_commit_id)
        .bind(path)
        .bind(&content)
        .bind(format!(
            "edge-{}-{}",
            Uuid::new_v4().simple(),
            path.replace('/', "-")
        ))
        .bind(content.len() as i64)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_feed_event(
    pool: &PgPool,
    actor_user_id: Uuid,
    repository_id: Uuid,
    event_type: &str,
    title: &str,
    target_href: String,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            occurred_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, now())
        "#,
    )
    .bind(actor_user_id)
    .bind(repository_id)
    .bind(event_type)
    .bind(title)
    .bind(Some(
        "Seeded dashboard feed event for browser smoke tests".to_owned(),
    ))
    .bind(target_href)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_language(
    pool: &PgPool,
    repository_id: Uuid,
    language: &str,
    color: &str,
    byte_count: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO repository_languages (repository_id, language, color, byte_count)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, lower(language))
        DO UPDATE SET color = EXCLUDED.color, byte_count = EXCLUDED.byte_count
        "#,
    )
    .bind(repository_id)
    .bind(language)
    .bind(color)
    .bind(byte_count)
    .execute(pool)
    .await?;
    Ok(())
}
