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
        notifications::{create_notification, CreateNotification},
        permissions::RepositoryRole,
        pulls::{create_pull_request, CreatePullRequest},
        repositories::{
            create_organization, create_repository, create_repository_with_bootstrap,
            insert_commit, replace_repository_snapshot, upsert_git_ref, CreateCommit,
            CreateOrganization, CreateRepository, RepositoryBootstrapRequest, RepositoryOwner,
            RepositorySnapshot, RepositorySnapshotFile, RepositoryVisibility,
        },
        search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
        wiki::wiki_content_sha,
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
    profile_action_cookie_value: String,
    traffic_read_only_cookie_value: String,
    first_repository_href: String,
    private_profile_href: String,
    second_repository_href: String,
    social_source_repository_href: String,
    tree_repository_href: String,
    traffic_read_only_repository_href: String,
    fork_compare_href: String,
    pull_request_merge_href: String,
    actions_run_detail_href: String,
    actions_job_log_href: String,
    organization_profile_href: String,
    organization_empty_teams_href: String,
    repository_wiki_href: String,
    projects_workspace_href: String,
}

fn seed_empty_dashboard() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_EMPTY").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn skip_migrations() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_SKIP_MIGRATIONS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_tree_repository() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_TREE_REFS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_dependency_graph() -> bool {
    matches!(
        std::env::var("DASHBOARD_E2E_DEPENDENCY_GRAPH").as_deref(),
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

fn seed_organization_profile() -> bool {
    matches!(
        std::env::var("ORG_PROFILE_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_owner_packages() -> bool {
    matches!(
        std::env::var("OWNER_PACKAGES_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_account_security_second_identity() -> bool {
    matches!(
        std::env::var("ACCOUNT_SECURITY_E2E").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

fn seed_projects_workspace() -> bool {
    matches!(
        std::env::var("PROJECTS_WORKSPACE_E2E").as_deref(),
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
    if !skip_migrations() {
        MIGRATOR.run(&pool).await?;
    }

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
    opengithub_api::domain::identity::upsert_oauth_account(
        &pool,
        user.id,
        "google",
        &format!("dashboard-google-{suffix}"),
        &user.email,
    )
    .await?;
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&username)
        .bind(user.id)
        .execute(&pool)
        .await?;
    if seed_account_security_second_identity() {
        opengithub_api::domain::identity::upsert_oauth_account(
            &pool,
            user.id,
            "google",
            &format!("dashboard-google-second-{suffix}"),
            &format!("{username}+second@opengithub.local"),
        )
        .await?;
    }
    let profile_action_viewer_username = format!("profile-viewer-{}", &suffix[..12]);
    let profile_action_viewer = upsert_user_by_email(
        &pool,
        &format!("{profile_action_viewer_username}@opengithub.local"),
        Some("Profile Action Viewer"),
        None,
    )
    .await?;
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&profile_action_viewer_username)
        .bind(profile_action_viewer.id)
        .execute(&pool)
        .await?;
    let private_profile_username = format!("private-profile-{}", &suffix[..12]);
    let private_profile_user = upsert_user_by_email(
        &pool,
        &format!("{private_profile_username}@opengithub.local"),
        Some("Private Profile Tester"),
        None,
    )
    .await?;
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1,
            bio = 'Private profile smoke fixture.',
            profile_visibility = 'private'
        WHERE id = $2
        "#,
    )
    .bind(&private_profile_username)
    .bind(private_profile_user.id)
    .execute(&pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO user_profile_readmes (user_id, body, rendered_html, updated_by_user_id)
        VALUES ($1, $2, $3, $1)
        ON CONFLICT (user_id)
        DO UPDATE SET body = EXCLUDED.body,
                      rendered_html = EXCLUDED.rendered_html,
                      updated_by_user_id = EXCLUDED.updated_by_user_id
        "#,
    )
    .bind(private_profile_user.id)
    .bind("Private profile readme stays visible.")
    .bind("<p>Private profile readme stays visible.</p>")
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
    let (tree_repository_href, tree_repository_id, fork_compare_href) = if seed_tree_repository() {
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
        seed_repository_traffic(&pool, tree_repository.id).await?;
        if seed_dependency_graph() {
            seed_dependency_graph_fixture(&pool, tree_repository.id, &suffix).await?;
        }
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
            Some(tree_repository.id),
            fork_compare_href,
        )
    } else {
        (String::new(), None, String::new())
    };
    let mut pull_request_merge_href = String::new();
    let (first_repository_href, second_repository_href, repository_wiki_href) =
        if seed_empty_dashboard() {
            (String::new(), String::new(), String::new())
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
            seed_repository_wiki(&pool, first_repository.id, user.id).await?;
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
            if seed_owner_packages() {
                seed_user_owner_packages(&pool, first_repository.id, user.id, &suffix).await?;
            }

            sqlx::query(
                r#"
            UPDATE users
            SET bio = $1,
                company = $2,
                location = $3,
                website_url = $4,
                profile_visibility = 'public'
            WHERE id = $5
            "#,
            )
            .bind("Building calm developer tools at Namuh.")
            .bind("Namuh")
            .bind("San Francisco")
            .bind("https://namuh.co")
            .bind(user.id)
            .execute(&pool)
            .await?;
            sqlx::query(
                r#"
            INSERT INTO user_profile_readmes (user_id, body, rendered_html, updated_by_user_id)
            VALUES ($1, $2, $3, $1)
            ON CONFLICT (user_id)
            DO UPDATE SET body = EXCLUDED.body,
                          rendered_html = EXCLUDED.rendered_html,
                          updated_by_user_id = EXCLUDED.updated_by_user_id
            "#,
            )
            .bind(user.id)
            .bind("# Dashboard Tester\nSeeded profile overview for browser smoke.")
            .bind("<h1>Dashboard Tester</h1><p>Seeded profile overview for browser smoke.</p>")
            .execute(&pool)
            .await?;
            sqlx::query(
                r#"
            INSERT INTO profile_pins (user_id, repository_id, position)
            VALUES ($1, $2, 1)
            ON CONFLICT (user_id, repository_id)
            DO UPDATE SET position = EXCLUDED.position
            "#,
            )
            .bind(user.id)
            .bind(first_repository.id)
            .execute(&pool)
            .await?;
            sqlx::query(
                r#"
            INSERT INTO profile_contribution_days (user_id, day, contribution_count)
            VALUES ($1, CURRENT_DATE, 5)
            ON CONFLICT (user_id, day)
            DO UPDATE SET contribution_count = EXCLUDED.contribution_count
            "#,
            )
            .bind(user.id)
            .execute(&pool)
            .await?;
            sqlx::query(
                r#"
            INSERT INTO profile_contribution_events
                (user_id, repository_id, event_type, title, target_href)
            VALUES ($1, $2, 'push', 'Pushed seeded profile overview', $3)
            "#,
            )
            .bind(user.id)
            .bind(first_repository.id)
            .bind(format!(
                "/{username}/{first_repository_name}/commit/profile"
            ))
            .execute(&pool)
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
            let dashboard_commit = insert_commit(
                &pool,
                first_repository.id,
                CreateCommit {
                    oid: format!("{}abcdef", &suffix[..16]),
                    author_user_id: Some(user.id),
                    committer_user_id: Some(user.id),
                    message: "Wire dashboard activity feed".to_owned(),
                    tree_oid: Some(format!("tree-dashboard-{}", &suffix[..12])),
                    parent_oids: vec![],
                    committed_at: Utc::now(),
                },
            )
            .await?;
            upsert_git_ref(
                &pool,
                first_repository.id,
                "refs/heads/main",
                "branch",
                Some(dashboard_commit.id),
            )
            .await?;
            for (path, content) in [
                ("README.md", "# Dashboard workspace\n"),
                ("docs/index.html", "<h1>Dashboard Pages</h1>\n"),
            ] {
                sqlx::query(
                r#"
                INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(first_repository.id)
            .bind(dashboard_commit.id)
            .bind(path)
            .bind(content)
            .bind(format!(
                "{}-{}",
                dashboard_commit.oid,
                path.replace('/', "-")
            ))
            .bind(content.len() as i64)
            .execute(&pool)
            .await?;
            }
            let dashboard_issue = create_issue(
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
            create_notification(
                &pool,
                CreateNotification {
                    user_id: user.id,
                    repository_id: Some(first_repository.id),
                    subject_type: "issue".to_owned(),
                    subject_id: Some(dashboard_issue.id),
                    title: "Triage dashboard setup workflow".to_owned(),
                    reason: "mention".to_owned(),
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
                format!("/{username}/{first_repository_name}/wiki"),
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
    let (organization_profile_href, organization_empty_teams_href) = if seed_organization_profile()
    {
        seed_organization_profile_fixture(&pool, user.id, profile_action_viewer.id, &suffix).await?
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
    if seed_account_security_second_identity() {
        let desktop_session_id = Uuid::new_v4().to_string();
        let mobile_session_id = Uuid::new_v4().to_string();
        let expired_session_id = Uuid::new_v4().to_string();
        upsert_session(
            &pool,
            &desktop_session_id,
            Some(user.id),
            serde_json::json!({ "provider": "google", "fixture": "desktop" }),
            expires_at,
        )
        .await?;
        upsert_session(
            &pool,
            &mobile_session_id,
            Some(user.id),
            serde_json::json!({ "provider": "google", "fixture": "mobile" }),
            expires_at,
        )
        .await?;
        upsert_session(
            &pool,
            &expired_session_id,
            Some(user.id),
            serde_json::json!({ "provider": "google", "fixture": "expired" }),
            Utc::now() - Duration::hours(1),
        )
        .await?;
        sqlx::query(
            r#"
            UPDATE sessions
            SET user_agent = CASE id
                  WHEN $1 THEN 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0 Safari/537.36 VeryLongDeviceLabel/abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz'
                  WHEN $2 THEN 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1'
                  ELSE user_agent
                END,
                ip_inet = CASE id
                  WHEN $1 THEN '2001:db8::42'::inet
                  WHEN $2 THEN '10.44.55.66'::inet
                  ELSE ip_inet
                END,
                last_active_at = CASE id
                  WHEN $1 THEN now() - interval '10 minutes'
                  WHEN $2 THEN now() - interval '20 minutes'
                  ELSE last_active_at
                END
            WHERE id IN ($1, $2, $3)
            "#,
        )
        .bind(&desktop_session_id)
        .bind(&mobile_session_id)
        .bind(&expired_session_id)
        .execute(&pool)
        .await?;
    }
    let set_cookie = session::set_cookie_header(&config, &session_id, expires_at)?;
    let cookie_value = session::cookie_value_from_set_cookie(&set_cookie)
        .ok_or_else(|| anyhow::anyhow!("set-cookie did not include a value"))?;
    let profile_action_session_id = Uuid::new_v4().to_string();
    upsert_session(
        &pool,
        &profile_action_session_id,
        Some(profile_action_viewer.id),
        serde_json::json!({ "provider": "google" }),
        expires_at,
    )
    .await?;
    let profile_action_set_cookie =
        session::set_cookie_header(&config, &profile_action_session_id, expires_at)?;
    let profile_action_cookie_value =
        session::cookie_value_from_set_cookie(&profile_action_set_cookie)
            .ok_or_else(|| anyhow::anyhow!("profile action set-cookie did not include a value"))?;
    let (traffic_read_only_cookie_value, traffic_read_only_repository_href) =
        if let Some(tree_repository_id) = tree_repository_id {
            let traffic_reader_username = format!("traffic-reader-{}", &suffix[..12]);
            let traffic_reader = upsert_user_by_email(
                &pool,
                &format!("{traffic_reader_username}@opengithub.local"),
                Some("Traffic Read Only"),
                None,
            )
            .await?;
            sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
                .bind(&traffic_reader_username)
                .bind(traffic_reader.id)
                .execute(&pool)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO repository_permissions (repository_id, user_id, role, source)
                VALUES ($1, $2, 'read', 'direct')
                ON CONFLICT (repository_id, user_id)
                DO UPDATE SET role = EXCLUDED.role
                "#,
            )
            .bind(tree_repository_id)
            .bind(traffic_reader.id)
            .execute(&pool)
            .await?;
            let read_only_session_id = Uuid::new_v4().to_string();
            upsert_session(
                &pool,
                &read_only_session_id,
                Some(traffic_reader.id),
                serde_json::json!({ "provider": "google" }),
                expires_at,
            )
            .await?;
            let read_only_set_cookie =
                session::set_cookie_header(&config, &read_only_session_id, expires_at)?;
            let read_only_cookie_value =
                session::cookie_value_from_set_cookie(&read_only_set_cookie).ok_or_else(|| {
                    anyhow::anyhow!("traffic read-only set-cookie did not include a value")
                })?;
            (
                read_only_cookie_value.to_owned(),
                tree_repository_href.clone(),
            )
        } else {
            (String::new(), String::new())
        };

    let projects_workspace_href = if seed_projects_workspace() {
        seed_projects_workspace_fixture(&pool, user.id, &suffix).await?
    } else {
        String::new()
    };

    let output = SeedOutput {
        cookie_name: config.session_cookie_name,
        cookie_value: cookie_value.to_owned(),
        profile_action_cookie_value: profile_action_cookie_value.to_owned(),
        traffic_read_only_cookie_value,
        first_repository_href,
        private_profile_href: format!("/{private_profile_username}"),
        second_repository_href,
        social_source_repository_href: format!(
            "/{}/{}",
            social_source_repository.owner_login, social_source_repository.name
        ),
        tree_repository_href,
        traffic_read_only_repository_href,
        fork_compare_href,
        pull_request_merge_href,
        actions_run_detail_href,
        actions_job_log_href,
        organization_profile_href,
        organization_empty_teams_href,
        repository_wiki_href,
        projects_workspace_href,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

async fn seed_projects_workspace_fixture(
    pool: &PgPool,
    actor_user_id: Uuid,
    suffix: &str,
) -> anyhow::Result<String> {
    let organization_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organizations (slug, display_name, description, owner_user_id)
        VALUES ('namuh', 'Namuh', 'Seeded organization for Projects workspace smoke tests.', $1)
        ON CONFLICT (lower(slug))
        DO UPDATE SET display_name = EXCLUDED.display_name,
                      description = EXCLUDED.description
        RETURNING id
        "#,
    )
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT (organization_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::Organization {
                id: organization_id,
            },
            name: format!("projects-workspace-{}", &suffix[..12]),
            description: Some("Seeded Projects v2 workspace source repository.".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: actor_user_id,
        },
    )
    .await?;
    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 1, 'Wire the table shell', 'Seeded issue for Projects v2 workspace QA.', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
          owner_organization_id, number, title, short_description, visibility,
          default_repository_id, created_by_user_id
        )
        VALUES ($1, 1, 'Editorial table workspace', 'Projects v2 saved views and editable table.', 'private', $2, $3)
        ON CONFLICT (owner_organization_id, number) WHERE owner_organization_id IS NOT NULL
        DO UPDATE SET title = EXCLUDED.title,
                      short_description = EXCLUDED.short_description,
                      visibility = EXCLUDED.visibility,
                      default_repository_id = EXCLUDED.default_repository_id,
                      updated_at = now()
        RETURNING id
        "#,
    )
    .bind(organization_id)
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO project_permissions (project_id, user_id, role, source)
        VALUES ($1, $2, 'admin', 'direct')
        ON CONFLICT (project_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(project_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO project_repositories (project_id, repository_id, link_type)
        VALUES ($1, $2, 'default')
        ON CONFLICT (project_id, repository_id) DO NOTHING
        "#,
    )
    .bind(project_id)
    .bind(repository.id)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM project_items WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM project_fields WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM project_views WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO project_views (project_id, name, layout, position, configuration)
        VALUES ($1, 'Table', 'table', 1, '{"sort":"manual"}'),
               ($1, 'Bugs', 'table', 2, '{"query":"label:bug"}')
        "#,
    )
    .bind(project_id)
    .execute(pool)
    .await?;
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    let priority_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Priority', 'single_select', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    let target_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Target date', 'date', 3) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO project_field_options (project_field_id, name, color, position, description)
        VALUES ($1, 'Backlog', 'gray', 1, 'Not started'),
               ($1, 'In progress', 'blue', 2, 'Actively moving'),
               ($1, 'Done', 'green', 3, 'Completed work'),
               ($2, 'P1', 'red', 1, 'Highest priority'),
               ($2, 'P2', 'orange', 2, 'Normal priority')
        "#,
    )
    .bind(status_field)
    .bind(priority_field)
    .execute(pool)
    .await?;
    let iteration_field: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_fields (project_id, name, field_type, position, settings)
        VALUES ($1, 'Iteration', 'iteration', 4, '{"duration":2,"durationUnit":"weeks"}'::jsonb)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO project_iterations (project_field_id, name, start_date, duration_days, position)
        VALUES ($1, 'Iteration 1', '2026-05-04', 14, 1),
               ($1, 'Iteration 2', '2026-05-18', 14, 2),
               ($1, 'Iteration 3', '2026-06-01', 14, 3)
        "#,
    )
    .bind(iteration_field)
    .execute(pool)
    .await?;
    let issue_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, issue_id, position) VALUES ($1, 'issue', $2, 1) RETURNING id",
    )
    .bind(project_id)
    .bind(issue_id)
    .fetch_one(pool)
    .await?;
    let draft_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, body, position) VALUES ($1, 'draft_issue', 'Draft launch notes', 'Project-only draft for add-row and editor smoke.', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    for (item_id, status, priority, target) in [
        (issue_item_id, "In progress", "P1", "2026-05-20"),
        (draft_item_id, "Backlog", "P2", "2026-06-01"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id)
            VALUES ($1, $2, $3, $5), ($1, $4, $6, $5), ($1, $7, $8, $5), ($1, $9, $10, $5)
            "#,
        )
        .bind(item_id)
        .bind(status_field)
        .bind(serde_json::json!(status))
        .bind(priority_field)
        .bind(actor_user_id)
        .bind(serde_json::json!(priority))
        .bind(target_field)
        .bind(serde_json::json!(target))
        .bind(iteration_field)
        .bind(serde_json::json!("2026-05-11"))
        .execute(pool)
        .await?;
    }

    Ok("/orgs/namuh/projects/1/views/1".to_owned())
}

async fn seed_repository_wiki(
    pool: &PgPool,
    repository_id: Uuid,
    author_user_id: Uuid,
) -> anyhow::Result<()> {
    let wiki_repository_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO wiki_repositories (repository_id)
        VALUES ($1)
        ON CONFLICT (repository_id)
        DO UPDATE SET updated_at = now()
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;

    for (position, title, slug, markdown, is_sidebar, is_footer) in [
        (
            0,
            "Home",
            "Home",
            "# Home\n\nWelcome to the seeded repository wiki.\n\n## Getting started\n\nUse the page list to open the architecture guide.",
            false,
            false,
        ),
        (
            1,
            "Architecture Guide",
            "Architecture Guide",
            "# Architecture Guide\n\nThe wiki reader is backed by Rust-rendered Markdown.\n\n## Services\n\nAPI, web, worker, and storage responsibilities stay separated.\n\n## Operations\n\nKeep deployments observable.",
            false,
            false,
        ),
        (
            2,
            "Runbook",
            "Runbook",
            "# Runbook\n\n## Rollback\n\nUse the deployment runbook.",
            false,
            false,
        ),
        (
            3,
            "_Sidebar",
            "_sidebar",
            "## Wiki links\n\n- [Architecture](Architecture%20Guide)\n- [Runbook](Runbook)",
            true,
            false,
        ),
        (
            4,
            "_Footer",
            "_footer",
            "Maintained by platform engineering.",
            false,
            true,
        ),
    ] {
        let page_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO wiki_pages (
                wiki_repository_id,
                title,
                slug,
                path,
                is_sidebar,
                is_footer,
                position
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (wiki_repository_id, lower(slug))
            DO UPDATE SET title = EXCLUDED.title,
                          path = EXCLUDED.path,
                          is_sidebar = EXCLUDED.is_sidebar,
                          is_footer = EXCLUDED.is_footer,
                          position = EXCLUDED.position
            RETURNING id
            "#,
        )
        .bind(wiki_repository_id)
        .bind(title)
        .bind(slug)
        .bind(format!("{slug}.md"))
        .bind(is_sidebar)
        .bind(is_footer)
        .bind(position)
        .fetch_one(pool)
        .await?;
        let revision_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO wiki_page_revisions (
                page_id,
                author_user_id,
                commit_oid,
                message,
                markdown,
                content_sha
            )
            VALUES ($1, $2, $3, 'Seed wiki page', $4, $5)
            RETURNING id
            "#,
        )
        .bind(page_id)
        .bind(author_user_id)
        .bind(format!("wiki{}", Uuid::new_v4().simple()))
        .bind(markdown)
        .bind(wiki_content_sha(markdown))
        .fetch_one(pool)
        .await?;
        sqlx::query("UPDATE wiki_pages SET latest_revision_id = $1 WHERE id = $2")
            .bind(revision_id)
            .bind(page_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

struct OwnerPackageSeed<'a> {
    repository_id: Uuid,
    owner_user_id: Option<Uuid>,
    owner_organization_id: Option<Uuid>,
    created_by_user_id: Uuid,
    name: &'a str,
    package_type: &'a str,
    visibility: &'a str,
    version: &'a str,
    downloads: i64,
}

async fn seed_package_row(pool: &PgPool, seed: OwnerPackageSeed<'_>) -> anyhow::Result<()> {
    let package_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO packages (
            repository_id,
            owner_user_id,
            owner_organization_id,
            created_by_user_id,
            name,
            package_type,
            visibility
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(seed.repository_id)
    .bind(seed.owner_user_id)
    .bind(seed.owner_organization_id)
    .bind(seed.created_by_user_id)
    .bind(seed.name)
    .bind(seed.package_type)
    .bind(seed.visibility)
    .fetch_one(pool)
    .await?;

    let version_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO package_versions (
            package_id,
            version,
            published_by_user_id,
            created_at
        )
        VALUES ($1, $2, $3, now() - INTERVAL '1 hour')
        RETURNING id
        "#,
    )
    .bind(package_id)
    .bind(seed.version)
    .bind(seed.created_by_user_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO package_repository_links (package_id, repository_id, link_type)
        VALUES ($1, $2, 'source')
        "#,
    )
    .bind(package_id)
    .bind(seed.repository_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO package_downloads (package_id, package_version_id, download_count)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(package_id)
    .bind(version_id)
    .bind(seed.downloads)
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_user_owner_packages(
    pool: &PgPool,
    repository_id: Uuid,
    owner_user_id: Uuid,
    suffix: &str,
) -> anyhow::Result<()> {
    let marker = &suffix[..12];
    seed_package_row(
        pool,
        OwnerPackageSeed {
            repository_id,
            owner_user_id: Some(owner_user_id),
            owner_organization_id: None,
            created_by_user_id: owner_user_id,
            name: &format!("list-container-{marker}"),
            package_type: "container",
            visibility: "public",
            version: "2.0.0",
            downloads: 210,
        },
    )
    .await?;
    seed_package_row(
        pool,
        OwnerPackageSeed {
            repository_id,
            owner_user_id: Some(owner_user_id),
            owner_organization_id: None,
            created_by_user_id: owner_user_id,
            name: &format!("list-private-{marker}"),
            package_type: "npm",
            visibility: "private",
            version: "0.3.0",
            downloads: 7,
        },
    )
    .await?;
    Ok(())
}

async fn seed_organization_owner_packages(
    pool: &PgPool,
    repository_id: Uuid,
    organization_id: Uuid,
    owner_user_id: Uuid,
    marker: &str,
) -> anyhow::Result<()> {
    seed_package_row(
        pool,
        OwnerPackageSeed {
            repository_id,
            owner_user_id: None,
            owner_organization_id: Some(organization_id),
            created_by_user_id: owner_user_id,
            name: &format!("org-public-{marker}"),
            package_type: "maven",
            visibility: "public",
            version: "1.1.0",
            downloads: 88,
        },
    )
    .await?;
    seed_package_row(
        pool,
        OwnerPackageSeed {
            repository_id,
            owner_user_id: None,
            owner_organization_id: Some(organization_id),
            created_by_user_id: owner_user_id,
            name: &format!("org-internal-{marker}"),
            package_type: "nuget",
            visibility: "internal",
            version: "1.2.0",
            downloads: 33,
        },
    )
    .await?;
    seed_package_row(
        pool,
        OwnerPackageSeed {
            repository_id,
            owner_user_id: None,
            owner_organization_id: Some(organization_id),
            created_by_user_id: owner_user_id,
            name: &format!("org-private-{marker}"),
            package_type: "npm",
            visibility: "private",
            version: "9.9.9",
            downloads: 5,
        },
    )
    .await?;
    Ok(())
}

async fn seed_organization_profile_fixture(
    pool: &PgPool,
    owner_user_id: Uuid,
    member_user_id: Uuid,
    suffix: &str,
) -> anyhow::Result<(String, String)> {
    let slug = format!("org-profile-{}", &suffix[..12]);
    let organization = create_organization(
        pool,
        CreateOrganization {
            slug: slug.clone(),
            display_name: "Namuh Engineering".to_owned(),
            description: Some("Shipping side projects in the open.".to_owned()),
            owner_user_id,
        },
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE organizations
        SET avatar_url = $1,
            website_url = $2,
            location = $3,
            profile_visibility = 'public',
            public_members_visible = true
        WHERE id = $4
        "#,
    )
    .bind(Option::<String>::None)
    .bind("https://namuh.co")
    .bind("Seoul")
    .bind(organization.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO organization_verified_domains (organization_id, domain)
        VALUES ($1, 'namuh.co')
        ON CONFLICT (organization_id, lower(domain)) DO NOTHING
        "#,
    )
    .bind(organization.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role, created_at)
        VALUES ($1, $2, 'member', now() - INTERVAL '1 day')
        ON CONFLICT (organization_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(organization.id)
    .bind(member_user_id)
    .execute(pool)
    .await?;

    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::Organization {
                id: organization.id,
            },
            name: format!("opengithub-{}", &suffix[..8]),
            description: Some("A rust-first collaboration platform.".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner_user_id,
        },
    )
    .await?;
    let preview_repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::Organization {
                id: organization.id,
            },
            name: format!("ralph-{}", &suffix[..8]),
            description: Some("Autonomous build loop tooling.".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner_user_id,
        },
    )
    .await?;
    if seed_owner_packages() {
        seed_organization_owner_packages(
            pool,
            repository.id,
            organization.id,
            owner_user_id,
            &suffix[..12],
        )
        .await?;
    }
    sqlx::query(
        r#"
        UPDATE repositories
        SET license_template_slug = 'mit',
            is_template = true,
            updated_at = now() - INTERVAL '2 hours'
        WHERE id = $1
        "#,
    )
    .bind(repository.id)
    .execute(pool)
    .await?;
    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '1 hour' WHERE id = $1")
        .bind(preview_repository.id)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_profile_pins (organization_id, repository_id, position)
        VALUES ($1, $2, 1), ($1, $3, 2)
        ON CONFLICT (organization_id, repository_id)
        DO UPDATE SET position = EXCLUDED.position
        "#,
    )
    .bind(organization.id)
    .bind(repository.id)
    .bind(preview_repository.id)
    .execute(pool)
    .await?;
    upsert_language(pool, repository.id, "Rust", "#b7410e", 9000).await?;
    upsert_language(pool, preview_repository.id, "TypeScript", "#8c5a3c", 3000).await?;
    sqlx::query(
        r#"
        INSERT INTO repository_topics (repository_id, topic)
        VALUES ($1, 'developer-tools'), ($1, 'forge')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(repository.id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO repository_topics (repository_id, topic)
        VALUES ($1, 'automation')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(preview_repository.id)
    .execute(pool)
    .await?;
    let platform_team_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO teams (
            organization_id,
            slug,
            name,
            description,
            visibility,
            notifications_enabled,
            updated_at
        )
        VALUES ($1, $2, 'Platform Maintainers', 'Runtime, release, and repository access owners.', 'visible', true, now() - INTERVAL '3 hours')
        RETURNING id
        "#,
    )
    .bind(organization.id)
    .bind(format!("platform-{}", &suffix[..8]))
    .fetch_one(pool)
    .await?;
    let frontend_team_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO teams (
            organization_id,
            parent_team_id,
            slug,
            name,
            description,
            visibility,
            notifications_enabled,
            updated_at
        )
        VALUES ($1, $2, $3, 'Frontend Studio', 'Editorial interface work and design-system stewardship.', 'visible', true, now() - INTERVAL '2 hours')
        RETURNING id
        "#,
    )
    .bind(organization.id)
    .bind(platform_team_id)
    .bind(format!("frontend-{}", &suffix[..8]))
    .fetch_one(pool)
    .await?;
    let security_team_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO teams (
            organization_id,
            slug,
            name,
            description,
            visibility,
            notifications_enabled,
            updated_at
        )
        VALUES ($1, $2, 'Security Response', 'Private security and incident response coordination.', 'secret', false, now() - INTERVAL '1 hour')
        RETURNING id
        "#,
    )
    .bind(organization.id)
    .bind(format!("security-{}", &suffix[..8]))
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO team_memberships (team_id, user_id, role)
        VALUES ($1, $2, 'maintainer'),
               ($3, $4, 'member'),
               ($5, $2, 'maintainer')
        "#,
    )
    .bind(platform_team_id)
    .bind(owner_user_id)
    .bind(frontend_team_id)
    .bind(member_user_id)
    .bind(security_team_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO repository_team_permissions (repository_id, team_id, role, source)
        VALUES ($1, $2, 'maintain', 'team'),
               ($3, $4, 'write', 'team')
        "#,
    )
    .bind(repository.id)
    .bind(platform_team_id)
    .bind(preview_repository.id)
    .bind(frontend_team_id)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(owner_user_id)
    .bind(repository.id)
    .execute(pool)
    .await?;
    let issue_id: Uuid = sqlx::query_scalar(
        "INSERT INTO issues (repository_id, number, title, author_user_id) VALUES ($1, 1, 'Seeded organization issue', $2) RETURNING id",
    )
    .bind(repository.id)
    .bind(owner_user_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        "INSERT INTO pull_requests (repository_id, issue_id, number, title, author_user_id, head_ref, base_ref, head_repository_id, base_repository_id) VALUES ($1, $2, 2, 'Seeded organization PR', $3, 'feature/org-profile', 'main', $1, $1)",
    )
    .bind(repository.id)
    .bind(issue_id)
    .bind(owner_user_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO organization_invitations (
            organization_id,
            invited_email,
            role,
            status,
            token_hash,
            invited_by_user_id,
            email_delivery_status,
            email_delivery_error,
            failed_at,
            expires_at
        )
        VALUES ($1, $2, 'member', 'failed', $3, $4, 'failed', 'SES sandbox rejected recipient', now(), now() + INTERVAL '7 days')
        ON CONFLICT (organization_id, lower(invited_email)) WHERE status = 'pending'
        DO NOTHING
        "#,
    )
    .bind(organization.id)
    .bind(format!("failed-invite-{}@opengithub.local", &suffix[..12]))
    .bind(format!("sha256:{}", Uuid::new_v4().simple()))
    .bind(owner_user_id)
    .execute(pool)
    .await?;

    let empty_slug = format!("org-empty-teams-{}", &suffix[..12]);
    let empty_organization = create_organization(
        pool,
        CreateOrganization {
            slug: empty_slug.clone(),
            display_name: "Empty Team Directory".to_owned(),
            description: Some("Organization with no teams for smoke testing.".to_owned()),
            owner_user_id,
        },
    )
    .await?;
    sqlx::query(
        r#"
        UPDATE organizations
        SET profile_visibility = 'public',
            public_members_visible = true
        WHERE id = $1
        "#,
    )
    .bind(empty_organization.id)
    .execute(pool)
    .await?;

    Ok((format!("/orgs/{slug}"), format!("/orgs/{empty_slug}/teams")))
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
                    "pub fn {marker}() {{\n    let {marker}_router = true;\n    println!(\"search phase three {marker}\");\n    assert!({marker}_router);\n    tracing::info!(\"{marker} indexed\");\n}}\n"
                ),
                oid: format!("blob-{marker}"),
                byte_size: 180,
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

async fn seed_repository_traffic(pool: &PgPool, repository_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO repository_traffic_daily (
            repository_id, traffic_date, clones_total, clones_unique, visitors_total, visitors_unique
        )
        VALUES
            ($1, current_date - interval '2 days', 8, 3, 32, 14),
            ($1, current_date - interval '1 day', 12, 5, 48, 20),
            ($1, current_date, 14, 6, 55, 23)
        ON CONFLICT (repository_id, traffic_date)
        DO UPDATE SET
            clones_total = EXCLUDED.clones_total,
            clones_unique = EXCLUDED.clones_unique,
            visitors_total = EXCLUDED.visitors_total,
            visitors_unique = EXCLUDED.visitors_unique
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_referrers_daily (
            repository_id, traffic_date, referrer, total_views, unique_visitors
        )
        VALUES
            ($1, current_date - interval '1 day', 'https://search.opengithub.local/results?q=traffic', 24, 10),
            ($1, current_date - interval '1 day', 'https://example.com/docs', 12, 6),
            ($1, current_date - interval '1 day', 'https://very-long-referrer.example.com/docs/product/analytics/traffic/reports/2026/05/that-keeps-wrapping-in-the-table', 5, 2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_popular_content_daily (
            repository_id, traffic_date, path, title, total_views, unique_visitors
        )
        VALUES
            ($1, current_date - interval '1 day', 'README.md', 'README', 30, 12),
            ($1, current_date - interval '1 day', 'src/main.rs', 'Application entrypoint', 16, 7),
            ($1, current_date - interval '1 day', 'docs/product/analytics/traffic/reports/2026/05/very-long-file-name.md', 'Very long traffic report', 5, 2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(repository_id)
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
    let stale_commit = insert_commit(
        pool,
        repository_id,
        CreateCommit {
            oid: format!("tree-stale-{}", Uuid::new_v4().simple()),
            author_user_id: Some(user_id),
            committer_user_id: Some(user_id),
            message: "Archive old release branch".to_owned(),
            tree_oid: None,
            parent_oids: vec![],
            committed_at: Utc::now() - Duration::days(140),
        },
    )
    .await?;
    upsert_git_ref(
        pool,
        repository_id,
        "refs/heads/release/old-tree",
        "branch",
        Some(stale_commit.id),
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

async fn seed_dependency_graph_fixture(
    pool: &PgPool,
    repository_id: Uuid,
    suffix: &str,
) -> anyhow::Result<()> {
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = 'refs/heads/main'
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let short_suffix = &suffix[..12];
    for (path, content) in [
        (
            "package.json",
            r#"{"dependencies":{"@playwright/test":"^1.56.0"},"devDependencies":{"vitest":"^4.0.0"}}"#,
        ),
        (
            "package-lock.json",
            r#"{"packages":{"node_modules/@playwright/test":{"version":"1.56.0"},"node_modules/vitest":{"version":"4.0.0"}}}"#,
        ),
        (
            "crates/api/Cargo.toml",
            r#"[package]
name = "opengithub-api"
[dependencies]
sqlx = "0.8"
"#,
        ),
    ] {
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repository_id, commit_id, lower(path))
            DO UPDATE SET content = EXCLUDED.content, oid = EXCLUDED.oid, byte_size = EXCLUDED.byte_size
            "#,
        )
        .bind(repository_id)
        .bind(commit_id)
        .bind(path)
        .bind(content)
        .bind(format!("dependency-{short_suffix}-{}", path.replace('/', "-")))
        .bind(content.len() as i64)
        .execute(pool)
        .await?;
    }

    let package_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO dependency_packages (ecosystem, name, package_href)
        VALUES ('npm', '@playwright/test', 'https://www.npmjs.com/package/@playwright/test')
        ON CONFLICT (ecosystem, lower(name))
        DO UPDATE SET package_href = EXCLUDED.package_href, updated_at = now()
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await?;
    let manifest_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO dependency_manifests (repository_id, path, ecosystem, lockfile_path, dependency_count)
        VALUES ($1, 'package.json', 'npm', 'package-lock.json', 2)
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET ecosystem = EXCLUDED.ecosystem,
                      lockfile_path = EXCLUDED.lockfile_path,
                      dependency_count = EXCLUDED.dependency_count,
                      updated_at = now()
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO repository_dependencies (
            repository_id, manifest_id, package_id, package_version, relationship, license, lockfile_path
        )
        VALUES ($1, $2, $3, '1.56.0', 'direct', 'Apache-2.0', 'package-lock.json')
        ON CONFLICT (manifest_id, package_id, relationship)
        DO UPDATE SET package_version = EXCLUDED.package_version,
                      license = EXCLUDED.license,
                      lockfile_path = EXCLUDED.lockfile_path,
                      updated_at = now()
        "#,
    )
    .bind(repository_id)
    .bind(manifest_id)
    .bind(package_id)
    .execute(pool)
    .await?;

    let public_owner_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (username, email, display_name, avatar_url)
        VALUES ($1, $2, $3, NULL)
        ON CONFLICT (lower(email)) DO UPDATE SET username = EXCLUDED.username
        RETURNING id
        "#,
    )
    .bind(format!("public-consumer-{short_suffix}"))
    .bind(format!("public-consumer-{short_suffix}@opengithub.local"))
    .bind(format!("Public consumer {short_suffix}"))
    .fetch_one(pool)
    .await?;
    let private_owner_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (username, email, display_name, avatar_url)
        VALUES ($1, $2, $3, NULL)
        ON CONFLICT (lower(email)) DO UPDATE SET username = EXCLUDED.username
        RETURNING id
        "#,
    )
    .bind(format!("private-consumer-{short_suffix}"))
    .bind(format!("private-consumer-{short_suffix}@opengithub.local"))
    .bind(format!("Private consumer {short_suffix}"))
    .fetch_one(pool)
    .await?;
    let public_repo_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repositories (owner_user_id, name, description, visibility, default_branch, created_by_user_id)
        VALUES ($1, $2, 'Uses the opengithub package in production.', 'public', 'main', $1)
        ON CONFLICT (owner_user_id, lower(name)) WHERE owner_user_id IS NOT NULL
        DO UPDATE SET description = EXCLUDED.description, visibility = 'public'
        RETURNING id
        "#,
    )
    .bind(public_owner_id)
    .bind(format!("workflow-tools-{short_suffix}"))
    .fetch_one(pool)
    .await?;
    let private_repo_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repositories (owner_user_id, name, description, visibility, default_branch, created_by_user_id)
        VALUES ($1, $2, 'Private dependent repository.', 'private', 'main', $1)
        ON CONFLICT (owner_user_id, lower(name)) WHERE owner_user_id IS NOT NULL
        DO UPDATE SET description = EXCLUDED.description, visibility = 'private'
        RETURNING id
        "#,
    )
    .bind(private_owner_id)
    .bind(format!("private-workflow-tools-{short_suffix}"))
    .fetch_one(pool)
    .await?;
    for dependent_repo_id in [public_repo_id, private_repo_id] {
        sqlx::query(
            r#"
            INSERT INTO repository_dependents (
                source_repository_id, dependent_repository_id, package_id, manifest_path
            )
            VALUES ($1, $2, $3, 'package.json')
            ON CONFLICT (source_repository_id, dependent_repository_id, package_id)
            DO UPDATE SET manifest_path = EXCLUDED.manifest_path, detected_at = now()
            "#,
        )
        .bind(repository_id)
        .bind(dependent_repo_id)
        .bind(package_id)
        .execute(pool)
        .await?;
    }

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
