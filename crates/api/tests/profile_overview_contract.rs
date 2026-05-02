use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Datelike, Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
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

async fn create_profile_user(pool: &PgPool, username: &str, private: bool) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{username}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{username} display")),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1,
            bio = $2,
            company = $3,
            location = $4,
            website_url = $5,
            profile_visibility = $6
        WHERE id = $7
        "#,
    )
    .bind(username)
    .bind(format!("{username} builds in public"))
    .bind("@namuh")
    .bind("Seoul")
    .bind("https://namuh.co")
    .bind(if private { "private" } else { "public" })
    .bind(user.id)
    .execute(pool)
    .await
    .expect("profile columns should update");
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

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

#[tokio::test]
async fn public_profile_returns_overview_pins_counts_and_viewer_state() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile overview scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("profile{}", Uuid::new_v4().simple());
    let owner = create_profile_user(&pool, &marker, false).await;
    let follower = create_profile_user(&pool, &format!("{marker}-follower"), false).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let follower_cookie = cookie_header(&pool, &config, &follower).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    sqlx::query("INSERT INTO user_follows (follower_user_id, followed_user_id) VALUES ($1, $2)")
        .bind(follower.id)
        .bind(owner.id)
        .execute(&pool)
        .await
        .expect("follow should insert");
    sqlx::query(
        "INSERT INTO user_profile_readmes (user_id, body, rendered_html, updated_by_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(owner.id)
    .bind("# Hello from profile")
    .bind("<h1>Hello from profile</h1>")
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("readme should insert");

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-public"),
            description: Some("public pinned repo".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-private"),
            description: Some("private pinned repo".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");
    sqlx::query(
        "INSERT INTO profile_pins (user_id, repository_id, position) VALUES ($1, $2, 1), ($1, $3, 2)",
    )
    .bind(owner.id)
    .bind(public_repo.id)
    .bind(private_repo.id)
    .execute(&pool)
    .await
    .expect("pins should insert");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', '#b7410e', 900), ($1, 'TypeScript', '#8c5a3c', 100)",
    )
    .bind(public_repo.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query("INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2)")
        .bind(follower.id)
        .bind(public_repo.id)
        .execute(&pool)
        .await
        .expect("star should insert");

    let organization_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (slug, display_name, owner_user_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(format!("{marker}-org"))
    .bind("Profile Org")
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("org should insert");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(organization_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("membership should insert");

    let achievement_id: Uuid = sqlx::query_scalar(
        "INSERT INTO achievements (slug, name, description, icon) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(format!("{marker}-shipper"))
    .bind("Shipper")
    .bind("Opened the first pull request")
    .bind("spark")
    .fetch_one(&pool)
    .await
    .expect("achievement should insert");
    sqlx::query("INSERT INTO user_achievements (user_id, achievement_id) VALUES ($1, $2)")
        .bind(owner.id)
        .bind(achievement_id)
        .execute(&pool)
        .await
        .expect("user achievement should insert");
    sqlx::query(
        "INSERT INTO profile_contribution_days (user_id, day, contribution_count) VALUES ($1, CURRENT_DATE, 7)",
    )
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("contribution day should insert");
    sqlx::query(
        "INSERT INTO profile_contribution_days (user_id, day, contribution_count) VALUES ($1, DATE '2025-02-14', 3)",
    )
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("prior-year contribution day should insert");
    sqlx::query(
        "INSERT INTO profile_contribution_events (user_id, repository_id, event_type, title, target_href) VALUES ($1, $2, 'commit', 'Pushed profile contract', $3)",
    )
    .bind(owner.id)
    .bind(public_repo.id)
    .bind(format!("/{}/{}/commit/abc123", marker, public_repo.name))
    .execute(&pool)
    .await
    .expect("contribution event should insert");
    sqlx::query(
        "INSERT INTO profile_contribution_events (user_id, repository_id, event_type, title, target_href, occurred_at) VALUES ($1, $2, 'commit', 'Pushed prior-year profile contract', $3, TIMESTAMPTZ '2025-02-14 12:00:00Z')",
    )
    .bind(owner.id)
    .bind(public_repo.id)
    .bind(format!("/{}/{}/commit/def456", marker, public_repo.name))
    .execute(&pool)
    .await
    .expect("prior-year contribution event should insert");

    let (status, headers, anonymous) =
        get_json(app.clone(), &format!("/api/users/{marker}/profile"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(anonymous["identity"]["login"], marker);
    assert_eq!(anonymous["identity"]["followerCount"], 1);
    assert_eq!(anonymous["identity"]["followingCount"], 0);
    assert_eq!(anonymous["readme"]["body"], "# Hello from profile");
    assert_eq!(anonymous["pinnedRepositories"].as_array().unwrap().len(), 1);
    assert_eq!(
        anonymous["pinnedRepositories"][0]["name"],
        format!("{marker}-public")
    );
    assert_eq!(
        anonymous["pinnedRepositories"][0]["primaryLanguage"]["language"],
        "Rust"
    );
    assert_eq!(anonymous["achievements"][0]["name"], "Shipper");
    assert_eq!(anonymous["organizations"][0]["name"], "Profile Org");
    assert_eq!(anonymous["contributionSummary"]["total"], 7);
    assert_eq!(
        anonymous["contributionSummary"]["year"],
        Utc::now().date_naive().year()
    );
    assert_eq!(anonymous["contributionSummary"]["days"][0]["intensity"], 3);
    assert_eq!(anonymous["tabCounts"]["repositories"], 1);
    assert_eq!(anonymous["tabCounts"]["stars"], 1);
    assert_eq!(anonymous["viewerState"]["authenticated"], false);
    assert_eq!(anonymous["viewerState"]["canFollow"], true);

    let (status, _, owner_view) = get_json(
        app.clone(),
        &format!("/api/users/{}/profile", marker.to_uppercase()),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        owner_view["pinnedRepositories"].as_array().unwrap().len(),
        2
    );
    assert_eq!(owner_view["tabCounts"]["repositories"], 2);
    assert_eq!(owner_view["viewerState"]["isSelf"], true);
    assert_eq!(owner_view["viewerState"]["canFollow"], false);

    let (status, _, follower_view) = get_json(
        app.clone(),
        &format!("/api/users/{marker}/profile"),
        Some(&follower_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(follower_view["viewerState"]["authenticated"], true);
    assert_eq!(follower_view["viewerState"]["isFollowing"], true);

    let (status, _, year_view) = get_json(
        app,
        &format!("/api/users/{marker}/profile?year=2025"),
        Some(&follower_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(year_view["contributionSummary"]["year"], 2025);
    assert_eq!(year_view["contributionSummary"]["total"], 3);
    assert_eq!(
        year_view["contributionSummary"]["days"][0]["date"],
        "2025-02-14"
    );
    assert_eq!(
        year_view["contributionSummary"]["recentEvents"][0]["title"],
        "Pushed prior-year profile contract"
    );
}

#[tokio::test]
async fn private_profile_redacts_secondary_data_and_missing_user_404s() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping profile overview scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("private{}", Uuid::new_v4().simple());
    let private_user = create_profile_user(&pool, &marker, true).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User {
                id: private_user.id,
            },
            name: format!("{marker}-repo"),
            description: Some("hidden repo".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: private_user.id,
        },
    )
    .await
    .expect("repo should create");
    sqlx::query("INSERT INTO profile_pins (user_id, repository_id, position) VALUES ($1, $2, 1)")
        .bind(private_user.id)
        .bind(repo.id)
        .execute(&pool)
        .await
        .expect("pin should insert");
    sqlx::query(
        "INSERT INTO user_profile_readmes (user_id, body, rendered_html) VALUES ($1, $2, $3)",
    )
    .bind(private_user.id)
    .bind("Private readme")
    .bind("<p>Private readme</p>")
    .execute(&pool)
    .await
    .expect("readme should insert");

    let (status, headers, body) =
        get_json(app.clone(), &format!("/api/users/{marker}/profile"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["identity"]["login"], marker);
    assert_eq!(body["identity"]["isPrivate"], true);
    assert_eq!(body["identity"]["followerCount"], Value::Null);
    assert_eq!(body["readme"]["body"], "Private readme");
    assert!(body["pinnedRepositories"].as_array().unwrap().is_empty());
    assert!(body["achievements"].as_array().unwrap().is_empty());
    assert!(body["organizations"].as_array().unwrap().is_empty());
    assert!(body["contributionSummary"]["days"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(body["tabCounts"]["repositories"], 0);
    assert_eq!(body["viewerState"]["canFollow"], false);

    let (missing_status, _, missing) =
        get_json(app, "/api/users/does-not-exist/profile", None).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing["error"]["code"], "not_found");
}
