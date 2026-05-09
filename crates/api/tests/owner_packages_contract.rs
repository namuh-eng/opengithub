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
        repositories::{
            create_organization, create_repository, CreateOrganization, CreateRepository,
            RepositoryOwner, RepositoryVisibility,
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
    let set_cookie =
        session::set_cookie_header(config, &session_id, expires_at).expect("cookie should create");
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

fn package_names(body: &Value) -> Vec<String> {
    body["items"]
        .as_array()
        .expect("items should be array")
        .iter()
        .map(|item| {
            item["name"]
                .as_str()
                .expect("name should be string")
                .to_owned()
        })
        .collect()
}

struct PackageSeed<'a> {
    repository_id: Uuid,
    owner_user_id: Option<Uuid>,
    owner_organization_id: Option<Uuid>,
    created_by_user_id: Uuid,
    name: &'a str,
    package_type: &'a str,
    visibility: &'a str,
    downloads: i64,
}

async fn insert_package(pool: &PgPool, seed: PackageSeed<'_>) -> Uuid {
    let package_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO packages (repository_id, owner_user_id, owner_organization_id, created_by_user_id, name, package_type, visibility)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(seed.repository_id)
    .bind(seed.owner_user_id)
    .bind(seed.owner_organization_id)
    .bind(seed.created_by_user_id)
    .bind(seed.name)
    .bind(seed.package_type)
    .bind(seed.visibility)
    .fetch_one(pool)
    .await
    .expect("package should insert");
    sqlx::query(
        "INSERT INTO package_versions (package_id, version, published_by_user_id) VALUES ($1, '1.0.0', $2)",
    )
    .bind(package_id)
    .bind(seed.created_by_user_id)
    .execute(pool)
    .await
    .expect("version should insert");
    sqlx::query("INSERT INTO package_downloads (package_id, download_count) VALUES ($1, $2)")
        .bind(package_id)
        .bind(seed.downloads)
        .execute(pool)
        .await
        .expect("downloads should insert");
    package_id
}

#[tokio::test]
async fn user_packages_filter_sort_and_redact_private_rows() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping owner package scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkguser{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &marker).await;
    let collaborator = create_user(&pool, &format!("{marker}-collab")).await;
    let collaborator_cookie = cookie_header(&pool, &config, &collaborator).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-public-repo"),
            description: Some("package source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-private-repo"),
            description: Some("private package source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");

    insert_package(
        &pool,
        PackageSeed {
            repository_id: public_repo.id,
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            created_by_user_id: owner.id,
            name: &format!("{marker}-container"),
            package_type: "container",
            visibility: "public",
            downloads: 90,
        },
    )
    .await;
    let private_package = insert_package(
        &pool,
        PackageSeed {
            repository_id: private_repo.id,
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            created_by_user_id: owner.id,
            name: &format!("{marker}-npm-private"),
            package_type: "npm",
            visibility: "private",
            downloads: 5,
        },
    )
    .await;
    sqlx::query(
        "INSERT INTO package_permissions (package_id, user_id, role) VALUES ($1, $2, 'read')",
    )
    .bind(private_package)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("package permission should insert");

    let (status, headers, public_body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/packages?sort=downloads-desc"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(
        package_names(&public_body),
        vec![format!("{marker}-container")]
    );
    assert_eq!(public_body["total"], 1);
    assert_eq!(public_body["items"][0]["downloadCount"], 90);
    assert_eq!(
        public_body["items"][0]["linkedRepository"]["fullName"],
        format!("{marker}/{marker}-public-repo")
    );

    let (status, _, filtered_body) = get_json(
        app,
        &format!(
            "/api/users/{marker}/packages?q=npm&type=npm&visibility=private&sort=downloads-asc"
        ),
        Some(&collaborator_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        package_names(&filtered_body),
        vec![format!("{marker}-npm-private")]
    );
    assert_eq!(filtered_body["filters"]["query"], "npm");
    assert_eq!(filtered_body["filters"]["packageType"], "npm");
    assert_eq!(filtered_body["filters"]["visibility"], "private");
}

#[tokio::test]
async fn organization_packages_show_internal_to_members_only() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization package scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkgorg{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Package Guild".to_owned(),
            description: Some("Owner package contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("org should create");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("member should insert");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-repo"),
            description: Some("org package source".to_owned()),
            visibility: RepositoryVisibility::Internal,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    let internal_package_name = format!("{marker}-maven");
    insert_package(
        &pool,
        PackageSeed {
            repository_id: repo.id,
            owner_user_id: None,
            owner_organization_id: Some(org.id),
            created_by_user_id: owner.id,
            name: &internal_package_name,
            package_type: "maven",
            visibility: "internal",
            downloads: 12,
        },
    )
    .await;
    let private_package_name = format!("{marker}-nuget-private");
    let private_package = insert_package(
        &pool,
        PackageSeed {
            repository_id: repo.id,
            owner_user_id: None,
            owner_organization_id: Some(org.id),
            created_by_user_id: owner.id,
            name: &private_package_name,
            package_type: "nuget",
            visibility: "private",
            downloads: 99,
        },
    )
    .await;

    let (status, _, public_body) =
        get_json(app.clone(), &format!("/api/orgs/{marker}/packages"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(public_body["total"], 0);

    let (status, _, member_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/packages?type=maven&visibility=internal"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(package_names(&member_body), vec![internal_package_name]);
    assert_eq!(member_body["items"][0]["visibility"], "internal");

    let (status, _, private_body) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/packages?type=nuget&visibility=private"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(private_body["total"], 0);
    assert!(package_names(&private_body).is_empty());

    sqlx::query(
        "INSERT INTO package_permissions (package_id, user_id, role) VALUES ($1, $2, 'read')",
    )
    .bind(private_package)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("private package permission should insert");

    let (status, _, permitted_private_body) = get_json(
        app,
        &format!("/api/orgs/{marker}/packages?type=nuget&visibility=private"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(package_names(&permitted_private_body), vec![private_package_name]);
    assert_eq!(permitted_private_body["items"][0]["visibility"], "private");
}
