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

    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping repository discussions scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_discussion_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.discussions') IS NOT NULL
               AND to_regclass('public.discussion_categories') IS NOT NULL
               AND to_regclass('public.discussion_votes') IS NOT NULL
               AND to_regclass('public.discussion_form_answers') IS NOT NULL
               AND to_regclass('public.discussion_subscriptions') IS NOT NULL
               AND to_regclass('public.discussion_polls') IS NOT NULL
               AND to_regclass('public.discussion_reactions') IS NOT NULL
               AND to_regclass('public.discussion_answers') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_discussion_tables {
            eprintln!("skipping repository discussions scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository discussions scenario with pre-applied schema after migration warning: {error}"
        );
    }
    Some(pool)
}

#[tokio::test]
async fn repository_discussion_detail_returns_timeline_sidebar_and_answer_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussion detail scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-detail-owner").await;
    let reader = create_user(&pool, "discussion-detail-reader").await;
    let commenter = create_user(&pool, "discussion-detail-commenter").await;
    let outsider = create_user(&pool, "discussion-detail-outsider").await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussion-detail-{}", Uuid::new_v4().simple()),
            description: Some("Discussion detail contract".to_owned()),
            visibility: RepositoryVisibility::Private,
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
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    let category_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position, accepts_answers)
        VALUES ($1, 'q-a', 'Q&A', '💬', 'Questions with accepted answers.', 1, true)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("category should insert");
    let label_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO labels (repository_id, name, color, description)
        VALUES ($1, 'help-wanted', 'a16207', 'Needs community help')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("label should insert");
    let discussion_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (
            repository_id, category_id, number, title, body, state, answered,
            author_user_id, comments_count, votes_count, last_activity_at
        )
        VALUES (
            $1, $2, 7, 'How do discussion answers work?',
            'Use **Markdown** safely <script>bad()</script> in a discussion body.',
            'open', true, $3, 3, 2, now()
        )
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(category_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("discussion should insert");
    sqlx::query("INSERT INTO discussion_labels (discussion_id, label_id) VALUES ($1, $2)")
        .bind(discussion_id)
        .bind(label_id)
        .execute(&pool)
        .await
        .expect("discussion label should insert");
    let first_comment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_comments (discussion_id, author_user_id, body, created_at)
        VALUES ($1, $2, 'First timeline comment', now() - interval '2 hours')
        RETURNING id
        "#,
    )
    .bind(discussion_id)
    .bind(commenter.id)
    .fetch_one(&pool)
    .await
    .expect("first comment should insert");
    let answer_comment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_comments (discussion_id, author_user_id, body, created_at)
        VALUES ($1, $2, 'This is the accepted answer.', now() - interval '1 hour')
        RETURNING id
        "#,
    )
    .bind(discussion_id)
    .bind(commenter.id)
    .fetch_one(&pool)
    .await
    .expect("answer comment should insert");
    let reply_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_comments (discussion_id, parent_comment_id, author_user_id, body)
        VALUES ($1, $2, $3, 'Nested reply context')
        RETURNING id
        "#,
    )
    .bind(discussion_id)
    .bind(answer_comment_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("reply should insert");
    sqlx::query("UPDATE discussions SET answer_comment_id = $1 WHERE id = $2")
        .bind(answer_comment_id)
        .bind(discussion_id)
        .execute(&pool)
        .await
        .expect("answer pointer should update");
    sqlx::query(
        "INSERT INTO discussion_answers (discussion_id, comment_id, marked_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(discussion_id)
    .bind(answer_comment_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("answer should insert");
    sqlx::query(
        r#"
        INSERT INTO discussion_form_answers (discussion_id, field_id, field_label, value)
        VALUES ($1, 'context', 'Context', 'Readers need answer summaries.')
        "#,
    )
    .bind(discussion_id)
    .execute(&pool)
    .await
    .expect("form answer should insert");
    let poll_id: Uuid = sqlx::query_scalar(
        "INSERT INTO discussion_polls (discussion_id, question, allows_multiple) VALUES ($1, 'Which path?', false) RETURNING id",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("poll should insert");
    sqlx::query(
        "INSERT INTO discussion_poll_options (poll_id, position, label) VALUES ($1, 0, 'Read'), ($1, 1, 'Write')",
    )
    .bind(poll_id)
    .execute(&pool)
    .await
    .expect("poll options should insert");
    sqlx::query(
        r#"
        INSERT INTO discussion_reactions (discussion_id, comment_id, user_id, content)
        VALUES ($1, NULL, $2, 'heart'), ($1, $3, $2, '+1'), ($1, $4, $5, 'eyes')
        "#,
    )
    .bind(discussion_id)
    .bind(reader.id)
    .bind(answer_comment_id)
    .bind(reply_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("reactions should insert");
    sqlx::query(
        "INSERT INTO discussion_subscriptions (discussion_id, user_id, state, reason) VALUES ($1, $2, 'subscribed', 'manual')",
    )
    .bind(discussion_id)
    .bind(reader.id)
    .execute(&pool)
    .await
    .expect("subscription should insert");
    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, 'answer_marked', jsonb_build_object('commentId', $3::text))
        "#,
    )
    .bind(discussion_id)
    .bind(owner.id)
    .bind(answer_comment_id)
    .execute(&pool)
    .await
    .expect("event should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let path = format!(
        "/api/repos/{owner_login}/{}/discussions/7?sort=oldest",
        repository.name
    );

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &path, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert!(!outsider_body.to_string().contains("accepted answer"));

    let (status, body) = get_json(app.clone(), &path, Some(&reader_cookie)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["discussion"]["number"], 7);
    assert_eq!(body["discussion"]["answered"], true);
    assert_eq!(body["viewer"]["canComment"], true);
    assert_eq!(body["viewer"]["canMarkAnswer"], false);
    assert_eq!(body["category"]["slug"], "q-a");
    assert_eq!(body["labels"][0]["name"], "help-wanted");
    assert!(body["body"]["html"]
        .as_str()
        .expect("html")
        .contains("<strong>Markdown</strong>"));
    assert!(!body["body"]["html"]
        .as_str()
        .expect("html")
        .contains("<script>"));
    assert_eq!(body["formAnswers"][0]["fieldLabel"], "Context");
    assert_eq!(
        body["poll"]["options"].as_array().expect("options").len(),
        2
    );
    assert_eq!(body["answer"]["commentId"], answer_comment_id.to_string());
    assert!(body["answer"]["href"]
        .as_str()
        .expect("answer href")
        .contains("#discussioncomment-"));
    assert_eq!(body["reactions"][0]["content"], "heart");
    assert_eq!(body["reactions"][0]["viewerReacted"], true);
    assert_eq!(body["subscription"]["state"], "subscribed");
    assert_eq!(
        body["sidebar"]["participants"]
            .as_array()
            .expect("participants")
            .len(),
        2
    );
    assert_eq!(body["sidebar"]["events"][0]["eventType"], "answer_marked");
    assert_eq!(body["timeline"][0]["kind"], "comment");
    assert_eq!(body["timeline"][1]["answer"], true);
    assert_eq!(
        body["timeline"][1]["replies"][0]["body"]["markdown"],
        "Nested reply context"
    );
    assert!(!body.to_string().contains("test-session-secret"));

    let (invalid_status, invalid_body) = get_json(
        app,
        &format!(
            "/api/repos/{owner_login}/{}/discussions/7?sort=primer-blue",
            repository.name
        ),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");
    assert_ne!(first_comment_id, answer_comment_id);
}

#[tokio::test]
async fn repository_discussion_creation_returns_forms_and_persists_normal_discussion() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussion creation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-create-owner").await;
    let maintainer = create_user(&pool, "discussion-create-maintainer").await;
    let reader = create_user(&pool, "discussion-create-reader").await;
    let outsider = create_user(&pool, "discussion-create-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussion-create-{}", Uuid::new_v4().simple()),
            description: Some("Discussion creation contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        maintainer.id,
        RepositoryRole::Write,
        "direct",
    )
    .await
    .expect("maintainer permission should grant");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    let ideas_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position, accepts_answers)
        VALUES ($1, 'ideas', 'Ideas', '💡', 'Feature proposals.', 1, true)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("ideas category should insert");
    let polls_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position, accepts_answers)
        VALUES ($1, 'polls', 'Polls', '📊', 'Vote on options.', 2, false)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("poll category should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_community_links (repository_id, label, href, kind, position)
        VALUES ($1, 'Contributing guide', '/CONTRIBUTING.md', 'guide', 1)
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("community link should insert");

    let commit_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, $2, $3, $3, 'Add discussion template')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(format!("template-{}", Uuid::new_v4().simple()))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("commit should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, 'refs/heads/main', 'branch', $2)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .execute(&pool)
    .await
    .expect("default branch ref should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        VALUES ($1, $2, '.github/DISCUSSION_TEMPLATE/ideas.yml', $3, $4, length($3))
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .bind(
        r#"
name: Idea
description: <script>bad()</script>Share a feature idea.
body:
  - type: textarea
    id: context
    attributes:
      label: Context
      description: Tell us why this matters.
      placeholder: What should happen?
    validations:
      required: true
  - type: dropdown
    id: area
    attributes:
      label: Area
      options:
        - UI
        - API
"#,
    )
    .bind(format!("blob-{}", Uuid::new_v4().simple()))
    .execute(&pool)
    .await
    .expect("template file should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!("/api/repos/{owner_login}/{}/discussions", repository.name);

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &format!("{base}/new"), Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert!(!outsider_body.to_string().contains("Feature proposals"));

    let (metadata_status, metadata_body) = get_json(
        app.clone(),
        &format!("{base}/new?category=ideas&title=Search%20syntax"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(metadata_status, StatusCode::OK, "{metadata_body}");
    assert_eq!(metadata_body["viewer"]["canCreate"], true);
    assert_eq!(metadata_body["selectedCategory"]["slug"], "ideas");
    assert_eq!(
        metadata_body["categories"]
            .as_array()
            .expect("categories")
            .len(),
        2
    );
    assert_eq!(
        metadata_body["form"]["templatePath"],
        ".github/DISCUSSION_TEMPLATE/ideas.yml"
    );
    assert_eq!(metadata_body["form"]["fields"][0]["id"], "context");
    assert_eq!(metadata_body["form"]["fields"][0]["required"], true);
    assert!(metadata_body["form"]["description"]
        .as_str()
        .expect("description")
        .contains("Share a feature idea."));
    assert!(!metadata_body.to_string().contains("<script>"));
    assert_eq!(
        metadata_body["similarSearch"]["query"],
        "is:open Search syntax"
    );
    assert_eq!(
        metadata_body["communityLinks"][0]["label"],
        "Contributing guide"
    );
    assert!(!metadata_body.to_string().contains("test-session-secret"));

    let (reader_create_status, reader_create_body) = post_json(
        app.clone(),
        &base,
        Some(&reader_cookie),
        json!({
            "categorySlug": "ideas",
            "title": "Reader should not create",
            "body": "Missing write permission.",
            "similarSearchAcknowledged": true,
            "formAnswers": [{ "fieldId": "context", "value": "No write permission." }]
        }),
    )
    .await;
    assert_eq!(reader_create_status, StatusCode::FORBIDDEN);
    assert_eq!(reader_create_body["error"]["code"], "forbidden");

    let (missing_ack_status, missing_ack_body) = post_json(
        app.clone(),
        &base,
        Some(&owner_cookie),
        json!({
            "categorySlug": "ideas",
            "title": "Search syntax ideas",
            "body": "Support saved discussion searches.",
            "similarSearchAcknowledged": false,
            "formAnswers": [{ "fieldId": "context", "value": "Users repeat search discussions." }]
        }),
    )
    .await;
    assert_eq!(missing_ack_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing_ack_body["error"]["code"], "validation_failed");

    let (create_status, create_body) = post_json(
        app.clone(),
        &base,
        Some(&owner_cookie),
        json!({
            "categorySlug": "ideas",
            "title": "Search syntax ideas",
            "body": "Support saved discussion searches.",
            "similarSearchAcknowledged": true,
            "formAnswers": [
                { "fieldId": "context", "value": "Users repeat search discussions." },
                { "fieldId": "area", "value": "UI" }
            ],
            "attachmentDrafts": [{
                "fileName": "sketch.png",
                "contentType": "image/png",
                "byteSize": 128,
                "storageKey": "discussion-drafts/sketch.png"
            }]
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK, "{create_body}");
    assert_eq!(create_body["discussionNumber"], 1);
    assert_eq!(
        create_body["href"],
        format!("/{owner_login}/{}/discussions/1", repository.name)
    );

    let discussion_id = Uuid::parse_str(create_body["discussionId"].as_str().expect("id"))
        .expect("discussion id should parse");
    let discussion_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussions WHERE id = $1 AND category_id = $2 AND comments_count = 1",
    )
    .bind(discussion_id)
    .bind(ideas_id)
    .fetch_one(&pool)
    .await
    .expect("discussion should count");
    assert_eq!(discussion_count, 1);
    let answer_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_form_answers WHERE discussion_id = $1",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("answers should count");
    assert_eq!(answer_count, 2);
    let subscription_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_subscriptions WHERE discussion_id = $1 AND user_id = $2 AND state = 'subscribed'",
    )
    .bind(discussion_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("subscription should count");
    assert_eq!(subscription_count, 1);
    let attachment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_attachments WHERE discussion_id = $1 AND status = 'attached'",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("attachment should count");
    assert_eq!(attachment_count, 1);
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_activity_events WHERE discussion_id = $1 AND event_type = 'created'",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("activity should count");
    assert_eq!(event_count, 1);
    let notification_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM notifications WHERE subject_type = 'discussion' AND subject_id = $1 AND reason = 'discussion_created'",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("notification should count");
    assert_eq!(notification_count, 1);

    let (missing_poll_status, missing_poll_body) = post_json(
        app.clone(),
        &base,
        Some(&owner_cookie),
        json!({
            "categorySlug": "polls",
            "title": "Choose a default branch policy",
            "body": null,
            "similarSearchAcknowledged": true
        }),
    )
    .await;
    assert_eq!(missing_poll_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(missing_poll_body["error"]["message"]
        .as_str()
        .expect("poll error")
        .contains("poll question"));

    let (poll_status, poll_body) = post_json(
        app.clone(),
        &base,
        Some(&owner_cookie),
        json!({
            "categorySlug": "polls",
            "title": "Choose a default branch policy",
            "body": null,
            "similarSearchAcknowledged": true,
            "poll": {
                "question": "Which branch policy should ship first?",
                "options": ["Linear history", "Required reviews", "Signed commits"],
                "allowsMultiple": true
            }
        }),
    )
    .await;
    assert_eq!(poll_status, StatusCode::OK, "{poll_body}");
    let poll_discussion_id = Uuid::parse_str(poll_body["discussionId"].as_str().expect("poll id"))
        .expect("poll discussion id should parse");
    let poll_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM discussion_polls
        JOIN discussions ON discussions.id = discussion_polls.discussion_id
        WHERE discussion_polls.discussion_id = $1
          AND discussions.category_id = $2
          AND discussion_polls.allows_multiple = true
        "#,
    )
    .bind(poll_discussion_id)
    .bind(polls_id)
    .fetch_one(&pool)
    .await
    .expect("poll should count");
    assert_eq!(poll_count, 1);
    let poll_option_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM discussion_poll_options
        JOIN discussion_polls ON discussion_polls.id = discussion_poll_options.poll_id
        WHERE discussion_polls.discussion_id = $1
        "#,
    )
    .bind(poll_discussion_id)
    .fetch_one(&pool)
    .await
    .expect("poll options should count");
    assert_eq!(poll_option_count, 3);
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
    request_json(app, "GET", uri, cookie).await
}

async fn request_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    builder = builder.method(method);
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
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
async fn repository_discussions_return_screen_ready_list_and_category_filters() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussions scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-owner").await;
    let reader = create_user(&pool, "discussion-reader").await;
    let voter = create_user(&pool, "discussion-voter").await;
    let outsider = create_user(&pool, "discussion-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let voter_cookie = cookie_header(&pool, &config, &voter).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussions-{}", Uuid::new_v4().simple()),
            description: Some("Repository discussions contract".to_owned()),
            visibility: RepositoryVisibility::Private,
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
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");
    grant_repository_permission(
        &pool,
        repository.id,
        voter.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("voter permission should grant");

    let general_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position)
        VALUES ($1, 'general', 'General', '💬', 'Open-ended product discussion.', 1)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("general category should insert");
    let ideas_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position)
        VALUES ($1, 'ideas', 'Ideas', '💡', 'Feature proposals and experiments.', 2)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("ideas category should insert");
    let label_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO labels (repository_id, name, color, description)
        VALUES ($1, 'roadmap', 'a16207', 'Planning conversations')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("label should insert");
    let pinned_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (
            repository_id, category_id, number, title, body, state, answered,
            author_user_id, comments_count, votes_count, last_activity_at
        )
        VALUES (
            $1, $2, 1, 'How should the roadmap be shaped?', 'Discuss milestones and release scope.',
            'open', true, $3, 2, 5, now() - interval '1 hour'
        )
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(general_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("pinned discussion should insert");
    let closed_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (
            repository_id, category_id, number, title, body, state, locked,
            author_user_id, comments_count, votes_count, last_activity_at
        )
        VALUES (
            $1, $2, 2, 'Archive the old onboarding thread', 'Closed migration discussion.',
            'closed', true, $3, 1, 1, now() - interval '2 days'
        )
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(ideas_id)
    .bind(reader.id)
    .fetch_one(&pool)
    .await
    .expect("closed discussion should insert");
    sqlx::query("INSERT INTO discussion_labels (discussion_id, label_id) VALUES ($1, $2)")
        .bind(pinned_id)
        .bind(label_id)
        .execute(&pool)
        .await
        .expect("discussion label should insert");
    sqlx::query("INSERT INTO discussion_votes (discussion_id, user_id) VALUES ($1, $2), ($1, $3)")
        .bind(pinned_id)
        .bind(owner.id)
        .bind(reader.id)
        .execute(&pool)
        .await
        .expect("votes should insert");
    sqlx::query(
        "INSERT INTO discussion_pins (discussion_id, pinned_by_user_id, position) VALUES ($1, $2, 1)",
    )
    .bind(pinned_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("pin should insert");
    sqlx::query(
        "INSERT INTO discussion_comments (discussion_id, author_user_id, body) VALUES ($1, $2, 'Helpful context'), ($3, $2, 'Follow-up')",
    )
    .bind(pinned_id)
    .bind(reader.id)
    .bind(closed_id)
    .execute(&pool)
    .await
    .expect("comments should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_community_links (repository_id, label, href, kind, position)
        VALUES ($1, 'Code of conduct', '/conduct', 'conduct', 1)
        "#,
    )
    .bind(repository.id)
    .execute(&pool)
    .await
    .expect("community link should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!("/api/repos/{owner_login}/{}/discussions", repository.name);

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert!(!outsider_body.to_string().contains("roadmap"));

    let (reader_status, reader_body) = get_json(
        app.clone(),
        &format!("{base}?q=roadmap&label=roadmap&sort=top"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["enabled"], true);
    assert_eq!(reader_body["viewer"]["canCreate"], false);
    assert_eq!(reader_body["filters"]["query"], "roadmap");
    assert_eq!(reader_body["filters"]["sort"], "top");
    assert_eq!(reader_body["openCount"], 1);
    assert_eq!(reader_body["closedCount"], 1);
    assert_eq!(reader_body["items"].as_array().expect("items").len(), 1);
    assert_eq!(
        reader_body["items"][0]["title"],
        "How should the roadmap be shaped?"
    );
    assert_eq!(reader_body["items"][0]["viewerVoted"], true);
    assert_eq!(reader_body["pinned"].as_array().expect("pins").len(), 1);
    assert_eq!(reader_body["labels"][0]["name"], "roadmap");
    assert_eq!(reader_body["categories"][0]["slug"], "general");
    assert_eq!(reader_body["helpfulContributors"][0]["commentsCount"], 2);
    assert_eq!(reader_body["communityLinks"][0]["label"], "Code of conduct");
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let category_path = format!("{base}/categories/ideas?state=closed&locked=true");
    let (category_status, category_body) =
        get_json(app.clone(), &category_path, Some(&owner_cookie)).await;
    assert_eq!(category_status, StatusCode::OK, "{category_body}");
    assert_eq!(category_body["filters"]["category"], "ideas");
    assert_eq!(category_body["items"][0]["number"], 2);
    assert_eq!(category_body["categories"][1]["active"], true);

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}?sort=primer-blue"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let vote_path = format!("{base}/1/vote");
    let (vote_status, vote_body) =
        request_json(app.clone(), "PUT", &vote_path, Some(&voter_cookie)).await;
    assert_eq!(vote_status, StatusCode::OK, "{vote_body}");
    assert_eq!(vote_body["discussionNumber"], 1);
    assert_eq!(vote_body["viewerVoted"], true);
    assert_eq!(vote_body["votesCount"], 3);
    let (idempotent_status, idempotent_body) =
        request_json(app.clone(), "PUT", &vote_path, Some(&voter_cookie)).await;
    assert_eq!(idempotent_status, StatusCode::OK, "{idempotent_body}");
    assert_eq!(idempotent_body["votesCount"], 3);
    let (unvote_status, unvote_body) =
        request_json(app.clone(), "DELETE", &vote_path, Some(&voter_cookie)).await;
    assert_eq!(unvote_status, StatusCode::OK, "{unvote_body}");
    assert_eq!(unvote_body["viewerVoted"], false);
    assert_eq!(unvote_body["votesCount"], 2);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_activity_events WHERE discussion_id = $1 AND event_type IN ('voted', 'unvoted')",
    )
    .bind(pinned_id)
    .fetch_one(&pool)
    .await
    .expect("vote events should count");
    assert_eq!(event_count, 2);
    let notification_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM notifications WHERE subject_type = 'discussion' AND subject_id = $1 AND reason = 'discussion_vote'",
    )
    .bind(pinned_id)
    .fetch_one(&pool)
    .await
    .expect("vote notifications should count");
    assert_eq!(notification_count, 1);

    let (anonymous_vote_status, anonymous_vote_body) =
        request_json(app.clone(), "PUT", &vote_path, None).await;
    assert_eq!(anonymous_vote_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_vote_body["error"]["code"], "not_authenticated");

    sqlx::query("UPDATE repositories SET is_archived = true WHERE id = $1")
        .bind(repository.id)
        .execute(&pool)
        .await
        .expect("repository should archive");
    let (archived_vote_status, archived_vote_body) =
        request_json(app, "PUT", &vote_path, Some(&voter_cookie)).await;
    assert_eq!(archived_vote_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(archived_vote_body["error"]["code"], "validation_failed");
}
