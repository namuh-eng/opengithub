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
async fn repository_actions_secrets_are_admin_only_write_only_and_audited() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository Actions secrets scenario; set TEST_DATABASE_URL");
        return;
    };

    std::env::set_var(
        "ACTIONS_SECRETS_KEY",
        "test-actions-secret-key-with-enough-entropy",
    );
    let config = app_config();
    let marker = format!("actsec{}", Uuid::new_v4().simple());
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
            description: Some("Actions secrets contract".to_owned()),
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

    let uri = format!("/api/repos/{}/{}/settings/secrets", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (writer_status, _, writer_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&writer_cookie), None).await;
    assert_eq!(writer_status, StatusCode::FORBIDDEN);
    assert_eq!(writer_body["error"]["code"], "forbidden");
    assert!(!writer_body.to_string().contains("Actions secrets contract"));

    let (invalid_status, _, invalid_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/secrets"),
        Some(&owner_cookie),
        Some(json!({ "name": "GITHUB_TOKEN", "value": "blocked" })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let secret_plaintext = "prod-super-secret-value";
    let (create_secret_status, _, create_secret_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/secrets"),
        Some(&owner_cookie),
        Some(json!({ "name": "deploy_key", "value": secret_plaintext })),
    )
    .await;
    assert_eq!(create_secret_status, StatusCode::CREATED);
    assert_eq!(create_secret_body["secrets"][0]["name"], "DEPLOY_KEY");
    assert_eq!(create_secret_body["secrets"][0]["secretConfigured"], true);
    assert!(create_secret_body["secrets"][0]["value"].is_null());
    assert!(!create_secret_body.to_string().contains(secret_plaintext));
    assert!(!create_secret_body.to_string().contains("encrypted_value"));
    assert!(!create_secret_body.to_string().contains("ciphertext"));
    assert!(!create_secret_body.to_string().contains("fingerprint"));

    let stored = sqlx::query(
        r#"
        SELECT encrypted_value_ciphertext, encrypted_value_nonce, value_fingerprint
        FROM actions_secrets
        WHERE repository_id = $1 AND name = 'DEPLOY_KEY'
        "#,
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("secret row should load");
    let ciphertext: String = sqlx::Row::get(&stored, "encrypted_value_ciphertext");
    let nonce: String = sqlx::Row::get(&stored, "encrypted_value_nonce");
    let fingerprint: String = sqlx::Row::get(&stored, "value_fingerprint");
    assert!(!ciphertext.is_empty());
    assert!(!nonce.is_empty());
    assert!(fingerprint.starts_with("sha256:"));
    assert_ne!(ciphertext, secret_plaintext);
    assert_ne!(fingerprint, secret_plaintext);

    let replacement_secret = "new-secret-material";
    let (update_secret_status, _, update_secret_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/secrets/DEPLOY_KEY"),
        Some(&owner_cookie),
        Some(json!({ "name": "deploy_key_rotated", "value": replacement_secret })),
    )
    .await;
    assert_eq!(update_secret_status, StatusCode::OK);
    assert_eq!(
        update_secret_body["secrets"][0]["name"],
        "DEPLOY_KEY_ROTATED"
    );
    assert!(!update_secret_body.to_string().contains(replacement_secret));

    let (create_variable_status, _, create_variable_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/variables"),
        Some(&owner_cookie),
        Some(json!({ "name": "release_channel", "value": "stable" })),
    )
    .await;
    assert_eq!(create_variable_status, StatusCode::CREATED);
    assert_eq!(
        create_variable_body["variables"][0]["name"],
        "RELEASE_CHANNEL"
    );
    assert_eq!(create_variable_body["variables"][0]["value"], "stable");

    let (duplicate_status, _, duplicate_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/variables"),
        Some(&owner_cookie),
        Some(json!({ "name": "release_channel", "value": "canary" })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let (update_variable_status, _, update_variable_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/variables/RELEASE_CHANNEL"),
        Some(&owner_cookie),
        Some(json!({ "value": "canary" })),
    )
    .await;
    assert_eq!(update_variable_status, StatusCode::OK);
    assert_eq!(update_variable_body["variables"][0]["value"], "canary");

    let (outside_status, _, outside_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&outsider_cookie), None).await;
    assert_eq!(outside_status, StatusCode::FORBIDDEN);
    assert!(!outside_body.to_string().contains("DEPLOY_KEY"));
    assert!(!outside_body.to_string().contains("RELEASE_CHANNEL"));

    let (delete_secret_status, _, delete_secret_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/secrets/DEPLOY_KEY_ROTATED"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_secret_status, StatusCode::OK);
    assert!(delete_secret_body["secrets"]
        .as_array()
        .expect("secrets should be an array")
        .is_empty());

    let (delete_variable_status, _, delete_variable_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/variables/RELEASE_CHANNEL"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_variable_status, StatusCode::OK);
    assert!(delete_variable_body["variables"]
        .as_array()
        .expect("variables should be an array")
        .is_empty());

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.actions_%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit events should load");
    assert!(audit_count >= 5);

    let leaked_secret = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_settings_audit_events
            WHERE repository_id = $1
              AND (
                before_state::text LIKE '%' || $2 || '%'
                OR after_state::text LIKE '%' || $2 || '%'
                OR before_state::text LIKE '%' || $3 || '%'
                OR after_state::text LIKE '%' || $3 || '%'
                OR before_state::text LIKE '%encrypted_value%'
                OR after_state::text LIKE '%encrypted_value%'
                OR before_state::text LIKE '%ciphertext%'
                OR after_state::text LIKE '%ciphertext%'
                OR before_state::text LIKE '%fingerprint%'
                OR after_state::text LIKE '%fingerprint%'
              )
        )
        "#,
    )
    .bind(repo.id)
    .bind(secret_plaintext)
    .bind(replacement_secret)
    .fetch_one(&pool)
    .await
    .expect("audit leakage check should run");
    assert!(!leaked_secret);

    let persisted_plaintext = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM actions_secrets
            WHERE repository_id = $1
              AND (
                encrypted_value_ciphertext = $2
                OR value_fingerprint = $2
                OR encrypted_value_ciphertext = $3
                OR value_fingerprint = $3
              )
        )
        "#,
    )
    .bind(repo.id)
    .bind(secret_plaintext)
    .bind(replacement_secret)
    .fetch_one(&pool)
    .await
    .expect("plaintext persistence check should run");
    assert!(!persisted_plaintext);
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
