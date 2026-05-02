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

struct CodeDocumentFixture {
    repository_id: Uuid,
    visibility: RepositoryVisibility,
    resource_id: String,
    title: String,
    body: String,
    path: String,
    language: String,
    branch: String,
}

async fn seed_code_document(pool: &PgPool, actor: &User, fixture: CodeDocumentFixture) {
    upsert_search_document(
        pool,
        actor.id,
        UpsertSearchDocument {
            repository_id: Some(fixture.repository_id),
            owner_user_id: Some(actor.id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Code,
            resource_id: fixture.resource_id,
            title: fixture.title.clone(),
            body: Some(fixture.body.clone()),
            path: Some(fixture.path.clone()),
            language: Some(fixture.language.clone()),
            branch: Some(fixture.branch.clone()),
            visibility: fixture.visibility,
            metadata: json!({
                "lineNumber": 2,
                "fragment": fixture.body,
                "branch": fixture.branch,
                "symbol": fixture.title,
            }),
        },
    )
    .await
    .expect("code document should persist");
}

#[tokio::test]
async fn code_search_returns_facets_chips_counts_and_validation_errors() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping code search contract scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "code-search-owner").await;
    let outsider = create_user(&pool, "code-search-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("codesearch{}", Uuid::new_v4().simple());

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("public-code-{marker}"),
            description: Some(format!("Public code search fixture {marker}")),
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
            name: format!("private-code-{marker}"),
            description: Some(format!("Private code search fixture {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let archived_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("archived-code-{marker}"),
            description: Some(format!("Archived code search fixture {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("archived repository should create");
    sqlx::query("UPDATE repositories SET is_archived = true WHERE id = $1")
        .bind(archived_repo.id)
        .execute(&pool)
        .await
        .expect("repository should archive");

    seed_code_document(
        &pool,
        &owner,
        CodeDocumentFixture {
            repository_id: public_repo.id,
            visibility: RepositoryVisibility::Public,
            resource_id: format!("public-rust-{marker}"),
            title: format!("Router {marker}"),
            body: format!(
                "fn router_{marker}() {{ build_routes(); }}\nlet {marker}_middleware = tower_layer();\nassert!({marker}_middleware.ready());\ntracing::info!(\"{marker} complete\");"
            ),
            path: "src/router.rs".to_owned(),
            language: "Rust".to_owned(),
            branch: "main".to_owned(),
        },
    )
    .await;
    seed_code_document(
        &pool,
        &owner,
        CodeDocumentFixture {
            repository_id: public_repo.id,
            visibility: RepositoryVisibility::Public,
            resource_id: format!("public-ts-{marker}"),
            title: format!("Client {marker}"),
            body: format!("export function router{marker}() {{ return 'client'; }}"),
            path: "web/src/router.ts".to_owned(),
            language: "TypeScript".to_owned(),
            branch: "main".to_owned(),
        },
    )
    .await;
    seed_code_document(
        &pool,
        &owner,
        CodeDocumentFixture {
            repository_id: public_repo.id,
            visibility: RepositoryVisibility::Public,
            resource_id: format!("feature-rust-{marker}"),
            title: format!("Feature branch {marker}"),
            body: format!("fn feature_branch_{marker}() {{ hidden_from_default(); }}"),
            path: "src/feature_only.rs".to_owned(),
            language: "Rust".to_owned(),
            branch: "feature/search-preview".to_owned(),
        },
    )
    .await;
    seed_code_document(
        &pool,
        &owner,
        CodeDocumentFixture {
            repository_id: private_repo.id,
            visibility: RepositoryVisibility::Private,
            resource_id: format!("private-rust-{marker}"),
            title: format!("Private {marker}"),
            body: format!("fn private_router_{marker}() {{ secret(); }}"),
            path: "crates/api/src/private_router.rs".to_owned(),
            language: "Rust".to_owned(),
            branch: "main".to_owned(),
        },
    )
    .await;
    seed_code_document(
        &pool,
        &owner,
        CodeDocumentFixture {
            repository_id: archived_repo.id,
            visibility: RepositoryVisibility::Public,
            resource_id: format!("archived-rust-{marker}"),
            title: format!("Archived {marker}"),
            body: format!("fn archived_router_{marker}() {{ old(); }}"),
            path: "src/archive.rs".to_owned(),
            language: "Rust".to_owned(),
            branch: "main".to_owned(),
        },
    )
    .await;
    upsert_search_document(
        &pool,
        owner.id,
        UpsertSearchDocument {
            repository_id: Some(public_repo.id),
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Issue,
            resource_id: format!("issue-{marker}"),
            title: format!("Router issue {marker}"),
            body: Some(format!("Issue body {marker}")),
            path: None,
            language: None,
            branch: None,
            visibility: RepositoryVisibility::Public,
            metadata: json!({ "number": 1 }),
        },
    )
    .await
    .expect("issue document should persist");

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/search?q={marker}+language:Rust+path:src&type=code&page=0&pageSize=1000"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 50);
    assert_eq!(body["total"], 3);
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["document"]["branch"] == "main"));
    assert!(!body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["document"]["path"] == "src/feature_only.rs"));
    assert!(body["queryDurationMs"].as_i64().is_some());
    assert_eq!(body["diagnostics"].as_array().unwrap().len(), 0);
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["type"] == "code"));
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["document"]["visibility"] == "private"));
    let grouped_rust_item = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["document"]["path"] == "src/router.rs")
        .expect("public Rust result should be present");
    assert_eq!(grouped_rust_item["match_count"], 4);
    assert_eq!(grouped_rust_item["hidden_match_count"], 1);
    assert_eq!(
        grouped_rust_item["blob_href"],
        format!(
            "/{}/{}/blob/main/src/router.rs",
            owner.email.replace('@', "%40"),
            public_repo.name
        )
    );
    assert_eq!(grouped_rust_item["snippets"].as_array().unwrap().len(), 4);
    assert_eq!(grouped_rust_item["snippets"][0]["line_number"], 1);
    assert!(grouped_rust_item["snippets"][0]["match_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|range| range["end"].as_i64().unwrap() > range["start"].as_i64().unwrap()));
    assert!(body["facets"]["languages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|facet| facet["value"] == "Rust" && facet["selected"] == true));
    assert!(body["facets"]["paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|facet| facet["value"] == "src" && facet["selected"] == true));
    assert!(body["activeChips"]
        .as_array()
        .unwrap()
        .iter()
        .any(|chip| chip["label"] == "language:Rust"
            && chip["removeQuery"].as_str().unwrap().contains(&marker)));
    assert!(body["typeCounts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|count| count["resultType"] == "code" && count["count"].as_i64().unwrap() >= 4));
    assert!(body["typeCounts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|count| count["resultType"] == "issues" && count["count"] == 1));

    let (outsider_status, _headers, outsider_body) = get_json(
        app.clone(),
        &format!("/api/search?q={marker}&type=code"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::OK);
    assert_eq!(outsider_body["total"], 3);
    assert!(outsider_body.to_string().contains(&marker));
    assert!(!outsider_body.to_string().contains("private_router"));
    assert!(outsider_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["document"]["visibility"] == "public"));

    let (repo_status, _headers, repo_body) = get_json(
        app.clone(),
        &format!(
            "/api/search?q={marker}+repo:{}/{}&type=code",
            owner.email, public_repo.name
        ),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(repo_status, StatusCode::OK);
    assert_eq!(repo_body["total"], 2);
    assert!(repo_body["activeChips"]
        .as_array()
        .unwrap()
        .iter()
        .any(|chip| chip["qualifier"] == "repo"));

    let (archived_status, _headers, archived_body) = get_json(
        app.clone(),
        &format!("/api/search?q={marker}+archived:true&type=code"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(archived_status, StatusCode::OK);
    assert_eq!(archived_body["total"], 1);
    assert_eq!(
        archived_body["items"][0]["repository_name"],
        archived_repo.name
    );

    for uri in [
        "/api/search?q=language:&type=code".to_owned(),
        "/api/search?q=router+fork:true&type=code".to_owned(),
        "/api/search?q=/router.*/&type=code".to_owned(),
        format!("/api/search?q={}+repo:broken&type=code", marker),
        format!("/api/search?q={}&type=code", "x".repeat(257)),
    ] {
        let (bad_status, _headers, bad_body) =
            get_json(app.clone(), &uri, Some(&owner_cookie)).await;
        assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(bad_body["status"], 422);
        assert_eq!(bad_body["error"]["code"], "validation_failed");
        let serialized = bad_body.to_string();
        assert!(!serialized.contains("DATABASE_URL"));
        assert!(!serialized.contains("stack backtrace"));
        assert!(!serialized.contains("postgres://"));
    }
}
