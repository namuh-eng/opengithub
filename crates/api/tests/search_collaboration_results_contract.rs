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
    let cookie_value = session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn send_json(app: axum::Router, uri: &str, cookie: &str) -> (StatusCode, HeaderMap, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
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

#[tokio::test]
async fn issue_and_pull_request_search_returns_facets_filters_sort_and_private_redaction() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search collaboration contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "search-collab-owner").await;
    let outsider = create_user(&pool, "search-collab-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("collab{}", Uuid::new_v4().simple());

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("public-collab-{}", Uuid::new_v4().simple()),
            description: Some(format!("Public collaboration search {marker}")),
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
            name: format!("private-collab-{}", Uuid::new_v4().simple()),
            description: Some(format!("Private collaboration search {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");

    for (repo, visibility, suffix) in [
        (&public_repo, RepositoryVisibility::Public, "public"),
        (&private_repo, RepositoryVisibility::Private, "private"),
    ] {
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind: SearchDocumentKind::Issue,
                resource_id: format!("{}:41", repo.id),
                title: format!("{marker} {suffix} issue"),
                body: Some(format!("{marker} issue body label urgent")),
                path: None,
                language: None,
                branch: None,
                visibility: visibility.clone(),
                metadata: json!({
                    "number": 41,
                    "state": if suffix == "public" { "open" } else { "closed" },
                    "labels": [{ "name": "urgent", "color": "accent" }],
                    "assignees": [{ "login": "mona" }],
                    "milestone": { "title": "M1" },
                    "authorLogin": "mona",
                    "commentCount": if suffix == "public" { 7 } else { 1 },
                    "interactionCount": if suffix == "public" { 13 } else { 2 },
                    "href": format!("/{}/{}/issues/41", repo.owner_login, repo.name),
                }),
            },
        )
        .await
        .expect("issue search document should persist");
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind: SearchDocumentKind::PullRequest,
                resource_id: format!("{}:42", repo.id),
                title: format!("{marker} {suffix} pull"),
                body: Some(format!("{marker} pull body reviewable")),
                path: None,
                language: None,
                branch: Some("feature/search-004".to_owned()),
                visibility,
                metadata: json!({
                    "number": 42,
                    "state": if suffix == "public" { "merged" } else { "open" },
                    "labels": [{ "name": "review", "color": "accent" }],
                    "assignees": [{ "login": "mona" }],
                    "reviewers": [{ "login": "octavia" }],
                    "milestone": { "title": "M2" },
                    "authorLogin": "mona",
                    "headRef": "feature/search-004",
                    "baseRef": "main",
                    "commentCount": if suffix == "public" { 9 } else { 3 },
                    "interactionCount": if suffix == "public" { 17 } else { 4 },
                    "href": format!("/{}/{}/pull/42", repo.owner_login, repo.name),
                }),
            },
        )
        .await
        .expect("pull request search document should persist");
    }

    let (status, headers, body) = send_json(
        app.clone(),
        &format!(
            "/api/search?q={marker}%20state:open%20label:urgent&type=issues&sort=comments-desc"
        ),
        &owner_cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["type"], "issues");
    assert_eq!(body["items"][0]["document"]["metadata"]["commentCount"], 7);
    assert_eq!(body["activeSort"], "comments-desc");
    assert_eq!(body["activeChips"].as_array().expect("chips").len(), 2);
    assert!(body["facets"]["states"].to_string().contains("open"));
    assert!(body["facets"]["labels"].to_string().contains("urgent"));
    assert!(body["facets"]["assignees"].to_string().contains("mona"));
    assert!(body["typeCounts"].to_string().contains("pull_requests"));

    let (pr_status, _headers, pr_body) = send_json(
        app.clone(),
        &format!(
            "/api/search?q={marker}%20reviewer:octavia&type=pullrequests&sort=interactions-desc"
        ),
        &owner_cookie,
    )
    .await;
    assert_eq!(pr_status, StatusCode::OK);
    assert_eq!(pr_body["total"], 2);
    assert_eq!(pr_body["items"][0]["type"], "pull_requests");
    assert_eq!(pr_body["facets"]["reviewers"][0]["value"], "octavia");
    assert_eq!(pr_body["activeSort"], "interactions-desc");

    let (outsider_status, _headers, outsider_body) = send_json(
        app,
        &format!("/api/search?q={marker}&type=pull_requests"),
        &outsider_cookie,
    )
    .await;
    assert_eq!(outsider_status, StatusCode::OK);
    assert_eq!(outsider_body["total"], 1);
    assert!(!outsider_body.to_string().contains("private"));
}
