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
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
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

async fn create_user(pool: &PgPool, login: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{login} display")),
        Some("https://images.opengithub.local/avatar.png"),
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

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request_body = if let Some(value) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&value).expect("body should serialize"))
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
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

async fn insert_branch(pool: &PgPool, repository_id: Uuid, user_id: Uuid, branch: &str) {
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, $2, $3, $3, $4)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(format!("{}{}", branch, Uuid::new_v4().simple()))
    .bind(user_id)
    .bind(format!("Seed {branch}"))
    .fetch_one(pool)
    .await
    .expect("commit should insert");

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
    .expect("branch ref should insert");
}

#[tokio::test]
async fn repository_settings_are_admin_only_and_persist_audited_updates() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("settings{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Original settings description".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let sibling = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-taken"),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("sibling repository should create");
    insert_branch(&pool, repo.id, owner.id, "main").await;
    insert_branch(&pool, repo.id, owner.id, "release").await;
    insert_branch(&pool, sibling.id, owner.id, "main").await;
    grant_repository_permission(
        &pool,
        repo.id,
        writer.id,
        opengithub_api::domain::permissions::RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("writer permission should grant");

    let uri = format!("/api/repos/{}/{}/settings", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (writer_status, _, writer_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&writer_cookie), None).await;
    assert_eq!(writer_status, StatusCode::FORBIDDEN);
    assert_eq!(writer_body["error"]["code"], "forbidden");

    let (read_status, read_headers, read_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&owner_cookie), None).await;
    assert_eq!(read_status, StatusCode::OK);
    assert_json(&read_headers);
    assert_eq!(read_body["name"], repo.name);
    assert_eq!(read_body["features"]["issuesEnabled"], true);
    assert_eq!(read_body["merge"]["defaultMethod"], "squash");
    assert_eq!(read_body["danger"]["deleteSupported"], false);
    assert!(read_body["branches"]
        .as_array()
        .expect("branches should be array")
        .iter()
        .any(|value| value == "release"));

    let patch = json!({
        "description": "Updated settings description",
        "visibility": "private",
        "defaultBranch": "release",
        "isTemplate": true,
        "allowForking": false,
        "webCommitSignoffRequired": true,
        "features": {
            "issuesEnabled": false,
            "projectsEnabled": true,
            "wikiEnabled": false
        },
        "merge": {
            "allowSquash": false,
            "allowMergeCommit": true,
            "allowRebase": false,
            "defaultMethod": "merge_commit"
        }
    });
    let (patch_status, _, patch_body) = send_json(
        app.clone(),
        Method::PATCH,
        &uri,
        Some(&owner_cookie),
        Some(patch),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["description"], "Updated settings description");
    assert_eq!(patch_body["visibility"], "private");
    assert_eq!(patch_body["defaultBranch"], "release");
    assert_eq!(patch_body["features"]["issuesEnabled"], false);
    assert_eq!(patch_body["merge"]["allowSquash"], false);
    assert_eq!(patch_body["merge"]["defaultMethod"], "merge_commit");
    assert_eq!(
        patch_body["auditEvents"][0]["eventType"],
        "repository.settings.update"
    );
    assert!(patch_body["auditEvents"][0]["changedFields"]
        .as_array()
        .expect("changed fields should be array")
        .iter()
        .any(|value| value == "default_branch"));

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 1);

    let (merge_status, _, merge_body) = send_json(
        app.clone(),
        Method::PATCH,
        &uri,
        Some(&owner_cookie),
        Some(json!({
            "merge": {
                "allowSquash": false,
                "allowMergeCommit": false,
                "allowRebase": false
            }
        })),
    )
    .await;
    assert_eq!(merge_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(merge_body["error"]["code"], "validation_failed");

    let (branch_status, _, branch_body) = send_json(
        app.clone(),
        Method::PATCH,
        &uri,
        Some(&owner_cookie),
        Some(json!({ "defaultBranch": "missing" })),
    )
    .await;
    assert_eq!(branch_status, StatusCode::CONFLICT);
    assert_eq!(branch_body["error"]["code"], "conflict");

    let (rename_status, _, rename_body) = send_json(
        app,
        Method::PATCH,
        &uri,
        Some(&owner_cookie),
        Some(json!({ "name": sibling.name })),
    )
    .await;
    assert_eq!(rename_status, StatusCode::CONFLICT);
    assert_eq!(rename_body["error"]["code"], "conflict");
    assert!(!rename_body.to_string().contains("DATABASE_URL"));
}
