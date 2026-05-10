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

    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping projects list scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("skipping projects list scenario; migration failed: {error}");
        return None;
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

async fn create_user(pool: &PgPool, login: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{login} display")),
        None,
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

async fn get_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
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
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn projects_lists_filter_templates_and_repository_links_without_leaking_private_rows() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects list scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("projects{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Projects Org".to_owned(),
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
        INSERT INTO organization_policy_settings (organization_id, projects_base_permission, projects_enabled)
        VALUES ($1, 'write', true)
        ON CONFLICT (organization_id)
        DO UPDATE SET projects_base_permission = EXCLUDED.projects_base_permission,
                      projects_enabled = EXCLUDED.projects_enabled
        "#,
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("policy should upsert");

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("planning-{}", Uuid::new_v4().simple()),
            description: Some("Project linked repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Read, "direct")
        .await
        .expect("repository permission should grant");

    let public_project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
            owner_organization_id, number, title, short_description, visibility,
            default_repository_id, created_by_user_id, updated_at
        )
        VALUES ($1, 1, 'Platform roadmap', 'Tracks delivery status', 'public', $2, $3, now())
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    let template_project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
            owner_organization_id, number, title, short_description, visibility,
            is_template, created_by_user_id, updated_at
        )
        VALUES ($1, 2, 'Launch template', 'Reusable launch checklist', 'public', true, $2, now() - interval '1 day')
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("template project should insert");
    let private_project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
            owner_organization_id, number, title, short_description, visibility,
            state, created_by_user_id, updated_at, closed_at
        )
        VALUES ($1, 3, 'Private acquisition plan', 'Hidden from outsiders', 'private', 'closed', $2, now() - interval '2 days', now())
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("private project should insert");
    sqlx::query("INSERT INTO project_repositories (project_id, repository_id, link_type) VALUES ($1, $2, 'default')")
        .bind(public_project_id)
        .bind(repo.id)
        .execute(&pool)
        .await
        .expect("repo link should insert");
    sqlx::query("INSERT INTO project_templates (project_id, title, description, is_public) VALUES ($1, 'Launch template', 'Copy this setup', true)")
        .bind(template_project_id)
        .execute(&pool)
        .await
        .expect("template should insert");
    sqlx::query("INSERT INTO project_status_updates (project_id, author_user_id, status, body) VALUES ($1, $2, 'on_track', 'Shipping steadily')")
        .bind(public_project_id)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("status should insert");
    sqlx::query("INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Write spec', 1), ($2, 'draft_issue', 'Secret draft', 1)")
        .bind(public_project_id)
        .bind(private_project_id)
        .execute(&pool)
        .await
        .expect("items should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position) VALUES ($1, 'Table', 'table', 1), ($1, 'Roadmap', 'roadmap', 2)",
    )
    .bind(public_project_id)
    .execute(&pool)
    .await
    .expect("views should insert");
    sqlx::query(
        "INSERT INTO project_fields (project_id, name, field_type, position, settings) VALUES ($1, 'Status', 'single_select', 1, '{\"options\":[\"Todo\",\"Done\"]}'::jsonb)",
    )
    .bind(public_project_id)
    .execute(&pool)
    .await
    .expect("fields should insert");
    sqlx::query(
        "INSERT INTO project_workflows (project_id, workflow_key, name, enabled, trigger_event) VALUES ($1, 'auto-archive', 'Auto archive', true, 'item_closed')",
    )
    .bind(public_project_id)
    .execute(&pool)
    .await
    .expect("workflows should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org_uri = format!("/api/orgs/{marker}/projects?q=roadmap&sort=name_asc");
    let (status, headers, body) = get_json(app.clone(), &org_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["scope"]["kind"], "organization");
    assert_eq!(body["counts"]["open"], 1);
    assert_eq!(body["items"][0]["title"], "Platform roadmap");
    assert_eq!(body["items"][0]["status"]["label"], "On track");
    assert_eq!(body["items"][0]["counts"]["draft"], 1);
    assert_eq!(body["items"][0]["defaultRepository"]["name"], repo.name);

    let repo_uri = format!("/api/repos/{marker}/{}/projects", repo.name);
    let (status, _, body) = get_json(app.clone(), &repo_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["scope"]["kind"], "repository");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["id"], public_project_id.to_string());

    let templates_uri = format!("/api/orgs/{marker}/projects?tab=templates");
    let (status, _, body) = get_json(app.clone(), &templates_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["templates"]["total"], 1);
    assert_eq!(
        body["templates"]["items"][0]["projectId"],
        template_project_id.to_string()
    );

    let copy_uri = format!("/api/projects/{public_project_id}/copies");
    let (status, headers, body) = post_json(
        app.clone(),
        &copy_uri,
        Some(&member_cookie),
        json!({ "title": "[COPY] Platform roadmap", "includeDraftIssues": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_json(&headers);
    let copied_project_id = Uuid::parse_str(body["id"].as_str().expect("copy id")).expect("uuid");
    assert_eq!(body["title"], "[COPY] Platform roadmap");
    assert_eq!(body["number"], 4);
    assert_eq!(body["copiedViews"], 2);
    assert_eq!(body["copiedFields"], 1);
    assert_eq!(body["copiedWorkflows"], 1);
    assert_eq!(body["copiedDraftItems"], 1);
    assert_eq!(
        body["workspaceHref"],
        format!("/{marker}/projects/4/views/1")
    );
    let cloned_counts = sqlx::query(
        r#"
        SELECT
          (SELECT count(*)::bigint FROM project_views WHERE project_id = $1) AS views,
          (SELECT count(*)::bigint FROM project_fields WHERE project_id = $1) AS fields,
          (SELECT count(*)::bigint FROM project_workflows WHERE project_id = $1) AS workflows,
          (SELECT count(*)::bigint FROM project_items WHERE project_id = $1 AND item_type = 'draft_issue') AS drafts,
          (SELECT count(*)::bigint FROM project_items WHERE project_id = $1 AND item_type <> 'draft_issue') AS linked_items,
          (SELECT count(*)::bigint FROM audit_events WHERE target_id = $1::text AND event_type = 'project.copy') AS audits,
          (SELECT count(*)::bigint FROM project_recent_visits WHERE project_id = $1 AND user_id = $2 AND reason = 'copy') AS visits
        "#
    )
    .bind(copied_project_id)
    .bind(member.id)
    .fetch_one(&pool)
    .await
    .expect("copy counts should load");
    assert_eq!(cloned_counts.try_get::<i64, _>("views").ok(), Some(2));
    assert_eq!(cloned_counts.try_get::<i64, _>("fields").ok(), Some(1));
    assert_eq!(cloned_counts.try_get::<i64, _>("workflows").ok(), Some(1));
    assert_eq!(cloned_counts.try_get::<i64, _>("drafts").ok(), Some(1));
    assert_eq!(
        cloned_counts.try_get::<i64, _>("linked_items").ok(),
        Some(0)
    );
    assert_eq!(cloned_counts.try_get::<i64, _>("audits").ok(), Some(1));
    assert_eq!(cloned_counts.try_get::<i64, _>("visits").ok(), Some(1));

    let (status, _, body) = post_json(
        app.clone(),
        &copy_uri,
        Some(&outsider_cookie),
        json!({ "title": "[COPY] Forbidden", "includeDraftIssues": false }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");

    let sorted_uri = format!("/api/orgs/{marker}/projects?sort=name_desc&page=1&pageSize=1");
    let (status, _, body) = get_json(app.clone(), &sorted_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 1);
    assert_eq!(body["total"], 3);
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["title"], "Platform roadmap");

    let closed_uri = format!("/api/orgs/{marker}/projects?state=closed");
    let (status, _, body) = get_json(app.clone(), &closed_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["closed"], 1);
    assert_eq!(body["items"][0]["id"], private_project_id.to_string());

    let searched_closed_uri = format!("/api/orgs/{marker}/projects?q=acquisition&state=closed");
    let (status, _, body) = get_json(app.clone(), &searched_closed_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["total"], 1);
    assert_eq!(body["items"][0]["title"], "Private acquisition plan");

    let (status, _, body) = get_json(app.clone(), &closed_uri, Some(&outsider_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().expect("items").len(), 0);
    assert!(
        !body.to_string().contains("Private acquisition plan"),
        "private project title must not leak"
    );

    let invalid_uri = format!("/api/orgs/{marker}/projects?sort=random");
    let (status, _, body) = get_json(app, &invalid_uri, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}
