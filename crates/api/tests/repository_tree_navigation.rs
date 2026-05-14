use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        permissions::RepositoryRole,
        repositories::{
            create_repository_with_bootstrap, grant_repository_permission, insert_commit,
            upsert_git_ref, CreateCommit, CreateRepository, RepositoryBootstrapRequest,
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    let suffix = Uuid::new_v4().simple();
    let user = upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(label),
        None,
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

async fn send_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
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

async fn insert_file(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Uuid,
    path: &str,
    content: &str,
) {
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
    .bind(format!(
        "oid-{}-{}",
        commit_id.simple(),
        path.replace('/', "-")
    ))
    .bind(content.len() as i64)
    .execute(pool)
    .await
    .expect("file should insert");
}

#[tokio::test]
async fn repository_tree_contract_resolves_branches_tags_and_recovery_links() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository tree navigation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "tree-owner").await;
    let outsider = create_user(&pool, "tree-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository_with_bootstrap(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("tree-nav-{}", Uuid::new_v4().simple()),
            description: Some("Tree navigation repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: owner.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: true,
            template_slug: Some("rust-axum".to_owned()),
            ..RepositoryBootstrapRequest::default()
        },
    )
    .await
    .expect("repository should create");
    let default_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = 'refs/heads/main'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("default branch should exist");
    let default_commit_oid =
        sqlx::query_scalar::<_, String>("SELECT oid FROM commits WHERE id = $1")
            .bind(default_commit_id)
            .fetch_one(&pool)
            .await
            .expect("default commit should exist");
    let feature_commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("feature-{}", Uuid::new_v4().simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Add docs on feature branch".to_owned(),
            tree_oid: None,
            parent_oids: vec![default_commit_oid],
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("feature commit should insert");
    insert_file(
        &pool,
        repository.id,
        feature_commit.id,
        "README.md",
        "# Feature branch\n",
    )
    .await;
    insert_file(
        &pool,
        repository.id,
        feature_commit.id,
        "docs/guide.md",
        "# Guide\n",
    )
    .await;
    for index in 0..105 {
        insert_file(
            &pool,
            repository.id,
            feature_commit.id,
            &format!("docs/example-{index:03}.md"),
            &format!("# Example {index}\n"),
        )
        .await;
    }
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/feature/tree-nav",
        "branch",
        Some(feature_commit.id),
    )
    .await
    .expect("feature branch ref should upsert");
    upsert_git_ref(
        &pool,
        repository.id,
        "refs/tags/v1.0.0",
        "tag",
        Some(default_commit_id),
    )
    .await
    .expect("tag ref should upsert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let encoded_feature = "feature%2Ftree-nav";
    let (feature_status, feature_body) = send_json(
        app.clone(),
        &format!("{base}/contents/docs?ref={encoded_feature}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(feature_status, StatusCode::OK);
    assert_eq!(feature_body["resolvedRef"]["kind"], "branch");
    assert_eq!(feature_body["resolvedRef"]["shortName"], "feature/tree-nav");
    assert_eq!(
        feature_body["resolvedRef"]["qualifiedName"],
        "refs/heads/feature/tree-nav"
    );
    assert_eq!(feature_body["path"], "docs");
    assert_eq!(feature_body["page"], 1);
    assert_eq!(feature_body["pageSize"], 30);
    assert_eq!(feature_body["total"], 106);
    assert_eq!(feature_body["hasMore"], true);
    assert_eq!(
        feature_body["entries"][0]["name"], "example-000.md",
        "files sort by stable path name"
    );
    assert!(feature_body["entries"]
        .as_array()
        .expect("entries should be an array")
        .iter()
        .any(|entry| entry["name"] == "example-000.md"
            && entry["href"]
                .as_str()
                .expect("entry href")
                .contains("/blob/feature%2Ftree-nav/docs/example-000.md")));

    let (paged_status, paged_body) = send_json(
        app.clone(),
        &format!("{base}/contents/docs?ref={encoded_feature}&page=3&pageSize=50"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(paged_status, StatusCode::OK);
    assert_eq!(paged_body["page"], 3);
    assert_eq!(paged_body["pageSize"], 50);
    assert_eq!(paged_body["total"], 106);
    assert_eq!(paged_body["hasMore"], false);
    assert_eq!(
        paged_body["entries"]
            .as_array()
            .expect("entries should be an array")
            .len(),
        6
    );

    let (tag_status, tag_body) = send_json(
        app.clone(),
        &format!("{base}/blobs/README.md?ref=v1.0.0"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(tag_status, StatusCode::OK);
    assert_eq!(tag_body["resolvedRef"]["kind"], "tag");
    assert_eq!(tag_body["resolvedRef"]["shortName"], "v1.0.0");
    assert!(tag_body["file"]["content"]
        .as_str()
        .expect("README content should be a string")
        .starts_with(&format!("# {}", repository.name)));

    let (finder_status, finder_body) = send_json(
        app.clone(),
        &format!("{base}/file-finder?ref={encoded_feature}&q=guide"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(finder_status, StatusCode::OK);
    assert_eq!(finder_body["resolvedRef"]["shortName"], "feature/tree-nav");
    assert_eq!(finder_body["page"], 1);
    assert_eq!(finder_body["pageSize"], 20);
    assert_eq!(finder_body["items"][0]["path"], "docs/guide.md");

    let (finder_page_status, finder_page_body) = send_json(
        app.clone(),
        &format!("{base}/file-finder?ref={encoded_feature}&q=example&page=2&pageSize=40"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(finder_page_status, StatusCode::OK);
    assert_eq!(finder_page_body["page"], 2);
    assert_eq!(finder_page_body["pageSize"], 40);
    assert_eq!(finder_page_body["total"], 105);
    assert_eq!(finder_page_body["items"][0]["path"], "docs/example-040.md");

    let (path_cache_status, path_cache_body) = send_json(
        app.clone(),
        &format!("{base}/find?ref={encoded_feature}&q=guide"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(path_cache_status, StatusCode::OK);
    assert_eq!(path_cache_body["resolvedRef"]["shortName"], "feature/tree-nav");
    assert_eq!(
        path_cache_body["total"], 107,
        "the /find path-cache contract returns the full ref path list and ignores q"
    );
    assert_eq!(path_cache_body["pageSize"], 5000);
    assert!(path_cache_body["items"]
        .as_array()
        .expect("path cache items should be an array")
        .iter()
        .any(|entry| entry["path"] == "README.md"));
    assert!(path_cache_body["items"]
        .as_array()
        .expect("path cache items should be an array")
        .iter()
        .any(|entry| entry["path"] == "docs/example-104.md"));
    let cached_paths: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT paths
        FROM repository_ref_files
        WHERE repository_id = $1 AND ref = 'feature/tree-nav'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("finder request should refresh repository_ref_files");
    assert!(cached_paths
        .as_array()
        .expect("cached paths should be a JSON array")
        .iter()
        .any(|path| path == "docs/guide.md"));

    let (bad_path_status, bad_path_body) = send_json(
        app.clone(),
        &format!("{base}/contents/%2E%2E/secrets?ref={encoded_feature}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(bad_path_status, StatusCode::NOT_FOUND);
    assert!(!bad_path_body.to_string().to_lowercase().contains("stack"));

    let (refs_status, refs_body) = send_json(
        app.clone(),
        &format!("{base}/refs?q=feature&currentPath=docs&activeRef={encoded_feature}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(refs_status, StatusCode::OK);
    assert_eq!(refs_body["total"], 1);
    assert_eq!(refs_body["items"][0]["shortName"], "feature/tree-nav");
    assert_eq!(refs_body["items"][0]["active"], true);
    assert!(refs_body["items"][0]["samePathHref"]
        .as_str()
        .expect("same path href")
        .ends_with("/tree/feature%2Ftree-nav/docs"));

    let (tag_refs_status, tag_refs_body) = send_json(
        app.clone(),
        &format!("{base}/refs?q=v1&currentPath=docs&activeRef={encoded_feature}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(tag_refs_status, StatusCode::OK);
    assert_eq!(tag_refs_body["items"][0]["shortName"], "v1.0.0");
    assert_eq!(tag_refs_body["items"][0]["active"], false);
    assert!(tag_refs_body["items"][0]["samePathHref"]
        .as_str()
        .expect("tag same path href")
        .ends_with("/tree/v1.0.0"));

    let (missing_ref_status, missing_ref_body) = send_json(
        app.clone(),
        &format!("{base}/contents/docs?ref=nope"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_ref_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_ref_body["error"]["code"], "ref_not_found");
    assert_eq!(
        missing_ref_body["details"]["defaultBranchHref"],
        format!("/{}/{}/tree/main", repository.owner_login, repository.name)
    );

    let (missing_path_status, missing_path_body) = send_json(
        app.clone(),
        &format!("{base}/contents/docs?ref=v1.0.0"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_path_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_path_body["error"]["code"], "path_not_found");
    assert_eq!(missing_path_body["details"]["path"], "docs");

    let (private_status, private_body) = send_json(
        app.clone(),
        &format!("{base}/contents/docs?ref={encoded_feature}"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");

    grant_repository_permission(
        &pool,
        repository.id,
        outsider.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("permission should grant");
    let (allowed_status, allowed_body) = send_json(
        app,
        &format!("{base}/contents/docs?ref={encoded_feature}&page=0&pageSize=1000"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(allowed_status, StatusCode::OK);
    assert_eq!(allowed_body["page"], 1);
    assert_eq!(allowed_body["pageSize"], 100);
    assert_eq!(allowed_body["entries"][0]["name"], "example-000.md");
}
