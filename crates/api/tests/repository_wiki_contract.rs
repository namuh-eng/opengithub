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
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        wiki::wiki_content_sha,
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
            eprintln!("skipping repository wiki scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_wiki_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.wiki_repositories') IS NOT NULL
               AND to_regclass('public.wiki_pages') IS NOT NULL
               AND to_regclass('public.wiki_page_revisions') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_wiki_tables {
            eprintln!("skipping repository wiki scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository wiki scenario with pre-applied schema after migration warning: {error}"
        );
    }
    Some(pool)
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("https://opengithub.test").expect("app URL"),
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
    let set_cookie =
        session::set_cookie_header(config, &session_id, expires_at).expect("cookie should sign");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn get_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Option<String>, Value) {
    let mut builder = Request::builder().uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        content_type,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

struct WikiPageFixture<'a> {
    title: &'a str,
    slug: &'a str,
    markdown: &'a str,
    is_sidebar: bool,
    is_footer: bool,
}

async fn insert_wiki_page(
    pool: &PgPool,
    wiki_repository_id: Uuid,
    author: &User,
    page: WikiPageFixture<'_>,
) -> Uuid {
    let page_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO wiki_pages (wiki_repository_id, title, slug, path, is_sidebar, is_footer)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(wiki_repository_id)
    .bind(page.title)
    .bind(page.slug)
    .bind(format!("{}.md", page.slug))
    .bind(page.is_sidebar)
    .bind(page.is_footer)
    .fetch_one(pool)
    .await
    .expect("wiki page should insert");
    let revision_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO wiki_page_revisions (
            page_id, author_user_id, commit_oid, message, markdown, content_sha
        )
        VALUES ($1, $2, $3, 'Publish wiki page', $4, $5)
        RETURNING id
        "#,
    )
    .bind(page_id)
    .bind(author.id)
    .bind(format!("wiki{}", Uuid::new_v4().simple()))
    .bind(page.markdown)
    .bind(wiki_content_sha(page.markdown))
    .fetch_one(pool)
    .await
    .expect("wiki revision should insert");
    sqlx::query("UPDATE wiki_pages SET latest_revision_id = $1 WHERE id = $2")
        .bind(revision_id)
        .bind(page_id)
        .execute(pool)
        .await
        .expect("latest wiki revision should link");
    page_id
}

#[tokio::test]
async fn repository_wiki_read_contract_returns_pages_markdown_clone_and_states() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository wiki scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "wiki-owner").await;
    let owner_login = owner.username.clone().expect("owner username");
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("wiki-{}", Uuid::new_v4().simple()),
            description: Some("Wiki reader repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repository should create");
    let wiki_repository_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO wiki_repositories (repository_id) VALUES ($1) RETURNING id",
    )
    .bind(public_repo.id)
    .fetch_one(&pool)
    .await
    .expect("wiki repository should insert");
    insert_wiki_page(
        &pool,
        wiki_repository_id,
        &owner,
        WikiPageFixture {
            title: "Home",
            slug: "Home",
            markdown:
                "# Home\n\nWelcome to the **wiki**.\n\n## Install\n\n<script>alert('x')</script>",
            is_sidebar: false,
            is_footer: false,
        },
    )
    .await;
    insert_wiki_page(
        &pool,
        wiki_repository_id,
        &owner,
        WikiPageFixture {
            title: "Install Guide",
            slug: "Install Guide",
            markdown: "# Install Guide\n\nUse `cargo test`.\n\n## Troubleshooting",
            is_sidebar: false,
            is_footer: false,
        },
    )
    .await;
    insert_wiki_page(
        &pool,
        wiki_repository_id,
        &owner,
        WikiPageFixture {
            title: "_Sidebar",
            slug: "_sidebar",
            markdown: "## Contents\n\n- [Install](Install-Guide)",
            is_sidebar: true,
            is_footer: false,
        },
    )
    .await;
    insert_wiki_page(
        &pool,
        wiki_repository_id,
        &owner,
        WikiPageFixture {
            title: "_Footer",
            slug: "_footer",
            markdown: "Last updated by maintainers.",
            is_sidebar: false,
            is_footer: true,
        },
    )
    .await;

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let uri = format!("/api/repos/{}/{}/wiki", owner_login, public_repo.name);
    let (status, content_type, body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type
        .as_deref()
        .is_some_and(|value| value.starts_with("application/json")));
    assert_eq!(body["state"]["kind"], "ready");
    assert_eq!(body["page"]["title"], "Home");
    assert!(body["page"]["html"].as_str().unwrap().contains("Welcome"));
    assert!(!body["page"]["html"].as_str().unwrap().contains("<script>"));
    assert_eq!(body["page"]["outline"][0]["text"], "Home");
    assert_eq!(body["viewer"]["canEditWiki"], false);
    assert_eq!(
        body["clone"]["httpsUrl"],
        format!(
            "https://opengithub.test{}/{}.wiki.git",
            body["repository"]["ownerLogin"].as_str().unwrap(),
            public_repo.name
        )
    );
    assert_eq!(body["sidebar"]["title"], "_Sidebar");
    assert_eq!(body["footer"]["title"], "_Footer");
    assert!(body["pages"].as_array().unwrap().len() >= 2);
    assert!(body.to_string().contains("wiki"));
    assert!(!body.to_string().contains("google-client-secret"));
    assert!(!body.to_string().contains("test-session-secret"));

    let owner_uri = format!(
        "/api/repos/{}/{}/wiki/Install%20Guide",
        body["repository"]["ownerLogin"].as_str().unwrap(),
        public_repo.name
    );
    let (owner_status, _, owner_body) =
        get_json(app.clone(), &owner_uri, Some(&owner_cookie)).await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["page"]["title"], "Install Guide");
    assert_eq!(owner_body["viewer"]["permission"], "owner");
    assert_eq!(owner_body["viewer"]["canEditWiki"], true);
    assert_eq!(owner_body["page"]["outline"][1]["text"], "Troubleshooting");

    sqlx::query("UPDATE repositories SET wiki_enabled = false WHERE id = $1")
        .bind(public_repo.id)
        .execute(&pool)
        .await
        .expect("wiki should disable");
    let (disabled_status, _, disabled_body) = get_json(app.clone(), &uri, None).await;
    assert_eq!(disabled_status, StatusCode::OK);
    assert_eq!(disabled_body["state"]["kind"], "disabled");
    assert!(disabled_body["page"].is_null());

    let private_owner = create_user(&pool, "wiki-private-owner").await;
    let private_owner_login = private_owner
        .username
        .clone()
        .expect("private owner username");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: private_owner.id,
            },
            name: format!("wiki-private-{}", Uuid::new_v4().simple()),
            description: Some("Private wiki".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let private_uri = format!(
        "/api/repos/{}/{}/wiki",
        private_owner_login, private_repo.name
    );
    let (private_status, _, private_body) = get_json(app, &private_uri, None).await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");
}
