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
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
        },
    },
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
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

#[tokio::test]
async fn repository_releases_read_contract_filters_privacy_and_exposes_tags_assets() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository Releases scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("rel{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outsider = create_user(&pool, &format!("{marker}-outside")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-public"),
            description: Some("Release contract public repository".to_owned()),
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
            name: format!("{marker}-private"),
            description: Some("Release contract private repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    grant_repository_permission(
        &pool,
        private_repo.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader grant should persist");

    let public_v1_commit = seed_commit_and_tag(&pool, public_repo.id, &owner, "v1.0.0", 10).await;
    let public_v2_commit = seed_commit_and_tag(&pool, public_repo.id, &owner, "v2.0.0", 5).await;
    let public_beta_commit =
        seed_commit_and_tag(&pool, public_repo.id, &owner, "v2.1.0-beta.1", 2).await;
    let release_v1 = seed_release(
        &pool,
        public_repo.id,
        &owner,
        "v1.0.0",
        public_v1_commit,
        "First release",
        false,
        false,
        false,
        10,
    )
    .await;
    let release_v2 = seed_release(
        &pool,
        public_repo.id,
        &owner,
        "v2.0.0",
        public_v2_commit,
        "Stable release",
        false,
        false,
        true,
        5,
    )
    .await;
    let release_beta = seed_release(
        &pool,
        public_repo.id,
        &owner,
        "v2.1.0-beta.1",
        public_beta_commit,
        "Beta release",
        false,
        true,
        false,
        2,
    )
    .await;
    let _draft = seed_release(
        &pool,
        public_repo.id,
        &owner,
        "v3.0.0-draft",
        public_beta_commit,
        "Draft should hide",
        true,
        false,
        false,
        1,
    )
    .await;
    seed_asset(&pool, public_repo.id, release_v2, &owner).await;
    seed_reaction(&pool, public_repo.id, release_v2, &reader, "rocket").await;
    seed_reaction(&pool, public_repo.id, release_v2, &owner, "heart").await;

    let private_commit = seed_commit_and_tag(&pool, private_repo.id, &owner, "v9.0.0", 1).await;
    seed_release(
        &pool,
        private_repo.id,
        &owner,
        "v9.0.0",
        private_commit,
        "Private release",
        false,
        false,
        true,
        1,
    )
    .await;

    let public_uri = format!("/api/repos/{}/{}/releases", owner.email, public_repo.name);
    let (list_status, _, list_body) =
        send_json(app.clone(), Method::GET, &public_uri, None, None).await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_body["total"], 3);
    assert_eq!(list_body["items"][0]["tagName"], "v2.1.0-beta.1");
    assert_eq!(list_body["items"][0]["prerelease"], true);
    assert!(!list_body.to_string().contains("Draft should hide"));

    let latest_uri = format!("{public_uri}/latest");
    let (latest_status, _, latest_body) = send_json(
        app.clone(),
        Method::GET,
        &latest_uri,
        Some(&reader_cookie),
        None,
    )
    .await;
    assert_eq!(latest_status, StatusCode::OK);
    assert_eq!(latest_body["tagName"], "v2.0.0");
    assert_eq!(latest_body["latest"], true);
    assert_eq!(latest_body["assets"][0]["name"], "opengithub.tar.gz");
    assert_eq!(latest_body["reactions"]["totalCount"], 2);
    assert_eq!(latest_body["reactions"]["viewerReaction"], "rocket");
    assert!(latest_body["bodyHtml"]
        .as_str()
        .unwrap()
        .contains("<strong>safe</strong>"));
    assert!(!latest_body["bodyHtml"]
        .as_str()
        .unwrap()
        .contains("<script"));

    let by_id_uri = format!("{public_uri}/{release_v2}");
    let (by_id_status, _, by_id_body) =
        send_json(app.clone(), Method::GET, &by_id_uri, None, None).await;
    assert_eq!(by_id_status, StatusCode::OK);
    assert_eq!(by_id_body["id"], release_v2.to_string());

    let by_tag_uri = format!("{public_uri}/tag/v1.0.0");
    let (by_tag_status, _, by_tag_body) =
        send_json(app.clone(), Method::GET, &by_tag_uri, None, None).await;
    assert_eq!(by_tag_status, StatusCode::OK);
    assert_eq!(by_tag_body["id"], release_v1.to_string());

    let (tags_status, _, tags_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{public_uri}/tags"),
        None,
        None,
    )
    .await;
    assert_eq!(tags_status, StatusCode::OK);
    assert_eq!(tags_body["total"], 3);
    assert!(tags_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tag| tag["name"] == "v2.0.0" && tag["releaseId"] == release_v2.to_string()));

    let (archive_status, _, archive_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{public_uri}/zipball/v2.0.0"),
        None,
        None,
    )
    .await;
    assert_eq!(archive_status, StatusCode::OK);
    assert_eq!(archive_body["format"], "zipball");
    assert_eq!(archive_body["tagName"], "v2.0.0");

    let private_uri = format!("/api/repos/{}/{}/releases", owner.email, private_repo.name);
    let (anonymous_private_status, _, anonymous_private_body) =
        send_json(app.clone(), Method::GET, &private_uri, None, None).await;
    assert_eq!(anonymous_private_status, StatusCode::FORBIDDEN);
    assert!(!anonymous_private_body
        .to_string()
        .contains("Private release"));

    let (outside_private_status, _, outside_private_body) = send_json(
        app.clone(),
        Method::GET,
        &private_uri,
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(outside_private_status, StatusCode::FORBIDDEN);
    assert!(!outside_private_body
        .to_string()
        .contains(&private_repo.name));

    let (reader_private_status, _, reader_private_body) = send_json(
        app.clone(),
        Method::GET,
        &private_uri,
        Some(&reader_cookie),
        None,
    )
    .await;
    assert_eq!(reader_private_status, StatusCode::OK);
    assert_eq!(reader_private_body["items"][0]["tagName"], "v9.0.0");

    let (owner_list_status, _, owner_list_body) = send_json(
        app.clone(),
        Method::GET,
        &public_uri,
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(owner_list_status, StatusCode::OK);
    assert_eq!(owner_list_body["total"], 4);
    assert!(owner_list_body.to_string().contains("Draft should hide"));

    let deleted_visible = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM releases WHERE id = $1 AND deleted_at IS NOT NULL)",
    )
    .bind(release_beta)
    .fetch_one(&pool)
    .await
    .expect("deleted check should run");
    assert!(!deleted_visible);
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

