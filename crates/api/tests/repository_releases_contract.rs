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
    let public_slash_commit =
        seed_commit_and_tag(&pool, public_repo.id, &owner, "release/2026", 20).await;
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
    let release_slash = seed_release(
        &pool,
        public_repo.id,
        &owner,
        "release/2026",
        public_slash_commit,
        "Slash tag release",
        false,
        false,
        false,
        20,
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
    let asset_v2 = seed_asset(&pool, public_repo.id, release_v2, &owner).await;
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
    assert_eq!(list_status, StatusCode::OK, "{list_body:?}");
    assert_eq!(list_body["total"], 4);
    assert_eq!(list_body["items"][0]["tagName"], "v2.1.0-beta.1");
    assert_eq!(list_body["items"][0]["prerelease"], true);
    assert!(!list_body.to_string().contains("Draft should hide"));
    let slash_release = list_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|release| release["tagName"] == "release/2026")
        .expect("slash-containing tag release should be listed");
    assert_eq!(slash_release["id"], release_slash.to_string());
    assert!(slash_release["links"]["htmlHref"]
        .as_str()
        .unwrap()
        .ends_with("/releases/tag/release%2F2026"));
    assert!(slash_release["links"]["tagHref"]
        .as_str()
        .unwrap()
        .ends_with("/tree/release%2F2026"));
    assert!(slash_release["links"]["zipballHref"]
        .as_str()
        .unwrap()
        .ends_with("/releases/zipball/release%2F2026"));

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

    let (unreacted_status, _, unreacted_body) = send_json(
        app.clone(),
        Method::GET,
        &latest_uri,
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(unreacted_status, StatusCode::OK);
    assert_eq!(unreacted_body["reactions"]["viewerReaction"], Value::Null);

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

    let by_slash_tag_uri = format!("{public_uri}/tag/release%2F2026");
    let (by_slash_tag_status, _, by_slash_tag_body) =
        send_json(app.clone(), Method::GET, &by_slash_tag_uri, None, None).await;
    assert_eq!(by_slash_tag_status, StatusCode::OK);
    assert_eq!(by_slash_tag_body["id"], release_slash.to_string());

    let (tags_status, _, tags_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{public_uri}/tags"),
        None,
        None,
    )
    .await;
    assert_eq!(tags_status, StatusCode::OK);
    assert_eq!(tags_body["total"], 4);
    assert!(tags_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tag| tag["name"] == "v2.0.0" && tag["releaseId"] == release_v2.to_string()));
    let stable_tag = tags_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tag| tag["name"] == "v2.0.0")
        .expect("stable tag should be listed");
    assert_eq!(stable_tag["verified"], true);
    assert_eq!(
        stable_tag["signatureSummary"],
        "Verified tag signature for v2.0.0"
    );
    assert!(stable_tag["compareHref"]
        .as_str()
        .unwrap()
        .ends_with("/compare/v2.0.0...main"));
    let slash_tag = tags_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tag| tag["name"] == "release/2026")
        .expect("slash tag should be listed");
    assert_eq!(
        slash_tag["releaseHref"],
        format!(
            "/{}-owner/{}/releases/tag/release%2F2026",
            marker, public_repo.name
        )
    );
    assert!(slash_tag["compareHref"]
        .as_str()
        .unwrap()
        .ends_with("/compare/release%2F2026...main"));

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
    let (tar_status, _, tar_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{public_uri}/tarball/v2.0.0"),
        None,
        None,
    )
    .await;
    assert_eq!(tar_status, StatusCode::OK);
    assert_eq!(tar_body["format"], "tarball");
    let archive_cache_rows = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_archives WHERE repository_id = $1 AND ref_name = 'refs/tags/v2.0.0' AND target_oid IS NOT NULL AND format IN ('zip', 'tar')",
    )
    .bind(public_repo.id)
    .fetch_one(&pool)
    .await
    .expect("archive cache rows should read");
    assert_eq!(archive_cache_rows, 2);
    let archive_downloads = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM release_downloads WHERE release_id = $1 AND source = 'zipball'",
    )
    .bind(release_v2)
    .fetch_one(&pool)
    .await
    .expect("archive download count should read");
    assert_eq!(archive_downloads, 1);

    let (asset_status, _, asset_body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{public_uri}/assets/{asset_v2}"),
        None,
        None,
    )
    .await;
    assert_eq!(asset_status, StatusCode::OK);
    assert_eq!(asset_body["asset"]["downloadCount"], 43);
    assert_eq!(asset_body["releaseTagName"], "v2.0.0");
    assert!(!asset_body
        .to_string()
        .contains("releases/test/opengithub.tar.gz"));
    let asset_downloads = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM release_downloads WHERE release_id = $1 AND asset_id = $2 AND source = 'asset'",
    )
    .bind(release_v2)
    .bind(asset_v2)
    .fetch_one(&pool)
    .await
    .expect("asset download count should read");
    assert_eq!(asset_downloads, 1);

    let (anonymous_reaction_status, _, anonymous_reaction_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{public_uri}/{release_v2}/reactions"),
        None,
        Some(json!({ "content": "eyes" })),
    )
    .await;
    assert_eq!(anonymous_reaction_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_reaction_body["error"]["code"], "unauthorized");

    let (reaction_status, _, reaction_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{public_uri}/{release_v2}/reactions"),
        Some(&reader_cookie),
        Some(json!({ "content": "eyes" })),
    )
    .await;
    assert_eq!(reaction_status, StatusCode::CREATED);
    assert_eq!(reaction_body["eyes"], 1);
    assert_eq!(reaction_body["rocket"], 0);
    assert_eq!(reaction_body["totalCount"], 2);
    assert_eq!(reaction_body["viewerReaction"], "eyes");

    let (reaction_toggle_status, _, reaction_toggle_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{public_uri}/{release_v2}/reactions"),
        Some(&reader_cookie),
        Some(json!({ "content": "eyes" })),
    )
    .await;
    assert_eq!(reaction_toggle_status, StatusCode::CREATED);
    assert_eq!(reaction_toggle_body["eyes"], 0);
    assert_eq!(reaction_toggle_body["totalCount"], 1);
    assert_eq!(reaction_toggle_body["viewerReaction"], Value::Null);

    let (invalid_reaction_status, _, invalid_reaction_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{public_uri}/{release_v2}/reactions"),
        Some(&reader_cookie),
        Some(json!({ "content": "sparkles" })),
    )
    .await;
    assert_eq!(invalid_reaction_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_reaction_body["error"]["code"], "validation_failed");

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
    assert_eq!(owner_list_body["total"], 5);
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

