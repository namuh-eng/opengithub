use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{self, User},
        tokens::hash_personal_access_token,
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
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("migrator could not run cleanly for PAT scenario ({error}); applying credentials-001 additive migration directly");
        sqlx::raw_sql(include_str!(
            "../migrations/202605040007_personal_access_token_management.up.sql"
        ))
        .execute(&pool)
        .await
        .ok()?;
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    let unique = Uuid::new_v4();
    let user = identity::upsert_user_by_email(
        pool,
        &format!("{label}-{unique}@opengithub.local"),
        Some("Token Manager"),
        None,
    )
    .await
    .expect("user should persist");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("{label}-{unique}"))
        .bind(user.id)
        .execute(pool)
        .await
        .expect("username should update");
    user
}

async fn cookie_header(pool: &PgPool, config: &AppConfig, user: &User) -> (String, String) {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    identity::upsert_session(
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
    (
        session_id,
        format!("{}={cookie_value}", config.session_cookie_name),
    )
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
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let response = app
        .oneshot(
            builder
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
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
async fn token_settings_reject_anonymous_requests_with_json_401() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, headers, body) =
        send_json(app, Method::GET, "/api/settings/tokens", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["status"], 401);
    assert_eq!(body["error"]["code"], "not_authenticated");
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn token_settings_list_context_and_sudo_are_redacted_and_session_bound() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping personal access token settings scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool, "pat-owner").await;
    let other = create_user(&pool, "pat-other").await;
    let (_session_id, cookie) = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (slug, display_name, owner_user_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(format!("token-org-{}", Uuid::new_v4().simple()))
    .bind("Token Org")
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("org should insert");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'owner')",
    )
    .bind(org_id)
    .bind(user.id)
    .execute(&pool)
    .await
    .expect("org membership should insert");
    let repo_id: Uuid = sqlx::query_scalar(
        "INSERT INTO repositories (owner_organization_id, name, visibility, created_by_user_id) VALUES ($1, $2, 'private', $3) RETURNING id",
    )
    .bind(org_id)
    .bind(format!("token-repo-{}", Uuid::new_v4().simple()))
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("repo should insert");
    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role, source) VALUES ($1, $2, 'admin', 'organization')",
    )
    .bind(repo_id)
    .bind(user.id)
    .execute(&pool)
    .await
    .expect("repo permission should insert");

    let secret = format!("oghp_{}_secret", Uuid::new_v4().simple());
    let prefix = secret[..15].to_owned();
    let token_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO personal_access_tokens (
            user_id, name, description, prefix, token_hash, scopes, token_type,
            resource_owner_organization_id, repository_access, expires_at
        )
        VALUES ($1, 'Deploy token', 'Used by release automation', $2, $3,
            ARRAY['repo:read', 'packages:write'], 'fine_grained', $4, 'selected', $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&prefix)
    .bind(hash_personal_access_token(&secret))
    .bind(org_id)
    .bind(Utc::now() + Duration::days(30))
    .fetch_one(&pool)
    .await
    .expect("token should insert");
    sqlx::query(
        "INSERT INTO personal_access_token_repositories (token_id, repository_id) VALUES ($1, $2)",
    )
    .bind(token_id)
    .bind(repo_id)
    .execute(&pool)
    .await
    .expect("selected repository should insert");
    sqlx::query(
        "INSERT INTO personal_access_tokens (user_id, name, prefix, token_hash, scopes, resource_owner_user_id) VALUES ($1, 'Other token', $2, $3, ARRAY['repo'], $1)",
    )
    .bind(other.id)
    .bind(format!("oghp_{}", &Uuid::new_v4().simple().to_string()[..8]))
    .bind(hash_personal_access_token("other-secret-token"))
    .execute(&pool)
    .await
    .expect("other token should insert");

    let (status, headers, list_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/settings/tokens",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(list_body["tokens"].as_array().expect("tokens").len(), 1);
    assert_eq!(list_body["tokens"][0]["name"], "Deploy token");
    assert_eq!(list_body["tokens"][0]["type"], "fine_grained");
    assert_eq!(list_body["tokens"][0]["prefix"], prefix);
    assert_eq!(
        list_body["tokens"][0]["resourceOwner"]["kind"],
        "organization"
    );
    assert_eq!(
        list_body["tokens"][0]["selectedRepositories"][0]["id"],
        repo_id.to_string()
    );
    assert_eq!(list_body["sudo"]["active"], false);
    let rendered = list_body.to_string();
    assert!(!rendered.contains(&secret));
    assert!(!rendered.contains("sha256:"));
    assert!(!rendered.contains("Other token"));

    let (status, _headers, context_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/settings/tokens/new",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(context_body["resourceOwners"]
        .as_array()
        .expect("resource owners")
        .iter()
        .any(|owner| owner["id"] == user.id.to_string() && owner["kind"] == "user"));
    assert!(context_body["resourceOwners"]
        .as_array()
        .expect("resource owners")
        .iter()
        .any(|owner| owner["id"] == org_id.to_string() && owner["kind"] == "organization"));
    assert!(context_body["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .any(|repo| repo["id"] == repo_id.to_string()));
    assert!(context_body["permissionGroups"]
        .as_array()
        .expect("permission groups")
        .iter()
        .any(|group| group["key"] == "repositories"));

    let (status, _headers, bad_sudo) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/sudo",
        Some(&cookie),
        Some(json!({ "confirmation": "wrong@example.test" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(bad_sudo["error"]["code"], "sudo_confirmation_failed");

    let (status, _headers, sudo_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/sudo",
        Some(&cookie),
        Some(json!({ "confirmation": user.email })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sudo_body["sudo"]["active"], true);
    assert!(sudo_body["sudo"]["expiresAt"].as_str().is_some());
    assert!(!sudo_body.to_string().contains("sha256:"));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM security_audit_events WHERE actor_user_id = $1 AND event_type = 'sudo.grant.create'",
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("audit should count");
    assert!(audit_count >= 1);
}
