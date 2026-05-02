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
        issues::{create_issue, CreateIssue},
        notifications::{create_notification, CreateNotification},
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert")
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

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder.body(Body::empty()).expect("request should build");
    let response = app.oneshot(request).await.expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response should be json")
    };
    (status, value)
}

#[tokio::test]
async fn notifications_inbox_contract_filters_groups_and_marks_read() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping notifications inbox scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "notifications-owner").await;
    let viewer = create_user(&pool, "notifications-viewer").await;
    let hidden = create_user(&pool, "notifications-hidden").await;
    let repo_name = format!("notifications-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    sqlx::query(
        "INSERT INTO repository_watches (user_id, repository_id, reason) VALUES ($1, $2, 'subscribed') ON CONFLICT DO NOTHING",
    )
    .bind(viewer.id)
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("watch should persist");

    let issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Inbox search keeps mention filters".to_owned(),
            body: Some("notification body".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![],
            attachments: vec![],
        },
    )
    .await
    .expect("issue should create");

    let mention = create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Inbox search keeps mention filters".to_owned(),
            reason: "mention".to_owned(),
        },
    )
    .await
    .expect("mention notification should create");
    let assigned = create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Assigned notification older".to_owned(),
            reason: "assigned".to_owned(),
        },
    )
    .await
    .expect("assigned notification should create");
    sqlx::query("UPDATE notifications SET unread = false, updated_at = now() - interval '2 days' WHERE id = $1")
        .bind(assigned.id)
        .execute(&pool)
        .await
        .expect("assigned notification should age");
    create_notification(
        &pool,
        CreateNotification {
            user_id: hidden.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Hidden user notification".to_owned(),
            reason: "mention".to_owned(),
        },
    )
    .await
    .expect("hidden notification should create");

    let cookie = cookie_header(&pool, &config, &viewer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (anon_status, anon_body) =
        send_json(app.clone(), Method::GET, "/api/notifications", None).await;
    assert_eq!(anon_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anon_body["error"]["code"], "not_authenticated");

    let uri = format!(
        "/api/notifications?q=reason%3Amention&group=repository&repo={}%2F{}",
        owner.email, repo_name
    );
    let (status, body) = send_json(app.clone(), Method::GET, &uri, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["unreadCount"], 1);
    assert_eq!(
        body["groups"][0]["label"],
        format!("{}/{}", owner.email, repo_name)
    );
    assert_eq!(
        body["groups"][0]["rows"][0]["title"],
        "Inbox search keeps mention filters"
    );
    assert_eq!(body["groups"][0]["rows"][0]["subjectNumber"], issue.number);
    assert_eq!(body["groups"][0]["rows"][0]["subscribed"], true);
    assert!(body["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|bucket| {
            bucket["label"] == format!("{}/{}", owner.email, repo_name) && bucket["count"] == 2
        }));

    let (unread_status, unread_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications?tab=unread&sort=oldest",
        Some(&cookie),
    )
    .await;
    assert_eq!(unread_status, StatusCode::OK);
    assert_eq!(unread_body["total"], 1);
    assert_eq!(unread_body["query"]["tab"], "unread");

    let (mark_status, mark_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/read", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(mark_status, StatusCode::OK);
    assert_eq!(mark_body["unread"], false);

    let unread_after = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notifications WHERE user_id = $1 AND unread = true",
    )
    .bind(viewer.id)
    .fetch_one(&pool)
    .await
    .expect("unread count should load");
    assert_eq!(unread_after, 0);
}
