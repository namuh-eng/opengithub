use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_organization, create_repository_with_bootstrap, CreateOrganization,
            CreateRepository, RepositoryBootstrapRequest, RepositoryOwner, RepositoryVisibility,
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

async fn send_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
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
async fn organization_overview_returns_verified_public_profile_contract() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization overview scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "org-owner").await;
    let member = create_user(&pool, "org-member").await;
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("open-labs-{}", Uuid::new_v4().simple()),
            display_name: "Open Labs".to_owned(),
            description: Some("Verified maintainers building calm developer tools.".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("member should insert");
    sqlx::query(
        "INSERT INTO organization_verified_domains (organization_id, domain) VALUES ($1, 'openlabs.example')",
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("domain should insert");
    sqlx::query("INSERT INTO organization_follows (user_id, organization_id) VALUES ($1, $2)")
        .bind(member.id)
        .bind(org.id)
        .execute(&pool)
        .await
        .expect("follow should insert");

    let repo = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("editorial-shell-{}", Uuid::new_v4().simple()),
            description: Some("A pinned organization repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("nextjs".to_owned()),
            gitignore_template_slug: Some("node".to_owned()),
            license_template_slug: Some("mit".to_owned()),
        },
    )
    .await
    .expect("repository should create");
    let private_repo = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("private-roadmap-{}", Uuid::new_v4().simple()),
            description: Some("Private roadmap".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest::default(),
    )
    .await
    .expect("private repository should create");

    sqlx::query("INSERT INTO profile_pins (owner_organization_id, repository_id, position) VALUES ($1, $2, 0)")
        .bind(org.id)
        .bind(repo.id)
        .execute(&pool)
        .await
        .expect("pin should insert");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'TypeScript', '3178c6', 3000), ($1, 'Rust', 'dea584', 1000)",
    )
    .bind(repo.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query(
        "INSERT INTO repository_topics (repository_id, topic) VALUES ($1, 'editorial'), ($1, 'organizations')",
    )
    .bind(repo.id)
    .execute(&pool)
    .await
    .expect("topics should insert");
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(member.id)
        .bind(repo.id)
        .execute(&pool)
        .await
        .expect("star should insert");
    sqlx::query(
        "INSERT INTO issues (repository_id, number, title, author_user_id) VALUES ($1, 1, 'Open org shell polish', $2)",
    )
    .bind(repo.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("issue should insert");
    let pr_issue_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO issues (repository_id, number, title, author_user_id) VALUES ($1, 2, 'Ship org shell', $2) RETURNING id",
    )
    .bind(repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("pr backing issue should insert");
    sqlx::query(
        "INSERT INTO pull_requests (repository_id, issue_id, number, title, author_user_id, head_ref, base_ref) VALUES ($1, $2, 1, 'Ship org shell', $3, 'feature/orgs', 'main')",
    )
    .bind(repo.id)
    .bind(pr_issue_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pull request should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let (public_status, public_body) =
        send_json(app.clone(), &format!("/api/orgs/{}", org.slug), None).await;

    assert_eq!(public_status, StatusCode::OK);
    assert_eq!(public_body["slug"], org.slug);
    assert_eq!(public_body["displayName"], "Open Labs");
    assert_eq!(public_body["verifiedDomain"]["domain"], "openlabs.example");
    assert_eq!(public_body["followerCount"], 1);
    assert_eq!(public_body["memberCount"], 2);
    assert_eq!(public_body["repositoryCount"], 1);
    assert_eq!(public_body["pinnedRepositories"][0]["name"], repo.name);
    assert_eq!(public_body["pinnedRepositories"][0]["isPinned"], true);
    assert_eq!(public_body["pinnedRepositories"][0]["starsCount"], 1);
    assert_eq!(public_body["pinnedRepositories"][0]["openIssuesCount"], 1);
    assert_eq!(
        public_body["pinnedRepositories"][0]["openPullRequestsCount"],
        1
    );
    assert_eq!(
        public_body["pinnedRepositories"][0]["primaryLanguage"]["language"],
        "TypeScript"
    );
    assert!(public_body["topics"]
        .as_array()
        .expect("topics should be array")
        .iter()
        .any(|topic| topic["topic"] == "editorial"));
    assert!(public_body["repositories"]
        .as_array()
        .expect("repositories should be array")
        .iter()
        .all(|item| item["name"] != private_repo.name));
    assert_eq!(public_body["viewerCanAdmin"], false);
    assert!(public_body["settingsHref"].is_null());
    assert_eq!(public_body["sponsorship"]["enabled"], false);

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let (owner_status, owner_body) = send_json(
        opengithub_api::build_app_with_config(Some(pool), config),
        &format!("/api/orgs/{}", org.slug),
        Some(&owner_cookie),
    )
    .await;

    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["viewerRole"], "owner");
    assert_eq!(owner_body["viewerCanAdmin"], true);
    assert_eq!(owner_body["repositoryCount"], 2);
    assert_eq!(
        owner_body["settingsHref"],
        format!("/orgs/{}/settings", org.slug)
    );
}

#[tokio::test]
async fn organization_overview_reports_not_found() {
    let app = opengithub_api::build_app_with_config(None, app_config());
    let (status, body) = send_json(app, "/api/orgs/missing", None).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "database_unavailable");
}
