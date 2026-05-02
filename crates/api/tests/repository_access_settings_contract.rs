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
            create_organization, create_repository, grant_repository_permission,
            CreateOrganization, CreateRepository, RepositoryOwner, RepositoryVisibility,
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
async fn repository_access_settings_cover_admin_privacy_invites_and_team_grants() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository access settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("access{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let admin = create_user(&pool, &format!("{marker}-admin")).await;
    let writer = create_user(&pool, &format!("{marker}-writer")).await;
    let outside = create_user(&pool, &format!("{marker}-outside")).await;
    let invitee = create_user(&pool, &format!("{marker}-invitee")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let organization = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("{marker}-org"),
            display_name: "Access Settings Org".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization {
                id: organization.id,
            },
            name: format!("{marker}-repo"),
            description: Some("Private access surface".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, admin.id, RepositoryRole::Admin, "direct")
        .await
        .expect("admin grant should persist");
    grant_repository_permission(&pool, repo.id, writer.id, RepositoryRole::Write, "direct")
        .await
        .expect("writer grant should persist");

    let team_id = insert_team(
        &pool,
        organization.id,
        &format!("{marker}-core"),
        "Core Team",
    )
    .await;
    insert_team_member(&pool, team_id, outside.id).await;

    let uri = format!(
        "/api/repos/{}/{}/settings/access",
        organization.slug, repo.name
    );
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (writer_status, _, writer_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&writer_cookie), None).await;
    assert_eq!(writer_status, StatusCode::FORBIDDEN);
    assert_eq!(writer_body["error"]["code"], "forbidden");
    assert!(!writer_body.to_string().contains("Private access surface"));

    let (read_status, read_headers, read_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&owner_cookie), None).await;
    assert_eq!(read_status, StatusCode::OK);
    assert_json(&read_headers);
    assert_eq!(read_body["visibility"], "private");
    assert_eq!(read_body["viewerPermission"], "owner");
    assert!(read_body["roles"]
        .as_array()
        .expect("roles should be present")
        .iter()
        .any(|role| role["role"] == "triage"));
    assert!(read_body["people"]
        .as_array()
        .expect("people should be present")
        .iter()
        .any(
            |person| person["login"] == writer.username.as_deref().unwrap()
                && person["role"] == "write"
                && person["source"] == "direct"
                && person["canRemove"] == true
        ));

    let (invite_status, _, invite_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({ "emailOrLogin": invitee.email, "role": "triage" })),
    )
    .await;
    assert_eq!(invite_status, StatusCode::OK);
    let invitation_id = invite_body["invitations"][0]["id"]
        .as_str()
        .expect("invitation id should be returned")
        .to_owned();
    assert_eq!(invite_body["invitations"][0]["role"], "triage");
    assert_eq!(invite_body["invitations"][0]["status"], "pending");
    assert_eq!(
        invite_body["invitations"][0]["emailDeliveryStatus"],
        "degraded"
    );

    let (duplicate_invite_status, _, duplicate_invite_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({ "emailOrLogin": invitee.email, "role": "write" })),
    )
    .await;
    assert_eq!(duplicate_invite_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_invite_body["error"]["code"], "conflict");
    assert!(!duplicate_invite_body.to_string().contains("token_hash"));

    let (team_status, _, team_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/teams"),
        Some(&owner_cookie),
        Some(json!({ "teamSlug": format!("{marker}-core"), "role": "maintain" })),
    )
    .await;
    assert_eq!(team_status, StatusCode::OK);
    assert_eq!(team_body["teams"][0]["role"], "maintain");
    assert_eq!(team_body["teams"][0]["canEdit"], true);
    assert!(team_body["people"]
        .as_array()
        .expect("team people should be present")
        .iter()
        .any(
            |person| person["login"] == outside.username.as_deref().unwrap()
                && person["source"] == "team"
                && person["canEdit"] == false
        ));

    let (missing_team_status, _, missing_team_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/teams"),
        Some(&owner_cookie),
        Some(json!({ "teamSlug": format!("{marker}-missing"), "role": "write" })),
    )
    .await;
    assert_eq!(missing_team_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_team_body["error"]["code"], "not_found");
    assert!(!missing_team_body.to_string().contains("Private access surface"));

    let (role_status, _, role_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/collaborators/{}", writer.id),
        Some(&owner_cookie),
        Some(json!({ "role": "maintain" })),
    )
    .await;
    assert_eq!(role_status, StatusCode::OK);
    assert!(role_body["people"]
        .as_array()
        .expect("updated people should be present")
        .iter()
        .any(|person| person["userId"] == writer.id.to_string() && person["role"] == "maintain"));

    let (remove_status, _, remove_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/collaborators/{}", writer.id),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(remove_status, StatusCode::OK);
    assert!(!remove_body["people"]
        .as_array()
        .expect("people should be present after removal")
        .iter()
        .any(|person| person["userId"] == writer.id.to_string()));

    let (cancel_status, _, cancel_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/invitations/{invitation_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(cancel_status, StatusCode::OK);
    assert!(cancel_body["invitations"]
        .as_array()
        .expect("invitations should be present")
        .is_empty());

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.access.%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 4);
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
    User {
        username: Some(login.to_owned()),
        ..user
    }
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

async fn insert_team(pool: &PgPool, organization_id: Uuid, slug: &str, name: &str) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO teams (organization_id, slug, name)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(organization_id)
    .bind(slug)
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("team should insert")
}

async fn insert_team_member(pool: &PgPool, team_id: Uuid, user_id: Uuid) {
    sqlx::query("INSERT INTO team_memberships (team_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(team_id)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("team membership should insert");
}
