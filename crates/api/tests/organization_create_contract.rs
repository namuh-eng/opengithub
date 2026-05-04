use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::identity::{upsert_session, upsert_user_by_email, User},
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    let suffix = Uuid::new_v4().simple();
    let user = upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(label),
        None,
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
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("request should build")
    } else {
        builder.body(Body::empty()).expect("request should build")
    };

    let response = app.oneshot(request).await.expect("request should run");
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
async fn create_organization_normalizes_slug_persists_defaults_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization create contract; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "org-create").await;
    let cookie = cookie_header(&pool, &config, &actor).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let unique = Uuid::new_v4().simple().to_string();
    let name = format!("Acme Labs {unique}");

    let (availability_status, availability_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/organizations/slug-availability?name=Acme%20Labs%20{unique}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(availability_status, StatusCode::OK);
    let slug = availability_body["normalizedSlug"]
        .as_str()
        .expect("normalized slug should exist")
        .to_owned();
    assert!(slug.starts_with("acme-labs-"));
    assert_eq!(availability_body["available"], true);

    let (status, body) = send_json(
        app,
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": name,
            "contactEmail": "  ORG-OWNER@Example.COM ",
            "ownershipType": "business",
            "companyName": "  Acme Incorporated  ",
            "termsAccepted": true
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["slug"], slug);
    assert_eq!(body["displayName"], format!("Acme Labs {unique}"));
    assert_eq!(body["contactEmail"], "org-owner@example.com");
    assert_eq!(body["ownershipType"], "business");
    assert_eq!(body["companyName"], "Acme Incorporated");
    assert_eq!(body["termsOfServiceType"], "free_organization_terms");
    assert_eq!(body["role"], "owner");
    assert_eq!(body["href"], format!("/orgs/{slug}"));
    assert_eq!(
        body["settingsHref"],
        format!("/organizations/{slug}/settings/profile")
    );

    let organization_id =
        Uuid::parse_str(body["id"].as_str().expect("organization id")).expect("id should parse");
    let membership_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_memberships WHERE organization_id = $1 AND user_id = $2 AND role = 'owner'",
    )
    .bind(organization_id)
    .bind(actor.id)
    .fetch_one(&pool)
    .await
    .expect("membership should count");
    assert_eq!(membership_count, 1);

    let policy_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_policy_settings WHERE organization_id = $1 AND base_repository_permission = 'read'",
    )
    .bind(organization_id)
    .fetch_one(&pool)
    .await
    .expect("policy should count");
    assert_eq!(policy_count, 1);

    let audit_metadata = sqlx::query_scalar::<_, Value>(
        "SELECT metadata FROM organization_audit_events WHERE organization_id = $1 AND event_type = 'organization.create'",
    )
    .bind(organization_id)
    .fetch_one(&pool)
    .await
    .expect("audit metadata should load");
    assert_eq!(audit_metadata["slug"], slug);
    assert_eq!(audit_metadata["ownershipType"], "business");
    assert_eq!(audit_metadata.get("contactEmail"), None);
}

#[tokio::test]
async fn create_organization_rejects_anonymous_reserved_duplicates_and_invalid_fields() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization create validation contract; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "org-validation").await;
    let cookie = cookie_header(&pool, &config, &actor).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);
    let unique = Uuid::new_v4().simple().to_string();
    let org_name = format!("Duplicate Org {unique}");

    let (anonymous_status, anonymous_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/organizations",
        None,
        Some(json!({
            "name": org_name,
            "contactEmail": "owner@example.com",
            "ownershipType": "personal",
            "termsAccepted": true
        })),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (reserved_status, reserved_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/organizations/slug-availability?name=settings",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(reserved_status, StatusCode::OK);
    assert_eq!(reserved_body["available"], false);
    assert_eq!(reserved_body["reserved"], true);

    let (missing_terms_status, missing_terms_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": format!("Terms Org {unique}"),
            "contactEmail": "owner@example.com",
            "ownershipType": "personal",
            "termsAccepted": false
        })),
    )
    .await;
    assert_eq!(missing_terms_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing_terms_body["error"]["code"], "validation_failed");

    let (email_status, email_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": format!("Email Org {unique}"),
            "contactEmail": "not-an-email",
            "ownershipType": "personal",
            "termsAccepted": true
        })),
    )
    .await;
    assert_eq!(email_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(email_body["error"]["code"], "validation_failed");

    let (business_status, business_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": format!("Business Org {unique}"),
            "contactEmail": "owner@example.com",
            "ownershipType": "business",
            "companyName": " ",
            "termsAccepted": true
        })),
    )
    .await;
    assert_eq!(business_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(business_body["error"]["code"], "validation_failed");

    let (first_status, _) = send_json(
        app.clone(),
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": org_name,
            "contactEmail": "owner@example.com",
            "ownershipType": "personal",
            "companyName": "ignored for personal",
            "termsAccepted": true
        })),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (duplicate_status, duplicate_body) = send_json(
        app,
        Method::POST,
        "/api/organizations",
        Some(&cookie),
        Some(json!({
            "name": format!("duplicate_org_{unique}"),
            "contactEmail": "owner@example.com",
            "ownershipType": "personal",
            "termsAccepted": true
        })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");
}
