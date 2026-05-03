use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity::{self, User},
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
        eprintln!("migrator could not run cleanly for signing-key scenario ({error}); applying credentials-002 additive migration directly");
        sqlx::raw_sql(include_str!(
            "../migrations/202605040008_personal_signing_keys.up.sql"
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
        Some("Signing Key Manager"),
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

async fn grant_sudo(pool: &PgPool, user_id: Uuid, session_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO sudo_grants (session_id, user_id, method, expires_at)
        VALUES ($1, $2, 'test', now() + interval '30 minutes')
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("sudo grant should insert");
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

fn ssh_public_key() -> String {
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAABAgMEBQYHCAkKCwwNDg8QERITFBUWFxgZGhscHR4f".to_owned()
}

fn gpg_public_key(email: &str) -> String {
    format!(
        "-----BEGIN PGP PUBLIC KEY BLOCK-----\nComment: {email}\n\nAAECAwQFBgcICQoLDA0ODxAREhM=\n-----END PGP PUBLIC KEY BLOCK-----"
    )
}

#[tokio::test]
async fn key_settings_reject_anonymous_requests_with_json_401() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, headers, body) =
        send_json(app, Method::GET, "/api/settings/keys", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["status"], 401);
    assert_eq!(body["error"]["code"], "not_authenticated");
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn signing_key_settings_validate_persist_revoke_and_redact() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping signing key settings scenario; set TEST_DATABASE_URL");
        return;
    };
    let config = app_config();
    let user = create_user(&pool, "signing-owner").await;
    let (session_id, cookie) = cookie_header(&pool, &config, &user).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, _headers, empty) = send_json(
        app.clone(),
        Method::GET,
        "/api/settings/keys",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty["sshKeys"].as_array().expect("ssh keys").len(), 0);
    assert_eq!(empty["gpgKeys"].as_array().expect("gpg keys").len(), 0);
    assert_eq!(empty["vigilantMode"], false);
    assert_eq!(empty["sudo"]["active"], false);

    let (status, _headers, invalid_ssh) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/keys/ssh",
        Some(&cookie),
        Some(json!({
            "title": "Bad key",
            "keyType": "ssh-ed25519",
            "publicKey": "ssh-ed25519 not-base64",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_ssh["error"]["code"], "validation_failed");

    let (status, _headers, ssh_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/keys/ssh",
        Some(&cookie),
        Some(json!({
            "title": "Laptop",
            "keyType": "ssh-ed25519",
            "publicKey": ssh_public_key(),
            "accessMode": "read_write",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let ssh_key_id = ssh_body["sshKey"]["id"].as_str().expect("ssh key id");
    assert_eq!(ssh_body["sshKey"]["title"], "Laptop");
    assert_eq!(ssh_body["sshKey"]["keyType"], "ssh-ed25519");
    assert!(ssh_body["sshKey"]["fingerprintSha256"]
        .as_str()
        .expect("fingerprint")
        .starts_with("SHA256:"));
    assert!(!ssh_body.to_string().contains("AAAAC3NzaC1lZDI1NTE5"));

    let (status, _headers, duplicate_ssh) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/keys/ssh",
        Some(&cookie),
        Some(json!({
            "title": "Duplicate",
            "publicKey": ssh_public_key(),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(duplicate_ssh["error"]["message"]
        .as_str()
        .expect("duplicate message")
        .contains("fingerprint"));

    let email = format!("signing-{}@opengithub.local", Uuid::new_v4());
    let (status, _headers, bad_gpg) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/keys/gpg",
        Some(&cookie),
        Some(json!({ "title": "Bad GPG", "armoredPublicKey": "not a key" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_gpg["error"]["code"], "validation_failed");

    let (status, _headers, gpg_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/settings/keys/gpg",
        Some(&cookie),
        Some(json!({
            "title": "Release signing",
            "armoredPublicKey": gpg_public_key(&email),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let gpg_key_id = gpg_body["gpgKey"]["id"].as_str().expect("gpg key id");
    assert_eq!(gpg_body["gpgKey"]["title"], "Release signing");
    assert_eq!(gpg_body["gpgKey"]["emails"][0], email.to_ascii_lowercase());
    assert!(!gpg_body.to_string().contains("BEGIN PGP PUBLIC KEY BLOCK"));

    let (status, _headers, vigilant_body) = send_json(
        app.clone(),
        Method::PATCH,
        "/api/settings/keys/vigilant-mode",
        Some(&cookie),
        Some(json!({ "enabled": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(vigilant_body["vigilantMode"], true);

    let (status, _headers, no_sudo_revoke) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/keys/ssh/{ssh_key_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(no_sudo_revoke["error"]["code"], "sudo_required");

    grant_sudo(&pool, user.id, &session_id).await;
    let (status, _headers, revoked_ssh) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/keys/ssh/{ssh_key_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(revoked_ssh["sshKey"]["revokedAt"].as_str().is_some());
    assert!(revoked_ssh["revokedAt"].as_str().is_some());

    let (status, _headers, revoked_gpg) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/settings/keys/gpg/{gpg_key_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(revoked_gpg["gpgKey"]["revokedAt"].as_str().is_some());

    let (status, _headers, final_body) =
        send_json(app, Method::GET, "/api/settings/keys", Some(&cookie), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(final_body["vigilantMode"], true);
    assert_eq!(final_body["sshKeys"].as_array().expect("ssh keys").len(), 1);
    assert_eq!(final_body["gpgKeys"].as_array().expect("gpg keys").len(), 1);
    assert!(!final_body.to_string().contains("AAAAC3NzaC1lZDI1NTE5"));
    assert!(!final_body
        .to_string()
        .contains("BEGIN PGP PUBLIC KEY BLOCK"));

    let audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM security_audit_events
        WHERE actor_user_id = $1
          AND event_type = ANY($2)
        "#,
    )
    .bind(user.id)
    .bind(vec![
        "signing_key.ssh.create",
        "signing_key.ssh.revoke",
        "signing_key.gpg.create",
        "signing_key.gpg.revoke",
        "vigilant_mode.update",
    ])
    .fetch_one(&pool)
    .await
    .expect("audit count should query");
    assert_eq!(audit_count, 5);
}
