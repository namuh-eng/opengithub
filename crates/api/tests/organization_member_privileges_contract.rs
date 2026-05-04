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
        .map_err(|error| eprintln!("organization member privileges DB connect failed: {error}"))
        .ok()?;
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("organization member privileges migration warning: {error}");
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
    let login = format!("{label}-{}", Uuid::new_v4().simple());
    let user = upsert_user_by_email(
        pool,
        &format!("{login}@opengithub.local"),
        Some(label),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&login)
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

async fn insert_org_member(pool: &PgPool, organization_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, $3)",
    )
    .bind(organization_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("organization membership should insert");
}

#[tokio::test]
async fn organization_member_privileges_are_owner_only_validate_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization member privileges contract; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpriv{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, "org-priv-owner").await;
    let admin = create_user(&pool, "org-priv-admin").await;
    let member = create_user(&pool, "org-priv-member").await;
    let outsider = create_user(&pool, "org-priv-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Member Privileges".to_owned(),
            description: Some("Policy controls".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    insert_org_member(&pool, org.id, admin.id, "admin").await;
    insert_org_member(&pool, org.id, member.id, "member").await;

    let (anonymous_status, anonymous_headers, anonymous_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{marker}/settings/member-privileges"),
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
        &format!("/api/orgs/{marker}/settings/member-privileges"),
        Some(&member_cookie),
        None,
    )
    .await;
    assert_eq!(member_status, StatusCode::FORBIDDEN);
    assert_eq!(member_body["error"]["code"], "forbidden");

    let (admin_status, _, admin_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/member-privileges"),
        Some(&admin_cookie),
        Some(json!({ "membersCanCreateTeams": false })),
    )
    .await;
    assert_eq!(admin_status, StatusCode::FORBIDDEN);
    assert_eq!(admin_body["error"]["code"], "forbidden");

    let (owner_status, owner_headers, owner_body) = send_json(
        app.clone(),
        Method::GET,
        &format!(
            "/api/orgs/{}/settings/member-privileges",
            marker.to_uppercase()
        ),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_json(&owner_headers);
    assert_eq!(owner_body["organization"]["slug"], marker);
    assert_eq!(
        owner_body["organization"]["settingsHref"],
        format!("/organizations/{marker}/settings/member_privileges")
    );
    assert_eq!(owner_body["policies"]["baseRepositoryPermission"], "read");
    assert_eq!(owner_body["policies"]["projectsBasePermission"], "write");
    assert_eq!(
        owner_body["capabilities"]["requiresConfirmationFields"][0],
        "baseRepositoryPermission"
    );
    assert_eq!(
        owner_body["capabilities"]["locks"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let (invalid_status, _, invalid_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/member-privileges"),
        Some(&owner_cookie),
        Some(json!({ "baseRepositoryPermission": "super-admin" })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert!(!invalid_body.to_string().contains("stack"));

    let (confirm_status, _, confirm_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/member-privileges"),
        Some(&owner_cookie),
        Some(json!({ "baseRepositoryPermission": "admin" })),
    )
    .await;
    assert_eq!(confirm_status, StatusCode::CONFLICT);
    assert_eq!(confirm_body["error"]["code"], "confirmation_required");
    assert_eq!(
        confirm_body["details"]["fields"][0],
        "baseRepositoryPermission"
    );

    let (patch_status, _, patch_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/orgs/{marker}/settings/member-privileges"),
        Some(&owner_cookie),
        Some(json!({
            "baseRepositoryPermission": "admin",
            "projectsBasePermission": "read",
            "membersCanCreatePublicRepositories": false,
            "membersCanCreatePrivateRepositories": true,
            "membersCanCreateInternalRepositories": true,
            "membersCanForkPrivateRepositories": false,
            "repositoryDiscussionsEnabled": false,
            "pagesPublicPublishing": false,
            "pagesPrivatePublishing": true,
            "appAccessRequestPolicy": "owners_only",
            "membersCanChangeRepositoryVisibility": true,
            "membersCanDeleteRepositories": false,
            "membersCanTransferRepositories": true,
            "membersCanDeleteIssues": true,
            "membersCanCreateTeams": false,
            "confirmation": "confirm"
        })),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["policies"]["baseRepositoryPermission"], "admin");
    assert_eq!(patch_body["policies"]["projectsBasePermission"], "read");
    assert_eq!(
        patch_body["policies"]["membersCanCreateInternalRepositories"],
        true
    );
    assert_eq!(patch_body["policies"]["membersCanCreateTeams"], false);

    let stored_policy = sqlx::query_as::<_, (String, bool, bool)>(
        r#"
        SELECT base_repository_permission,
               members_can_create_internal_repositories,
               members_can_create_teams
        FROM organization_policy_settings
        WHERE organization_id = $1
        "#,
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("policy should persist");
    assert_eq!(stored_policy, ("admin".to_owned(), true, false));

    let audit_metadata = sqlx::query_scalar::<_, Value>(
        "SELECT metadata FROM organization_audit_events WHERE organization_id = $1 AND event_type = 'organization.policy.update' ORDER BY created_at DESC LIMIT 1",
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
        .any(|value| value == "baseRepositoryPermission"));
    assert!(audit_metadata["changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| change["field"] == "membersCanCreateTeams"
            && change["before"] == true
            && change["after"] == false));
    let audit_text = audit_metadata.to_string();
    assert!(!audit_text.contains(&owner.email));
    assert!(!audit_text.contains("test-session-secret"));

    let private_org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("{marker}-private"),
            display_name: "Private Privileges".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("private organization should create");
    sqlx::query("UPDATE organizations SET profile_visibility = 'private' WHERE id = $1")
        .bind(private_org.id)
        .execute(&pool)
        .await
        .expect("organization should become private");
    let (private_status, _, private_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/orgs/{marker}-private/settings/member-privileges"),
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");
}
