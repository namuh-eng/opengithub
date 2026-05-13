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
    let mut builder = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(
            "x-forwarded-for",
            format!("package-detail-contract-{}", Uuid::new_v4()),
        );
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
    let value = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response should be JSON: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, value)
}

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: &str,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(uri)
                .header(
                    "x-forwarded-for",
                    format!("package-detail-contract-{}", Uuid::new_v4()),
                )
                .header(header::COOKIE, cookie)
                .header(header::CONTENT_TYPE, "application/json")
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
    let value = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response should be JSON: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, value)
}

async fn patch_raw(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: &str,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(Method::PATCH)
        .uri(uri)
        .header(
            "x-forwarded-for",
            format!("package-detail-contract-{}", Uuid::new_v4()),
        )
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_owned()))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "response should be JSON: {error}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, headers, value)
}

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

struct PackageSeed<'a> {
    repository_id: Uuid,
    owner_user_id: Option<Uuid>,
    owner_organization_id: Option<Uuid>,
    created_by_user_id: Uuid,
    name: &'a str,
    package_type: &'a str,
    visibility: &'a str,
}

async fn insert_package_with_versions(pool: &PgPool, seed: PackageSeed<'_>) -> (Uuid, Uuid, Uuid) {
    let package_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO packages (
            repository_id, owner_user_id, owner_organization_id, created_by_user_id,
            name, package_type, visibility
        )
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

    let older_version = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO package_versions (
            package_id, version, digest, platform_os, platform_arch, size_bytes,
            published_by_user_id, created_at
        )
        VALUES ($1, '1.0.0', 'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'linux', 'amd64', 42, $2, now() - interval '1 day')
        RETURNING id
        "#,
    )
    .bind(package_id)
    .bind(seed.created_by_user_id)
    .fetch_one(pool)
    .await
    .expect("older version should insert");
    let latest_version = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO package_versions (
            package_id, version, digest, platform_os, platform_arch, size_bytes,
            published_by_user_id
        )
        VALUES ($1, '2.0.0', 'sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                'linux', 'arm64', 84, $2)
        RETURNING id
        "#,
    )
    .bind(package_id)
    .bind(seed.created_by_user_id)
    .fetch_one(pool)
    .await
    .expect("latest version should insert");

    sqlx::query(
        r#"
        INSERT INTO package_blobs (
            package_id, package_version_id, digest, media_type, platform_os,
            platform_arch, size_bytes, storage_key
        )
        VALUES
            ($1, $2, 'sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
             'application/vnd.oci.image.layer.v1.tar+gzip', 'linux', 'arm64', 84,
             's3://secret-package-bucket/hidden-layer'),
            ($1, $3, 'sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
             'application/vnd.oci.image.layer.v1.tar+gzip', 'linux', 'amd64', 42,
             's3://secret-package-bucket/older-layer')
        "#,
    )
    .bind(package_id)
    .bind(latest_version)
    .bind(older_version)
    .execute(pool)
    .await
    .expect("package blobs should insert");
    sqlx::query(
        "INSERT INTO package_downloads (package_id, package_version_id, download_count) VALUES ($1, $2, 17)",
    )
    .bind(package_id)
    .bind(latest_version)
    .execute(pool)
    .await
    .expect("downloads should insert");
    (package_id, older_version, latest_version)
}

async fn insert_readme(pool: &PgPool, repository_id: Uuid, body: &str) {
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (repository_id, oid, message)
        VALUES ($1, $2, 'Add package README')
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(format!("oid-{}", Uuid::new_v4().simple()))
    .fetch_one(pool)
    .await
    .expect("commit should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, 'refs/heads/main', 'branch', $2)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("ref should upsert");
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, 'README.md', $3, $4, $5)
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(body)
    .bind(format!("blob-{}", Uuid::new_v4().simple()))
    .bind(body.len() as i64)
    .execute(pool)
    .await
    .expect("readme should insert");
}

