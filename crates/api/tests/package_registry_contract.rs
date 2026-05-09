use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
};
use base64::Engine as _;
use chrono::{Duration, Utc};
use opengithub_api::{
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        tokens::hash_personal_access_token,
    },
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::sync::LazyLock;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
static REGISTRY_STORAGE_ENV_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

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
        None,
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

async fn create_pat(pool: &PgPool, user_id: Uuid, scopes: &[&str]) -> String {
    let token = format!("oghp_{}_registry", Uuid::new_v4().simple());
    let prefix = token
        .split("_registry")
        .next()
        .expect("prefix marker")
        .to_owned();
    sqlx::query(
        r#"
        INSERT INTO personal_access_tokens (
            user_id, name, prefix, token_hash, scopes, expires_at, resource_owner_user_id
        )
        VALUES ($1, 'Registry contract token', $2, $3, $4, $5, $1)
        "#,
    )
    .bind(user_id)
    .bind(prefix)
    .bind(hash_personal_access_token(&token))
    .bind(
        scopes
            .iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<_>>(),
    )
    .bind(Utc::now() + Duration::hours(1))
    .execute(pool)
    .await
    .expect("PAT should insert");
    token
}

async fn create_workflow_token(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_run_id: Uuid,
    workflow_job_id: Uuid,
    actor_user_id: Uuid,
    scopes: &[&str],
) -> String {
    let token = format!("ogwt_{}_registry", Uuid::new_v4().simple());
    sqlx::query(
        r#"
        INSERT INTO package_workflow_tokens (
            token_hash, repository_id, workflow_run_id, workflow_job_id,
            actor_user_id, scopes, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(hash_personal_access_token(&token))
    .bind(repository_id)
    .bind(workflow_run_id)
    .bind(workflow_job_id)
    .bind(actor_user_id)
    .bind(
        scopes
            .iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<_>>(),
    )
    .bind(Utc::now() + Duration::hours(1))
    .execute(pool)
    .await
    .expect("workflow token should insert");
    token
}

async fn create_workflow_run_fixture(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    marker: &str,
) -> (Uuid, Uuid, Uuid) {
    let workflow_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO actions_workflows (repository_id, name, path, state, trigger_events)
        VALUES ($1, $2, '.github/workflows/publish.yml', 'active', ARRAY['push']::text[])
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(format!("{marker} publish"))
    .fetch_one(pool)
    .await
    .expect("workflow should insert");
    let run_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
            head_branch, head_sha, event, started_at, completed_at
        )
        VALUES ($1, $2, $3, 1, 'completed', 'success', 'main', $4, 'push', now(), now())
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(workflow_id)
    .bind(actor_user_id)
    .bind(format!(
        "{:0<40}",
        marker.chars().take(12).collect::<String>()
    ))
    .fetch_one(pool)
    .await
    .expect("workflow run should insert");
    let job_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO workflow_jobs (run_id, name, status, conclusion, runner_label)
        VALUES ($1, 'publish-container', 'completed', 'success', 'ubuntu-latest')
        RETURNING id
        "#,
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .expect("workflow job should insert");
    (workflow_id, run_id, job_id)
}

async fn insert_container_package(
    pool: &PgPool,
    owner: &User,
    name: &str,
    visibility: &str,
) -> (Uuid, Uuid) {
    let repo = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{name}-repo"),
            description: Some("registry source".to_owned()),
            visibility: if visibility == "public" {
                RepositoryVisibility::Public
            } else {
                RepositoryVisibility::Private
            },
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    let package_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO packages (
            repository_id, owner_user_id, created_by_user_id,
            name, package_type, visibility
        )
        VALUES ($1, $2, $2, $3, 'container', $4)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(owner.id)
    .bind(name)
    .bind(visibility)
    .fetch_one(pool)
    .await
    .expect("package should insert");
    let version_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO package_versions (
            package_id, version, digest, manifest, manifest_media_type,
            config_digest, manifest_size_bytes, size_bytes, published_by_user_id
        )
        VALUES (
            $1,
            'latest',
            'sha256:1111111111111111111111111111111111111111111111111111111111111111',
            $2,
            'application/vnd.oci.image.manifest.v1+json',
            'sha256:2222222222222222222222222222222222222222222222222222222222222222',
            512,
            1024,
            $3
        )
        RETURNING id
        "#,
    )
    .bind(package_id)
    .bind(json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
            "size": 42
        },
        "layers": [{
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "digest": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
            "size": 84
        }]
    }))
    .bind(owner.id)
    .fetch_one(pool)
    .await
    .expect("package version should insert");
    sqlx::query(
        r#"
        INSERT INTO package_blobs (
            package_id, package_version_id, digest, media_type, size_bytes, storage_key
        )
        VALUES (
            $1, $2,
            'sha256:3333333333333333333333333333333333333333333333333333333333333333',
            'application/vnd.oci.image.layer.v1.tar+gzip',
            84,
            's3://secret-registry-bucket/private-layer'
        )
        "#,
    )
    .bind(package_id)
    .bind(version_id)
    .execute(pool)
    .await
    .expect("blob should insert");
    (package_id, version_id)
}

