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
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn send_json(
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
async fn search_route_rejects_auth_and_validation_failures_without_leaks() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search global contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "search-guard-actor").await;
    let cookie = cookie_header(&pool, &config, &actor).await;
    let app = opengithub_api::build_app_with_config(Some(pool), config);

    let (status, headers, body) = send_json(app.clone(), "/api/search?q=router", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_json(&headers);
    assert_eq!(body["status"], 401);
    assert_eq!(body["error"]["code"], "not_authenticated");

    for uri in ["/api/search?q=x", "/api/search?q=router&type=secrets"] {
        let (status, headers, body) = send_json(app.clone(), uri, Some(&cookie)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_json(&headers);
        assert_eq!(body["status"], 422);
        assert_eq!(body["error"]["code"], "validation_failed");
        let rendered = body.to_string();
        assert!(!rendered.contains(&cookie));
        assert!(!rendered.contains("DATABASE_URL"));
        assert!(!rendered.contains("test-session-secret"));
        assert!(!rendered.to_ascii_lowercase().contains("stack"));
        assert!(!rendered.to_ascii_lowercase().contains("panic"));
    }
}

#[tokio::test]
async fn search_route_clamps_pages_orders_deterministically_and_filters_private_kinds() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search global contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "search-guard-owner").await;
    let outsider = create_user(&pool, "search-guard-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("guard{}", Uuid::new_v4().simple());

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("public-search-{}", Uuid::new_v4().simple()),
            description: Some(format!("Public search guard {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repository should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-search-{}", Uuid::new_v4().simple()),
            description: Some(format!("Private search guard {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");

    for (index, kind) in [
        SearchDocumentKind::Repository,
        SearchDocumentKind::Code,
        SearchDocumentKind::Commit,
        SearchDocumentKind::Issue,
        SearchDocumentKind::PullRequest,
    ]
    .into_iter()
    .enumerate()
    {
        let suffix = kind.as_str().replace('_', "-");
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(public_repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind: kind.clone(),
                resource_id: format!("{suffix}-public-{marker}"),
                title: format!("{marker} public {suffix}"),
                body: Some(format!("Public indexed {suffix} result {marker}")),
                path: Some(format!("src/{suffix}.rs")),
                language: Some("Rust".to_owned()),
                branch: Some("main".to_owned()),
                visibility: RepositoryVisibility::Public,
                metadata: json!({ "number": index + 1, "state": "open" }),
            },
        )
        .await
        .expect("public document should persist");
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(private_repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind,
                resource_id: format!("{suffix}-private-{marker}"),
                title: format!("{marker} private {suffix}"),
                body: Some(format!("Private indexed {suffix} result {marker}")),
                path: Some(format!("src/private-{suffix}.rs")),
                language: Some("Rust".to_owned()),
                branch: Some("main".to_owned()),
                visibility: RepositoryVisibility::Private,
                metadata: json!({ "number": index + 10, "state": "closed" }),
            },
        )
        .await
        .expect("private document should persist");
    }

    sqlx::query(
        r#"
        UPDATE search_documents
        SET updated_at = now() - interval '1 hour'
        WHERE resource_id = $1
        "#,
    )
    .bind(format!("repository-public-{marker}"))
    .execute(&pool)
    .await
    .expect("old public document timestamp should update");

    let (owner_status, _headers, owner_body) = send_json(
        app.clone(),
        &format!("/api/search?q={marker}&type=repositories&page=0&pageSize=1000"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["page"], 1);
    assert_eq!(owner_body["pageSize"], 50);
    assert_eq!(owner_body["total"], 2);
    let owner_visibilities = owner_body["items"]
        .as_array()
        .expect("owner items should be an array")
        .iter()
        .map(|item| item["document"]["visibility"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(owner_visibilities.contains(&"private"));
    assert!(owner_visibilities.contains(&"public"));
    let (_repeat_status, _headers, repeat_body) = send_json(
        app.clone(),
        &format!("/api/search?q={marker}&type=repositories&page=0&pageSize=1000"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(repeat_body["items"], owner_body["items"]);

    for result_type in ["code", "commits", "issues", "pull_requests"] {
        let (owner_status, _headers, owner_body) = send_json(
            app.clone(),
            &format!("/api/search?q={marker}&type={result_type}"),
            Some(&owner_cookie),
        )
        .await;
        assert_eq!(owner_status, StatusCode::OK);
        assert_eq!(owner_body["total"], 2);

        let (outsider_status, _headers, outsider_body) = send_json(
            app.clone(),
            &format!("/api/search?q={marker}&type={result_type}"),
            Some(&outsider_cookie),
        )
        .await;
        assert_eq!(outsider_status, StatusCode::OK);
        assert_eq!(outsider_body["total"], 1);
        let visibility = if matches!(result_type, "issues" | "pull_requests") {
            &outsider_body["items"][0]["repository"]["visibility"]
        } else {
            &outsider_body["items"][0]["document"]["visibility"]
        };
        assert_eq!(visibility, "public");
        assert!(!outsider_body.to_string().contains("private"));
    }
}
