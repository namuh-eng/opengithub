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
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, insert_commit,
            repository_dependencies_for_actor_by_owner_name,
            repository_dependents_for_actor_by_owner_name, upsert_git_ref, CreateCommit,
            CreateRepository, RepositoryDependencyQuery, RepositoryDependentsQuery,
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

    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping dependency graph scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_dependency_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.dependency_manifests') IS NOT NULL
               AND to_regclass('public.repository_dependencies') IS NOT NULL
               AND to_regclass('public.sbom_exports') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_dependency_tables {
            eprintln!("skipping dependency graph scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing dependency graph scenario with pre-applied schema after migration warning: {error}"
        );
    }
    if let Err(error) =
        sqlx::query("ALTER TABLE sbom_exports ADD COLUMN IF NOT EXISTS artifact_json jsonb")
            .execute(&pool)
            .await
    {
        eprintln!("skipping dependency graph scenario; sbom schema failed: {error}");
        return None;
    }
    if let Err(error) = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS sbom_exports_ready_download_idx
        ON sbom_exports (repository_id, id, status)
        WHERE status = 'ready'
        "#,
    )
    .execute(&pool)
    .await
    {
        eprintln!("skipping dependency graph scenario; sbom index failed: {error}");
        return None;
    }
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
        Some(&format!("https://avatars.opengithub.local/{label}.png")),
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

async fn get_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    request_json(app, Method::GET, uri, cookie).await
}

async fn request_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    builder = builder.method(method);
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

async fn get_response(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
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
    (status, headers, bytes.to_vec())
}

async fn seed_file(pool: &PgPool, repository_id: Uuid, commit_id: Uuid, path: &str, content: &str) {
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(path)
    .bind(content)
    .bind(format!("blob-{}", Uuid::new_v4().simple()))
    .bind(content.len() as i64)
    .execute(pool)
    .await
    .expect("repository file should insert");
}

#[tokio::test]
async fn dependency_graph_extracts_filters_and_protects_private_repositories() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping dependency graph scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "deps-owner").await;
    let actor = create_user(&pool, "deps-actor").await;
    let outsider = create_user(&pool, "deps-outsider").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("deps-{}", Uuid::new_v4().simple()),
            description: Some("Dependency graph source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        actor.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("actor should read source");
    let commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Add dependency manifests".to_owned(),
            tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
            parent_oids: Vec::new(),
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("commit should insert");
    upsert_git_ref(&pool, repository.id, "main", "branch", Some(commit.id))
        .await
        .expect("main ref should upsert");

    seed_file(
        &pool,
        repository.id,
        commit.id,
        "package.json",
        r#"{"dependencies":{"@namuh/flow":"^1.2.3"},"devDependencies":{"vite":"5.0.0"}}"#,
    )
    .await;
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "package-lock.json",
        r#"{"packages":{"":{"name":"app"},"node_modules/@namuh/flow":{"version":"1.2.4","license":"MIT"},"node_modules/ansi-regex":{"version":"6.0.1","license":"MIT"}}}"#,
    )
    .await;
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "crates/api/Cargo.toml",
        r#"[package]
name = "api"
[dependencies]
axum = "0.7"
serde = { version = "1", features = ["derive"] }
"#,
    )
    .await;
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "crates/api/Cargo.lock",
        r#"[[package]]