async fn request(
    app: axum::Router,
    method: Method,
    uri: &str,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    request_with_body(app, method, uri, headers, Body::empty()).await
}

async fn request_with_body(
    app: axum::Router,
    method: Method,
    uri: &str,
    headers: HeaderMap,
    body: Body,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut builder = Request::builder().method(method).uri(uri).header(
        "x-forwarded-for",
        format!("198.51.100.{}", Uuid::new_v4().as_u128() % 250 + 1),
    );
    for (name, value) in headers {
        if let Some(name) = name {
            builder = builder.header(name, value);
        }
    }
    let response = app
        .oneshot(builder.body(body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read")
        .to_vec();
    (status, headers, bytes)
}

fn sha256_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    format!("sha256:{hex}")
}

fn basic_auth(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("opengithub:{token}"))
    ))
    .expect("basic auth should build")
}

#[tokio::test]
async fn registry_blob_upload_manifest_push_tag_list_and_pull_record_downloads() {
    let _env_guard = REGISTRY_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package registry contract; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir = std::env::temp_dir().join(format!("opengithub-registry-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_PACKAGE_REGISTRY_STORAGE_DIR", &storage_dir);

    let marker = format!("push{}", Uuid::new_v4().simple());
    let namespace = format!("{marker}-owner");
    let owner = create_user(&pool, &namespace).await;
    let image = format!("{marker}-image");
    let (package_id, _) = insert_container_package(&pool, &owner, &image, "private").await;
    let write_token = create_pat(&pool, owner.id, &["packages:write"]).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());

    let upload_blob = |app: axum::Router,
                       token: String,
                       namespace: String,
                       image: String,
                       bytes: Vec<u8>| async move {
        let digest = sha256_digest(&bytes);
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, headers, _) = request(
            app.clone(),
            Method::POST,
            &format!("/v2/{namespace}/{image}/blobs/uploads/"),
            headers,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let upload_location = headers
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("upload location")
            .to_owned();

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, headers, _) = request_with_body(
            app.clone(),
            Method::PATCH,
            &upload_location,
            headers,
            Body::from(bytes),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(headers.contains_key(header::RANGE));

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, headers, body) = request(
            app,
            Method::PUT,
            &format!("{upload_location}?digest={digest}"),
            headers,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            headers
                .get("docker-content-digest")
                .and_then(|value| value.to_str().ok())
                .expect("blob digest"),
            digest
        );
        assert!(!String::from_utf8(body)
            .expect("utf8 body")
            .contains("storage"));
        digest
    };

    let config_bytes = br#"{"architecture":"amd64","os":"linux"}"#.to_vec();
    let layer_bytes = b"container layer bytes".to_vec();
    let config_digest = upload_blob(
        app.clone(),
        write_token.clone(),
        namespace.clone(),
        image.clone(),
        config_bytes.clone(),
    )
    .await;
    let layer_digest = upload_blob(
        app.clone(),
        write_token.clone(),
        namespace.clone(),
        image.clone(),
        layer_bytes.clone(),
    )
    .await;
    let manifest = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": config_digest,
            "size": config_bytes.len()
        },
        "layers": [{
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "digest": layer_digest,
            "size": layer_bytes.len()
        }]
    });
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.oci.image.manifest.v1+json"),
    );
    let (status, headers, _) = request_with_body(
        app.clone(),
        Method::PUT,
        &format!("/v2/{namespace}/{image}/manifests/v2"),
        headers,
        Body::from(serde_json::to_vec(&manifest).expect("manifest json")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let manifest_digest = headers
        .get("docker-content-digest")
        .and_then(|value| value.to_str().ok())
        .expect("manifest digest")
        .to_owned();
    assert!(manifest_digest.starts_with("sha256:"));

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{namespace}/{image}/tags/list"),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tags = json_body(&body);
    assert!(tags["tags"]
        .as_array()
        .expect("tags array")
        .iter()
        .any(|tag| tag == "v2"));

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    let (status, headers, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{namespace}/{image}/blobs/{layer_digest}"),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, layer_bytes);
    assert_eq!(
        headers
            .get("docker-content-digest")
            .and_then(|value| value.to_str().ok())
            .expect("pulled digest"),
        layer_digest
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{namespace}/{image}/manifests/{manifest_digest}"),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json_body(&body)["schemaVersion"], 2);

    let download_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(download_count), 0)::bigint FROM package_downloads WHERE package_id = $1",
    )
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("download count should load");
    assert!(download_count >= 2);

    let manifest_version_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM package_versions WHERE package_id = $1 AND version = 'v2' AND digest = $2",
    )
    .bind(package_id)
    .bind(&manifest_digest)
    .fetch_one(&pool)
    .await
    .expect("manifest version count should load");
    assert_eq!(manifest_version_count, 1);

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    let (status, headers, _) = request(
        app.clone(),
        Method::DELETE,
        &format!("/v2/{namespace}/{image}/manifests/v2"),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(
        headers
            .get("docker-content-digest")
            .and_then(|value| value.to_str().ok())
            .expect("deleted digest"),
        manifest_digest
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&write_token));
    let (status, _, _) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{namespace}/{image}/manifests/v2"),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let deleted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM package_versions WHERE package_id = $1 AND version = 'v2' AND deleted_at IS NOT NULL",
    )
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("deleted manifest count should load");
    assert_eq!(deleted_count, 1);

    let storage_key_leaks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM package_registry_audit_events WHERE package_id = $1 AND COALESCE(reference, '') LIKE '%opengithub-registry%'",
    )
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("audit leak count should load");
    assert_eq!(storage_key_leaks, 0);
}

