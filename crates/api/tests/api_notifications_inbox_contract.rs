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
        notifications::{
            create_notification, should_deliver_notification, CreateNotification,
            NotificationDeliveryCheck,
        },
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
            RepositoryWatchEvent,
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
    if MIGRATOR.run(&pool).await.is_err() {
        let schema_ready = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.notifications') IS NOT NULL
               AND to_regclass('public.notification_delivery_preferences') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .ok()?;
        if !schema_ready {
            return None;
        }
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

async fn send_json_body(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder
        .body(Body::from(body.to_string()))
        .expect("request should build");
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
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());

    let (anon_status, anon_body) =
        send_json(app.clone(), Method::GET, "/api/notifications", None).await;
    assert_eq!(anon_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anon_body["error"]["code"], "not_authenticated");

    let uri = format!(
        "/api/notifications?q=reason%3Amention&group=repository&repo={}%2F{}",
        owner.username.as_deref().unwrap_or(&owner.email),
        repo_name
    );
    let (status, body) = send_json(app.clone(), Method::GET, &uri, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["unreadCount"], 1);
    assert_eq!(
        body["groups"][0]["label"],
        format!(
            "{}/{}",
            owner.username.as_deref().unwrap_or(&owner.email),
            repo_name
        )
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
            bucket["label"]
                == format!(
                    "{}/{}",
                    owner.username.as_deref().unwrap_or(&owner.email),
                    repo_name
                )
                && bucket["count"] == 2
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
    assert_eq!(mark_body["saved"], false);

    let unread_after = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notifications WHERE user_id = $1 AND unread = true",
    )
    .bind(viewer.id)
    .fetch_one(&pool)
    .await
    .expect("unread count should load");
    assert_eq!(unread_after, 0);

    let (unread_status, unread_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/unread", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(unread_status, StatusCode::OK);
    assert_eq!(unread_body["unread"], true);
    assert_eq!(unread_body["unreadCount"], 1);

    let (save_status, save_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/save", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(save_status, StatusCode::OK);
    assert_eq!(save_body["saved"], true);
    assert_eq!(save_body["folderCounts"]["saved"], 1);

    let (saved_list_status, saved_list_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications?folder=saved",
        Some(&cookie),
    )
    .await;
    assert_eq!(saved_list_status, StatusCode::OK);
    assert_eq!(saved_list_body["total"], 1);
    assert_eq!(saved_list_body["groups"][0]["rows"][0]["saved"], true);
    assert_eq!(saved_list_body["folders"][1]["count"], 1);

    let (unsave_status, unsave_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/unsave", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(unsave_status, StatusCode::OK);
    assert_eq!(unsave_body["saved"], false);
    assert_eq!(unsave_body["folderCounts"]["saved"], 0);

    let (resave_status, resave_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/save", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(resave_status, StatusCode::OK);
    assert_eq!(resave_body["folderCounts"]["saved"], 1);

    let (done_status, done_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/done", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(done_status, StatusCode::OK);
    assert_eq!(done_body["done"], true);
    assert_eq!(done_body["saved"], true);
    assert_eq!(done_body["unread"], true);
    assert_eq!(done_body["unreadCount"], 0);
    assert_eq!(done_body["folderCounts"]["inbox"], 1);
    assert_eq!(done_body["folderCounts"]["saved"], 1);
    assert_eq!(done_body["folderCounts"]["done"], 1);

    let (inbox_after_done_status, inbox_after_done_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications",
        Some(&cookie),
    )
    .await;
    assert_eq!(inbox_after_done_status, StatusCode::OK);
    assert_eq!(inbox_after_done_body["total"], 1);
    assert_eq!(
        inbox_after_done_body["groups"][0]["rows"][0]["title"],
        "Assigned notification older"
    );

    let (done_list_status, done_list_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications?folder=done",
        Some(&cookie),
    )
    .await;
    assert_eq!(done_list_status, StatusCode::OK);
    assert_eq!(done_list_body["total"], 1);
    assert_eq!(done_list_body["groups"][0]["rows"][0]["done"], true);
    assert_eq!(done_list_body["groups"][0]["rows"][0]["saved"], true);

    let (move_status, move_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/inbox", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(move_status, StatusCode::OK);
    assert_eq!(move_body["done"], false);
    assert_eq!(move_body["unread"], true);
    assert_eq!(move_body["unreadCount"], 1);
    assert_eq!(move_body["folderCounts"]["inbox"], 2);
    assert_eq!(move_body["folderCounts"]["done"], 0);

    let (unsubscribe_status, unsubscribe_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/unsubscribe", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(unsubscribe_status, StatusCode::OK);
    assert_eq!(unsubscribe_body["subscribed"], false);
    assert_eq!(unsubscribe_body["folderCounts"]["inbox"], 0);
    assert_eq!(unsubscribe_body["unreadCount"], 0);

    let (inbox_after_unsubscribe_status, inbox_after_unsubscribe_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications",
        Some(&cookie),
    )
    .await;
    assert_eq!(inbox_after_unsubscribe_status, StatusCode::OK);
    assert_eq!(inbox_after_unsubscribe_body["total"], 0);

    create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Suppressed subscribed notification".to_owned(),
            reason: "subscribed".to_owned(),
        },
    )
    .await
    .expect("suppressed notification should create for retention");

    let (suppressed_status, suppressed_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications",
        Some(&cookie),
    )
    .await;
    assert_eq!(suppressed_status, StatusCode::OK);
    assert_eq!(suppressed_body["total"], 0);

    create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Mention reactivates notification thread".to_owned(),
            reason: "mention".to_owned(),
        },
    )
    .await
    .expect("reactivating notification should create");

    let (reactivated_status, reactivated_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications",
        Some(&cookie),
    )
    .await;
    assert_eq!(reactivated_status, StatusCode::OK);
    assert!(reactivated_body["total"].as_i64().unwrap_or_default() >= 1);
    assert!(reactivated_body["groups"][0]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |row| row["title"] == "Mention reactivates notification thread"
                && row["subscribed"] == true
        ));

    let (resubscribe_status, resubscribe_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/subscribe", mention.id),
        Some(&cookie),
    )
    .await;
    assert_eq!(resubscribe_status, StatusCode::OK);
    assert_eq!(resubscribe_body["subscribed"], true);

    let (forbidden_status, forbidden_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/{}/save", mention.id),
        Some(&cookie_header(&pool, &config, &hidden).await),
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::NOT_FOUND);
    assert_eq!(forbidden_body["error"]["code"], "notification_not_found");

    let (bulk_empty_status, bulk_empty_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/bulk",
        Some(&cookie),
        json!({
            "notificationIds": [],
            "action": "read"
        }),
    )
    .await;
    assert_eq!(bulk_empty_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bulk_empty_body["error"]["code"], "validation_failed");

    let bulk_first = create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Bulk triage first".to_owned(),
            reason: "mention".to_owned(),
        },
    )
    .await
    .expect("first bulk notification should create");
    let bulk_second = create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Bulk triage second".to_owned(),
            reason: "assigned".to_owned(),
        },
    )
    .await
    .expect("second bulk notification should create");

    let (bulk_status, bulk_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/bulk",
        Some(&cookie),
        json!({
            "notificationIds": [bulk_first.id, bulk_second.id, Uuid::new_v4()],
            "action": "done"
        }),
    )
    .await;
    assert_eq!(bulk_status, StatusCode::OK);
    assert_eq!(bulk_body["action"], "done");
    assert_eq!(bulk_body["updated"].as_array().unwrap().len(), 2);
    assert_eq!(bulk_body["failed"].as_array().unwrap().len(), 1);
    assert_eq!(bulk_body["failed"][0]["code"], "notification_not_found");
    assert!(
        bulk_body["folderCounts"]["done"]
            .as_i64()
            .unwrap_or_default()
            >= 2
    );

    let done_bulk_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notifications WHERE id = ANY($1) AND done_at IS NOT NULL",
    )
    .bind(vec![bulk_first.id, bulk_second.id])
    .fetch_one(&pool)
    .await
    .expect("bulk done count should load");
    assert_eq!(done_bulk_count, 2);
}

