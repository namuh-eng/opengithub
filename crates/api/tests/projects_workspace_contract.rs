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
        json!({ "afterItemId": null, "beforeItemId": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

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
