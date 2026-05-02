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
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
    },
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use url::{form_urlencoded, Url};
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
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert")
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

fn encode_query_component(value: &str) -> String {
    form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

struct SearchDocumentFixture {
    owner_user_id: Uuid,
    repository_id: Option<Uuid>,
    kind: SearchDocumentKind,
    resource_id: String,
    title: String,
    body: String,
    visibility: RepositoryVisibility,
    metadata: Value,
}

async fn seed_document(pool: &PgPool, actor: &User, fixture: SearchDocumentFixture) {
    upsert_search_document(
        pool,
        actor.id,
        UpsertSearchDocument {
            repository_id: fixture.repository_id,
            owner_user_id: Some(fixture.owner_user_id),
            owner_organization_id: None,
            kind: fixture.kind,
            resource_id: fixture.resource_id,
            title: fixture.title,
            body: Some(fixture.body),
            path: Some("src/rest_search.rs".to_owned()),
            language: Some("Rust".to_owned()),
            branch: Some("main".to_owned()),
            visibility: fixture.visibility,
            metadata: fixture.metadata,
        },
    )
    .await
    .expect("search document should persist");
}

#[tokio::test]
async fn search_rest_endpoints_return_github_compatible_envelopes_and_respect_authz() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search REST compatibility scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "search005-owner").await;
    let outsider = create_user(&pool, "search005-outsider").await;
    let person = create_user(&pool, "search005-person").await;
    let owner_login = format!("search005-owner-{}", Uuid::new_v4().simple());
    let person_login = format!("search005-person-{}", Uuid::new_v4().simple());
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&owner_login)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("owner username should persist");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(&person_login)
        .bind(person.id)
        .execute(&pool)
        .await
        .expect("person username should persist");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("restsearch{}", Uuid::new_v4().simple());
    let repo_name = format!("rest-search-{marker}");

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: Some(format!("REST search fixture {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-{repo_name}"),
            description: Some(format!("Private REST search fixture {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");

    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: owner.id,
            repository_id: Some(public_repo.id),
            kind: SearchDocumentKind::Repository,
            resource_id: format!("repo-{marker}"),
            title: format!("{owner_login} {repo_name} {marker}"),
            body: format!("Repository result for {marker}"),
            visibility: RepositoryVisibility::Public,
            metadata: json!({ "description": "Repository REST result" }),
        },
    )
    .await;
    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: owner.id,
            repository_id: Some(public_repo.id),
            kind: SearchDocumentKind::Code,
            resource_id: format!("code-{marker}"),
            title: format!("Code {marker}"),
            body: format!("fn {marker}() {{ rest_search(); }}"),
            visibility: RepositoryVisibility::Public,
            metadata: json!({ "lineNumber": 1, "fragment": format!("fn {marker}() {{ rest_search(); }}") }),
        },
    )
    .await;
    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: owner.id,
            repository_id: Some(public_repo.id),
            kind: SearchDocumentKind::Commit,
            resource_id: format!("{}abcdef1234567890", Uuid::new_v4().simple()),
            title: format!("Add REST search endpoint {marker}"),
            body: format!(
                "Add REST search endpoint {marker}\n\nExpose GitHub-compatible envelope."
            ),
            visibility: RepositoryVisibility::Public,
            metadata: json!({ "authorLogin": owner_login, "committedAt": Utc::now() }),
        },
    )
    .await;
    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: owner.id,
            repository_id: Some(public_repo.id),
            kind: SearchDocumentKind::Issue,
            resource_id: format!("issue-{marker}"),
            title: format!("Open REST search issue {marker}"),
            body: format!("Issue body {marker}"),
            visibility: RepositoryVisibility::Public,
            metadata: json!({ "number": 42, "state": "open" }),
        },
    )
    .await;
    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: person.id,
            repository_id: None,
            kind: SearchDocumentKind::User,
            resource_id: person_login.clone(),
            title: format!("Search REST Person {marker} {person_login}"),
            body: format!("User result {marker}"),
            visibility: RepositoryVisibility::Public,
            metadata: json!({}),
        },
    )
    .await;
    seed_document(
        &pool,
        &owner,
        SearchDocumentFixture {
            owner_user_id: owner.id,
            repository_id: Some(private_repo.id),
            kind: SearchDocumentKind::Code,
            resource_id: format!("private-code-{marker}"),
            title: format!("Private code {marker}"),
            body: format!("secret {marker}"),
            visibility: RepositoryVisibility::Private,
            metadata: json!({ "lineNumber": 1 }),
        },
    )
    .await;

    let (anonymous_status, anonymous_headers, anonymous_body) =
        get_json(app.clone(), "/api/search/code?q=router", None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_json(&anonymous_headers);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let encoded_code_query = encode_query_component(&format!("{marker} language:Rust path:src"));
    let (code_status, code_headers, code_body) = get_json(
        app.clone(),
        &format!("/api/search/code?q={encoded_code_query}&per_page=100&page=0"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK);
    assert_json(&code_headers);
    assert_eq!(code_body["total_count"], 2);
    assert_eq!(code_body["incomplete_results"], false);
    assert_eq!(code_body["page"], 1);
    assert_eq!(
        code_body["items"][0]["repository"]["full_name"],
        format!("{owner_login}/{repo_name}")
    );
    assert!(code_body["items"][0]["html_url"]
        .as_str()
        .expect("code result should include html_url")
        .contains("/blob/main/src/rest_search.rs"));

    let (outsider_code_status, _headers, outsider_code_body) = get_json(
        app.clone(),
        &format!("/api/search/code?q={encoded_code_query}"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_code_status, StatusCode::OK);
    assert_eq!(outsider_code_body["total_count"], 1);
    assert!(!outsider_code_body.to_string().contains("Private code"));

    let encoded_repo_query =
        encode_query_component(&format!("{marker} repo:{owner_login}/{repo_name}"));
    let (repos_status, _headers, repos_body) = get_json(
        app.clone(),
        &format!("/api/search/repositories?q={encoded_repo_query}&sort=updated&order=desc"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(repos_status, StatusCode::OK);
    assert_eq!(repos_body["total_count"], 1);
    assert_eq!(
        repos_body["items"][0]["full_name"],
        format!("{owner_login}/{repo_name}")
    );
    assert_eq!(repos_body["items"][0]["private"], false);

    let encoded_user_query = encode_query_component(&format!("{marker} user:{person_login}"));
    let (users_status, _headers, users_body) = get_json(
        app.clone(),
        &format!("/api/search/users?q={encoded_user_query}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(users_status, StatusCode::OK);
    assert_eq!(users_body["total_count"], 1);
    assert_eq!(users_body["items"][0]["login"], person_login);
    assert_eq!(
        users_body["items"][0]["html_url"],
        format!("/{person_login}")
    );

    let (commits_status, _headers, commits_body) = get_json(
        app.clone(),
        &format!("/api/search/commits?q={marker}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(commits_status, StatusCode::OK);
    assert_eq!(commits_body["total_count"], 1);
    assert!(commits_body["items"][0]["sha"]
        .as_str()
        .expect("sha should render")
        .ends_with("abcdef1234567890"));
    assert!(commits_body["items"][0]["commit"]["message"]
        .as_str()
        .expect("message should render")
        .contains("GitHub-compatible envelope"));

    let encoded_issue_query = encode_query_component(&format!("{marker} state:open is:open"));
    let (issues_status, _headers, issues_body) = get_json(
        app.clone(),
        &format!("/api/search/issues?q={encoded_issue_query}&sort=updated&order=asc"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(issues_status, StatusCode::OK);
    assert_eq!(issues_body["total_count"], 1);
    assert_eq!(issues_body["items"][0]["number"], 42);
    assert_eq!(issues_body["items"][0]["state"], "open");
    assert_eq!(issues_body["items"][0]["repository"]["name"], repo_name);

    let (bad_status, bad_headers, bad_body) =
        get_json(app, "/api/search/code?q=x", Some(&owner_cookie)).await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_json(&bad_headers);
    assert_eq!(bad_body["status"], 422);
    assert_eq!(bad_body["error"]["code"], "validation_failed");
    let rendered = bad_body.to_string();
    assert!(!rendered.contains("DATABASE_URL"));
    assert!(!rendered.contains("test-session-secret"));
    assert!(!rendered.to_ascii_lowercase().contains("panic"));
}
