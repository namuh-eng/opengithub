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

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(Method::PATCH)
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

async fn delete_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::DELETE).uri(uri);
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
    assert_eq!(body["fields"].as_array().expect("fields").len(), 2);
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["title"], "Draft launch notes");
    assert_eq!(
        body["items"][0]["fieldValues"][0]["fieldId"],
        title_field.to_string()
    );
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
    assert_eq!(body["items"][0]["fieldValues"][1]["displayValue"], "Done");
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
        json!({ "optionIds": [done_option, todo_option] }),
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
