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
            create_repository, grant_repository_permission,
            repository_traffic_for_actor_by_owner_name, CreateRepository, RepositoryOwner,
            RepositoryTrafficQuery, RepositoryVisibility,
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
            eprintln!("skipping repository traffic scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_traffic_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_insight_snapshots') IS NOT NULL
               AND to_regclass('public.recent_insight_views') IS NOT NULL
               AND to_regclass('public.repository_traffic_daily') IS NOT NULL
               AND to_regclass('public.repository_referrers_daily') IS NOT NULL
               AND to_regclass('public.repository_popular_content_daily') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_traffic_tables {
            eprintln!("skipping repository traffic scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository traffic scenario with pre-applied schema after migration warning: {error}"
        );
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
    let mut builder = Request::builder()
        .uri(uri)
        // Keep anonymous traffic auth assertions independent from the shared
        // integration-test anonymous rate-limit bucket.
        .header(
            "x-forwarded-for",
            format!("repository-traffic-contract-{}", Uuid::new_v4()),
        );
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

#[tokio::test]
async fn repository_traffic_returns_push_access_analytics_privacy_and_cache_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository traffic scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "traffic-owner").await;
    let writer = create_user(&pool, "traffic-writer").await;
    let reader = create_user(&pool, "traffic-reader").await;
    let outsider = create_user(&pool, "traffic-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let writer_cookie = cookie_header(&pool, &config, &writer).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("traffic-{}", Uuid::new_v4().simple()),
            description: Some("Traffic analytics repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("release/main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        writer.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("writer permission should grant");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    let today = Utc::now().date_naive();
    sqlx::query(
        r#"
        INSERT INTO repository_traffic_daily (
            repository_id, traffic_date, clones_total, clones_unique, visitors_total, visitors_unique
        )
        VALUES
            ($1, $2, 18, 7, 91, 40),
            ($1, $3, 4, 3, 22, 12),
            ($1, $4, 99, 44, 101, 50)
        "#,
    )
    .bind(repository.id)
    .bind(today - Duration::days(1))
    .bind(today - Duration::days(8))
    .bind(today - Duration::days(20))
    .execute(&pool)
    .await
    .expect("traffic daily rows should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_referrers_daily (
            repository_id, traffic_date, referrer, total_views, unique_visitors
        )
        VALUES
            ($1, $2, 'docs.opengithub.local', 30, 12),
            ($1, $3, 'https://search.opengithub.local/results?q=traffic', 18, 9),
            ($1, $2, 'zzz.example.com/traffic-tie', 5, 2),
            ($1, $2, 'aaa.example.com/traffic-tie', 5, 2),
            ($1, $4, 'stale.opengithub.local', 99, 80)
        "#,
    )
    .bind(repository.id)
    .bind(today - Duration::days(1))
    .bind(today - Duration::days(8))
    .bind(today - Duration::days(20))
    .execute(&pool)
    .await
    .expect("referrer rows should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_popular_content_daily (
            repository_id, traffic_date, path, title, total_views, unique_visitors
        )
        VALUES
            ($1, $2, 'src/lib.rs', 'Library entrypoint', 45, 20),
            ($1, $3, 'docs/traffic report.md', 'Traffic report', 16, 7),
            ($1, $2, 'docs/very/long/path/that/should/remain/linkable/and/sorted/<script>alert(1)</script>.md', 'Long traffic report <script>alert(1)</script>', 5, 2),
            ($1, $4, 'stale.md', 'Stale page', 88, 22)
        "#,
    )
    .bind(repository.id)
    .bind(today - Duration::days(1))
    .bind(today - Duration::days(8))
    .bind(today - Duration::days(20))
    .execute(&pool)
    .await
    .expect("popular content rows should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let base = format!("/api/repos/{}/{}", repository.owner_login, repository.name);
    let (anonymous_status, anonymous_body) =
        get_json(app.clone(), &format!("{base}/graphs/traffic"), None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert!(!anonymous_body.to_string().contains("test-session-secret"));

    let (private_status, private_body) = get_json(
        app.clone(),
        &format!("{base}/graphs/traffic"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(private_status, StatusCode::NOT_FOUND);
    assert_eq!(private_body["error"]["code"], "not_found");

    let (reader_status, reader_body) = get_json(
        app.clone(),
        &format!("{base}/graphs/traffic"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(reader_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_body["error"]["code"], "traffic_access_required");
    assert_eq!(reader_body["details"]["countsVisible"], false);
    assert!(!reader_body.to_string().contains("91"));
    assert!(!reader_body.to_string().contains("docs.opengithub.local"));

    let direct_traffic = repository_traffic_for_actor_by_owner_name(
        &pool,
        writer.id,
        &repository.owner_login,
        &repository.name,
        RepositoryTrafficQuery,
    )
    .await;
    assert!(
        direct_traffic.is_ok(),
        "direct traffic error: {direct_traffic:?}"
    );

    let (status, body) = get_json(
        app.clone(),
        &format!("{base}/graphs/traffic"),
        Some(&writer_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["repository"]["name"], repository.name);
    assert_eq!(body["repository"]["viewerPermission"], "write");
    assert_eq!(body["repository"]["defaultBranch"], "release/main");
    assert_eq!(body["window"]["key"], "14d");
    assert_eq!(body["window"]["timezone"], "UTC");
    assert_eq!(body["window"]["dayCount"], 14);
    assert_eq!(body["window"]["clonesUpdateCadence"], "hourly");
    assert_eq!(body["window"]["referrersUpdateCadence"], "daily");
    assert_eq!(body["window"]["internalTrafficExcluded"], true);
    assert_eq!(body["summaries"]["clonesTotal"], 22);
    assert_eq!(body["summaries"]["clonesUnique"], 10);
    assert_eq!(body["summaries"]["visitorsTotal"], 113);
    assert_eq!(body["summaries"]["visitorsUnique"], 52);
    assert_eq!(body["summaries"]["referrersTotal"], 58);
    assert_eq!(body["summaries"]["popularContentTotal"], 66);
    assert_eq!(body["summaries"]["activeDays"], 2);
    assert_eq!(body["summaries"]["hasTraffic"], true);
    assert_eq!(body["clones"].as_array().expect("clone series").len(), 14);
    assert_eq!(
        body["visitors"].as_array().expect("visitor series").len(),
        14
    );
    assert!(body["clones"]
        .as_array()
        .expect("clone series")
        .iter()
        .any(|point| point["total"] == 18 && point["unique"] == 7));
    assert_eq!(body["referrers"][0]["referrer"], "docs.opengithub.local");
    assert_eq!(
        body["referrers"][0]["href"],
        "https://docs.opengithub.local"
    );
    assert_eq!(
        body["referrers"][1]["href"],
        "https://search.opengithub.local/results?q=traffic"
    );
    assert_eq!(
        body["referrers"][2]["referrer"],
        "aaa.example.com/traffic-tie"
    );
    assert_eq!(
        body["referrers"][3]["referrer"],
        "zzz.example.com/traffic-tie"
    );
    assert_eq!(body["popularContent"][0]["path"], "src/lib.rs");
    assert_eq!(body["popularContent"][0]["title"], "Library entrypoint");
    assert!(body["popularContent"][1]["href"]
        .as_str()
        .expect("content href")
        .contains("/blob/release%2Fmain/docs/traffic%20report.md"));
    assert!(body["popularContent"][2]["path"]
        .as_str()
        .expect("long path")
        .contains("<script>alert(1)</script>"));
    assert_eq!(body["snapshot"]["stale"], false);
    assert!(body["snapshot"]["cacheKey"]
        .as_str()
        .expect("cache key")
        .starts_with("traffic:"));
    assert!(!body.to_string().contains("SESSION_SECRET"));

    let snapshot_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_insight_snapshots
        WHERE repository_id = $1 AND period_key = '14d' AND cache_key LIKE 'traffic:%'
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("snapshot count should query");
    assert_eq!(snapshot_count, 1);
    let view_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM recent_insight_views
        WHERE repository_id = $1 AND user_id = $2 AND period_key = '14d'
        "#,
    )
    .bind(repository.id)
    .bind(writer.id)
    .fetch_one(&pool)
    .await
    .expect("view count should query");
    assert_eq!(view_count, 1);

    let public_repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("traffic-public-{}", Uuid::new_v4().simple()),
            description: Some("Public traffic repository".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repository should create");
    let public_base = format!(
        "/api/repos/{}/{}",
        public_repository.owner_login, public_repository.name
    );
    let (public_reader_status, public_reader_body) = get_json(
        app.clone(),
        &format!("{public_base}/graphs/traffic"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(public_reader_status, StatusCode::FORBIDDEN);
    assert_eq!(public_reader_body["details"]["requiredPermission"], "write");

    let (owner_status, owner_body) = get_json(
        app,
        &format!("{public_base}/graphs/traffic"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["summaries"]["clonesTotal"], 0);
    assert_eq!(owner_body["summaries"]["activeDays"], 0);
    assert_eq!(owner_body["summaries"]["hasTraffic"], false);
    assert_eq!(owner_body["referrers"].as_array().unwrap().len(), 0);
}