#[tokio::test]
async fn notification_delivery_preferences_validate_verified_email_and_audit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping notification delivery scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let viewer = create_user(&pool, "delivery-viewer").await;
    let cookie = cookie_header(&pool, &config, &viewer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let (status, body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications/delivery-preferences",
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["emailChannelAvailable"], true);
    assert_eq!(body["preferences"][0]["key"], "watching");
    assert_eq!(body["preferences"][0]["channels"][0], "web");
    assert_eq!(
        body["customRoutingHref"],
        "/settings/notifications#custom-routing"
    );
    let verified_email_id = body["emails"][0]["id"].as_str().expect("email id");

    let (status, body) = send_json_body(
        app.clone(),
        Method::PATCH,
        "/api/notifications/delivery-preferences",
        Some(&cookie),
        json!({
            "defaultEmailId": verified_email_id,
            "preferences": [
                { "key": "watching", "channels": ["web", "email", "cli"] }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let watching = body["preferences"]
        .as_array()
        .expect("preferences")
        .iter()
        .find(|preference| preference["key"] == "watching")
        .expect("watching preference");
    assert_eq!(watching["channels"], json!(["web", "email", "cli"]));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM security_audit_events WHERE actor_user_id = $1 AND event_type = 'notifications.delivery_preferences.update'",
    )
    .bind(viewer.id)
    .fetch_one(&pool)
    .await
    .expect("audit should count");
    assert!(audit_count >= 1);

    let unverified_email_id: Uuid = sqlx::query_scalar(
        "INSERT INTO user_email_addresses (user_id, email, is_primary, is_public, verified_at) VALUES ($1, $2, false, false, NULL) RETURNING id",
    )
    .bind(viewer.id)
    .bind(format!("unverified-{}@opengithub.local", Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .expect("unverified email should insert");

    let (status, body) = send_json_body(
        app.clone(),
        Method::PATCH,
        "/api/notifications/delivery-preferences",
        Some(&cookie),
        json!({
            "defaultEmailId": unverified_email_id,
            "preferences": [
                { "key": "actions", "channels": ["email"] }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, body) = send_json_body(
        app,
        Method::PATCH,
        "/api/notifications/delivery-preferences",
        Some(&cookie),
        json!({
            "preferences": [
                { "key": "dependabot", "channels": ["email"] }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn notification_fanout_respects_repository_watch_and_thread_overrides() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping notification fanout scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "fanout-owner").await;
    let actor = create_user(&pool, "fanout-actor").await;
    let watcher = create_user(&pool, "fanout-watcher").await;
    let repo_name = format!("fanout-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name,
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Fanout respects saved notification settings".to_owned(),
            body: Some("watch and thread preferences should gate recipients".to_owned()),
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

    sqlx::query(
        r#"
        INSERT INTO repository_watches (user_id, repository_id, reason, level, custom_events)
        VALUES ($1, $2, 'all', 'all', '[]'::jsonb)
        ON CONFLICT (user_id, repository_id)
        DO UPDATE SET reason = EXCLUDED.reason, level = EXCLUDED.level, custom_events = EXCLUDED.custom_events
        "#,
    )
    .bind(watcher.id)
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("watch all should persist");

    let issue_event = NotificationDeliveryCheck {
        user_id: watcher.id,
        repository_id: repository.id,
        subject_type: "issue".to_owned(),
        subject_id: Some(issue.id),
        reason: "comment".to_owned(),
        repository_event: Some(RepositoryWatchEvent::Issues),
        actor_user_id: Some(actor.id),
        participating: false,
        direct: false,
    };
    assert!(
        should_deliver_notification(&pool, issue_event.clone())
            .await
            .expect("watch all should evaluate"),
        "all activity watch should receive repository issue events"
    );

    sqlx::query(
        r#"
        UPDATE repository_watches
        SET reason = 'custom', level = 'custom', custom_events = '["pull_requests"]'::jsonb
        WHERE user_id = $1 AND repository_id = $2
        "#,
    )
    .bind(watcher.id)
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("custom watch should persist");
    assert!(
        !should_deliver_notification(&pool, issue_event.clone())
            .await
            .expect("custom watch should evaluate"),
        "custom watch should suppress unselected repository event categories"
    );
    assert!(
        should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                repository_event: Some(RepositoryWatchEvent::PullRequests),
                subject_type: "pull_request".to_owned(),
                subject_id: None,
                ..issue_event.clone()
            },
        )
        .await
        .expect("selected custom event should evaluate"),
        "custom watch should deliver selected repository event categories"
    );

    sqlx::query(
        "UPDATE repository_watches SET reason = 'ignore', level = 'ignore' WHERE user_id = $1 AND repository_id = $2",
    )
    .bind(watcher.id)
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("ignore should persist");
    assert!(
        !should_deliver_notification(&pool, issue_event.clone())
            .await
            .expect("ignore should evaluate"),
        "ignore should suppress repository watch delivery"
    );
    assert!(
        should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                reason: "mention".to_owned(),
                repository_event: Some(RepositoryWatchEvent::Issues),
                direct: true,
                ..issue_event.clone()
            },
        )
        .await
        .expect("direct mention should evaluate"),
        "direct mentions should still reactivate despite ignored repository watch state"
    );

    let thread_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM notification_threads
        WHERE repository_id = $1 AND subject_type = 'issue' AND subject_id = $2
        "#,
    )
    .bind(repository.id)
    .bind(issue.id)
    .fetch_one(&pool)
    .await
    .expect("delivery check should ensure thread");
    sqlx::query(
        r#"
        INSERT INTO notification_subscriptions (thread_id, user_id, state, reason)
        VALUES ($1, $2, 'unsubscribed', 'manual_unsubscribe')
        ON CONFLICT (thread_id, user_id)
        DO UPDATE SET state = 'unsubscribed', reason = 'manual_unsubscribe'
        "#,
    )
    .bind(thread_id)
    .bind(watcher.id)
    .execute(&pool)
    .await
    .expect("generic thread unsubscribe should persist");
    assert!(
        !should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                participating: true,
                reason: "comment".to_owned(),
                ..issue_event.clone()
            },
        )
        .await
        .expect("generic thread unsubscribe should evaluate"),
        "inbox-level thread unsubscribe should suppress future fanout"
    );
    sqlx::query("DELETE FROM notification_subscriptions WHERE thread_id = $1 AND user_id = $2")
        .bind(thread_id)
        .bind(watcher.id)
        .execute(&pool)
        .await
        .expect("generic thread override should clear");

    sqlx::query(
        r#"
        INSERT INTO issue_subscriptions (issue_id, user_id, subscribed, reason, custom_events)
        VALUES ($1, $2, false, 'ignored', '{}'::text[])
        ON CONFLICT (issue_id, user_id)
        DO UPDATE SET subscribed = false, reason = 'ignored', custom_events = '{}'::text[]
        "#,
    )
    .bind(issue.id)
    .bind(watcher.id)
    .execute(&pool)
    .await
    .expect("thread unsubscribe should persist");
    assert!(
        !should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                participating: true,
                reason: "closed".to_owned(),
                ..issue_event.clone()
            },
        )
        .await
        .expect("thread unsubscribe should evaluate"),
        "thread unsubscribe should beat participating and repository watch delivery"
    );

    sqlx::query(
        r#"
        UPDATE issue_subscriptions
        SET subscribed = true, reason = 'subscribed', custom_events = ARRAY['closed']::text[]
        WHERE issue_id = $1 AND user_id = $2
        "#,
    )
    .bind(issue.id)
    .bind(watcher.id)
    .execute(&pool)
    .await
    .expect("thread custom events should persist");
    assert!(
        should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                participating: true,
                reason: "closed".to_owned(),
                ..issue_event.clone()
            },
        )
        .await
        .expect("closed event should evaluate"),
        "selected thread state-change events should deliver"
    );
    assert!(
        !should_deliver_notification(
            &pool,
            NotificationDeliveryCheck {
                participating: true,
                reason: "reopened".to_owned(),
                ..issue_event
            },
        )
        .await
        .expect("reopened event should evaluate"),
        "unselected thread state-change events should be suppressed"
    );
}

#[tokio::test]
async fn notification_custom_filters_validate_persist_and_feed_inbox_facets() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping notification custom filters scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "filters-owner").await;
    let viewer = create_user(&pool, "filters-viewer").await;
    let hidden = create_user(&pool, "filters-hidden").await;
    let repo_name = format!("filters-{}", Uuid::new_v4().simple());
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
    let hidden_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: hidden.id },
            name: format!("hidden-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: None,
            created_by_user_id: hidden.id,
        },
    )
    .await
    .expect("private repository should create");
    let owner_login = owner.username.as_deref().unwrap_or(&owner.email);
    let hidden_login = hidden.username.as_deref().unwrap_or(&hidden.email);

    let cookie = cookie_header(&pool, &config, &viewer).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());

    let (settings_status, settings_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications/custom-filters",
        Some(&cookie),
    )
    .await;
    assert_eq!(settings_status, StatusCode::OK);
    assert_eq!(settings_body["limit"], 15);
    assert_eq!(settings_body["remaining"], 15);
    assert!(settings_body["defaultFilters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|filter| filter["queryString"] == "reason:assigned"));

    let (invalid_status, invalid_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/custom-filters",
        Some(&cookie),
        json!({ "name": "Bad", "queryString": "reason:mention NOT repo:anything" }),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert!(invalid_body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("NOT"));

    let (hidden_status, hidden_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/custom-filters",
        Some(&cookie),
        json!({
            "name": "Hidden",
            "queryString": format!("repo:{}/{}", hidden_login, hidden_repo.name),
        }),
    )
    .await;
    assert_eq!(hidden_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        hidden_body["error"]["message"],
        "repo: is not available for this account."
    );

    let create_query = format!("repo:{owner_login}/{repo_name} reason:mention");
    let (create_status, create_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/custom-filters",
        Some(&cookie),
        json!({ "name": "Review mentions", "queryString": create_query }),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK);
    assert_eq!(create_body["remaining"], 14);
    assert_eq!(create_body["customFilters"][0]["name"], "Review mentions");
    assert!(create_body["customFilters"][0]["href"]
        .as_str()
        .unwrap()
        .starts_with("/notifications?q="));
    let filter_id = create_body["customFilters"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Custom filter target".to_owned(),
            body: None,
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
    create_notification(
        &pool,
        CreateNotification {
            user_id: viewer.id,
            repository_id: Some(repository.id),
            subject_type: "issue".to_owned(),
            subject_id: Some(issue.id),
            title: "Custom filter target".to_owned(),
            reason: "mention".to_owned(),
        },
    )
    .await
    .expect("notification should create");

    let (inbox_status, inbox_body) = send_json(
        app.clone(),
        Method::GET,
        "/api/notifications",
        Some(&cookie),
    )
    .await;
    assert_eq!(inbox_status, StatusCode::OK);
    assert!(inbox_body["filters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|filter| {
            filter["label"] == "Review mentions"
                && filter["query"] == format!("repo:{owner_login}/{repo_name} reason:mention")
                && filter["count"] == 1
        }));

    let (update_status, update_body) = send_json_body(
        app.clone(),
        Method::PATCH,
        &format!("/api/notifications/custom-filters/{filter_id}"),
        Some(&cookie),
        json!({ "name": "Unread mentions", "queryString": "reason:mention is:unread" }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["customFilters"][0]["name"], "Unread mentions");

    for index in 2..=15 {
        let (status, _) = send_json_body(
            app.clone(),
            Method::POST,
            "/api/notifications/custom-filters",
            Some(&cookie),
            json!({
                "name": format!("Filter {index}"),
                "queryString": format!("reason:assigned author:user{index}"),
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
    let (limit_status, limit_body) = send_json_body(
        app.clone(),
        Method::POST,
        "/api/notifications/custom-filters",
        Some(&cookie),
        json!({ "name": "Too many", "queryString": "reason:mention" }),
    )
    .await;
    assert_eq!(limit_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        limit_body["error"]["message"],
        "You can create up to 15 custom notification filters."
    );

    let (delete_status, delete_body) = send_json_body(
        app.clone(),
        Method::DELETE,
        &format!("/api/notifications/custom-filters/{filter_id}"),
        Some(&cookie),
        json!({}),
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
    assert_eq!(delete_body["customFilters"].as_array().unwrap().len(), 14);
    let first_position = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(min(position), 0) FROM notification_custom_filters WHERE user_id = $1",
    )
    .bind(viewer.id)
    .fetch_one(&pool)
    .await
    .expect("positions should load");
    assert_eq!(first_position, 1);
}