#[tokio::test]
async fn repository_releases_management_contract_writes_assets_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository Releases management scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("relmgmt{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Release management repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let commit = seed_commit_and_tag(&pool, repo.id, &owner, "seed", 1).await;
    sqlx::query(
        "INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id) VALUES ($1, 'refs/heads/main', 'branch', $2)",
    )
    .bind(repo.id)
    .bind(commit)
    .execute(&pool)
    .await
    .expect("branch should persist");

    let uri = format!("/api/repos/{}/{}/releases", owner.email, repo.name);
    let (reader_create_status, _, reader_create_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&reader_cookie),
        Some(json!({
            "tagName": "v4.0.0",
            "target": "main",
            "title": "Reader cannot publish"
        })),
    )
    .await;
    assert_eq!(reader_create_status, StatusCode::FORBIDDEN);
    assert!(!reader_create_body.to_string().contains(&repo.name));

    let (create_status, _, create_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({
            "tagName": "v4.0.0",
            "target": "main",
            "title": "Version four",
            "body": "Draft notes with **safe** markdown",
            "draft": true,
            "prerelease": true
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let release_id = Uuid::parse_str(create_body["id"].as_str().unwrap()).unwrap();
    assert_eq!(create_body["draft"], true);
    assert_eq!(create_body["prerelease"], true);
    assert!(!create_body.to_string().contains("storage_key"));

    let (duplicate_status, _, duplicate_body) = send_json(
        app.clone(),
        Method::POST,
        &uri,
        Some(&owner_cookie),
        Some(json!({ "tagName": "v4.0.0", "target": "main" })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let (asset_status, _, asset_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/{release_id}/assets"),
        Some(&owner_cookie),
        Some(json!({
            "name": "opengithub-darwin.tar.gz",
            "label": "macOS build",
            "contentType": "application/gzip",
            "byteSize": 2048,
            "checksumSha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        })),
    )
    .await;
    assert_eq!(asset_status, StatusCode::CREATED);
    assert_eq!(asset_body["assets"][0]["name"], "opengithub-darwin.tar.gz");
    let asset_text = asset_body.to_string();
    assert!(!asset_text.contains("storageKey"));
    assert!(!asset_text.contains("storage_key"));
    assert!(!asset_text.contains(&format!("releases/{release_id}/assets")));
    let asset_id = Uuid::parse_str(asset_body["assets"][0]["id"].as_str().unwrap()).unwrap();

    let (publish_status, _, publish_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/{release_id}/publish"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(publish_status, StatusCode::OK);
    assert_eq!(publish_body["draft"], false);
    assert_eq!(publish_body["latest"], false);

    let (update_status, _, update_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/{release_id}"),
        Some(&owner_cookie),
        Some(json!({
            "title": "Version four updated",
            "body": "Updated release notes",
            "prerelease": false
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["title"], "Version four updated");
    assert_eq!(update_body["prerelease"], false);
    assert_eq!(update_body["latest"], true);

    let (delete_asset_status, _, delete_asset_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/{release_id}/assets/{asset_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_asset_status, StatusCode::OK);
    assert!(delete_asset_body["assets"].as_array().unwrap().is_empty());

    let (delete_status, _, _) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/{release_id}"),
        Some(&owner_cookie),
        Some(json!({})),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
    let deleted_at = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        "SELECT deleted_at FROM releases WHERE id = $1",
    )
    .bind(release_id)
    .fetch_one(&pool)
    .await
    .expect("deleted release should read");
    assert!(deleted_at.is_some());
    let audit_events = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM release_audit_events WHERE repository_id = $1 AND release_id = $2",
    )
    .bind(repo.id)
    .bind(release_id)
    .fetch_one(&pool)
    .await
    .expect("audit events should read");
    assert!(audit_events >= 5);
    let audit_text = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(string_agg(after_state::text, ' '), '') FROM release_audit_events WHERE release_id = $1",
    )
    .bind(release_id)
    .fetch_one(&pool)
    .await
    .expect("audit text should read");
    assert!(!audit_text.contains("storage_key"));
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
            tag_signature_summary, published_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '', $9, $10, true, $11, $12, $12)
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
    .bind(format!("Verified tag signature for {tag}"))
    .bind(Utc::now() - Duration::days(days_ago))
    .fetch_one(pool)
    .await
    .expect("release should persist");
    row.get("id")
}

async fn seed_asset(pool: &PgPool, repository_id: Uuid, release_id: Uuid, uploader: &User) -> Uuid {
    let row = sqlx::query(
        r#"
        INSERT INTO release_assets (
            repository_id, release_id, name, label, content_type, byte_size,
            storage_key, checksum_sha256, download_count, uploaded_by_user_id
        )
        VALUES ($1, $2, 'opengithub.tar.gz', 'Linux build', 'application/gzip',
                128, 'releases/test/opengithub.tar.gz', $3, 42, $4)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(release_id)
    .bind(format!("{:064x}", Uuid::new_v4().as_u128()))
    .bind(uploader.id)
    .fetch_one(pool)
    .await
    .expect("asset should persist");
    row.get("id")
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
    let mut builder = Request::builder().method(method).uri(uri).header(
        "x-forwarded-for",
        format!("198.51.100.{}", Uuid::new_v4().as_u128() % 250 + 1),
    );
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
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response should be JSON")
    };
    (status, headers, value)
}
