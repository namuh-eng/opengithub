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

fn logins(body: &Value) -> Vec<String> {
    body["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .map(|item| {
            item["login"]
                .as_str()
                .expect("login should be string")
                .to_owned()
        })
        .collect()
}

#[tokio::test]
async fn organization_people_respects_public_visibility_search_and_pagination() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization people scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpeople{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner"), "Org Owner").await;
    let admin = create_user(&pool, &format!("{marker}-admin"), "Admin Person").await;
    let member = create_user(&pool, &format!("{marker}-member"), "Member Person").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "People Guild".to_owned(),
            description: Some("People list contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role, created_at)
        VALUES ($1, $2, 'admin', now() - INTERVAL '2 days'),
               ($1, $3, 'member', now() - INTERVAL '1 day')
        "#,
    )
    .bind(org.id)
    .bind(admin.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("members should insert");

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people?pageSize=2"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["mode"], "people");
    assert_eq!(body["total"], 3);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 2);
    assert_eq!(body["tabCounts"]["people"], 3);
    assert_eq!(body["viewerState"]["isMember"], false);
    assert_eq!(
        logins(&body),
        vec![format!("{marker}-owner"), format!("{marker}-admin")]
    );
    assert_eq!(body["items"][0]["name"], "Org Owner");
    assert_eq!(body["items"][0]["href"], format!("/{marker}-owner"));
    assert!(body["items"][0]["role"].is_null());

    let (_, _, second_page) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people?page=2&pageSize=2"),
        None,
    )
    .await;
    assert_eq!(logins(&second_page), vec![format!("{marker}-member")]);

    let (_, _, searched) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/people?q=Admin&pageSize=10"),
        None,
    )
    .await;
    assert_eq!(searched["total"], 1);
    assert_eq!(logins(&searched), vec![format!("{marker}-admin")]);

    let (_, _, owner_body) = get_json(
        app,
        &format!("/api/orgs/{marker}/people?pageSize=10"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_body["viewerState"]["canAdmin"], true);
    assert_eq!(owner_body["items"][0]["role"], "owner");
    assert_eq!(owner_body["items"][1]["role"], "admin");
    assert_eq!(owner_body["items"][2]["role"], "member");
}

#[tokio::test]
async fn organization_people_hides_private_orgs_and_hidden_public_members() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization people privacy scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpeople{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner"), "Org Owner").await;
    let outsider = create_user(&pool, &format!("{marker}-outsider"), "Outside Viewer").await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let public_org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("{marker}-hidden"),
            display_name: "Hidden Members".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("public organization should create");
    sqlx::query("UPDATE organizations SET public_members_visible = false WHERE id = $1")
        .bind(public_org.id)
        .execute(&pool)
        .await
        .expect("public members should hide");

    let private_org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("{marker}-private"),
            display_name: "Private Members".to_owned(),
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
        .expect("private org should hide");

    let (status, _, hidden_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{}-hidden/people", marker),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(hidden_body["total"], 0);
    assert_eq!(hidden_body["items"].as_array().expect("items").len(), 0);

    let (private_status, _, private_body) = get_json(
        app,
        &format!("/api/orgs/{}-private/people", marker),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");
    let private_text = private_body.to_string();
    assert!(!private_text.contains("DATABASE_URL"));
    assert!(!private_text.contains("SESSION_SECRET"));
    assert!(!private_text.contains("panicked"));
}