async fn seed_commit_and_tag(
    pool: &PgPool,
    repository_id: Uuid,
    author: &User,
    tag: &str,
    days_ago: i64,
) -> Uuid {
    let oid = format!("{:040x}", Uuid::new_v4().as_u128());
    let row = sqlx::query(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, committed_at)
        VALUES ($1, $2, $3, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(&oid)
    .bind(author.id)
    .bind(format!("Release {tag}"))
    .bind(Utc::now() - Duration::days(days_ago))
    .fetch_one(pool)
    .await
    .expect("commit should persist");
    let commit_id = row.get("id");
    sqlx::query(
        "INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id) VALUES ($1, $2, 'tag', $3)",
    )
    .bind(repository_id)
    .bind(format!("refs/tags/{tag}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("tag should persist");
    commit_id
}

#[allow(clippy::too_many_arguments)]
async fn seed_release(
    pool: &PgPool,
    repository_id: Uuid,
    author: &User,
    tag: &str,
    target_commit_id: Uuid,
    title: &str,
    draft: bool,
    prerelease: bool,
    latest: bool,
    days_ago: i64,
) -> Uuid {
    let row = sqlx::query(
        r#"
        INSERT INTO releases (
            repository_id, tag_name, name, body, draft, prerelease, author_user_id,
            target_commit_id, body_html, rendered_body_excerpt, is_latest, tag_verified,
            published_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '', $9, $10, true, $11, $11)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(tag)
    .bind(title)
    .bind("Release notes with **safe** markdown and <script>alert('x')</script>")
    .bind(draft)
    .bind(prerelease)
    .bind(author.id)
    .bind(target_commit_id)
    .bind(format!("{title} excerpt"))
    .bind(latest)
    .bind(Utc::now() - Duration::days(days_ago))
    .fetch_one(pool)
    .await
    .expect("release should persist");
    row.get("id")
}

async fn seed_asset(pool: &PgPool, repository_id: Uuid, release_id: Uuid, uploader: &User) {
    sqlx::query(
        r#"
        INSERT INTO release_assets (
            repository_id, release_id, name, label, content_type, byte_size,
            storage_key, checksum_sha256, download_count, uploaded_by_user_id
        )
        VALUES ($1, $2, 'opengithub.tar.gz', 'Linux build', 'application/gzip',
                128, 'releases/test/opengithub.tar.gz', $3, 42, $4)
        "#,
    )
    .bind(repository_id)
    .bind(release_id)
    .bind(format!("{:064x}", Uuid::new_v4().as_u128()))
    .bind(uploader.id)
    .execute(pool)
    .await
    .expect("asset should persist");
}

async fn seed_reaction(
    pool: &PgPool,
    repository_id: Uuid,
    release_id: Uuid,
    user: &User,
    reaction: &str,
) {
    sqlx::query(
        "INSERT INTO release_reactions (repository_id, release_id, user_id, reaction) VALUES ($1, $2, $3, $4)",
    )
    .bind(repository_id)
    .bind(release_id)
    .bind(user.id)
    .bind(reaction)
    .execute(pool)
    .await
    .expect("reaction should persist");
}

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request_body = if let Some(value) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&value).expect("body should serialize"))
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
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