#[tokio::test]
async fn public_package_detail_returns_versions_blobs_install_commands_and_readme() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkgdetail{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &marker).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("package detail source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    insert_readme(&pool, repo.id, "# Package README\n\nInstall it safely.").await;
    insert_package_with_versions(
        &pool,
        PackageSeed {
            repository_id: repo.id,
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            created_by_user_id: owner.id,
            name: &format!("{marker}-image"),
            package_type: "container",
            visibility: "public",
        },
    )
    .await;

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/packages/container/{marker}-image"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["name"], format!("{marker}-image"));
    assert_eq!(body["selectedVersion"]["version"], "2.0.0");
    assert_eq!(body["versions"].as_array().expect("versions").len(), 2);
    assert_eq!(body["blobs"][0]["platformArch"], "arm64");
    assert_eq!(body["downloadCount"], 17);
    assert_eq!(body["about"]["source"], "linked_repository_readme");
    assert!(body["about"]["html"]
        .as_str()
        .expect("html")
        .contains("Package README"));
    assert!(body["installCommands"][0]["command"]
        .as_str()
        .expect("command")
        .contains("docker pull ghcr.io/"));
    assert_eq!(body["admin"]["canAdmin"], false);
    assert!(!body.to_string().contains("secret-package-bucket"));

    let (status, _, selected_body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/packages/container/{marker}-image?version=1.0.0"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(selected_body["selectedVersion"]["version"], "1.0.0");
    assert_eq!(selected_body["blobs"][0]["platformArch"], "amd64");

    let (status, _, digest_body) = get_json(
        app.clone(),
        &format!(
            "/api/users/{marker}/packages/container/{marker}-image?version=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(digest_body["selectedVersion"]["version"], "1.0.0");

    let before_downloads: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(download_count), 0)::bigint FROM package_downloads WHERE package_id = $1",
    )
    .bind(body["id"].as_str().expect("package id").parse::<Uuid>().expect("uuid"))
    .fetch_one(&pool)
    .await
    .expect("downloads should count");
    let (status, _, download_body) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/packages/container/{marker}-image/download?version=1.0.0"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(download_body["version"], "1.0.0");
    assert!(download_body["command"]
        .as_str()
        .expect("command")
        .contains("@sha256:aaaaaaaa"));
    assert_eq!(download_body["storageAvailable"], false);
    assert_eq!(download_body["downloadCount"], before_downloads + 1);

    let (status, _, invalid_body) = get_json(
        app,
        &format!("/api/users/{marker}/packages/container/{marker}-image/download?version=sha256:not-real"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn private_detail_redacts_until_package_or_linked_repository_permission_grants_read() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package detail permission scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkgperm{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let package_reader = create_user(&pool, &format!("{marker}-package-reader")).await;
    let repo_reader = create_user(&pool, &format!("{marker}-repo-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let package_reader_cookie = cookie_header(&pool, &config, &package_reader).await;
    let repo_reader_cookie = cookie_header(&pool, &config, &repo_reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("private package detail source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    let (package_id, _, _) = insert_package_with_versions(
        &pool,
        PackageSeed {
            repository_id: repo.id,
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            created_by_user_id: owner.id,
            name: &format!("{marker}-private"),
            package_type: "npm",
            visibility: "private",
        },
    )
    .await;

    let path = format!("/api/users/{marker}-owner/packages/npm/{marker}-private");
    let (status, _, public_body) = get_json(app.clone(), &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!public_body.to_string().contains(marker.as_str()));

    let (status, headers, malformed_body) =
        patch_raw(app.clone(), &format!("{path}/settings"), None, "{bad-json").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_json(&headers);
    assert_eq!(malformed_body["error"]["code"], "invalid_json");
    assert!(!malformed_body.to_string().contains(marker.as_str()));

    let (status, _, download_body) =
        get_json(app.clone(), &format!("{path}/download?version=2.0.0"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!download_body.to_string().contains(marker.as_str()));

    sqlx::query(
        "INSERT INTO package_permissions (package_id, user_id, role) VALUES ($1, $2, 'read')",
    )
    .bind(package_id)
    .bind(package_reader.id)
    .execute(&pool)
    .await
    .expect("package permission should insert");
    let (status, _, package_reader_body) =
        get_json(app.clone(), &path, Some(&package_reader_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(package_reader_body["admin"]["canAdmin"], false);

    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role) VALUES ($1, $2, 'read')",
    )
    .bind(repo.id)
    .bind(repo_reader.id)
    .execute(&pool)
    .await
    .expect("repo permission should insert");
    let (status, _, repo_reader_body) =
        get_json(app.clone(), &path, Some(&repo_reader_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        repo_reader_body["linkedRepository"]["fullName"],
        format!("{marker}-owner/{marker}-repo")
    );

    let (status, _, owner_body) = get_json(app.clone(), &path, Some(&owner_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owner_body["admin"]["canAdmin"], true);
    assert_eq!(
        owner_body["admin"]["settingsHref"],
        format!("/{marker}-owner/npm/{marker}-private/settings")
    );

    let (status, _, settings_public_body) =
        get_json(app.clone(), &format!("{path}/settings"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!settings_public_body.to_string().contains(marker.as_str()));

    let (status, _, settings_reader_body) = get_json(
        app.clone(),
        &format!("{path}/settings"),
        Some(&package_reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(!settings_reader_body.to_string().contains(marker.as_str()));

    sqlx::query(
        r#"
        INSERT INTO package_permissions (package_id, user_id, role)
        VALUES ($1, $2, 'admin')
        ON CONFLICT (package_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(package_id)
    .bind(package_reader.id)
    .execute(&pool)
    .await
    .expect("package admin permission should insert");
    let (status, headers, settings_body) = get_json(
        app,
        &format!("{path}/settings"),
        Some(&package_reader_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(
        settings_body["package"]["name"],
        format!("{marker}-private")
    );
    assert_eq!(settings_body["package"]["visibility"], "private");
    assert_eq!(
        settings_body["explicitPermissions"][0]["login"],
        format!("{marker}-package-reader")
    );
    assert_eq!(settings_body["explicitPermissions"][0]["role"], "admin");
    assert_eq!(
        settings_body["linkedRepositories"][0]["fullName"],
        format!("{marker}-owner/{marker}-repo")
    );
    assert!(settings_body["inheritedRepositoryAccess"]
        .as_array()
        .expect("inherited repository access")
        .iter()
        .any(
            |access| access["login"] == format!("{marker}-repo-reader") && access["role"] == "read"
        ));
    assert!(settings_body["registryWriteCapabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .any(|capability| capability["enabled"] == true
            && capability["reason"]
                .as_str()
                .expect("reason")
                .contains("audit")));
    assert!(!settings_body.to_string().contains("secret-package-bucket"));
}

#[tokio::test]
async fn package_settings_admin_mutations_soft_delete_and_audit_without_leaking_storage() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package settings mutation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkgadmin{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let grantee = create_user(&pool, &format!("{marker}-grantee")).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let source_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("package source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    let linked_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-linked"),
            description: Some("linked source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("linked repo should create");
    let (_package_id, _older_version_id, latest_version_id) = insert_package_with_versions(
        &pool,
        PackageSeed {
            repository_id: source_repo.id,
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            created_by_user_id: owner.id,
            name: &format!("{marker}-image"),
            package_type: "container",
            visibility: "private",
        },
    )
    .await;
    let path = format!(
        "/api/users/{}-owner/packages/container/{}-image/settings",
        marker, marker
    );

    let (status, headers, visibility_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({ "action": "updateVisibility", "visibility": "public" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(visibility_body["package"]["visibility"], "public");

    let (status, _, grant_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({
            "action": "grantAccess",
            "username": format!("{marker}-grantee"),
            "role": "write"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(grant_body["explicitPermissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .any(|permission| permission["userId"] == grantee.id.to_string()
            && permission["role"] == "write"));

    let (status, _, link_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({
            "action": "linkRepository",
            "owner": format!("{marker}-owner"),
            "repo": format!("{marker}-linked")
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(link_body["linkedRepositories"]
        .as_array()
        .expect("linked repos")
        .iter()
        .any(|repo| repo["id"] == linked_repo.id.to_string()));

    let (status, _, unlink_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({
            "action": "unlinkRepository",
            "repositoryId": linked_repo.id.to_string()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{unlink_body}");
    assert!(!unlink_body["linkedRepositories"]
        .as_array()
        .expect("linked repos")
        .iter()
        .any(|repo| repo["id"] == linked_repo.id.to_string()));

    let (status, _, revoke_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({
            "action": "revokeAccess",
            "userId": grantee.id.to_string()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!revoke_body["explicitPermissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .any(|permission| permission["userId"] == grantee.id.to_string()));

    let (status, _, deleted_version_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({ "action": "deleteVersion", "versionId": latest_version_id.to_string() }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{deleted_version_body}");
    assert_ne!(deleted_version_body["package"]["latestVersion"], "2.0.0");

    let (status, _, restored_version_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({ "action": "restoreVersion", "versionId": latest_version_id.to_string() }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored_version_body["package"]["latestVersion"], "2.0.0");

    let (status, _, package_deleted_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({ "action": "deletePackage" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(package_deleted_body["package"]["deletedAt"].is_string());

    let (status, _, restored_body) = patch_json(
        app.clone(),
        &path,
        &owner_cookie,
        json!({ "action": "restorePackage" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(restored_body["package"]["deletedAt"].is_null());
    assert!(!restored_body.to_string().contains("secret-package-bucket"));

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM package_registry_audit_events WHERE event_type LIKE 'settings.%'",
    )
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 8);
}

#[tokio::test]
async fn organization_internal_detail_is_visible_to_members_and_admin_state_to_owners() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping org package detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("pkgorgdetail{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Package Detail Org".to_owned(),
            description: Some("package detail org".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("org should create");
    sqlx::query("INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')")
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
    insert_package_with_versions(
        &pool,
        PackageSeed {
            repository_id: repo.id,
            owner_user_id: None,
            owner_organization_id: Some(org.id),
            created_by_user_id: owner.id,
            name: &format!("{marker}-gem"),
            package_type: "rubygems",
            visibility: "internal",
        },
    )
    .await;

    let path = format!("/api/orgs/{marker}/packages/rubygems/{marker}-gem");
    let (status, _, public_body) = get_json(app.clone(), &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!public_body.to_string().contains("gem"));

    let (status, _, member_body) = get_json(app.clone(), &path, Some(&member_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member_body["owner"]["kind"], "organization");
    assert_eq!(member_body["admin"]["canAdmin"], false);

    let (status, _, owner_body) = get_json(app, &path, Some(&owner_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owner_body["admin"]["canAdmin"], true);
    assert_eq!(
        owner_body["admin"]["settingsHref"],
        format!("/orgs/{marker}/packages/rubygems/{marker}-gem/settings")
    );
}