fn json_body(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("response should be json")
}

#[tokio::test]
async fn registry_workflow_token_publish_links_repository_and_enqueues_package_events() {
    let _env_guard = REGISTRY_STORAGE_ENV_LOCK.lock().await;
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package registry contract; set TEST_DATABASE_URL");
        return;
    };
    let storage_dir = std::env::temp_dir().join(format!("opengithub-registry-{}", Uuid::new_v4()));
    std::env::set_var("OPENGITHUB_PACKAGE_REGISTRY_STORAGE_DIR", &storage_dir);

    let marker = format!("wf{}", Uuid::new_v4().simple());
    let namespace = format!("{marker}-owner");
    let owner = create_user(&pool, &namespace).await;
    let image = format!("{marker}-image");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: image.clone(),
            description: Some("workflow package source".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");
    sqlx::query(
        "INSERT INTO webhooks (repository_id, url, events, created_by_user_id) VALUES ($1, 'https://example.test/package-hook', ARRAY['package']::text[], $2)",
    )
    .bind(repo.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("package webhook should insert");
    let (_, run_id, job_id) = create_workflow_run_fixture(&pool, repo.id, owner.id, &marker).await;
    let workflow_token = create_workflow_token(
        &pool,
        repo.id,
        run_id,
        job_id,
        owner.id,
        &["packages:write"],
    )
    .await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&workflow_token));
    let (status, _, body) = request(app.clone(), Method::GET, "/v2/token", headers).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json_body(&body)["token"], workflow_token);
    assert_eq!(json_body(&body)["expiresIn"], 900);

    let upload_blob = |app: axum::Router,
                       token: String,
                       namespace: String,
                       image: String,
                       bytes: Vec<u8>| async move {
        let digest = sha256_digest(&bytes);
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, headers, _) = request(
            app.clone(),
            Method::POST,
            &format!("/v2/{namespace}/{image}/blobs/uploads/"),
            headers,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let upload_location = headers
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("upload location")
            .to_owned();

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, _, _) = request_with_body(
            app.clone(),
            Method::PATCH,
            &upload_location,
            headers,
            Body::from(bytes),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_auth(&token));
        let (status, headers, _) = request(
            app,
            Method::PUT,
            &format!("{upload_location}?digest={digest}"),
            headers,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            headers
                .get("docker-content-digest")
                .and_then(|value| value.to_str().ok())
                .expect("blob digest"),
            digest
        );
        digest
    };

    let config = json!({
        "architecture": "amd64",
        "os": "linux",
        "config": {
            "Labels": {
                "org.opencontainers.image.source": format!("https://opengithub.namuh.co/{namespace}/{image}"),
                "org.opencontainers.image.description": "Published from Actions",
                "org.opencontainers.image.revision": "abc123"
            }
        }
    });
    let config_bytes = serde_json::to_vec(&config).expect("config json");
    let layer_bytes = b"workflow layer bytes".to_vec();
    let config_digest = upload_blob(
        app.clone(),
        workflow_token.clone(),
        namespace.clone(),
        image.clone(),
        config_bytes.clone(),
    )
    .await;
    let layer_digest = upload_blob(
        app.clone(),
        workflow_token.clone(),
        namespace.clone(),
        image.clone(),
        layer_bytes.clone(),
    )
    .await;
    let manifest = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": config_digest,
            "size": config_bytes.len()
        },
        "layers": [{
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "digest": layer_digest,
            "size": layer_bytes.len()
        }]
    });
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&workflow_token));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.oci.image.manifest.v1+json"),
    );
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("opengithub-actions/1.0"),
    );
    let (status, headers, _) = request_with_body(
        app.clone(),
        Method::PUT,
        &format!("/v2/{namespace}/{image}/manifests/actions-build"),
        headers,
        Body::from(serde_json::to_vec(&manifest).expect("manifest json")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let manifest_digest = headers
        .get("docker-content-digest")
        .and_then(|value| value.to_str().ok())
        .expect("manifest digest")
        .to_owned();

    let row = sqlx::query(
        r#"
        SELECT packages.id AS package_id,
               package_versions.id AS version_id,
               package_versions.workflow_run_id,
               package_versions.workflow_job_id,
               package_versions.source_repository_id,
               package_versions.oci_annotations,
               packages.visibility
        FROM packages
        JOIN package_versions ON package_versions.package_id = packages.id
        WHERE packages.repository_id = $1 AND packages.name = $2 AND package_versions.version = 'actions-build'
        "#,
    )
    .bind(repo.id)
    .bind(&image)
    .fetch_one(&pool)
    .await
    .expect("workflow-published package version should load");
    let package_id: Uuid = row.get("package_id");
    assert_eq!(row.get::<Option<Uuid>, _>("workflow_run_id"), Some(run_id));
    assert_eq!(row.get::<Option<Uuid>, _>("workflow_job_id"), Some(job_id));
    assert_eq!(
        row.get::<Option<Uuid>, _>("source_repository_id"),
        Some(repo.id)
    );
    assert_eq!(row.get::<String, _>("visibility"), "private");
    assert_eq!(
        row.get::<Value, _>("oci_annotations")["org.opencontainers.image.description"],
        "Published from Actions"
    );

    let linked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM package_repository_links WHERE package_id = $1 AND repository_id = $2 AND link_type = 'workflow'",
    )
    .bind(package_id)
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("package link count should load");
    assert_eq!(linked_count, 1);

    let package_webhooks: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM webhook_deliveries
        JOIN webhooks ON webhooks.id = webhook_deliveries.webhook_id
        WHERE webhooks.repository_id = $1
          AND webhook_deliveries.event = 'package'
          AND webhook_deliveries.payload->'payload'->>'digest' = $2
        "#,
    )
    .bind(repo.id)
    .bind(&manifest_digest)
    .fetch_one(&pool)
    .await
    .expect("package webhook count should load");
    assert_eq!(package_webhooks, 1);

    let workflow_audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM package_registry_audit_events
        WHERE package_id = $1
          AND actor_kind = 'workflow'
          AND workflow_run_id = $2
          AND event_type = 'manifest.write'
          AND metadata->>'sourceRepositoryId' = $3
        "#,
    )
    .bind(package_id)
    .bind(run_id)
    .bind(repo.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("workflow audit count should load");
    assert_eq!(workflow_audit_count, 1);
}

