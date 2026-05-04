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

async fn create_user(pool: &PgPool, login: &str, display_name: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(display_name),
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

async fn get_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
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

#[tokio::test]
async fn organization_people_admin_lists_tabs_counts_and_capabilities() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization people admin scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpeopleadmin{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner"), "Org Owner").await;
    let admin = create_user(&pool, &format!("{marker}-admin"), "Admin Person").await;
    let member = create_user(&pool, &format!("{marker}-member"), "Member Person").await;
    let outside = create_user(&pool, &format!("{marker}-outside"), "Outside Person").await;
    let invitee = create_user(&pool, &format!("{marker}-invitee"), "Invitee Person").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "People Admin Guild".to_owned(),
            description: Some("People admin contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships
            (organization_id, user_id, role, membership_visibility, outside_collaborator, security_manager, created_at)
        VALUES
            ($1, $2, 'admin', 'private', false, false, now() - INTERVAL '3 days'),
            ($1, $3, 'member', 'public', false, false, now() - INTERVAL '2 days'),
            ($1, $4, 'member', 'private', true, false, now() - INTERVAL '1 day'),
            ($1, $5, 'admin', 'public', false, true, now())
        "#,
    )
    .bind(org.id)
    .bind(admin.id)
    .bind(member.id)
    .bind(outside.id)
    .bind(invitee.id)
    .execute(&pool)
    .await
    .expect("members should insert");

    let team_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO teams (organization_id, slug, name) VALUES ($1, $2, 'Core Team') RETURNING id",
    )
    .bind(org.id)
    .bind(format!("{marker}-core"))
    .fetch_one(&pool)
    .await
    .expect("team should insert");
    sqlx::query("INSERT INTO team_memberships (team_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(team_id)
        .bind(member.id)
        .execute(&pool)
        .await
        .expect("team member should insert");

    sqlx::query(
        r#"
        INSERT INTO organization_invitations
            (organization_id, invited_user_id, invited_email, role, team_ids, status, token_hash,
             invited_by_user_id, email_delivery_status, email_delivery_error, failed_at, expires_at)
        VALUES
            ($1, $2, $3, 'member', ARRAY[$4]::uuid[], 'pending', 'sha256:pending-secret-token',
             $5, 'degraded', NULL, NULL, now() + INTERVAL '7 days'),
            ($1, NULL, $6, 'admin', ARRAY[]::uuid[], 'failed', 'sha256:failed-secret-token',
             $5, 'failed', 'SES sandbox rejected recipient', now(), now() + INTERVAL '7 days')
        "#,
    )
    .bind(org.id)
    .bind(invitee.id)
    .bind(invitee.email.clone())
    .bind(team_id)
    .bind(owner.id)
    .bind(format!("{marker}-failed@example.com"))
    .execute(&pool)
    .await
    .expect("invitations should insert");

    let (anonymous_status, _, anonymous_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin"),
        None,
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (member_status, _, member_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(member_status, StatusCode::FORBIDDEN);
    assert_eq!(member_body["error"]["code"], "forbidden");
    assert!(!member_body.to_string().contains("pending-secret-token"));

    let (owner_status, owner_headers, owner_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin?pageSize=2"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_json(&owner_headers);
    assert_eq!(owner_body["tab"], "members");
    assert_eq!(owner_body["counts"]["members"], 3);
    assert_eq!(owner_body["counts"]["outsideCollaborators"], 1);
    assert_eq!(owner_body["counts"]["pendingCollaborators"], 1);
    assert_eq!(owner_body["counts"]["invitations"], 1);
    assert_eq!(owner_body["counts"]["failedInvitations"], 1);
    assert_eq!(owner_body["counts"]["securityManagers"], 1);
    assert_eq!(owner_body["rows"]["total"], 3);
    assert_eq!(owner_body["rows"]["pageSize"], 2);
    assert_eq!(owner_body["viewerState"]["role"], "owner");
    assert_eq!(owner_body["exports"][0]["format"], "json");
    assert!(owner_body["rows"]["items"]
        .as_array()
        .expect("member rows")
        .iter()
        .any(|row| row["login"] == format!("{marker}-owner")
            && row["actionState"]["finalOwner"] == true
            && row["actionState"]["canRemove"] == false));
    assert!(owner_body["rows"]["items"]
        .as_array()
        .expect("member rows")
        .iter()
        .any(|row| row["login"] == format!("{marker}-admin")
            && row["membershipVisibility"] == "private"
            && row["hasActiveSession"] == true));

    let (search_status, _, search_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin?q=member&pageSize=10"),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(search_body["viewerState"]["role"], "admin");
    assert_eq!(search_body["rows"]["total"], 1);
    assert_eq!(
        search_body["rows"]["items"][0]["login"],
        format!("{marker}-member")
    );
    assert_eq!(search_body["rows"]["items"][0]["teamCount"], 1);
    assert_eq!(
        search_body["rows"]["items"][0]["membershipSource"],
        "organization"
    );

    let (_, _, outside_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin?tab=outside_collaborators"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(outside_body["rows"]["total"], 1);
    assert_eq!(
        outside_body["rows"]["items"][0]["membershipSource"],
        "outside_collaborator"
    );

    let (_, _, pending_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people/admin?tab=pending_collaborators"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(pending_body["invitations"]["total"], 1);
    assert_eq!(pending_body["invitations"]["items"][0]["teamCount"], 1);
    assert_eq!(pending_body["invitations"]["items"][0]["canCancel"], true);
    assert!(pending_body["invitations"]["items"][0]["canRetry"]
        .as_bool()
        .is_some_and(|value| !value));

    let (_, _, failed_body) = get_json(
        app,
        &format!("/api/orgs/{marker}/people/admin?tab=failed_invitations&q=failed"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(failed_body["invitations"]["total"], 1);
    assert_eq!(failed_body["invitations"]["items"][0]["canRetry"], true);
    let failed_text = failed_body.to_string();
    assert!(!failed_text.contains("token_hash"));
    assert!(!failed_text.contains("pending-secret-token"));
    assert!(!failed_text.contains("failed-secret-token"));
    assert!(!failed_text.contains("DATABASE_URL"));
    assert!(!failed_text.contains("SESSION_SECRET"));
}

#[tokio::test]
async fn organization_people_admin_hides_private_organizations_from_outsiders() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization people admin privacy scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpeopleadmin{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner"), "Org Owner").await;
    let outsider = create_user(&pool, &format!("{marker}-outsider"), "Outside Viewer").await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Private People Admin".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query("UPDATE organizations SET profile_visibility = 'private' WHERE id = $1")
        .bind(org.id)
        .execute(&pool)
        .await
        .expect("private org should update");

    let (status, _, body) = get_json(
        app,
        &format!("/api/orgs/{marker}/people/admin"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert!(!body.to_string().contains("Private People Admin"));
}
