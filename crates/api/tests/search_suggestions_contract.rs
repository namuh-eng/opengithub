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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    (status, headers, value)
}

async fn delete_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::DELETE).uri(uri);
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
    let value = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn search_suggestions_return_modal_data_without_private_leaks() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search suggestions contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "suggest-owner").await;
    let outsider = create_user(&pool, "suggest-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("sugg{}", Uuid::new_v4().simple());

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("public-{marker}"),
            description: Some(format!("Public suggestion {marker}")),
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
            name: format!("private-{marker}"),
            description: Some(format!("Private suggestion {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");

    for (repo, visibility, name) in [
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
                kind: SearchDocumentKind::Repository,
                resource_id: format!("repo-{name}-{marker}"),
                title: repo.name.clone(),
                body: Some(format!("{name} repository body {marker}")),
                path: None,
                language: None,
                branch: Some("main".to_owned()),
                visibility: visibility.clone(),
                metadata: json!({ "description": format!("{name} repo suggestion") }),
            },
        )
        .await
        .expect("repository suggestion document should persist");
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind: SearchDocumentKind::Code,
                resource_id: format!("code-{name}-{marker}"),
                title: format!("src/{name}_{marker}.rs"),
                body: Some(format!("fn {name}_{marker}() {{}}")),
                path: Some(format!("src/{name}_{marker}.rs")),
                language: Some("Rust".to_owned()),
                branch: Some("main".to_owned()),
                visibility,
                metadata: json!({ "lineNumber": 7 }),
            },
        )
        .await
        .expect("code suggestion document should persist");
    }

    sqlx::query(
        r#"
        INSERT INTO saved_searches (user_id, name, query, scope)
        VALUES ($1, $2, $3, 'code')
        "#,
    )
    .bind(owner.id)
    .bind(format!("Rust files {marker}"))
    .bind(format!("language:rust {marker}"))
    .execute(&pool)
    .await
    .expect("saved search should insert");

    sqlx::query(
        r#"
        INSERT INTO recent_searches (user_id, query, scope, result_type)
        VALUES ($1, $2, 'all', 'repositories')
        "#,
    )
    .bind(owner.id)
    .bind(format!("recent {marker}"))
    .execute(&pool)
    .await
    .expect("recent search should insert");

    let (owner_status, owner_headers, owner_body) = get_json(
        app.clone(),
        &format!("/api/search/suggestions?q={marker}&scope=all&limit=20"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_json(&owner_headers);
    assert_eq!(owner_body["query"], marker);
    assert_eq!(owner_body["scope"], "all");
    assert!(owner_body["groups"]
        .as_array()
        .unwrap()
        .iter()
        .any(|group| group["id"] == "scopes"));
    let owner_rendered = owner_body.to_string();
    assert!(owner_rendered.contains(&public_repo.name));
    assert!(owner_rendered.contains(&private_repo.name));
    assert!(owner_rendered.contains("direct_repository_jump"));
    assert!(owner_rendered.contains("direct_code_jump"));
    assert!(owner_rendered.contains("language:rust"));
    assert!(owner_rendered.contains("recent"));

    let (outsider_status, _outsider_headers, outsider_body) = get_json(
        app.clone(),
        &format!("/api/search/suggestions?q={marker}&scope=all&limit=20"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::OK);
    let outsider_rendered = outsider_body.to_string();
    assert!(outsider_rendered.contains(&public_repo.name));
    assert!(!outsider_rendered.contains(&private_repo.name));
    assert!(!outsider_rendered.contains(&format!("private_{marker}")));
    assert_eq!(outsider_body["savedSearches"].as_array().unwrap().len(), 0);
    assert_eq!(outsider_body["recentSearches"].as_array().unwrap().len(), 0);

    let (unauth_status, unauth_headers, unauth_body) =
        get_json(app, "/api/search/suggestions?q=repo", None).await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
    assert_json(&unauth_headers);
    assert_eq!(unauth_body["error"]["code"], "not_authenticated");
}

#[tokio::test]
async fn search_suggestions_include_people_teams_and_qualifier_replacements() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping search suggestions contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "suggest-team-actor").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("team{}", Uuid::new_v4().simple());

    let org_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organizations (slug, display_name, description, owner_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(format!("org-{marker}"))
    .bind(format!("Organization {marker}"))
    .bind(format!("Organization suggestion {marker}"))
    .bind(actor.id)
    .fetch_one(&pool)
    .await
    .expect("organization should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
    )
    .bind(org_id)
    .bind(actor.id)
    .execute(&pool)
    .await
    .expect("membership should insert");
    sqlx::query(
        r#"
        INSERT INTO teams (organization_id, slug, name, description)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(org_id)
    .bind(format!("platform-{marker}"))
    .bind(format!("Platform {marker}"))
    .bind("Owns search relevance")
    .execute(&pool)
    .await
    .expect("team should insert");

    let (status, _headers, body) = get_json(
        app.clone(),
        &format!("/api/search/suggestions?q=language%3Aru&scope=org:{marker}&limit=2"),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["scope"], format!("org:{marker}"));
    assert_eq!(body["token"]["prefix"], "language");

    let rendered = body.to_string();
    assert!(rendered.contains("replace_token"));
    assert!(rendered.contains("\"action\":\"replace_token\""));
    assert!(rendered.contains("language:"));
    assert!(rendered.contains("language:rust"));

    let (directory_status, _headers, directory_body) = get_json(
        app,
        &format!("/api/search/suggestions?q={marker}&scope=org:{marker}&limit=6"),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(directory_status, StatusCode::OK);
    let directory_rendered = directory_body.to_string();
    assert!(directory_rendered.contains(&format!("Organization {marker}")));
    assert!(directory_rendered.contains(&format!("Platform {marker}")));
    assert!(
        directory_body["groups"]
            .as_array()
            .expect("groups array")
            .len()
            >= 3
    );
}

#[tokio::test]
async fn saved_search_mutations_validate_persist_and_enforce_ownership() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping saved search contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let actor = create_user(&pool, "saved-search-actor").await;
    let outsider = create_user(&pool, "saved-search-outsider").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("saved{}", Uuid::new_v4().simple());

    let (invalid_status, invalid_headers, invalid_body) = post_json(
        app.clone(),
        "/api/search/saved-searches",
        Some(&actor_cookie),
        json!({ "name": "", "query": "" }),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_json(&invalid_headers);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (created_status, created_headers, created_body) = post_json(
        app.clone(),
        "/api/search/saved-searches",
        Some(&actor_cookie),
        json!({
            "name": format!("Rust routers {marker}"),
            "query": format!("  router   language:rust   {marker} "),
            "scope": "code"
        }),
    )
    .await;
    assert_eq!(created_status, StatusCode::OK);
    assert_json(&created_headers);
    assert_eq!(created_body["name"], format!("Rust routers {marker}"));
    assert_eq!(
        created_body["query"],
        format!("router language:rust {marker}")
    );
    assert_eq!(created_body["scope"], "code");
    assert!(created_body["href"]
        .as_str()
        .expect("href")
        .contains("type=code"));

    let (duplicate_status, _duplicate_headers, duplicate_body) = post_json(
        app.clone(),
        "/api/search/saved-searches",
        Some(&actor_cookie),
        json!({
            "name": format!("rust routers {marker}"),
            "query": format!("other {marker}"),
            "scope": "repositories"
        }),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "duplicate_saved_search");

    let (suggest_status, _suggest_headers, suggest_body) = get_json(
        app.clone(),
        "/api/search/suggestions?q=router&scope=all&limit=8",
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(suggest_status, StatusCode::OK);
    let rendered = suggest_body.to_string();
    assert!(rendered.contains(&format!("Rust routers {marker}")));
    assert!(rendered.contains("recentSearches"));
    assert!(rendered.contains(&format!("router language:rust {marker}")));

    let saved_id = created_body["id"].as_str().expect("saved id");
    let (outsider_delete_status, _headers, outsider_delete_body) = delete_json(
        app.clone(),
        &format!("/api/search/saved-searches/{saved_id}"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_delete_status, StatusCode::NOT_FOUND);
    assert_eq!(outsider_delete_body["error"]["code"], "not_found");

    let (deleted_status, _headers, deleted_body) = delete_json(
        app.clone(),
        &format!("/api/search/saved-searches/{saved_id}"),
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(deleted_status, StatusCode::NO_CONTENT);
    assert_eq!(deleted_body, json!({}));

    let (after_delete_status, _headers, after_delete_body) = get_json(
        app,
        "/api/search/suggestions?q=router&scope=all&limit=8",
        Some(&actor_cookie),
    )
    .await;
    assert_eq!(after_delete_status, StatusCode::OK);
    assert!(!after_delete_body
        .to_string()
        .contains(&format!("Rust routers {marker}")));
}
