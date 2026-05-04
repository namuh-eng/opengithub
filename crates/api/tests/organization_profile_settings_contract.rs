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
        repositories::{create_organization, CreateOrganization},
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
) -> (StatusCode, HeaderMap, Value) {
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
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        headers,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn organization_profile_settings_are_owner_only_and_persist_profile_fields() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization profile settings contract; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgsettings{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, "org-settings-owner").await;
    let member = create_user(&pool, "org-settings-member").await;
    let outsider = create_user(&pool, "org-settings-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Original Org".to_owned(),
            description: Some("Original description".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        r#"
        UPDATE organizations
        SET website_url = 'https://old.example.com',
            location = 'Seoul',
            contact_email = 'owner@example.com',
            public_email = 'hello@example.com',
            billing_email = 'billing@example.com',
            company_name = 'Original Company',
            avatar_url = 'https://images.opengithub.local/org.png'
        WHERE id = $1
        "#,
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("organization metadata should update");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("member should insert");

    let (anonymous_status, anonymous_headers, anonymous_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{marker}/settings/profile"),
        None,
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_json(&anonymous_headers);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (member_status, _, member_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{marker}/settings/profile"),
        Some(&member_cookie),
        None,
    )
    .await;
    assert_eq!(member_status, StatusCode::FORBIDDEN);
    assert_eq!(member_body["error"]["code"], "forbidden");
    assert!(!member_body.to_string().contains("billing@example.com"));

    let (outsider_status, _, _) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/profile"),
        Some(&outsider_cookie),
        Some(json!({ "displayName": "Blocked" })),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);

    let (owner_status, owner_headers, owner_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{}/settings/profile", marker.to_uppercase()),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_json(&owner_headers);
    assert_eq!(owner_body["organization"]["slug"], marker);
    assert_eq!(owner_body["organization"]["name"], "Original Org");
    assert_eq!(
        owner_body["organization"]["settingsHref"],
        format!("/organizations/{marker}/settings/profile")
    );
    assert_eq!(owner_body["profile"]["contactEmail"], "owner@example.com");
    assert_eq!(owner_body["profile"]["billingEmail"], "billing@example.com");
    assert_eq!(owner_body["profile"]["publicEmail"], "hello@example.com");
    assert_eq!(owner_body["viewerState"]["role"], "owner");
    assert_eq!(owner_body["viewerState"]["canEditProfile"], true);
    assert_eq!(owner_body["viewerState"]["canDelete"], false);
    assert_eq!(owner_body["avatar"]["uploadAvailable"], false);

    let (patch_status, _, patch_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/profile"),
        Some(&owner_cookie),
        Some(json!({
            "displayName": "  Updated   Org  ",
            "description": "Maintainer tools",
            "websiteUrl": "https://namuh.co",
            "location": "Seoul, KR",
            "publicEmail": " PUBLIC@Example.COM ",
            "contactEmail": " CONTACT@Example.COM ",
            "billingEmail": null,
            "companyName": "Namuh Labs",
            "socialAccounts": [
                { "provider": "x", "value": "@opengithub" },
                { "provider": "mastodon", "value": "https://social.example/@opengithub" },
                { "provider": "linkedin", "value": "opengithub" },
                { "provider": "bluesky", "value": "opengithub.example" }
            ]
        })),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["profile"]["displayName"], "Updated Org");
    assert_eq!(patch_body["profile"]["description"], "Maintainer tools");
    assert_eq!(patch_body["profile"]["websiteUrl"], "https://namuh.co");
    assert_eq!(patch_body["profile"]["publicEmail"], "public@example.com");
    assert_eq!(patch_body["profile"]["contactEmail"], "contact@example.com");
    assert_eq!(patch_body["profile"]["billingEmail"], Value::Null);
    assert_eq!(patch_body["socialAccounts"].as_array().unwrap().len(), 4);
    assert_eq!(patch_body["socialAccounts"][0]["provider"], "x");
    assert_eq!(patch_body["socialAccounts"][0]["position"], 1);
    assert_eq!(patch_body["socialAccounts"][3]["provider"], "bluesky");

    let stored_public_email = sqlx::query_scalar::<_, Option<String>>(
        "SELECT public_email FROM organizations WHERE id = $1",
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("public email should load");
    assert_eq!(stored_public_email.as_deref(), Some("public@example.com"));

    let social_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_social_accounts WHERE organization_id = $1",
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("social count should load");
    assert_eq!(social_count, 4);

    let audit_metadata = sqlx::query_scalar::<_, Value>(
        "SELECT metadata FROM organization_audit_events WHERE organization_id = $1 AND event_type = 'organization.profile_settings.update' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("audit metadata should load");
    assert_eq!(audit_metadata["slug"], marker);
    assert!(audit_metadata["changedFields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "socialAccounts"));
    let audit_text = audit_metadata.to_string();
    assert!(!audit_text.contains("public@example.com"));
    assert!(!audit_text.contains("contact@example.com"));
    assert!(!audit_text.contains("opengithub.example"));

    let conflict_slug = format!("{marker}-taken");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&conflict_slug)
        .bind(member.id)
        .execute(&pool)
        .await
        .expect("conflicting user slug should update");
    let (conflict_status, _, conflict_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/orgs/{marker}/settings/profile/rename"),
        Some(&owner_cookie),
        Some(json!({ "name": conflict_slug })),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
    assert_eq!(conflict_body["error"]["code"], "conflict");
    assert_eq!(
        conflict_body["error"]["message"],
        "organization slug is already taken"
    );
    assert!(!conflict_body.to_string().contains("user"));

    let reserved_slug = format!("{marker}-reserved");
    sqlx::query("INSERT INTO reserved_slugs (slug, reason) VALUES ($1, 'contract reserved slug')")
        .bind(&reserved_slug)
        .execute(&pool)
        .await
        .expect("reserved slug should insert");
    let (reserved_status, _, reserved_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/orgs/{marker}/settings/profile/rename"),
        Some(&owner_cookie),
        Some(json!({ "name": reserved_slug })),
    )
    .await;
    assert_eq!(reserved_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reserved_body["error"]["code"], "validation_failed");
    assert_eq!(
        reserved_body["error"]["message"],
        "This organization slug is not available."
    );
    assert!(!reserved_body.to_string().contains("contract reserved slug"));

    let renamed_slug = format!("{marker}-renamed");
    let (rename_status, rename_headers, rename_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/orgs/{marker}/settings/profile/rename"),
        Some(&owner_cookie),
        Some(json!({ "name": format!("  {renamed_slug}  ") })),
    )
    .await;
    assert_eq!(rename_status, StatusCode::OK);
    assert_json(&rename_headers);
    assert_eq!(rename_body["organization"]["slug"], renamed_slug);
    assert_eq!(
        rename_body["organization"]["settingsHref"],
        format!("/organizations/{renamed_slug}/settings/profile")
    );

    let (old_status, _, old_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{marker}/settings/profile"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(old_status, StatusCode::NOT_FOUND);
    assert_eq!(old_body["error"]["code"], "not_found");

    let rename_audit = sqlx::query_scalar::<_, Value>(
        "SELECT metadata FROM organization_audit_events WHERE organization_id = $1 AND event_type = 'organization.rename' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("rename audit metadata should load");
    assert_eq!(rename_audit["previousSlug"], marker);
    assert_eq!(rename_audit["newSlug"], renamed_slug);
}

#[tokio::test]
async fn organization_profile_settings_validate_inputs_and_redact_errors() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization profile settings validation; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgsettings{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, "org-settings-valid").await;
    let cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Validation Org".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");

    for (body, expected_message) in [
        (
            json!({ "displayName": "   " }),
            "Organization display name is required.",
        ),
        (
            json!({ "websiteUrl": "javascript:alert(1)" }),
            "Website URL must start with http:// or https://.",
        ),
        (
            json!({ "publicEmail": "not-an-email" }),
            "Enter a valid email address.",
        ),
        (
            json!({
                "socialAccounts": [
                    { "provider": "x", "value": "@one" },
                    { "provider": "x", "value": "@two" }
                ]
            }),
            "Duplicate social provider: x.",
        ),
        (
            json!({
                "socialAccounts": [
                    { "provider": "youtube", "value": "opengithub" }
                ]
            }),
            "Unsupported social provider: youtube.",
        ),
    ] {
        let (status, headers, body) = send_json(
            app.clone(),
            Method::PATCH,
            &format!("/api/orgs/{marker}/settings/profile"),
            Some(&cookie),
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_json(&headers);
        assert_eq!(body["error"]["code"], "validation_failed");
        assert_eq!(body["error"]["message"], expected_message);
        let text = body.to_string();
        assert!(!text.contains("DATABASE_URL"));
        assert!(!text.contains("SESSION_SECRET"));
        assert!(!text.contains("stack backtrace"));
        assert!(!text.contains("panicked"));
    }

    let (missing_status, _, missing_body) = send_json(
        app,
        Method::GET,
        "/api/orgs/does-not-exist/settings/profile",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "not_found");
    assert!(!missing_body.to_string().contains("DATABASE_URL"));
}
