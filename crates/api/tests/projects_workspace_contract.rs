use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, NaiveDate, Utc};
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
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

static REQUEST_SUBJECT_COUNTER: AtomicU64 = AtomicU64::new(1);

fn isolated_forwarded_for() -> String {
    let id = REQUEST_SUBJECT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("2001:db8:projects::{id:x}")
}

fn with_isolated_subject(builder: axum::http::request::Builder) -> axum::http::request::Builder {
    builder.header("x-forwarded-for", isolated_forwarded_for())
}

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
            eprintln!("skipping projects workspace scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("skipping projects workspace scenario; migration failed: {error}");
        return None;
    }
    Some(pool)
}

#[tokio::test]
async fn project_insights_read_contract_filters_private_items_and_returns_burnup_data() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping project insights scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("insights{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Insights Org".to_owned(),
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

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("public-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repository should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("private-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    grant_repository_permission(
        &pool,
        private_repo.id,
        member.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("member repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 88, 'Insights launch board', 'Burn-up source data', 'public', $2)
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write')",
    )
    .bind(project_id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("project permission should insert");
    sqlx::query(
        r#"
        INSERT INTO project_status_updates (project_id, author_user_id, status, body)
        VALUES ($1, $2, 'at_risk', 'Scope grew after beta feedback')
        "#,
    )
    .bind(project_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("status update should insert");
    let custom_chart_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_charts (project_id, owner_user_id, title, description, chart_type, filter, visibility)
        VALUES ($1, $2, 'Closed issues by week', 'Shared chart summary', 'line', 'is:closed type:issue', 'project')
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("custom chart should insert");

    sqlx::query(
        r#"
        INSERT INTO project_items (project_id, item_type, title, position, created_at)
        VALUES ($1, 'draft_issue', 'Draft launch checklist', 1, now() - interval '8 days')
        "#,
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("draft item should insert");
    let public_issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id, closed_at, created_at)
        VALUES ($1, 11, 'Ship public issue', 'Visible to everyone', 'closed', $2, now() - interval '2 days', now() - interval '7 days')
        RETURNING id
        "#,
    )
    .bind(public_repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("public issue should insert");
    let private_issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id, created_at)
        VALUES ($1, 12, 'Private security issue', 'Only repository readers should see this', 'open', $2, now() - interval '6 days')
        RETURNING id
        "#,
    )
    .bind(private_repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("private issue should insert");
    sqlx::query(
        r#"
        INSERT INTO project_items (project_id, item_type, issue_id, position, created_at)
        VALUES ($1, 'issue', $2, 2, now() - interval '7 days'),
               ($1, 'issue', $3, 3, now() - interval '6 days')
        "#,
    )
    .bind(project_id)
    .bind(public_issue_id)
    .bind(private_issue_id)
    .execute(&pool)
    .await
    .expect("linked project items should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = get_json(
        app.clone(),
        &format!(
            "/api/projects/{project_id}/insights?range=2w&filter=is:closed%20type:issue&table=true"
        ),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["project"]["title"], "Insights launch board");
    assert_eq!(body["navigation"]["selectedItem"], "insights");
    assert_eq!(body["selectedChart"]["id"], "burn-up");
    assert_eq!(body["selectedChart"]["sharedWithViewers"], true);
    assert!(body["selectedChart"]["shareHref"]
        .as_str()
        .expect("default share href")
        .contains("chart=burn-up"));
    assert_eq!(body["defaultCharts"][0]["title"], "Burn up");
    assert_eq!(body["customCharts"][0]["id"], custom_chart_id.to_string());
    assert!(body["customCharts"][0]["shareHref"]
        .as_str()
        .expect("custom share href")
        .contains("chart="));
    assert_eq!(body["range"]["key"], "2w");
    assert_eq!(
        body["filter"]["tokens"].as_array().expect("tokens").len(),
        2
    );
    assert_eq!(body["matchingItemCount"], 1);
    assert_eq!(body["dataRows"][0]["title"], "Ship public issue");
    assert_eq!(body["latestStatus"]["label"], "At risk");
    assert_eq!(body["viewerPermissions"]["canCreateCharts"], true);
    assert!(body["series"]
        .as_array()
        .expect("series")
        .iter()
        .any(|series| series["id"] == "completed"
            && !series["points"].as_array().expect("points").is_empty()));
    assert!(body["cache"]["version"].as_i64().unwrap_or_default() >= 0);
    let cache_rows: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_chart_series_cache WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("chart cache rows should read");
    assert!(cache_rows >= 1);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/insights?range=1m"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["viewerPermissions"]["canCreateCharts"], false);
    assert_eq!(body["matchingItemCount"], 2);
    assert!(!body["dataRows"]
        .as_array()
        .expect("rows")
        .iter()
        .any(|row| row["title"] == "Private security issue"));

    let (status, _, body) = get_json(
        app,
        &format!(
            "/api/projects/{project_id}/insights?range=custom&start=2026-04-01&end=2027-05-10"
        ),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_filter");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());
    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/insights?filter=label:bug"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_filter");
    assert!(body["error"]["message"]
        .as_str()
        .expect("message")
        .contains("Unsupported Insights filter token"));

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/charts"),
        Some(&member_cookie),
        json!({
            "title": "Closed issue trend",
            "description": "Visible to project viewers",
            "chartType": "line",
            "filter": "is:closed type:issue",
            "visibility": "project"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["selectedChart"]["title"], "Closed issue trend");
    assert_eq!(body["selectedChart"]["visibility"], "project");
    assert_eq!(body["selectedChart"]["sharedWithViewers"], true);
    assert!(body["selectedChart"]["shareHref"]
        .as_str()
        .expect("created share href")
        .contains("chart="));
    let created_chart_id = body["selectedChart"]["id"]
        .as_str()
        .expect("created chart id")
        .to_owned();
    let created_updated_at = body["selectedChart"]["updatedAt"]
        .as_str()
        .expect("created chart updated at")
        .to_owned();

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/charts/{created_chart_id}"),
        Some(&member_cookie),
        json!({
            "title": "Private closed issue trend",
            "description": "Editor-only chart",
            "chartType": "bar",
            "filter": "is:closed",
            "visibility": "private",
            "expectedUpdatedAt": created_updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selectedChart"]["title"], "Private closed issue trend");
    assert_eq!(body["selectedChart"]["visibility"], "private");
    assert_eq!(body["selectedChart"]["sharedWithViewers"], false);
    assert!(body["customCharts"]
        .as_array()
        .expect("custom charts")
        .iter()
        .any(|chart| chart["id"] == created_chart_id));

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/insights?chart={created_chart_id}"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/charts"),
        Some(&outsider_cookie),
        json!({
            "title": "Outsider chart",
            "chartType": "bar",
            "filter": "is:open",
            "visibility": "project"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let current_updated_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM project_charts WHERE id = $1")
            .bind(Uuid::parse_str(&created_chart_id).expect("created uuid"))
            .fetch_one(&pool)
            .await
            .expect("chart updated_at should read");
    let (status, _, body) = delete_json_body(
        app.clone(),
        &format!("/api/projects/{project_id}/charts/{created_chart_id}"),
        Some(&member_cookie),
        json!({ "expectedUpdatedAt": current_updated_at.to_rfc3339() }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selectedChart"]["id"], "burn-up");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit_events WHERE target_id = $1 AND event_type LIKE 'project.chart.%'",
    )
    .bind(project_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("chart audit count should read");
    assert!(audit_count >= 3);
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
    let mut user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(login),
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
    user.username = Some(login.to_owned());
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
    let mut builder = with_isolated_subject(Request::builder().method(Method::GET).uri(uri));
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

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = with_isolated_subject(
        Request::builder()
            .method(Method::PATCH)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json"),
    );
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = with_isolated_subject(
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json"),
    );
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

async fn delete_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = with_isolated_subject(Request::builder().method(Method::DELETE).uri(uri));
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

async fn delete_json_body(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = with_isolated_subject(
        Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json"),
    );
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

#[tokio::test]
async fn project_workspace_returns_table_fields_items_filters_and_private_guards() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects workspace scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("workspace{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Workspace Org".to_owned(),
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
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("planning-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Write, "direct")
        .await
        .expect("repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 41, 'Editorial table workspace', 'Screen-ready project data', 'private', $2)
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write')",
    )
    .bind(project_id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("permission should insert");
    let view_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_views (project_id, name, layout, position, configuration)
        VALUES ($1, 'Table', 'table', 1, '{"sort":"manual"}')
        RETURNING id
        "#,
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("view should insert");
    let title_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("title field should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    let target_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Target date', 'date', 3) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("target date field should insert");
    let board_view_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_views (project_id, name, layout, position, configuration)
        VALUES ($1, 'Board', 'board', 2, jsonb_build_object('columnFieldId', $2::text, 'swimlaneFieldId', $2::text))
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("board view should insert");
    let roadmap_view_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_views (project_id, name, layout, position, configuration)
        VALUES ($1, 'Roadmap', 'roadmap', 3, jsonb_build_object('startFieldId', $2::text, 'targetFieldId', $2::text, 'zoom', 'quarter'))
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(target_field)
    .fetch_one(&pool)
    .await
    .expect("roadmap view should insert");
    sqlx::query(
        r#"
        INSERT INTO project_board_column_settings (project_view_id, project_field_id, option_key, label, position, item_limit)
        VALUES ($1, $2, 'In progress', 'In progress', 1, 1),
               ($1, $2, 'Done', 'Done', 2, 3)
        "#,
    )
    .bind(board_view_id)
    .bind(status_field)
    .execute(&pool)
    .await
    .expect("board columns should insert");
    sqlx::query(
        r#"
        INSERT INTO project_roadmap_settings (project_view_id, start_field_id, target_field_id, marker_field_ids, zoom)
        VALUES ($1, $2, $2, ARRAY[$2]::uuid[], 'quarter')
        "#,
    )
    .bind(roadmap_view_id)
    .bind(target_field)
    .execute(&pool)
    .await
    .expect("roadmap settings should insert");
    let draft_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, body, position) VALUES ($1, 'draft_issue', 'Draft launch notes', 'Write the launch copy', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("draft item should insert");
    sqlx::query(
        "INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(draft_item_id)
    .bind(status_field)
    .bind(json!("In progress"))
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("field value should insert");

    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 7, 'Private linked issue', 'Only readable members should see this row', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let linked_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, issue_id, position) VALUES ($1, 'issue', $2, 2) RETURNING id",
    )
    .bind(project_id)
    .bind(issue_id)
    .fetch_one(&pool)
    .await
    .expect("issue project item should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, headers, body) = get_json(
        app.clone(),
        &format!(
            "/api/projects/{project_id}/workspace?view={view_id}&q=is:draft&sort=title_asc&group=Status&pageSize=10"
        ),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["project"]["title"], "Editorial table workspace");
    assert_eq!(body["selectedView"]["id"], view_id.to_string());
    assert_eq!(
        body["selectedView"]["href"],
        format!("/orgs/{marker}/projects/41/views/1")
    );
    assert_eq!(body["fields"].as_array().expect("fields").len(), 2);
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["title"], "Draft launch notes");
    assert!(!body["fields"]
        .as_array()
        .expect("workspace fields")
        .iter()
        .any(|field| field["id"] == title_field.to_string()));
    assert!(body["items"][0]["fieldValues"]
        .as_array()
        .expect("field values")
        .iter()
        .any(|field| field["fieldId"] == status_field.to_string()));
    assert_eq!(body["groups"][0]["label"], "In progress");
    assert_eq!(body["unsavedView"]["active"], true);
    assert_eq!(body["viewerPermissions"]["canEdit"], true);
    assert_eq!(body["viewerPermissions"]["canChangeLayout"], true);
    assert_eq!(body["layoutChoices"].as_array().expect("choices").len(), 3);
    assert!(body["layoutChoices"]
        .as_array()
        .expect("choices")
        .iter()
        .any(|choice| choice["layout"] == "board" && choice["keyboardHint"] == "b"));

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/fields/{status_field}"),
        Some(&member_cookie),
        json!({ "value": "Done" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["items"][0]["fieldValues"]
        .as_array()
        .expect("updated field values")
        .iter()
        .any(
            |field| field["fieldId"] == status_field.to_string() && field["displayValue"] == "Done",
        ));
    let stored_value: Value = sqlx::query_scalar(
        "SELECT value FROM project_item_field_values WHERE project_item_id = $1 AND project_field_id = $2",
    )
    .bind(draft_item_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("field value should persist");
    assert_eq!(stored_value, json!("Done"));

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{linked_item_id}/fields/{title_field}"),
        Some(&member_cookie),
        json!({ "value": "Renamed linked issue" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let issue_title: String = sqlx::query_scalar("SELECT title FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_one(&pool)
        .await
        .expect("issue title should load");
    assert_eq!(issue_title, "Renamed linked issue");
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE event_type = 'project.item_field.update'",
    )
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 2);

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workspace?view={board_view_id}&group=Status"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selectedView"]["layout"], "board");
    assert_eq!(
        body["boardConfig"]["columnField"]["id"],
        status_field.to_string()
    );
    assert_eq!(
        body["boardConfig"]["swimlaneField"]["id"],
        status_field.to_string()
    );
    assert_eq!(body["boardConfig"]["columns"][0]["label"], "In progress");
    assert_eq!(body["boardConfig"]["columns"][0]["itemLimit"], 1);
    assert_eq!(
        body["boardConfig"]["eligibleColumnFields"][0]["name"],
        "Status"
    );
    assert!(body["roadmapConfig"]["eligibleDateFields"]
        .as_array()
        .expect("date fields")
        .iter()
        .any(|field| field["id"] == target_field.to_string()));

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workspace?view={roadmap_view_id}&sort=manual"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selectedView"]["layout"], "roadmap");
    assert_eq!(
        body["roadmapConfig"]["startDateField"]["id"],
        target_field.to_string()
    );
    assert_eq!(body["roadmapConfig"]["zoom"], "quarter");
    assert!(body["roadmapConfig"]["zoomOptions"]
        .as_array()
        .expect("zoom options")
        .iter()
        .any(|option| option == "year"));

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/views/{roadmap_view_id}/roadmap-settings"),
        Some(&member_cookie),
        json!({
            "startFieldId": target_field,
            "targetFieldId": target_field,
            "markerFieldIds": [target_field, target_field],
            "zoom": "year"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selectedView"]["layout"], "roadmap");
    assert_eq!(body["roadmapConfig"]["zoom"], "year");
    assert_eq!(
        body["roadmapConfig"]["markerFields"][0]["id"],
        target_field.to_string()
    );
    let roadmap_zoom: String =
        sqlx::query_scalar("SELECT zoom FROM project_roadmap_settings WHERE project_view_id = $1")
            .bind(roadmap_view_id)
            .fetch_one(&pool)
            .await
            .expect("roadmap settings should persist");
    assert_eq!(roadmap_zoom, "year");
    let roadmap_audits: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE event_type = 'project.roadmap_settings.update'",
    )
    .fetch_one(&pool)
    .await
    .expect("roadmap audit count should load");
    assert!(roadmap_audits >= 1);

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/views/{roadmap_view_id}/roadmap-settings"),
        Some(&member_cookie),
        json!({
            "startFieldId": target_field,
            "targetFieldId": target_field,
            "markerFieldIds": [],
            "zoom": "week"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workspace?group=Missing"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(!body.to_string().contains("test-session-secret"));

    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/workspace"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn project_item_detail_and_archived_list_enforce_visibility() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects item detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("itemdetail{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let project_reader = create_user(&pool, &format!("{marker}-reader")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let reader_cookie = cookie_header(&pool, &config, &project_reader).await;

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-source-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Write, "direct")
        .await
        .expect("repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 71, 'Item detail project', 'Side panel read contract', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(member.id)
    .bind(project_reader.id)
    .execute(&pool)
    .await
    .expect("project permissions should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position, configuration) VALUES ($1, 'Table', 'table', 1, '{}')",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");
    sqlx::query(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1), ($1, 'Status', 'status', 2)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("fields should insert");

    let draft_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, body, position) VALUES ($1, 'draft_issue', 'Draft side panel', 'Project-only body', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("draft should insert");
    sqlx::query(
        "INSERT INTO project_item_comments (project_id, project_item_id, author_user_id, body) VALUES ($1, $2, $3, 'Draft-only comment')",
    )
    .bind(project_id)
    .bind(draft_item_id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("draft comment should insert");
    sqlx::query(
        "INSERT INTO project_item_events (project_id, project_item_id, actor_user_id, event_type, metadata) VALUES ($1, $2, $3, 'project.item.created', $4)",
    )
    .bind(project_id)
    .bind(draft_item_id)
    .bind(member.id)
    .bind(json!({ "source": "test" }))
    .execute(&pool)
    .await
    .expect("item event should insert");

    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 22, 'Private source issue', 'Only repo readers can see it', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let hidden_linked_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, issue_id, position) VALUES ($1, 'issue', $2, 2) RETURNING id",
    )
    .bind(project_id)
    .bind(issue_id)
    .fetch_one(&pool)
    .await
    .expect("linked item should insert");
    let archived_item_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_items (
          project_id, item_type, title, body, position, archived_at, archived_by_user_id, source_synced_at, source_sync_version
        )
        VALUES ($1, 'draft_issue', 'Archived draft', 'Restorable project-only body', 3, now(), $2, now(), 2)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(member.id)
    .fetch_one(&pool)
    .await
    .expect("archived item should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["item"]["title"], "Draft side panel");
    assert_eq!(body["draft"]["repositoryNotificationsEnabled"], false);
    assert_eq!(body["comments"][0]["body"], "Draft-only comment");
    assert_eq!(body["activity"][0]["eventType"], "project.item.created");
    assert_eq!(body["viewerPermissions"]["canConvert"], true);
    assert_eq!(body["archive"]["archived"], false);

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/archive"),
        Some(&member_cookie),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["archive"]["archived"], true);
    assert_eq!(
        body["archive"]["archivedBy"]["login"],
        member.username.as_deref().expect("member login")
    );
    assert_eq!(body["viewerPermissions"]["canArchive"], false);
    assert_eq!(body["viewerPermissions"]["canRestore"], true);
    assert!(body["activity"]
        .as_array()
        .expect("activity")
        .iter()
        .any(|event| event["eventType"] == "project.item.archive"));

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/restore"),
        Some(&member_cookie),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["archive"]["archived"], false);
    assert_eq!(
        body["archive"]["restoredBy"]["login"],
        member.username.as_deref().expect("member login")
    );
    assert_eq!(body["viewerPermissions"]["canArchive"], true);
    assert_eq!(body["viewerPermissions"]["canRestore"], false);
    assert!(body["activity"]
        .as_array()
        .expect("activity")
        .iter()
        .any(|event| event["eventType"] == "project.item.restore"));

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{hidden_linked_item_id}"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "not_found");
    assert!(!body.to_string().contains("Private source issue"));

    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/items/archived?itemType=draft_issue&pageSize=10"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["item"]["id"], archived_item_id.to_string());
    assert_eq!(body["items"][0]["item"]["title"], "Archived draft");
    assert_eq!(
        body["items"][0]["archivedBy"]["login"],
        member.username.as_deref().expect("member login")
    );
    assert_eq!(body["items"][0]["viewerPermissions"]["canRestore"], true);
}

#[tokio::test]
async fn project_draft_editing_and_comments_are_project_only() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects draft editing scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("draftedit{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 72, 'Draft edit project', 'Project-only mutation contract', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(member.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("project permissions should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position, configuration) VALUES ($1, 'Table', 'table', 1, '{}')",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");
    sqlx::query(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("fields should insert");
    let draft_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, body, position) VALUES ($1, 'draft_issue', 'Original draft', 'Original body', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("draft should insert");
    let archived_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, archived_at, position) VALUES ($1, 'draft_issue', 'Archived draft', now(), 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("archived draft should insert");

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("linked-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 73, 'Linked issue', 'Repository-backed issue', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let linked_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, issue_id, position) VALUES ($1, 'issue', $2, 3) RETURNING id",
    )
    .bind(project_id)
    .bind(issue_id)
    .fetch_one(&pool)
    .await
    .expect("linked project item should insert");

    let original_updated_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM project_items WHERE id = $1")
            .bind(draft_item_id)
            .fetch_one(&pool)
            .await
            .expect("draft timestamp should read");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/draft"),
        Some(&member_cookie),
        json!({
            "title": "Updated project-only draft",
            "body": "Updated project-only body",
            "expectedUpdatedAt": original_updated_at.to_rfc3339(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["item"]["title"], "Updated project-only draft");
    assert_eq!(body["item"]["body"], "Updated project-only body");
    assert_eq!(body["draft"]["repositoryNotificationsEnabled"], false);

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/draft"),
        Some(&member_cookie),
        json!({
            "title": "Stale draft title",
            "body": null,
            "expectedUpdatedAt": original_updated_at.to_rfc3339(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let current_updated_at = body["error"]["message"].clone();
    assert!(current_updated_at
        .as_str()
        .expect("error message")
        .contains("changed since it was loaded"));

    let refreshed_updated_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM project_items WHERE id = $1")
            .bind(draft_item_id)
            .fetch_one(&pool)
            .await
            .expect("refreshed timestamp should read");
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/comments"),
        Some(&member_cookie),
        json!({
            "body": "Project-only comment",
            "expectedUpdatedAt": refreshed_updated_at.to_rfc3339(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["comments"][0]["body"], "Project-only comment");
    let comment_id = body["comments"][0]["id"]
        .as_str()
        .expect("comment id")
        .to_owned();

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/comments/{comment_id}"),
        Some(&member_cookie),
        json!({
            "body": "Edited project-only comment",
            "expectedUpdatedAt": refreshed_updated_at.to_rfc3339(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["comments"][0]["body"], "Edited project-only comment");

    let (status, _, body) = delete_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/comments/{comment_id}"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["comments"][0]["isDeleted"], true);

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/draft"),
        Some(&reader_cookie),
        json!({ "title": "Reader edit", "body": null }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{linked_item_id}/draft"),
        Some(&member_cookie),
        json!({ "title": "Wrong item type", "body": null }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (status, _, body) = post_json(
        app,
        &format!("/api/projects/{project_id}/items/{archived_item_id}/comments"),
        Some(&member_cookie),
        json!({ "body": "Archived comment" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM project_item_events WHERE project_item_id = $1 AND event_type LIKE 'project.draft%'",
    )
    .bind(draft_item_id)
    .fetch_one(&pool)
    .await
    .expect("event count should read");
    assert_eq!(event_count, 4);
    let notification_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications WHERE subject_type = 'project_item' AND subject_id = $1",
    )
    .bind(draft_item_id)
    .fetch_one(&pool)
    .await
    .expect("notification count should read");
    assert_eq!(notification_count, 0);
    let timeline_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM timeline_events WHERE metadata->>'projectItemId' = $1",
    )
    .bind(draft_item_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("timeline count should read");
    assert_eq!(timeline_count, 0);
}

#[tokio::test]
async fn project_draft_conversion_creates_linked_issue() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects draft conversion scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("draftconvert{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let assignee = create_user(&pool, &format!("{marker}-assignee")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 76, 'Draft convert project', 'Convert project drafts', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(member.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("project permissions should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position, configuration) VALUES ($1, 'Table', 'table', 1, '{}')",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("convert-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Write, "direct")
        .await
        .expect("member repository permission");
    grant_repository_permission(&pool, repo.id, assignee.id, RepositoryRole::Read, "direct")
        .await
        .expect("assignee repository permission");
    sqlx::query("UPDATE projects SET default_repository_id = $2 WHERE id = $1")
        .bind(project_id)
        .bind(repo.id)
        .execute(&pool)
        .await
        .expect("project default repository");
    sqlx::query(
        "INSERT INTO project_repositories (project_id, repository_id, link_type) VALUES ($1, $2, 'default')",
    )
    .bind(project_id)
    .bind(repo.id)
    .execute(&pool)
    .await
    .expect("project repository link");
    sqlx::query(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("field should insert");
    let label_id: Uuid = sqlx::query_scalar(
        "INSERT INTO labels (repository_id, name, color) VALUES ($1, 'frontend', 'c44d2d') RETURNING id",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("label should insert");
    let milestone_id: Uuid = sqlx::query_scalar(
        "INSERT INTO milestones (repository_id, title, due_on, created_by_user_id) VALUES ($1, 'M1', now(), $2) RETURNING id",
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should insert");
    let draft_item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, body, position) VALUES ($1, 'draft_issue', 'Convert this draft', 'Draft issue body', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("draft should insert");
    let original_updated_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM project_items WHERE id = $1")
            .bind(draft_item_id)
            .fetch_one(&pool)
            .await
            .expect("draft timestamp should read");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/conversion-targets"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["repositories"][0]["id"], repo.id.to_string());
    assert!(body["repositories"][0]["labels"]
        .as_array()
        .expect("repository labels")
        .iter()
        .any(|label| label["name"] == "frontend"));

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/convert-to-issue"),
        Some(&member_cookie),
        json!({
            "repositoryId": repo.id,
            "labelIds": [label_id],
            "assigneeUserIds": [assignee.id],
            "milestoneId": milestone_id,
            "expectedUpdatedAt": original_updated_at.to_rfc3339(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["item"]["itemType"], "issue");
    assert_eq!(body["item"]["title"], "Convert this draft");
    assert_eq!(body["source"]["repository"]["id"], repo.id.to_string());
    assert_eq!(body["source"]["number"], 1);
    assert_eq!(body["viewerPermissions"]["canConvert"], false);

    let issue_id: Uuid = sqlx::query_scalar(
        "SELECT issue_id FROM project_items WHERE id = $1 AND item_type = 'issue'",
    )
    .bind(draft_item_id)
    .fetch_one(&pool)
    .await
    .expect("converted issue id should read");
    let label_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM issue_labels WHERE issue_id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("label count");
    assert_eq!(label_count, 1);
    let assignee_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM issue_assignees WHERE issue_id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("assignee count");
    assert_eq!(assignee_count, 1);
    let timeline_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM timeline_events WHERE issue_id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("timeline count");
    assert_eq!(timeline_count, 1);
    let notification_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM notifications WHERE subject_id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("notification count");
    assert_eq!(notification_count, 1);

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/convert-to-issue"),
        Some(&member_cookie),
        json!({ "repositoryId": repo.id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["source"]["id"], issue_id.to_string());

    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/conversion-targets"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn project_workflow_settings_seed_defaults_and_filter_targets() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects workflow settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("workflow{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("workflow-source-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, writer.id, RepositoryRole::Write, "direct")
        .await
        .expect("writer repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects
          (owner_user_id, number, title, short_description, visibility, default_repository_id, created_by_user_id)
        VALUES ($1, 81, 'Workflow settings project', 'Automation read contract', 'private', $2, $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(writer.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("project permissions should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'status', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    sqlx::query(
        "INSERT INTO project_field_options (project_field_id, name, color, position) VALUES ($1, 'Todo', 'gray', 1)",
    )
    .bind(status_field)
    .execute(&pool)
    .await
    .expect("status option should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows"),
        Some(&writer_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["project"]["title"], "Workflow settings project");
    assert_eq!(body["automationActor"], "@opengithub-project-automation");
    assert_eq!(body["workflows"].as_array().expect("workflows").len(), 4);
    assert_eq!(body["workflows"][0]["workflowKey"], "closed-item-to-done");
    assert_eq!(body["workflows"][0]["enabled"], true);
    assert_eq!(body["workflows"][1]["workflowKey"], "merged-pr-to-done");
    assert_eq!(body["workflows"][1]["enabled"], true);
    assert_eq!(
        body["workflows"][0]["configuration"]["target"]["fieldId"],
        status_field.to_string()
    );
    assert_eq!(
        body["workflows"][0]["configuration"]["target"]["missingOption"],
        true
    );
    assert_eq!(body["eligibleFields"][0]["supportsStatusTarget"], true);
    assert_eq!(body["eligibleFields"][0]["options"][0]["name"], "Todo");
    assert_eq!(
        body["repositoryTargets"][0]["fullName"],
        format!("{}/{}", repo.owner_login, repo.name)
    );
    assert_eq!(body["repositoryTargets"][0]["permission"], "write");
    assert_eq!(body["viewerPermissions"]["canManageWorkflows"], true);

    let workflow_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM project_workflows WHERE project_id = $1 AND workflow_key = 'closed-item-to-done'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("workflow should exist");
    let todo_option: Uuid = sqlx::query_scalar(
        "SELECT id FROM project_field_options WHERE project_field_id = $1 AND name = 'Todo'",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("todo option should exist");
    let workflow_updated_at: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT updated_at FROM project_workflows WHERE id = $1")
            .bind(workflow_id)
            .fetch_one(&pool)
            .await
            .expect("workflow timestamp should load");
    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows/{workflow_id}"),
        Some(&writer_cookie),
        json!({
            "enabled": true,
            "condition": "state:closed label:ready",
            "statusFieldId": status_field,
            "statusOptionId": todo_option,
            "repositoryTargetIds": [repo.id],
            "archiveAfterDays": 30,
            "closeOnStatus": true,
            "expectedUpdatedAt": workflow_updated_at,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["workflows"][0]["configuration"]["condition"],
        "state:closed label:ready"
    );
    assert_eq!(
        body["workflows"][0]["configuration"]["target"]["optionId"],
        todo_option.to_string()
    );
    assert_eq!(
        body["workflows"][0]["configuration"]["archiveAfterDays"],
        30
    );
    assert_eq!(body["workflows"][0]["configuration"]["closeOnStatus"], true);
    assert_eq!(
        body["workflows"][0]["repositoryTargetIds"][0],
        repo.id.to_string()
    );
    assert_eq!(body["workflows"][0]["source"], "ui");
    assert_eq!(body["recentLogs"][0]["source"], "ui");
    assert_eq!(body["recentLogs"][0]["status"], "success");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE event_type = 'project.workflow.update' AND target_id = $1",
    )
    .bind(workflow_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 1);

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows/{workflow_id}"),
        Some(&reader_cookie),
        json!({ "enabled": false }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows/{workflow_id}"),
        Some(&writer_cookie),
        json!({
            "enabled": false,
            "expectedUpdatedAt": workflow_updated_at,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("changed since it was loaded"));

    sqlx::query(
        r#"
        INSERT INTO workflow_execution_logs
          (project_id, project_workflow_id, actor_user_id, source, event_type, status, message, metadata)
        VALUES ($1, $2, $3, 'system', 'issue_closed', 'skipped', 'No Done option is configured.', $4)
        "#,
    )
    .bind(project_id)
    .bind(workflow_id)
    .bind(writer.id)
    .bind(json!({ "reason": "missing_done_option" }))
    .execute(&pool)
    .await
    .expect("execution log should insert");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows"),
        Some(&writer_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["recentLogs"][0]["workflowKey"], "closed-item-to-done");
    assert_eq!(
        body["recentLogs"][0]["actor"]["login"],
        writer.username.as_deref().unwrap_or(&writer.email)
    );
    assert_eq!(body["recentLogs"][0]["status"], "skipped");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["viewerPermissions"]["canManageWorkflows"], false);
    assert_eq!(body["repositoryTargets"].as_array().unwrap().len(), 0);

    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/workflows"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "forbidden");
    assert!(!body.to_string().contains("test-session-secret"));
}

#[tokio::test]
async fn project_settings_read_contract_filters_private_repositories_and_permissions() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("settings{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let admin = create_user(&pool, &format!("{marker}-admin")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Project Settings Org".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner'), ($1, $3, 'member'), ($1, $4, 'member')
        ON CONFLICT (organization_id, user_id) DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .bind(admin.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("memberships should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_policy_settings (
          organization_id, base_repository_permission, projects_base_permission, projects_enabled, members_can_change_repository_visibility
        )
        VALUES ($1, 'none', 'read', false, false)
        ON CONFLICT (organization_id)
        DO UPDATE SET base_repository_permission = EXCLUDED.base_repository_permission,
                      projects_base_permission = EXCLUDED.projects_base_permission,
                      projects_enabled = EXCLUDED.projects_enabled,
                      members_can_change_repository_visibility = EXCLUDED.members_can_change_repository_visibility
        "#,
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("policy should upsert");

    let visible_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("visible-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("visible repository should create");
    let hidden_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("hidden-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("hidden repository should create");
    grant_repository_permission(
        &pool,
        visible_repo.id,
        admin.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("admin visible repository permission should grant");
    grant_repository_permission(
        &pool,
        visible_repo.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader visible repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
          owner_organization_id, number, title, short_description, readme, visibility,
          default_repository_id, created_by_user_id
        )
        VALUES ($1, 91, 'Settings contract project', 'Settings read contract', '## Plan', 'private', $2, $3)
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(visible_repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(admin.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("project permissions should insert");
    sqlx::query(
        "INSERT INTO project_repositories (project_id, repository_id, link_type, linked_by_user_id) VALUES ($1, $2, 'linked', $4), ($1, $3, 'linked', $4)",
    )
    .bind(project_id)
    .bind(visible_repo.id)
    .bind(hidden_repo.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("repository links should insert");
    sqlx::query(
        "INSERT INTO project_status_updates (project_id, author_user_id, status, body, start_date, target_date) VALUES ($1, $2, 'at_risk', 'Blocked on review', '2026-05-01', '2026-05-31')",
    )
    .bind(project_id)
    .bind(admin.id)
    .execute(&pool)
    .await
    .expect("status update should insert");
    sqlx::query(
        "INSERT INTO project_readme_revisions (project_id, author_user_id, body) VALUES ($1, $2, 'Initial readme')",
    )
    .bind(project_id)
    .bind(admin.id)
    .execute(&pool)
    .await
    .expect("readme revision should insert");
    let team_id: Uuid = sqlx::query_scalar(
        "INSERT INTO teams (organization_id, slug, name) VALUES ($1, 'planning', 'Planning') RETURNING id",
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("team should insert");
    sqlx::query("INSERT INTO team_memberships (team_id, user_id) VALUES ($1, $2)")
        .bind(team_id)
        .bind(reader.id)
        .execute(&pool)
        .await
        .expect("team membership should insert");
    sqlx::query(
        "INSERT INTO project_team_permissions (project_id, team_id, role, created_by_user_id) VALUES ($1, $2, 'write', $3)",
    )
    .bind(project_id)
    .bind(team_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("team permission should insert");
    sqlx::query(
        "INSERT INTO project_templates (project_id, title, description, is_public) VALUES ($1, 'Roadmap template', 'Copy this plan', true)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("template should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings"),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["project"]["title"], "Settings contract project");
    assert_eq!(body["general"]["readme"], "## Plan");
    assert_eq!(body["general"]["readmeRevisionCount"], 1);
    assert_eq!(body["policy"]["projectsEnabled"], false);
    assert_eq!(body["policy"]["visibilityChangesAllowed"], false);
    assert_eq!(body["viewerPermissions"]["canManageAccess"], true);
    assert_eq!(body["viewerPermissions"]["canChangeVisibility"], false);
    assert_eq!(
        body["repositories"].as_array().expect("repositories").len(),
        1
    );
    assert!(body["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .all(|repo| repo["fullName"] != format!("{}/{}", marker, hidden_repo.name)));
    assert_eq!(body["accessGrants"].as_array().expect("grants").len(), 2);
    assert_eq!(body["teamGrants"][0]["team"]["slug"], "planning");
    assert_eq!(body["teamGrants"][0]["memberCount"], 1);
    assert_eq!(body["eligibleTeams"][0]["name"], "Planning");
    assert_eq!(body["statusUpdates"][0]["label"], "At risk");
    assert_eq!(body["template"]["title"], "Roadmap template");
    assert_eq!(
        body["dangerState"]["deleteConfirmation"],
        "Settings contract project"
    );

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["viewerPermissions"]["canManageAccess"], false);
    assert_eq!(body["viewerPermissions"]["canPublishStatus"], false);
    assert_eq!(
        body["repositories"].as_array().expect("repositories").len(),
        1
    );

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "forbidden");

    let (status, _, body) =
        get_json(app, &format!("/api/projects/{project_id}/settings"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn project_lifecycle_close_reopen_and_delete_are_confirmed_and_private() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects lifecycle scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("lifecycle{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let admin = create_user(&pool, &format!("{marker}-admin")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 92, 'Lifecycle project', 'Lifecycle contract', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(admin.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("permissions should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/close"),
        Some(&reader_cookie),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/close"),
        Some(&admin_cookie),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["dangerState"]["state"], "closed");
    assert_eq!(body["viewerPermissions"]["canEditGeneral"], false);
    assert_eq!(body["viewerPermissions"]["canReopen"], true);
    assert_eq!(
        body["dangerState"]["closedBy"]["login"],
        admin.username.as_deref().unwrap_or(&admin.email)
    );

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/reopen"),
        Some(&admin_cookie),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["dangerState"]["state"], "open");
    assert!(body["dangerState"]["closedAt"].is_null());

    let (status, _, body) = delete_json_body(
        app.clone(),
        &format!("/api/projects/{project_id}"),
        Some(&admin_cookie),
        json!({ "confirmation": "wrong title" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, _, body) = delete_json_body(
        app.clone(),
        &format!("/api/projects/{project_id}"),
        Some(&admin_cookie),
        json!({ "confirmation": "Lifecycle project" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deleted"], true);
    assert_eq!(
        body["destinationHref"],
        format!(
            "/{}/projects",
            owner.username.as_deref().unwrap_or(&owner.email)
        )
    );

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings"),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE target_id = $1 AND event_type LIKE 'project.lifecycle.%'",
    )
    .bind(project_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit events should count");
    assert_eq!(audit_count, 3);
}

#[tokio::test]
async fn project_workflow_engine_moves_closed_issue_to_done_idempotently() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects workflow execution scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("workflowexec{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("workflow-exec-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, writer.id, RepositoryRole::Write, "direct")
        .await
        .expect("writer repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects
          (owner_user_id, number, title, short_description, visibility, default_repository_id, created_by_user_id)
        VALUES ($1, 82, 'Workflow execution project', 'Automation execution contract', 'private', $2, $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write')",
    )
    .bind(project_id)
    .bind(writer.id)
    .execute(&pool)
    .await
    .expect("project permission should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'status', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    let done_option: Uuid = sqlx::query_scalar(
        "INSERT INTO project_field_options (project_field_id, name, color, position) VALUES ($1, 'Done', 'green', 1) RETURNING id",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("done option should insert");
    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 42, 'Close this project item', 'Automation should move this to Done', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, issue_id, position) VALUES ($1, 'issue', $2, 1) RETURNING id",
    )
    .bind(project_id)
    .bind(issue_id)
    .fetch_one(&pool)
    .await
    .expect("project item should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows"),
        Some(&writer_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["workflows"][0]["configuration"]["target"]["optionId"],
        done_option.to_string()
    );

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/repos/{}/{}/issues/42", repo.owner_login, repo.name),
        Some(&writer_cookie),
        json!({ "state": "closed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let status_value: Value = sqlx::query_scalar(
        "SELECT value FROM project_item_field_values WHERE project_item_id = $1 AND project_field_id = $2",
    )
    .bind(item_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("workflow field value should persist");
    assert_eq!(status_value, json!("Done"));
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_item_events WHERE project_item_id = $1 AND event_type = 'project.workflow.execute'",
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("project workflow event count should load");
    assert_eq!(event_count, 1);
    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workflow_execution_logs WHERE project_id = $1 AND project_item_id = $2 AND status = 'success'",
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("workflow log count should load");
    assert_eq!(log_count, 1);

    let (status, _, body) = patch_json(
        app,
        &format!("/api/repos/{}/{}/issues/42", repo.owner_login, repo.name),
        Some(&writer_cookie),
        json!({ "state": "closed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let repeated_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workflow_execution_logs WHERE project_id = $1 AND project_item_id = $2 AND status = 'success'",
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("repeated workflow log count should load");
    assert_eq!(repeated_log_count, 1);
}

#[tokio::test]
async fn project_automation_invocation_records_actions_and_graphql_attribution() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects automation invocation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("workflowinvoke{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("workflow-invoke-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, writer.id, RepositoryRole::Write, "direct")
        .await
        .expect("writer repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects
          (owner_user_id, number, title, short_description, visibility, default_repository_id, created_by_user_id)
        VALUES ($1, 83, 'Workflow invocation project', 'Actions and GraphQL hook contract', 'private', $2, $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write')",
    )
    .bind(project_id)
    .bind(writer.id)
    .execute(&pool)
    .await
    .expect("project permission should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'status', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    sqlx::query(
        "INSERT INTO project_field_options (project_field_id, name, color, position) VALUES ($1, 'Done', 'green', 1)",
    )
    .bind(status_field)
    .execute(&pool)
    .await
    .expect("done option should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Invoke automation', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("project item should insert");
    let actions_workflow_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO actions_workflows (repository_id, name, path, trigger_events)
        VALUES ($1, 'Project automation', '.github/workflows/project.yml', ARRAY['workflow_dispatch'])
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("actions workflow should insert");
    let actions_run_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_runs
          (repository_id, workflow_id, actor_user_id, run_number, head_branch, head_sha, event)
        VALUES ($1, $2, $3, 1, 'main', 'abc123', 'workflow_dispatch')
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(actions_workflow_id)
    .bind(writer.id)
    .fetch_one(&pool)
    .await
    .expect("workflow run should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workflows"),
        Some(&writer_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let workflow_id = Uuid::parse_str(
        body["workflows"][0]["id"]
            .as_str()
            .expect("workflow id should be present"),
    )
    .expect("workflow id should parse");

    let idempotency_key = format!("actions:{}:{item_id}", actions_run_id);
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/automation/invocations"),
        Some(&writer_cookie),
        json!({
            "source": "actions",
            "itemId": item_id,
            "workflowId": workflow_id,
            "actionsWorkflowRunId": actions_run_id,
            "idempotencyKey": idempotency_key,
            "fieldUpdates": [{ "fieldId": status_field, "value": "Done" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["source"], "actions");
    assert_eq!(body["status"], "success");
    assert_eq!(body["appliedUpdates"][0]["value"], "Done");

    let status_value: Value = sqlx::query_scalar(
        "SELECT value FROM project_item_field_values WHERE project_item_id = $1 AND project_field_id = $2",
    )
    .bind(item_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("workflow field value should persist");
    assert_eq!(status_value, json!("Done"));
    let log = sqlx::query(
        r#"
        SELECT source, event_type, status, metadata
        FROM workflow_execution_logs
        WHERE project_id = $1 AND project_item_id = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("execution log should load");
    assert_eq!(log.get::<String, _>("source"), "actions");
    assert_eq!(log.get::<String, _>("event_type"), "automation_invocation");
    assert_eq!(log.get::<String, _>("status"), "success");
    let metadata: Value = log.get("metadata");
    assert_eq!(metadata["idempotencyKey"], idempotency_key);
    assert_eq!(metadata["actionsWorkflowRunId"], actions_run_id.to_string());

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/automation/invocations"),
        Some(&writer_cookie),
        json!({
            "source": "actions",
            "itemId": item_id,
            "workflowId": workflow_id,
            "actionsWorkflowRunId": actions_run_id,
            "idempotencyKey": idempotency_key,
            "fieldUpdates": [{ "fieldId": status_field, "value": "Done" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "skipped");
    let success_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workflow_execution_logs WHERE project_id = $1 AND project_item_id = $2 AND source = 'actions' AND status = 'success'",
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("success log count should load");
    assert_eq!(success_log_count, 1);

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/automation/invocations"),
        Some(&writer_cookie),
        json!({
            "source": "graphql",
            "itemId": item_id,
            "workflowKey": "item-added-default-status",
            "idempotencyKey": format!("graphql:{item_id}"),
            "fieldUpdates": [{ "fieldId": status_field, "value": "Done" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["source"], "graphql");
    assert_eq!(body["workflowKey"], "item-added-default-status");

    let (status, _, body) = post_json(
        app,
        &format!("/api/projects/{project_id}/automation/invocations"),
        Some(&writer_cookie),
        json!({
            "source": "email",
            "itemId": item_id,
            "idempotencyKey": "bad-source",
            "fieldUpdates": [{ "fieldId": status_field, "value": "Done" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE event_type = 'project.workflow.invoke' AND target_id = $1",
    )
    .bind(item_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 2);
}

#[tokio::test]
async fn project_field_settings_returns_options_iterations_limits_and_guards() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects field settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("fieldsettings{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 51, 'Field settings project', 'Custom field admin contract', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(owner.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("permissions should insert");

    let title_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("title field should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    let iteration_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position, settings) VALUES ($1, 'Sprint', 'iteration', 3, '{\"durationUnit\":\"weeks\"}'::jsonb) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("iteration field should insert");
    sqlx::query(
        r#"
        INSERT INTO project_field_options (project_field_id, name, color, position, description)
        VALUES ($1, 'Todo', 'gray', 1, 'Not started'),
               ($1, 'Done', 'green', 2, 'Completed')
        "#,
    )
    .bind(status_field)
    .execute(&pool)
    .await
    .expect("options should insert");
    sqlx::query(
        r#"
        INSERT INTO project_iterations (project_field_id, name, start_date, duration_days, position)
        VALUES ($1, 'Sprint 1', $2, 14, 1),
               ($1, 'Sprint 2', $3, 14, 2)
        "#,
    )
    .bind(iteration_field)
    .bind(NaiveDate::from_ymd_opt(2026, 5, 4).expect("date"))
    .bind(NaiveDate::from_ymd_opt(2026, 5, 18).expect("date"))
    .execute(&pool)
    .await
    .expect("iterations should insert");
    sqlx::query(
        r#"
        INSERT INTO project_iteration_breaks (project_field_id, name, start_date, duration_days)
        VALUES ($1, 'Planning break', $2, 7)
        "#,
    )
    .bind(iteration_field)
    .bind(NaiveDate::from_ymd_opt(2026, 6, 1).expect("date"))
    .execute(&pool)
    .await
    .expect("break should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Backfill docs', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("item should insert");
    sqlx::query(
        "INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(item_id)
    .bind(status_field)
    .bind(json!("Todo"))
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("field value should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["project"]["title"], "Field settings project");
    assert_eq!(body["limits"]["usedFields"], 3);
    assert_eq!(body["limits"]["remainingFields"], 47);
    assert_eq!(body["viewerPermissions"]["canCreateFields"], true);
    assert_eq!(body["fields"][0]["id"], title_field.to_string());
    assert_eq!(body["fields"][0]["builtIn"], true);
    assert_eq!(body["fields"][0]["deletable"], false);
    assert_eq!(body["fields"][1]["id"], status_field.to_string());
    assert_eq!(
        body["fields"][1]["options"]
            .as_array()
            .expect("options")
            .len(),
        2
    );
    assert_eq!(body["fields"][1]["options"][1]["name"], "Done");
    assert_eq!(body["fields"][1]["usageCount"], 1);
    assert_eq!(body["fields"][2]["id"], iteration_field.to_string());
    assert_eq!(body["fields"][2]["iterations"][0]["name"], "Sprint 1");
    assert_eq!(body["fields"][2]["breaks"][0]["name"], "Planning break");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["viewerPermissions"]["viewerRole"], "read");
    assert_eq!(body["viewerPermissions"]["canCreateFields"], false);

    let (status, _, body) = get_json(
        app,
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn project_field_lifecycle_creates_renames_deletes_and_audits() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects field lifecycle scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("fieldlifecycle{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, visibility, created_by_user_id)
        VALUES ($1, 61, 'Field lifecycle project', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(owner.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("permissions should insert");
    let title_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("title field should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position) VALUES ($1, 'Table', 'table', 1)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Keep values scoped', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("item should insert");
    sqlx::query(
        "INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(item_id)
    .bind(status_field)
    .bind(json!("Todo"))
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("field value should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&owner_cookie),
        json!({ "name": "Priority", "fieldType": "single_select" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert!(body["fields"]
        .as_array()
        .expect("fields")
        .iter()
        .any(|field| field["name"] == "Priority" && field["fieldType"] == "single_select"));

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&owner_cookie),
        json!({ "name": "Priority", "fieldType": "date" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/settings/fields"),
        Some(&reader_cookie),
        json!({ "name": "Blocked", "fieldType": "text" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}"),
        Some(&owner_cookie),
        json!({ "name": "Stage" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["fields"]
        .as_array()
        .expect("fields")
        .iter()
        .any(|field| field["id"] == status_field.to_string()
            && field["name"] == "Stage"
            && field["cacheVersion"].as_i64().unwrap_or_default() > 1));

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{title_field}"),
        Some(&owner_cookie),
        json!({ "name": "Summary" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, _, body) = delete_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!body["fields"]
        .as_array()
        .expect("fields")
        .iter()
        .any(|field| field["id"] == status_field.to_string()));
    let removed_values: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_item_field_values WHERE project_field_id = $1",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("value count should load");
    assert_eq!(removed_values, 0);
    let audits: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit_events WHERE target_id = $1 AND event_type LIKE 'project.field.%'",
    )
    .bind(status_field.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audits, 2);
    let item_events: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_item_events WHERE project_item_id = $1 AND event_type = 'project.field_value.delete'",
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("item event count should load");
    assert_eq!(item_events, 1);
}

#[tokio::test]
async fn project_field_options_create_update_reorder_delete_and_sync_values() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects field options scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("fieldoptions{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, visibility, created_by_user_id)
        VALUES ($1, 62, 'Field options project', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(owner.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("permissions should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    let date_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Target', 'date', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("date field should insert");
    let todo_option: Uuid = sqlx::query_scalar(
        "INSERT INTO project_field_options (project_field_id, name, color, position) VALUES ($1, 'Todo', 'gray', 1) RETURNING id",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("todo option should insert");
    let done_option: Uuid = sqlx::query_scalar(
        "INSERT INTO project_field_options (project_field_id, name, color, position) VALUES ($1, 'Done', 'green', 2) RETURNING id",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("done option should insert");
    let view_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_views (project_id, name, layout, position) VALUES ($1, 'Board', 'board', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("view should insert");
    sqlx::query(
        "INSERT INTO project_board_column_settings (project_view_id, project_field_id, option_key, label, position) VALUES ($1, $2, 'Todo', 'Todo', 1)",
    )
    .bind(view_id)
    .bind(status_field)
    .execute(&pool)
    .await
    .expect("board setting should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Ship option sync', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("item should insert");
    sqlx::query(
        "INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(item_id)
    .bind(status_field)
    .bind(json!("Todo"))
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("field value should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}/options"),
        Some(&owner_cookie),
        json!({ "name": "Ready", "color": "blue", "description": "Ready to ship" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert!(body["fields"][0]["options"]
        .as_array()
        .expect("options")
        .iter()
        .any(|option| option["name"] == "Ready" && option["color"] == "blue"));
    let ready_option = body["fields"][0]["options"]
        .as_array()
        .expect("options")
        .iter()
        .find(|option| option["name"] == "Ready")
        .and_then(|option| option["id"].as_str())
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("ready option id should be returned");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{date_field}/options"),
        Some(&owner_cookie),
        json!({ "name": "Blocked", "color": "gray" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}/options"),
        Some(&reader_cookie),
        json!({ "name": "Blocked", "color": "gray" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}/options/{todo_option}"),
        Some(&owner_cookie),
        json!({ "name": "Queued", "color": "yellow", "description": "Queued work" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let renamed_value: Value = sqlx::query_scalar(
        "SELECT value FROM project_item_field_values WHERE project_item_id = $1 AND project_field_id = $2",
    )
    .bind(item_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("renamed value should load");
    assert_eq!(renamed_value, json!("Queued"));
    let renamed_column: String = sqlx::query_scalar(
        "SELECT option_key FROM project_board_column_settings WHERE project_view_id = $1 AND project_field_id = $2",
    )
    .bind(view_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("board column should load");
    assert_eq!(renamed_column, "Queued");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}/options/reorder"),
        Some(&owner_cookie),
        json!({ "optionIds": [done_option, todo_option, ready_option] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["fields"][0]["options"][0]["id"],
        done_option.to_string()
    );

    let (status, _, body) = delete_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{status_field}/options/{todo_option}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let removed_values: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_item_field_values WHERE project_field_id = $1",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("value count should load");
    assert_eq!(removed_values, 0);
    let removed_columns: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_board_column_settings WHERE project_field_id = $1 AND option_key = 'Queued'",
    )
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("column count should load");
    assert_eq!(removed_columns, 0);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit_events WHERE event_type LIKE 'project.field_option.%' AND target_type = 'project_field_option'",
    )
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 4);
    let item_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_item_events WHERE project_item_id = $1 AND event_type = 'project.field_option.delete'",
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("item event count should load");
    assert_eq!(item_event_count, 1);
}

#[tokio::test]
async fn project_iteration_settings_create_breaks_and_filter_tokens() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects iteration settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("fielditerations{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_user_id, number, title, visibility, created_by_user_id)
        VALUES ($1, 63, 'Iteration settings project', 'private', $1)
        RETURNING id
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'admin'), ($1, $3, 'read')",
    )
    .bind(project_id)
    .bind(owner.id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("permissions should insert");
    let iteration_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position, settings) VALUES ($1, 'Cycle', 'iteration', 1, '{\"duration\":2,\"durationUnit\":\"weeks\"}'::jsonb) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("iteration field should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position) VALUES ($1, 'Table', 'table', 1)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_items (project_id, item_type, title, position) VALUES ($1, 'draft_issue', 'Run iteration filter', 1) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("item should insert");
    sqlx::query(
        "INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(item_id)
    .bind(iteration_field)
    .bind(json!("2026-05-11"))
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("field value should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{iteration_field}/iterations/settings"),
        Some(&owner_cookie),
        json!({
            "startDate": "2026-05-04",
            "duration": 1,
            "durationUnit": "weeks",
            "generatedIterations": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["fields"][0]["iterations"].as_array().unwrap().len(), 3);

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{iteration_field}/iterations"),
        Some(&owner_cookie),
        json!({ "name": "Iteration 4", "startDate": "2026-05-25", "durationDays": 7 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let new_iteration_id = body["fields"][0]["iterations"][3]["id"]
        .as_str()
        .expect("new iteration id")
        .to_owned();

    let (status, _, body) = patch_json(
        app.clone(),
        &format!(
            "/api/projects/{project_id}/fields/{iteration_field}/iterations/{new_iteration_id}"
        ),
        Some(&owner_cookie),
        json!({ "name": "Iteration four", "startDate": "2026-05-25", "durationDays": 7 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["fields"][0]["iterations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|iteration| iteration["name"] == "Iteration four"));

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{iteration_field}/iteration-breaks"),
        Some(&owner_cookie),
        json!({ "name": "Holiday", "startDate": "2026-06-01", "durationDays": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let break_id = body["fields"][0]["breaks"][0]["id"]
        .as_str()
        .expect("break id")
        .to_owned();
    let (status, _, body) = delete_json(
        app.clone(),
        &format!("/api/projects/{project_id}/fields/{iteration_field}/iteration-breaks/{break_id}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, _, body) = get_json(
        app.clone(),
        &format!("/api/projects/{project_id}/workspace?q=cycle:2026-05-01..2026-05-31"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items"][0]["title"], "Run iteration filter");

    let (status, _, body) = post_json(
        app,
        &format!("/api/projects/{project_id}/fields/{iteration_field}/iterations"),
        Some(&reader_cookie),
        json!({ "name": "Blocked" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn project_workspace_adds_reorders_and_removes_items() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping projects workspace scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("workspace{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Workspace Org".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("planning-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, member.id, RepositoryRole::Write, "direct")
        .await
        .expect("repository permission should grant");

    let project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, created_by_user_id)
        VALUES ($1, 42, 'Editable table workspace', 'Adds rows and order', 'private', $2)
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("project should insert");
    sqlx::query(
        "INSERT INTO project_permissions (project_id, user_id, role) VALUES ($1, $2, 'write')",
    )
    .bind(project_id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("permission should insert");
    sqlx::query(
        "INSERT INTO project_views (project_id, name, layout, position, configuration) VALUES ($1, 'Table', 'table', 1, '{\"sort\":\"manual\"}')",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("view should insert");
    sqlx::query(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Title', 'title', 1)",
    )
    .bind(project_id)
    .execute(&pool)
    .await
    .expect("field should insert");
    let status_field: Uuid = sqlx::query_scalar(
        "INSERT INTO project_fields (project_id, name, field_type, position) VALUES ($1, 'Status', 'single_select', 2) RETURNING id",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("status field should insert");
    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 8, 'Linked add target', 'Can be added to a project', 'open', $2)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items"),
        Some(&member_cookie),
        json!({ "itemType": "draft_issue", "title": "Draft planning note", "body": "Keep it project-only" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["items"][0]["title"], "Draft planning note");
    let draft_item_id =
        Uuid::parse_str(body["items"][0]["id"].as_str().expect("draft id")).expect("uuid");

    let (status, _, body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items"),
        Some(&member_cookie),
        json!({ "itemType": "issue", "issueId": issue_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert!(body["items"]
        .as_array()
        .expect("items")
        .iter()
        .any(|item| item["title"] == "Linked add target"));

    let (duplicate_status, _, duplicate_body) = post_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items"),
        Some(&member_cookie),
        json!({ "itemType": "issue", "issueId": issue_id }),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(duplicate_body["error"]["code"], "validation_failed");

    let (status, _, body) = patch_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}/position"),
        Some(&member_cookie),
        json!({ "afterItemId": null, "beforeItemId": null, "groupFieldId": status_field, "groupValue": "Done" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["items"]
        .as_array()
        .expect("items")
        .iter()
        .any(|item| item["id"] == draft_item_id.to_string()
            && item["fieldValues"]
                .as_array()
                .expect("field values")
                .iter()
                .any(|field| field["fieldId"] == status_field.to_string()
                    && field["displayValue"] == "Done")));
    let status_value: Value = sqlx::query_scalar(
        "SELECT value FROM project_item_field_values WHERE project_item_id = $1 AND project_field_id = $2",
    )
    .bind(draft_item_id)
    .bind(status_field)
    .fetch_one(&pool)
    .await
    .expect("board move field value should persist");
    assert_eq!(status_value, json!("Done"));

    let (status, _, body) = delete_json(
        app.clone(),
        &format!("/api/projects/{project_id}/items/{draft_item_id}"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!body["items"]
        .as_array()
        .expect("items")
        .iter()
        .any(|item| item["id"] == draft_item_id.to_string()));
    let archived_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT archived_at FROM project_items WHERE id = $1")
            .bind(draft_item_id)
            .fetch_one(&pool)
            .await
            .expect("archived item should load");
    assert!(archived_at.is_some());
    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM project_item_events WHERE project_id = $1 AND event_type IN ('project.item.add', 'project.item.reorder', 'project.item.remove')",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("event count should load");
    assert!(event_count >= 3);
}