name = "axum"
version = "0.7.9"
[[package]]
name = "tower"
version = "0.5.2"
"#,
    )
    .await;
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "requirements.txt",
        "fastapi==0.110.0\nuvicorn>=0.29\n",
    )
    .await;

    let view = repository_dependencies_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryDependencyQuery {
            query: None,
            ecosystem: None,
            relationship: None,
        },
    )
    .await
    .expect("dependencies should load")
    .expect("repository should exist");
    assert!(view.availability.indexed);
    assert_eq!(view.summary.manifest_count, 3);
    assert!(view.summary.direct_count >= 5);
    assert!(view.summary.transitive_count >= 2);
    assert!(view
        .dependencies
        .iter()
        .any(|dependency| dependency.package.name == "@namuh/flow"
            && dependency.relationship == "direct"
            && dependency
                .manifest_href
                .ends_with("/blob/main/package.json")));
    assert!(view.dependencies.iter().any(|dependency| {
        dependency.package.name == "ansi-regex"
            && dependency.relationship == "transitive"
            && dependency
                .lockfile_href
                .as_deref()
                .unwrap_or("")
                .ends_with("package-lock.json")
    }));
    assert!(view
        .summary
        .ecosystem_counts
        .iter()
        .any(|count| count.ecosystem == "cargo" && count.count >= 2));
    assert!(view.export.supported);

    let filtered = repository_dependencies_for_actor_by_owner_name(
        &pool,
        actor.id,
        &repository.owner_login,
        &repository.name,
        RepositoryDependencyQuery {
            query: Some("tower"),
            ecosystem: Some("cargo"),
            relationship: Some("transitive"),
        },
    )
    .await
    .expect("filtered dependencies should load")
    .expect("repository should exist");
    assert_eq!(filtered.summary.total, 1);
    assert_eq!(filtered.dependencies[0].package.name, "tower");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!(
        "/api/repos/{}/{}/network/dependencies?ecosystem=npm",
        repository.owner_login, repository.name
    );
    let (status, body) = get_json(app.clone(), &uri, Some(&actor_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repository"]["name"], repository.name);
    assert!(body["dependencies"]
        .as_array()
        .expect("dependencies should be an array")
        .iter()
        .all(|dependency| dependency["package"]["ecosystem"] == "npm"));

    let (unauthenticated_status, unauthenticated_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(unauthenticated_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unauthenticated_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &uri, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert_eq!(outsider_body["error"]["code"], "not_found");
    assert!(!outsider_body.to_string().contains("package-lock"));

    let invalid_uri = format!(
        "/api/repos/{}/{}/network/dependencies?ecosystem=rubygems",
        repository.owner_login, repository.name
    );
    let (invalid_status, invalid_body) = get_json(app, &invalid_uri, Some(&actor_cookie)).await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn dependency_graph_exports_downloadable_spdx_sbom_and_audits() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping dependency graph export scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "sbom-owner").await;
    let actor = create_user(&pool, "sbom-actor").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("sbom-{}", Uuid::new_v4().simple()),
            description: Some("SBOM source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        actor.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("actor should read source");
    let commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Add package manifest".to_owned(),
            tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
            parent_oids: Vec::new(),
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("commit should insert");
    upsert_git_ref(&pool, repository.id, "main", "branch", Some(commit.id))
        .await
        .expect("main ref should upsert");
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "package.json",
        r#"{"dependencies":{"@namuh/flow":"^1.2.3"}}"#,
    )
    .await;
    seed_file(
        &pool,
        repository.id,
        commit.id,
        "package-lock.json",
        r#"{"packages":{"node_modules/@namuh/flow":{"version":"1.2.4","license":"MIT"}}}"#,
    )
    .await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let export_uri = format!(
        "/api/repos/{}/{}/network/dependencies/sbom",
        repository.owner_login, repository.name
    );
    let (export_status, export_body) =
        request_json(app.clone(), Method::POST, &export_uri, Some(&actor_cookie)).await;
    assert_eq!(export_status, StatusCode::CREATED);
    assert_eq!(export_body["status"], "ready");
    assert_eq!(export_body["format"], "spdx-json");
    assert!(export_body["downloadHref"]
        .as_str()
        .expect("download href")
        .contains("/network/dependencies/sbom/"));
    assert!(export_body["artifactByteSize"].as_i64().unwrap_or_default() > 0);

    let download_href = export_body["downloadHref"].as_str().expect("download href");
    let (download_status, headers, bytes) =
        get_response(app, download_href, Some(&actor_cookie)).await;
    assert_eq!(download_status, StatusCode::OK);
    assert!(headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .contains("attachment"));
    let artifact: Value = serde_json::from_slice(&bytes).expect("download should be json");
    assert_eq!(artifact["spdxVersion"], "SPDX-2.3");
    assert!(artifact["packages"]
        .as_array()
        .expect("packages should be an array")
        .iter()
        .any(|package| package["name"] == "@namuh/flow"));
    assert!(!artifact.to_string().contains("test-session-secret"));

    let audit_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM repository_settings_audit_events
        WHERE repository_id = $1
          AND actor_user_id = $2
          AND event_type = 'dependency_graph.sbom_export'
        "#,
    )
    .bind(repository.id)
    .bind(actor.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn dependency_graph_dependents_filter_public_usage_and_hide_private_consumers() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping dependency graph dependents scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "dependents-owner").await;
    let actor = create_user(&pool, "dependents-actor").await;
    let public_consumer_owner = create_user(&pool, "public-consumer").await;
    let private_consumer_owner = create_user(&pool, "private-consumer").await;
    let actor_cookie = cookie_header(&pool, &config, &actor).await;
    let package_name = format!("@namuh/flow-{}", Uuid::new_v4().simple());

    let source = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("source-{}", Uuid::new_v4().simple()),
            description: Some("Public package source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("source repository should create");
    grant_repository_permission(&pool, source.id, actor.id, RepositoryRole::Read, "direct")
        .await
        .expect("actor should read source");

    let public_consumer = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: public_consumer_owner.id,
            },
            name: format!("consumer-{}", Uuid::new_v4().simple()),
            description: Some("Public dependent repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: public_consumer_owner.id,
        },
    )
    .await
    .expect("public dependent should create");
    let private_consumer = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: private_consumer_owner.id,
            },
            name: format!("private-consumer-{}", Uuid::new_v4().simple()),
            description: Some("Private dependent repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: private_consumer_owner.id,
        },
    )
    .await
    .expect("private dependent should create");

    for repository in [&source, &public_consumer, &private_consumer] {
        let commit = insert_commit(
            &pool,
            repository.id,
            CreateCommit {
                oid: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
                author_user_id: Some(owner.id),
                committer_user_id: Some(owner.id),
                message: "Add package manifest".to_owned(),
                tree_oid: Some(format!("tree-{}", Uuid::new_v4().simple())),
                parent_oids: Vec::new(),
                committed_at: Utc::now(),
            },
        )
        .await
        .expect("commit should insert");
        upsert_git_ref(&pool, repository.id, "main", "branch", Some(commit.id))
            .await
            .expect("main ref should upsert");
        seed_file(
            &pool,
            repository.id,
            commit.id,
            "package.json",
            &format!(
                r#"{{"dependencies":{{"{}":"^1.2.3","vite":"5.0.0"}}}}"#,
                package_name
            ),
        )
        .await;
    }

    for repository in [&source, &public_consumer, &private_consumer] {
        repository_dependencies_for_actor_by_owner_name(
            &pool,
            if repository.id == private_consumer.id {
                private_consumer_owner.id
            } else {
                actor.id
            },
            &repository.owner_login,
            &repository.name,
            RepositoryDependencyQuery {
                query: None,
                ecosystem: None,
                relationship: None,
            },
        )
        .await
        .expect("dependency extraction should run")
        .expect("repository should exist");
    }

    let view = repository_dependents_for_actor_by_owner_name(
        &pool,
        actor.id,
        &source.owner_login,
        &source.name,
        RepositoryDependentsQuery {
            package: Some(&format!("npm:{package_name}")),
            owner: Some(&public_consumer.owner_login),
        },
    )
    .await
    .expect("dependents should load")
    .expect("source should exist");
    assert_eq!(
        view.filters.package.as_deref(),
        Some(format!("npm:{package_name}").as_str())
    );
    assert_eq!(
        view.filters.owner.as_deref(),
        Some(public_consumer.owner_login.as_str())
    );
    assert_eq!(view.summary.repository_count, 1);
    assert_eq!(view.summary.hidden_private_count, 0);
    assert!(view.summary.approximate);
    assert_eq!(view.dependents.len(), 1);
    assert_eq!(view.dependents[0].owner_login, public_consumer.owner_login);
    assert_eq!(view.dependents[0].package.name, package_name);
    assert!(view
        .packages
        .iter()
        .any(|package| package.selected && package.package.name == package_name));

    let unfiltered = repository_dependents_for_actor_by_owner_name(
        &pool,
        actor.id,
        &source.owner_login,
        &source.name,
        RepositoryDependentsQuery {
            package: Some(&package_name),
            owner: None,
        },
    )
    .await
    .expect("unfiltered dependents should load")
    .expect("source should exist");
    assert_eq!(unfiltered.summary.repository_count, 1);
    assert_eq!(unfiltered.summary.hidden_private_count, 1);
    assert!(!serde_json::to_string(&unfiltered)
        .expect("dependents should serialize")
        .contains(&private_consumer.name));

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let encoded_package =
        url::form_urlencoded::byte_serialize(format!("npm:{package_name}").as_bytes())
            .collect::<String>();
    let uri = format!(
        "/api/repos/{}/{}/network/dependents?package={}&owner={}",
        source.owner_login, source.name, encoded_package, public_consumer.owner_login
    );
    let (status, body) = get_json(app.clone(), &uri, Some(&actor_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["summary"]["repositoryCount"], 1);
    assert_eq!(
        body["dependents"][0]["href"],
        format!("/{}/{}", public_consumer.owner_login, public_consumer.name)
    );
    assert!(!body.to_string().contains(&private_consumer.name));

    let invalid_uri = format!(
        "/api/repos/{}/{}/network/dependents?owner=bad/owner",
        source.owner_login, source.name
    );
    let (invalid_status, invalid_body) =
        get_json(app.clone(), &invalid_uri, Some(&actor_cookie)).await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let private_source = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-source-{}", Uuid::new_v4().simple()),
            description: Some("Private package source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private source should create");
    grant_repository_permission(
        &pool,
        private_source.id,
        actor.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("actor should read private source");
    let private_uri = format!(
        "/api/repos/{}/{}/network/dependents",
        private_source.owner_login, private_source.name
    );
    let (private_status, private_body) = get_json(app, &private_uri, Some(&actor_cookie)).await;
    assert_eq!(private_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        private_body["error"]["code"],
        "dependency_graph_unavailable"
    );
    assert!(!private_body.to_string().contains("private-consumer"));
}