#[tokio::test]
async fn registry_v2_challenge_and_manifest_reads_enforce_visibility_and_pat_scopes() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping package registry contract; set TEST_DATABASE_URL");
        return;
    };

    let marker = format!("reg{}", Uuid::new_v4().simple());
    let namespace = format!("{marker}-owner");
    let owner = create_user(&pool, &namespace).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let public_image = format!("{marker}-public");
    let private_image = format!("{marker}-private");
    let (public_package_id, public_version_id) =
        insert_container_package(&pool, &owner, &public_image, "public").await;
    let (private_package_id, _) =
        insert_container_package(&pool, &owner, &private_image, "private").await;
    sqlx::query(
        "INSERT INTO package_permissions (package_id, user_id, role) VALUES ($1, $2, 'read')",
    )
    .bind(private_package_id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("package permission should insert");
    let read_token = create_pat(&pool, reader.id, &["packages:read"]).await;
    let repo_only_token = create_pat(&pool, reader.id, &["repo:read"]).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), app_config());

    let (status, headers, body) = request(app.clone(), Method::GET, "/v2/", HeaderMap::new()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(headers
        .get(header::WWW_AUTHENTICATE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("opengithub-registry")));
    assert_eq!(json_body(&body)["errors"][0]["code"], "UNAUTHORIZED");

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&read_token));
    let (status, _, body) = request(app.clone(), Method::GET, "/v2/token", headers).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json_body(&body)["token"], read_token);
    assert_eq!(json_body(&body)["expiresIn"], 900);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.oci.image.manifest.v1+json"),
    );
    let (status, headers, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, public_image),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type"),
        "application/vnd.oci.image.manifest.v1+json"
    );
    assert_eq!(
        headers
            .get("docker-content-digest")
            .and_then(|value| value.to_str().ok())
            .expect("digest"),
        "sha256:1111111111111111111111111111111111111111111111111111111111111111"
    );
    let body_text = String::from_utf8(body.clone()).expect("utf8 body");
    assert!(body_text.contains("schemaVersion"));
    assert!(!body_text.contains("secret-registry-bucket"));
    assert_eq!(json_body(&body)["schemaVersion"], 2);

    let (status, _, _) = request(
        app.clone(),
        Method::HEAD,
        &format!(
            "/v2/{}/{}/manifests/sha256:1111111111111111111111111111111111111111111111111111111111111111",
            namespace, public_image
        ),
        HeaderMap::new(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, headers, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, private_image),
        HeaderMap::new(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(headers.contains_key(header::WWW_AUTHENTICATE));
    assert!(!String::from_utf8(body.clone())
        .expect("utf8 body")
        .contains(&private_image));

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth("not-a-real-token"));
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, private_image),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        json_body(&body)["errors"][0]["message"],
        "invalid registry token"
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, basic_auth(&repo_only_token));
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, private_image),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(json_body(&body)["errors"][0]["code"], "DENIED");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {read_token}")).expect("bearer header"),
    );
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.docker.distribution.manifest.v2+json"),
    );
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, private_image),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        json_body(&body)["mediaType"],
        "application/vnd.oci.image.manifest.v1+json"
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::ACCEPT, HeaderValue::from_static("text/plain"));
    let (status, _, body) = request(
        app.clone(),
        Method::GET,
        &format!("/v2/{}/{}/manifests/latest", namespace, public_image),
        headers,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_ACCEPTABLE);
    assert_eq!(json_body(&body)["errors"][0]["code"], "MANIFEST_INVALID");

    let (status, _, body) = request(
        app,
        Method::GET,
        &format!(
            "/v2/{}/{}/manifests/sha256:not-real",
            namespace, public_image
        ),
        HeaderMap::new(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json_body(&body)["errors"][0]["code"], "NAME_INVALID");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM package_registry_audit_events WHERE package_id = $1 AND package_version_id = $2",
    )
    .bind(public_package_id)
    .bind(public_version_id)
    .fetch_one(&pool)
    .await
    .expect("audit rows should count");
    assert!(audit_count >= 2);
}
