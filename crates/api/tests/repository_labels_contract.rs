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
        issues::{create_issue, ensure_default_labels, CreateIssue},
        pulls::{create_pull_request, CreatePullRequest},
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
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
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
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
async fn repository_label_management_contract_counts_permissions_and_mutations() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository labels contract; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "labels-owner").await;
    let reader = create_user(&pool, "labels-reader").await;
    let outsider = create_user(&pool, "labels-outsider").await;
    let repo_name = format!("labels-contract-{}", Uuid::new_v4().simple());
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: repo_name.clone(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        opengithub_api::domain::permissions::RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader should grant");
    let labels = ensure_default_labels(&pool, repository.id)
        .await
        .expect("default labels should exist");
    let bug = labels.iter().find(|label| label.name == "bug").unwrap();

    create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Open issue keeps bug count".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![bug.id],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("issue should create");
    create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Open pull keeps bug count".to_owned(),
            body: None,
            head_ref: "feature".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![bug.id],
            milestone_id: None,
            assignee_user_ids: Vec::new(),
            reviewer_user_ids: Vec::new(),
            template_slug: None,
        },
    )
    .await
    .expect("pull request should create");
    let category_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name)
        VALUES ($1, 'general', 'General')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("category should create");
    let discussion_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (repository_id, category_id, number, title, body, author_user_id)
        VALUES ($1, $2, 1, 'Bug discussion', 'Discuss the label', $3)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(category_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("discussion should create");
    sqlx::query("INSERT INTO discussion_labels (discussion_id, label_id) VALUES ($1, $2)")
        .bind(discussion_id)
        .bind(bug.id)
        .execute(&pool)
        .await
        .expect("discussion label should create");

    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config.clone());
    let labels_path = format!("/api/repos/{}/{}/labels", owner.email_login(), repo_name);

    let (status, body) = send_json(
        app.clone(),
        Method::GET,
        &format!("{labels_path}?q=bug&sort=total_issue_count&direction=desc"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["name"], "bug");
    assert_eq!(body["items"][0]["counts"]["openIssues"], 1);
    assert_eq!(body["items"][0]["counts"]["openPullRequests"], 1);
    assert_eq!(body["items"][0]["counts"]["discussions"], 1);
    assert_eq!(body["items"][0]["counts"]["totalIssueCount"], 2);
    assert_eq!(body["viewer"]["canWrite"], false);
    assert!(body["items"][0]["issuesHref"]
        .as_str()
        .unwrap()
        .contains("label%3Abug"));

    let (reader_status, _) = send_json(
        app.clone(),
        Method::POST,
        &labels_path,
        Some(&reader_cookie),
        Some(json!({
            "name": "needs-design",
            "color": "b46838",
            "description": "Design review"
        })),
    )
    .await;
    assert_eq!(reader_status, StatusCode::FORBIDDEN);

    let (create_status, created) = send_json(
        app.clone(),
        Method::POST,
        &labels_path,
        Some(&owner_cookie),
        Some(json!({
            "name": "needs-design",
            "color": "#b46838",
            "description": "Design review"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(created["label"]["color"], "b46838");
    let created_id = created["label"]["id"].as_str().unwrap();

    let (duplicate_status, duplicate) = send_json(
        app.clone(),
        Method::POST,
        &labels_path,
        Some(&owner_cookie),
        Some(json!({
            "name": "NEEDS-DESIGN",
            "color": "b46838",
            "description": "Duplicate"
        })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate["error"]["code"], "conflict");

    let (invalid_status, invalid) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{labels_path}/{created_id}"),
        Some(&owner_cookie),
        Some(json!({
            "name": "needs-design",
            "color": "not-hex",
            "description": "Bad color"
        })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid["error"]["code"], "validation_failed");

    let (update_status, updated) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{labels_path}/{created_id}"),
        Some(&owner_cookie),
        Some(json!({
            "name": "needs-copy",
            "color": "8a5a44",
            "description": "Copy review"
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(updated["label"]["name"], "needs-copy");

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM repository_label_events WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("label events should count");
    assert!(event_count >= 2);

    let (delete_status, deleted) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{labels_path}/{created_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
    assert_eq!(deleted["label"]["name"], "needs-copy");

    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("labels-private-{}", Uuid::new_v4().simple()),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let (private_status, private_body) = send_json(
        app,
        Method::GET,
        &format!(
            "/api/repos/{}/{}/labels",
            owner.email_login(),
            private_repo.name
        ),
        Some(&outsider_cookie),
        None,
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert!(
        !private_body.to_string().contains("SESSION_SECRET"),
        "structured errors must not leak env values"
    );
}

trait TestUserLogin {
    fn email_login(&self) -> &str;
}

impl TestUserLogin for User {
    fn email_login(&self) -> &str {
        self.username
            .as_deref()
            .expect("test user should have generated username")
    }
}
