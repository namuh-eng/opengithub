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

#[tokio::test]
async fn repository_webhook_settings_cover_validation_redaction_delivery_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository webhook settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("hooks{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let outsider = create_user(&pool, &format!("{marker}-outside")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Webhook settings surface".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, writer.id, RepositoryRole::Write, "direct")
        .await
        .expect("writer grant should persist");

    let uri = format!("/api/repos/{}/{}/settings/hooks", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (writer_status, _, writer_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&writer_cookie), None).await;
    assert_eq!(writer_status, StatusCode::FORBIDDEN);
    assert_eq!(writer_body["error"]["code"], "forbidden");
    assert!(!writer_body.to_string().contains("Webhook settings surface"));

    let (invalid_status, _, invalid_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({
            "payloadUrl": "http://receiver.opengithub.local/hook",
            "eventSelection": "selected",
            "events": ["push"]
        })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (create_status, _, create_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({
            "payloadUrl": "https://receiver.opengithub.local/hook",
            "contentType": "json",
            "secret": "super-secret-value",
            "sslVerify": true,
            "eventSelection": "selected",
            "events": ["push", "issues"],
            "active": true
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(create_body["delivery"]["event"], "ping");
    assert_eq!(
        create_body["settings"]["hooks"][0]["secretConfigured"],
        true
    );
    assert_eq!(create_body["settings"]["hooks"][0]["events"][0], "issues");
    assert!(!create_body.to_string().contains("super-secret-value"));
    assert!(!create_body.to_string().contains("secret_hash"));
    let hook_id = create_body["settings"]["hooks"][0]["id"]
        .as_str()
        .expect("hook id should be returned")
        .to_owned();
    let ping_delivery_id = create_body["delivery"]["id"]
        .as_str()
        .expect("delivery id should be returned")
        .to_owned();

    let stored_secret = sqlx::query_scalar::<_, Option<String>>(
        "SELECT secret_hash FROM webhooks WHERE repository_id = $1 AND id = $2",
    )
    .bind(repo.id)
    .bind(Uuid::parse_str(&hook_id).expect("hook uuid"))
    .fetch_one(&pool)
    .await
    .expect("secret hash should load")
    .expect("secret hash should be configured");
    assert!(stored_secret.starts_with("sha256:"));
    assert_ne!(stored_secret, "super-secret-value");

    let (detail_status, _, detail_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{uri}/{hook_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(
        detail_body["hook"]["payloadUrl"],
        "https://receiver.opengithub.local/hook"
    );
    assert_eq!(detail_body["deliveries"][0]["id"], ping_delivery_id);

    let (delivery_status, _, delivery_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{uri}/{hook_id}/deliveries/{ping_delivery_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delivery_status, StatusCode::OK);
    assert_eq!(delivery_body["summary"]["event"], "ping");
    assert!(delivery_body["requestBodyExcerpt"]
        .as_str()
        .expect("request body excerpt should be returned")
        .contains("Keep it logically awesome"));

    let (update_status, _, update_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/{hook_id}"),
        Some(&owner_cookie),
        Some(json!({
            "payloadUrl": "https://receiver.opengithub.local/updated",
            "contentType": "form",
            "sslVerify": false,
            "eventSelection": "push",
            "events": [],
            "active": false
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(
        update_body["hooks"][0]["payloadUrl"],
        "https://receiver.opengithub.local/updated"
    );
    assert_eq!(update_body["hooks"][0]["events"][0], "push");
    assert_eq!(update_body["hooks"][0]["active"], false);
    assert_eq!(update_body["hooks"][0]["secretConfigured"], true);

    let secret_after_blank_update = sqlx::query_scalar::<_, Option<String>>(
        "SELECT secret_hash FROM webhooks WHERE repository_id = $1 AND id = $2",
    )
    .bind(repo.id)
    .bind(Uuid::parse_str(&hook_id).expect("hook uuid"))
    .fetch_one(&pool)
    .await
    .expect("secret hash should reload");
    assert_eq!(
        secret_after_blank_update.as_deref(),
        Some(stored_secret.as_str())
    );

    let (redeliver_status, _, redeliver_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/{hook_id}/deliveries/{ping_delivery_id}/redeliver"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(redeliver_status, StatusCode::OK);
    assert_eq!(
        redeliver_body["delivery"]["redeliveryOfId"],
        ping_delivery_id
    );

    let (outside_status, _, outside_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&outsider_cookie), None).await;
    assert_eq!(outside_status, StatusCode::FORBIDDEN);
    assert!(!outside_body
        .to_string()
        .contains("receiver.opengithub.local"));

    let (delete_status, _, delete_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/{hook_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
    assert!(delete_body["hooks"]
        .as_array()
        .expect("hooks should be present")
        .is_empty());

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.webhook.%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit events should load");
    assert!(audit_count >= 4);

    let leaked_audit_secret = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_settings_audit_events
            WHERE repository_id = $1
              AND (before_state::text LIKE '%super-secret-value%' OR after_state::text LIKE '%super-secret-value%')
        )
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit secret check should run");
    assert!(!leaked_audit_secret);
}

#[tokio::test]
async fn organization_webhook_settings_cover_owner_access_delivery_and_repository_events() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization webhook settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orghooks{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let admin = create_user(&pool, &format!("{marker}-admin")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let org_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO organizations (
            slug, display_name, owner_user_id, contact_email, terms_of_service_type
        )
        VALUES ($1, $2, $3, $4, 'standard')
        RETURNING id
        "#,
    )
    .bind(&marker)
    .bind(format!("{marker} organization"))
    .bind(owner.id)
    .bind(format!("{marker}@opengithub.local"))
    .fetch_one(&pool)
    .await
    .expect("organization should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner'), ($1, $3, 'admin')
        "#,
    )
    .bind(org_id)
    .bind(owner.id)
    .bind(admin.id)
    .execute(&pool)
    .await
    .expect("memberships should insert");

    let uri = format!("/api/orgs/{marker}/settings/hooks");
    let (admin_status, _, admin_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&admin_cookie), None).await;
    assert_eq!(admin_status, StatusCode::FORBIDDEN);
    assert_eq!(admin_body["error"]["code"], "forbidden");

    let (create_status, _, create_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({
            "payloadUrl": "https://receiver.opengithub.local/org-hook",
            "contentType": "json",
            "secret": "organization-secret",
            "eventSelection": "selected",
            "events": ["push", "workflow_run"],
            "active": true
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(create_body["settings"]["slug"], marker);
    assert_eq!(create_body["settings"]["hooks"][0]["events"][0], "push");
    assert_eq!(create_body["delivery"]["event"], "ping");
    assert!(!create_body.to_string().contains("organization-secret"));
    let hook_id = Uuid::parse_str(
        create_body["settings"]["hooks"][0]["id"]
            .as_str()
            .expect("hook id should be returned"),
    )
    .expect("hook id should parse");
    let ping_delivery_id = create_body["delivery"]["id"]
        .as_str()
        .expect("delivery id should exist")
        .to_owned();

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org_id },
            name: format!("{marker}-repo"),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("organization repository should create");

    let queued = opengithub_api::domain::webhooks::enqueue_repository_webhook_event(
        &pool,
        repo.id,
        "push",
        json!({ "ref": "refs/heads/main" }),
    )
    .await
    .expect("organization hook should enqueue for repository push");
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].webhook_id, hook_id);

    let (detail_status, _, detail_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{uri}/{hook_id}/deliveries/{ping_delivery_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(detail_body["summary"]["event"], "ping");
    assert!(detail_body["requestBodyExcerpt"]
        .as_str()
        .expect("request excerpt should exist")
        .contains("organization"));

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_audit_events WHERE organization_id = $1 AND event_type = 'organization.webhook.create'",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .expect("organization audit should load");
    assert_eq!(audit_count, 1);
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
