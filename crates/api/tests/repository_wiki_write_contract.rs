use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
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
    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping repository wiki write scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_wiki_write_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.wiki_git_commits') IS NOT NULL
               AND to_regclass('public.repository_activity_events') IS NOT NULL
               AND to_regclass('public.wiki_assets') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_wiki_write_tables {
            eprintln!("skipping repository wiki write scenario; migration failed: {error}");
            return None;
        }
    }
    Some(pool)
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("https://opengithub.test").expect("app URL"),
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
    let suffix = Uuid::new_v4().simple();
    let user = upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(label),
        Some(&format!("https://avatars.opengithub.local/{label}.png")),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("{label}-{suffix}"))
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
    payload: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

#[tokio::test]
async fn repository_wiki_write_contract_creates_previews_updates_and_records_git_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository wiki write scenario; set TEST_DATABASE_URL");
        return;
    };

    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-wiki-write-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);
    let config = app_config();
    let owner = create_user(&pool, "wiki-writer").await;
    let reader = create_user(&pool, "wiki-reader").await;
    let owner_login = owner.username.clone().expect("owner username");
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("wiki-write-{}", Uuid::new_v4().simple()),
            description: Some("Wiki writer repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let create_uri = format!("/api/repos/{}/{}/wiki/pages", owner_login, repository.name);
    let (status, create_body) = json_request(
        app.clone(),
        Method::POST,
        &create_uri,
        Some(&owner_cookie),
        json!({
            "title": "Operations Guide",
            "markdown": "# Operations\n\nRun the deploy checklist.\n\n![Deploy map](https://images.opengithub.local/deploy.png)",
            "message": "Create operations guide",
            "editMode": "markdown"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{create_body}");
    assert_eq!(create_body["page"]["slug"], "Operations Guide");
    assert_eq!(create_body["gitCommit"]["branch"], "master");
    assert!(create_body["gitCommit"]["shortOid"].as_str().unwrap().len() >= 7);

    let page_id = Uuid::parse_str(create_body["page"]["id"].as_str().unwrap()).unwrap();
    let revision_id =
        Uuid::parse_str(create_body["page"]["revision"]["id"].as_str().unwrap()).unwrap();
    let commit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM wiki_git_commits WHERE page_id = $1 AND revision_id = $2",
    )
    .bind(page_id)
    .bind(revision_id)
    .fetch_one(&pool)
    .await
    .expect("commit count should query");
    assert_eq!(commit_count, 1);
    assert!(storage_dir
        .join(format!("{}-{}.wiki.git", owner_login, repository.name))
        .exists());
    let asset_row = sqlx::query(
        "SELECT source_url, alt_text, storage_kind FROM wiki_assets WHERE page_id = $1 AND revision_id = $2",
    )
    .bind(page_id)
    .bind(revision_id)
    .fetch_one(&pool)
    .await
    .expect("wiki image reference should persist");
    assert_eq!(
        asset_row.get::<String, _>("source_url"),
        "https://images.opengithub.local/deploy.png"
    );
    assert_eq!(asset_row.get::<String, _>("alt_text"), "Deploy map");
    assert_eq!(asset_row.get::<String, _>("storage_kind"), "remote_url");

    let preview_uri = format!(
        "/api/repos/{}/{}/wiki/preview",
        owner_login, repository.name
    );
    let (status, preview_body) = json_request(
        app.clone(),
        Method::POST,
        &preview_uri,
        Some(&owner_cookie),
        json!({ "markdown": "## Preview\n\n<script>alert('x')</script>" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{preview_body}");
    assert!(preview_body["html"].as_str().unwrap().contains("Preview"));
    assert!(!preview_body["html"].as_str().unwrap().contains("<script>"));
    let revision_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM wiki_page_revisions WHERE page_id = $1")
            .bind(page_id)
            .fetch_one(&pool)
            .await
            .expect("revision count should query");
    assert_eq!(revision_count, 1, "preview must not persist revisions");

    let update_uri = format!(
        "/api/repos/{}/{}/wiki/Operations%20Guide",
        owner_login, repository.name
    );
    let (status, update_body) = json_request(
        app.clone(),
        Method::PATCH,
        &update_uri,
        Some(&owner_cookie),
        json!({
            "title": "Operations Guide",
            "markdown": "# Operations\n\nUpdated runbook.",
            "message": "Update operations guide",
            "expectedRevisionId": revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{update_body}");
    assert_ne!(
        update_body["page"]["revision"]["id"],
        revision_id.to_string()
    );

    let (status, stale_body) = json_request(
        app.clone(),
        Method::PATCH,
        &update_uri,
        Some(&owner_cookie),
        json!({
            "title": "Operations Guide",
            "markdown": "# Operations\n\nStale edit.",
            "message": "Try stale update",
            "expectedRevisionId": revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale_body}");

    let (status, unsupported_body) = json_request(
        app.clone(),
        Method::POST,
        &preview_uri,
        Some(&owner_cookie),
        json!({
            "markdown": "# Unsupported",
            "editMode": "asciidoc"
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{unsupported_body}"
    );
    assert_eq!(unsupported_body["error"]["code"], "validation_failed");
    assert!(unsupported_body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not supported"));

    let (status, duplicate_body) = json_request(
        app.clone(),
        Method::POST,
        &create_uri,
        Some(&owner_cookie),
        json!({
            "title": "Operations Guide",
            "markdown": "# Duplicate",
            "message": "Try duplicate title",
            "editMode": "markdown"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{duplicate_body}");

    let (status, reader_body) = json_request(
        app,
        Method::POST,
        &create_uri,
        Some(&reader_cookie),
        json!({
            "title": "Reader Page",
            "markdown": "# Reader",
            "message": "Reader should fail"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{reader_body}");

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit_events WHERE target_id = $1 AND event_type = 'repository.wiki_page.save'",
    )
    .bind(page_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("audit count should query");
    assert_eq!(audit_count, 2);
}

#[tokio::test]
async fn repository_wiki_revert_restores_base_revision_and_records_event() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository wiki revert scenario; set TEST_DATABASE_URL");
        return;
    };

    let storage_dir =
        std::env::temp_dir().join(format!("opengithub-wiki-revert-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_GIT_STORAGE_DIR", &storage_dir);
    let config = app_config();
    let owner = create_user(&pool, "wiki-reverter").await;
    let reader = create_user(&pool, "wiki-revert-reader").await;
    let owner_login = owner.username.clone().expect("owner username");
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("wiki-revert-{}", Uuid::new_v4().simple()),
            description: Some("Wiki revert repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let create_uri = format!("/api/repos/{}/{}/wiki/pages", owner_login, repository.name);
    let (status, create_body) = json_request(
        app.clone(),
        Method::POST,
        &create_uri,
        Some(&owner_cookie),
        json!({
            "title": "Rollback Guide",
            "markdown": "# Rollback\n\nStable baseline.",
            "message": "Create rollback guide",
            "editMode": "markdown"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{create_body}");
    let base_revision_id =
        Uuid::parse_str(create_body["page"]["revision"]["id"].as_str().unwrap()).unwrap();

    let update_uri = format!(
        "/api/repos/{}/{}/wiki/Rollback%20Guide",
        owner_login, repository.name
    );
    let (status, update_body) = json_request(
        app.clone(),
        Method::PATCH,
        &update_uri,
        Some(&owner_cookie),
        json!({
            "title": "Rollback Guide",
            "markdown": "# Rollback\n\nBroken change.",
            "message": "Break rollback guide",
            "expectedRevisionId": base_revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{update_body}");
    let head_revision_id =
        Uuid::parse_str(update_body["page"]["revision"]["id"].as_str().unwrap()).unwrap();

    let revert_uri = format!(
        "/api/repos/{}/{}/wiki/reverts",
        owner_login, repository.name
    );
    let (status, reader_body) = json_request(
        app.clone(),
        Method::POST,
        &revert_uri,
        Some(&reader_cookie),
        json!({
            "pageSlug": "Rollback Guide",
            "baseRevisionId": base_revision_id,
            "expectedHeadRevisionId": head_revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{reader_body}");

    let (status, revert_body) = json_request(
        app.clone(),
        Method::POST,
        &revert_uri,
        Some(&owner_cookie),
        json!({
            "pageSlug": "Rollback Guide",
            "baseRevisionId": base_revision_id,
            "expectedHeadRevisionId": head_revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revert_body}");
    assert_eq!(
        revert_body["page"]["markdown"],
        "# Rollback\n\nStable baseline."
    );
    assert!(revert_body["gitCommit"]["message"]
        .as_str()
        .unwrap()
        .starts_with("Revert wiki page to "));
    assert!(revert_body["redirectHref"]
        .as_str()
        .unwrap()
        .ends_with("/wiki/Rollback%20Guide/_history"));
    let restored_revision_id =
        Uuid::parse_str(revert_body["restoredRevisionId"].as_str().unwrap()).unwrap();
    assert_ne!(restored_revision_id, base_revision_id);
    assert_ne!(restored_revision_id, head_revision_id);

    let event_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM wiki_revert_events
        WHERE base_revision_id = $1
          AND head_revision_id = $2
          AND restored_revision_id = $3
        "#,
    )
    .bind(base_revision_id)
    .bind(head_revision_id)
    .bind(restored_revision_id)
    .fetch_one(&pool)
    .await
    .expect("revert event count should query");
    assert_eq!(event_count, 1);

    let (status, stale_body) = json_request(
        app,
        Method::POST,
        &revert_uri,
        Some(&owner_cookie),
        json!({
            "pageSlug": "Rollback Guide",
            "baseRevisionId": base_revision_id,
            "expectedHeadRevisionId": head_revision_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale_body}");
    assert!(!stale_body.to_string().contains("google-client-secret"));
}
