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
async fn organization_profile_returns_public_overview_and_redacts_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization profile scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgprofile{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let follower = create_user(&pool, &format!("{marker}-follower")).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Open Source Lab".to_owned(),
            description: Some("Tools for distributed maintainers".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        r#"
        UPDATE organizations
        SET avatar_url = $1, website_url = $2, location = $3
        WHERE id = $4
        "#,
    )
    .bind("https://images.opengithub.local/org.png")
    .bind("https://namuh.co")
    .bind("Seoul")
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("organization profile metadata should update");
    sqlx::query(
        "INSERT INTO organization_verified_domains (organization_id, domain) VALUES ($1, $2)",
    )
    .bind(org.id)
    .bind("namuh.co")
    .execute(&pool)
    .await
    .expect("verified domain should insert");
    sqlx::query("INSERT INTO organization_follows (user_id, organization_id) VALUES ($1, $2)")
        .bind(follower.id)
        .bind(org.id)
        .execute(&pool)
        .await
        .expect("organization follow should insert");

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-public"),
            description: Some("public org repo".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repo should create");
    let preview_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-preview"),
            description: Some("recent preview org repo".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("preview repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-private"),
            description: Some("private org repo".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");
    sqlx::query(
        "INSERT INTO organization_profile_pins (organization_id, repository_id, position) VALUES ($1, $2, 1), ($1, $3, 2)",
    )
    .bind(org.id)
    .bind(public_repo.id)
    .bind(private_repo.id)
    .execute(&pool)
    .await
    .expect("org pins should insert");
    sqlx::query(
        "INSERT INTO organization_profile_pins (organization_id, repository_id, position) VALUES ($1, $2, 3)",
    )
    .bind(org.id)
    .bind(preview_repo.id)
    .execute(&pool)
    .await
    .expect("second public org pin should insert");
    sqlx::query(
        r#"
        UPDATE repositories
        SET license_template_slug = 'mit',
            is_template = true,
            updated_at = now() - INTERVAL '2 hours'
        WHERE id = $1
        "#,
    )
    .bind(public_repo.id)
    .execute(&pool)
    .await
    .expect("public repo metadata should update");
    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '1 hour' WHERE id = $1")
        .bind(preview_repo.id)
        .execute(&pool)
        .await
        .expect("preview repo metadata should update");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', '#b7410e', 900), ($1, 'TypeScript', '#8c5a3c', 100)",
    )
    .bind(public_repo.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query(
        "INSERT INTO repository_topics (repository_id, topic) VALUES ($1, 'actions'), ($1, 'developer-tools')",
    )
    .bind(public_repo.id)
    .execute(&pool)
    .await
    .expect("topics should insert");
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(follower.id)
        .bind(public_repo.id)
        .execute(&pool)
        .await
        .expect("star should insert");
    sqlx::query(
        "INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(public_repo.id)
    .bind(private_repo.id)
    .bind(follower.id)
    .execute(&pool)
    .await
    .expect("fork should insert");
    let issue_id: Uuid = sqlx::query_scalar(
        "INSERT INTO issues (repository_id, number, title, author_user_id) VALUES ($1, 1, 'Open organization issue', $2) RETURNING id",
    )
    .bind(public_repo.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    sqlx::query(
        "INSERT INTO pull_requests (repository_id, issue_id, number, title, author_user_id, head_ref, base_ref, head_repository_id, base_repository_id) VALUES ($1, $2, 2, 'Open organization PR', $3, 'feature', 'main', $1, $1)",
    )
    .bind(public_repo.id)
    .bind(issue_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pull request should insert");

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/orgs/{}/profile", marker.to_uppercase()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["identity"]["slug"], marker);
    assert_eq!(body["identity"]["name"], "Open Source Lab");
    assert_eq!(body["identity"]["websiteUrl"], "https://namuh.co");
    assert_eq!(body["identity"]["followerCount"], 1);
    assert_eq!(body["identity"]["repositoryCount"], 2);
    assert_eq!(body["identity"]["publicMemberCount"], 1);
    assert_eq!(body["verifiedDomains"][0]["domain"], "namuh.co");
    assert_eq!(body["viewerState"]["authenticated"], false);
    assert_eq!(body["viewerState"]["isMember"], false);
    assert_eq!(body["sponsorship"]["enabled"], false);
    assert_eq!(body["pinnedRepositories"].as_array().unwrap().len(), 2);
    assert_eq!(body["pinnedRepositories"][0]["name"], public_repo.name);
    assert_eq!(body["pinnedRepositories"][1]["name"], preview_repo.name);
    assert_eq!(
        body["pinnedRepositories"][0]["href"],
        format!("/{marker}/{}", public_repo.name)
    );
    assert_eq!(
        body["pinnedRepositories"][0]["primaryLanguage"]["language"],
        "Rust"
    );
    assert_eq!(body["pinnedRepositories"][0]["topics"][0], "actions");
    assert_eq!(body["pinnedRepositories"][0]["starsCount"], 1);
    assert_eq!(body["pinnedRepositories"][0]["forksCount"], 1);
    assert_eq!(body["pinnedRepositories"][0]["openIssuesCount"], 1);
    assert_eq!(body["pinnedRepositories"][0]["openPullRequestsCount"], 1);
    assert_eq!(body["pinnedRepositories"][0]["license"]["slug"], "mit");
    assert_eq!(body["pinnedRepositories"][0]["isTemplate"], true);
    assert_eq!(body["repositoryPreview"].as_array().unwrap().len(), 2);
    assert_eq!(body["repositoryPreview"][0]["name"], preview_repo.name);
    assert_eq!(body["repositoryPreview"][1]["name"], public_repo.name);
    assert_eq!(body["topLanguages"][0]["language"], "Rust");
    assert_eq!(body["topTopics"][0]["topic"], "actions");

    let body_text = body.to_string();
    assert!(!body_text.contains(&private_repo.name));
}

#[tokio::test]
async fn organization_members_can_see_internal_data_and_private_orgs_hide_from_anonymous() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization member profile scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("privateorg{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Private Lab".to_owned(),
            description: Some("member-only organization".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        "UPDATE organizations SET profile_visibility = 'private', public_members_visible = false WHERE id = $1",
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("private org should update");
    let internal_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-internal"),
            description: Some("internal org repo".to_owned()),
            visibility: RepositoryVisibility::Internal,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("internal repo should create");

    let (anonymous_status, _, anonymous_body) =
        get_json(app.clone(), &format!("/api/orgs/{marker}/profile"), None).await;
    assert_eq!(anonymous_status, StatusCode::NOT_FOUND);
    assert_eq!(anonymous_body["error"]["code"], "not_found");

    let (outsider_status, _, _) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/profile"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);

    let (member_status, member_headers, member_body) = get_json(
        app,
        &format!("/api/orgs/{marker}/profile"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(member_status, StatusCode::OK);
    assert_json(&member_headers);
    assert_eq!(member_body["identity"]["isPrivate"], true);
    assert_eq!(member_body["viewerState"]["isMember"], true);
    assert_eq!(member_body["viewerState"]["canAdmin"], true);
    assert_eq!(member_body["peoplePreview"][0]["role"], "owner");
    assert_eq!(
        member_body["repositoryPreview"][0]["name"],
        internal_repo.name
    );
    assert_eq!(member_body["tabCounts"]["repositories"], 1);
}
