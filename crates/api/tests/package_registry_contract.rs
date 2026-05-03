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
        INSERT INTO personal_access_tokens (user_id, name, prefix, token_hash, scopes, expires_at)
        VALUES ($1, 'Registry contract token', $2, $3, $4, $5)
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
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        if let Some(name) = name {
            builder = builder.header(name, value);
        }
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
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

fn basic_auth(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("opengithub:{token}"))
    ))
    .expect("basic auth should build")
}

fn json_body(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("response should be json")
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
