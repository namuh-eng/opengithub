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

#[tokio::test]
async fn release_management_context_notes_latest_uploads_and_side_effects() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping release management contract; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("relmgmt{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Release management repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let previous_commit = seed_commit(&pool, repo.id, &owner, "Previous release", 8).await;
    seed_ref(&pool, repo.id, "refs/tags/v0.9.0", "tag", previous_commit).await;
    let target_commit = seed_commit(
        &pool,
        repo.id,
        &owner,
        "Ship management API\n\nDetailed commit body",
        1,
    )
    .await;
    seed_ref(&pool, repo.id, "refs/heads/main", "branch", target_commit).await;
    seed_merged_pull_request(&pool, repo.id, &owner, target_commit, 1).await;
    seed_release_webhook(&pool, repo.id, &owner).await;

    let manage_uri = format!("/api/repos/{}/{}/releases/manage", owner.email, repo.name);
    let (reader_context_status, _, reader_context_body) = send_json(
        app.clone(),
        Method::GET,
        &manage_uri,
        Some(&reader_cookie),
        None,
    )
    .await;
    assert_eq!(reader_context_status, StatusCode::FORBIDDEN);
    assert!(!reader_context_body.to_string().contains(&repo.name));

    let (context_status, _, context_body) = send_json(
        app.clone(),
        Method::GET,
        &manage_uri,
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(context_status, StatusCode::OK);
    assert_eq!(context_body["canWrite"], true);
    assert_eq!(context_body["defaultTarget"], "main");
    assert!(context_body["availableRefs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value["shortName"] == "main" && value["kind"] == "branch"));
    assert!(context_body["previousTagCandidates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value["shortName"] == "v0.9.0"));
    assert!(
        context_body["uploadLimits"]["maxAssetBytes"]
            .as_i64()
            .unwrap()
            > 0
    );

    let (notes_status, _, notes_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/generated-notes"),
        Some(&owner_cookie),
        Some(json!({
            "target": "main",
            "previousTag": "v0.9.0",
            "title": "v1.0.0"
        })),
    )
    .await;
    assert_eq!(notes_status, StatusCode::OK);
    assert_eq!(notes_body["commitCount"], 1);
    assert_eq!(notes_body["mergedPullRequestCount"], 1);
    assert!(notes_body["body"]
        .as_str()
        .unwrap()
        .contains("Ship management API"));
    assert!(notes_body["body"]
        .as_str()
        .unwrap()
        .contains("#1 Release polish"));

    let (intent_status, _, intent_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents"),
        Some(&owner_cookie),
        Some(json!({
            "name": "opengithub-linux.tar.gz",
            "contentType": "application/gzip",
            "byteSize": 4096,
            "checksumSha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        })),
    )
    .await;
    assert_eq!(intent_status, StatusCode::CREATED);
    assert_eq!(intent_body["assetName"], "opengithub-linux.tar.gz");
    assert_eq!(intent_body["status"], "pending");
    assert!(intent_body["uploadUrl"]
        .as_str()
        .unwrap()
        .contains("upload-intents"));
    assert!(!intent_body.to_string().contains("storageKey"));
    assert!(!intent_body.to_string().contains("releases/pending"));
    let upload_intent_id = Uuid::parse_str(intent_body["id"].as_str().unwrap()).unwrap();

    let (invalid_intent_status, _, invalid_intent_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents"),
        Some(&owner_cookie),
        Some(json!({
            "name": "bad.bin",
            "contentType": "bad mime",
            "byteSize": 1
        })),
    )
    .await;
    assert_eq!(invalid_intent_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_intent_body["error"]["code"], "validation_failed");

    let release_uri = format!("/api/repos/{}/{}/releases", owner.email, repo.name);
    let (create_status, _, create_body) = send_json(
        app.clone(),
        Method::POST,
        &release_uri,
        Some(&owner_cookie),
        Some(json!({
            "tagName": "v1.0.0",
            "target": "main",
            "title": "Version one",
            "body": notes_body["body"],
            "draft": false,
            "prerelease": false,
            "latestPolicy": "latest"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(create_body["latest"], true);
    assert!(!create_body.to_string().contains("storageKey"));
    let release_id = Uuid::parse_str(create_body["id"].as_str().unwrap()).unwrap();

    let (complete_status, _, complete_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents/{upload_intent_id}/complete"),
        Some(&owner_cookie),
        Some(json!({
            "releaseId": release_id,
            "handoffToken": intent_body["handoffToken"],
            "checksumSha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        })),
    )
    .await;
    assert_eq!(complete_status, StatusCode::OK);
    assert_eq!(
        complete_body["assets"][0]["name"],
        "opengithub-linux.tar.gz"
    );
    assert_eq!(complete_body["assets"][0]["downloadCount"], 0);
    assert_eq!(
        complete_body["assets"][0]["checksumSha256"],
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert!(!complete_body.to_string().contains("storageKey"));
    let asset_id = Uuid::parse_str(complete_body["assets"][0]["id"].as_str().unwrap()).unwrap();

    let upload_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM release_asset_upload_intents WHERE id = $1",
    )
    .bind(upload_intent_id)
    .fetch_one(&pool)
    .await
    .expect("upload intent status should read");
    assert_eq!(upload_status, "completed");

    let (asset_download_status, _, asset_download_body) = send_json(
        app.clone(),
        Method::GET,
        &format!(
            "/api/repos/{}/{}/releases/assets/{asset_id}",
            owner.email, repo.name
        ),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(asset_download_status, StatusCode::OK);
    assert_eq!(asset_download_body["asset"]["downloadCount"], 1);

    let (stale_intent_status, _, stale_intent_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents/{upload_intent_id}/complete"),
        Some(&owner_cookie),
        Some(json!({
            "releaseId": release_id,
            "handoffToken": intent_body["handoffToken"]
        })),
    )
    .await;
    assert_eq!(stale_intent_status, StatusCode::CONFLICT);
    assert_eq!(stale_intent_body["error"]["code"], "conflict");

    let (cancel_intent_status, _, cancel_intent_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents"),
        Some(&owner_cookie),
        Some(json!({
            "name": "cancelled.zip",
            "contentType": "application/zip",
            "byteSize": 128
        })),
    )
    .await;
    assert_eq!(cancel_intent_status, StatusCode::CREATED);
    let cancel_intent_id = Uuid::parse_str(cancel_intent_body["id"].as_str().unwrap()).unwrap();
    let (cancel_status, _, cancel_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{manage_uri}/upload-intents/{cancel_intent_id}/cancel"),
        Some(&owner_cookie),
        Some(json!({ "reason": "user removed queued asset" })),
    )
    .await;
    assert_eq!(cancel_status, StatusCode::OK);
    assert_eq!(cancel_body["status"], "cancelled");

    let (delete_asset_status, _, delete_asset_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{release_uri}/{release_id}/assets/{asset_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_asset_status, StatusCode::OK);
    assert_eq!(delete_asset_body["assets"].as_array().unwrap().len(), 0);

    let (edit_context_status, _, edit_context_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{manage_uri}/{release_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(edit_context_status, StatusCode::OK);
    assert_eq!(edit_context_body["release"]["id"], release_id.to_string());
    assert_eq!(edit_context_body["release"]["latest"], true);

    let webhook_events = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM webhook_deliveries
        JOIN webhooks ON webhooks.id = webhook_deliveries.webhook_id
        WHERE webhooks.repository_id = $1 AND webhook_deliveries.event = 'release'
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("webhook deliveries should read");
    assert_eq!(webhook_events, 2);

    let audit_text = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(string_agg(after_state::text, ' '), '') FROM release_audit_events WHERE repository_id = $1",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit text should read");
    assert!(audit_text.contains("upload_intent") || audit_text.contains("upload"));
    assert!(audit_text.contains("opengithub-linux.tar.gz"));
    assert!(audit_text.contains("cancelled.zip"));
    assert!(!audit_text.contains("storage_key"));
    assert!(!audit_text.contains("releases/pending"));

    let (draft_status, _, draft_body) = send_json(
        app.clone(),
        Method::POST,
        &release_uri,
        Some(&owner_cookie),
        Some(json!({
            "tagName": "v1.1.0",
            "target": "main",
            "title": "Delete candidate",
            "body": "Draft to delete",
            "draft": true,
            "prerelease": false,
            "latestPolicy": "automatic"
        })),
    )
    .await;
    assert_eq!(draft_status, StatusCode::CREATED);
    let draft_id = Uuid::parse_str(draft_body["id"].as_str().unwrap()).unwrap();
    let tag_exists_before = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM repository_git_refs
          WHERE repository_id = $1 AND kind = 'tag' AND name = 'refs/tags/v1.1.0'
        )
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("tag existence should read");
    assert!(tag_exists_before);

    let (delete_status, _, delete_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{release_uri}/{draft_id}"),
        Some(&owner_cookie),
        Some(json!({ "deleteTag": true })),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
    assert_eq!(delete_body, Value::Null);
    let tag_exists_after = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM repository_git_refs
          WHERE repository_id = $1 AND kind = 'tag' AND name = 'refs/tags/v1.1.0'
        )
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("tag existence should read after delete");
    assert!(!tag_exists_after);
    let delete_audit = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(string_agg(after_state::text, ' '), '') FROM release_audit_events WHERE repository_id = $1 AND event_type = 'release.deleted'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("delete audit should read");
    assert!(delete_audit.contains("deleteTag"));
    assert!(delete_audit.contains("v1.1.0"));
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

async fn seed_commit(
    pool: &PgPool,
    repository_id: Uuid,
    author: &User,
    message: &str,
    days_ago: i64,
) -> Uuid {
    let oid = format!("{:040x}", Uuid::new_v4().as_u128());
    let row = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, committed_at)
        VALUES ($1, $2, $3, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(&oid)
    .bind(author.id)
    .bind(message)
    .bind(Utc::now() - Duration::days(days_ago))
    .fetch_one(pool)
    .await
    .expect("commit should persist");
    row.get("id")
}

async fn seed_ref(pool: &PgPool, repository_id: Uuid, name: &str, kind: &str, commit_id: Uuid) {
    sqlx::query(
        "INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(repository_id)
    .bind(name)
    .bind(kind)
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("ref should persist");
}

async fn seed_merged_pull_request(
    pool: &PgPool,
    repository_id: Uuid,
    author: &User,
    merge_commit_id: Uuid,
    number: i64,
) {
    let issue_id = sqlx::query(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id, closed_by_user_id, closed_at)
        VALUES ($1, $2, $3, '', 'closed', $4, $4, now() - interval '12 hours')
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .bind("Release polish")
    .bind(author.id)
    .fetch_one(pool)
    .await
    .expect("issue should persist")
    .get::<Uuid, _>("id");
    sqlx::query(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, body, state, author_user_id,
            head_ref, base_ref, base_repository_id, merge_commit_id, merged_by_user_id, merged_at, closed_at
        )
        VALUES ($1, $2, $3, $4, '', 'merged', $5, 'feature/release', 'main', $1, $6, $5, now() - interval '6 hours', now() - interval '6 hours')
        "#,
    )
    .bind(repository_id)
    .bind(issue_id)
    .bind(number)
    .bind("Release polish")
    .bind(author.id)
    .bind(merge_commit_id)
    .execute(pool)
    .await
    .expect("pull request should persist");
}

async fn seed_release_webhook(pool: &PgPool, repository_id: Uuid, owner: &User) {
    sqlx::query(
        r#"
        INSERT INTO webhooks (repository_id, url, secret_hash, events, created_by_user_id)
        VALUES ($1, 'https://example.com/release-hook', 'hashed-secret', ARRAY['release'], $2)
        "#,
    )
    .bind(repository_id)
    .bind(owner.id)
    .execute(pool)
    .await
    .expect("release webhook should persist");
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
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response should be JSON")
    };
    (status, headers, value)
}
