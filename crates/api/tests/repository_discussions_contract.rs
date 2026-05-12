use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        discussions::{
            repository_discussion_detail_for_actor_by_owner_name, RepositoryDiscussionDetailQuery,
        },
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
               AND to_regclass('public.discussion_poll_votes') IS NOT NULL
               AND to_regclass('public.discussion_reactions') IS NOT NULL
               AND to_regclass('public.discussion_answers') IS NOT NULL
               AND EXISTS(
                   SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'public'
                     AND table_name = 'discussions'
                     AND column_name = 'lock_allows_reactions'
               )
               AND EXISTS(
                   SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'public'
                     AND table_name = 'discussion_polls'
                     AND column_name = 'allows_vote_changes'
               )
               AND EXISTS(
                   SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'public'
                     AND table_name = 'discussion_pins'
                     AND column_name = 'pin_scope'
               )
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
async fn repository_discussion_poll_vote_api_enforces_options_and_reconciles_results() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussion poll vote scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-poll-owner").await;
    let voter = create_user(&pool, "discussion-poll-voter").await;
    let outsider = create_user(&pool, "discussion-poll-outsider").await;
    let voter_cookie = cookie_header(&pool, &config, &voter).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussion-poll-vote-{}", Uuid::new_v4().simple()),
            description: Some("Discussion poll voting contract".to_owned()),
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
        voter.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("voter permission should grant");

    let category_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (
            repository_id, slug, name, emoji, description, position, format, accepts_answers
        )
        VALUES ($1, 'polls', 'Polls', '📊', 'Vote on repository choices.', 1, 'poll', false)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("poll category should insert");
    let discussion_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (repository_id, category_id, number, title, body, author_user_id)
        VALUES ($1, $2, 12, 'Pick the release train', 'Choose one path.', $3)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(category_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("discussion should insert");
    let poll_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_polls (discussion_id, question, allows_multiple)
        VALUES ($1, 'Which train should ship first?', false)
        RETURNING id
        "#,
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("poll should insert");
    let first_option_id: Uuid = sqlx::query_scalar(
        "INSERT INTO discussion_poll_options (poll_id, position, label) VALUES ($1, 0, 'Stable') RETURNING id",
    )
    .bind(poll_id)
    .fetch_one(&pool)
    .await
    .expect("first option should insert");
    let second_option_id: Uuid = sqlx::query_scalar(
        "INSERT INTO discussion_poll_options (poll_id, position, label) VALUES ($1, 1, 'Preview') RETURNING id",
    )
    .bind(poll_id)
    .fetch_one(&pool)
    .await
    .expect("second option should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let vote_path = format!(
        "/api/repos/{owner_login}/{}/discussions/12/poll/vote",
        repository.name
    );

    let (anonymous_status, anonymous_body) = put_json(
        app.clone(),
        &vote_path,
        None,
        json!({ "optionIds": [first_option_id] }),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) = put_json(
        app.clone(),
        &vote_path,
        Some(&outsider_cookie),
        json!({ "optionIds": [first_option_id] }),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::FORBIDDEN);
    assert!(!outsider_body.to_string().contains("release train"));

    let (invalid_status, invalid_body) = put_json(
        app.clone(),
        &vote_path,
        Some(&voter_cookie),
        json!({ "optionIds": [Uuid::new_v4()] }),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (vote_status, vote_body) = put_json(
        app.clone(),
        &vote_path,
        Some(&voter_cookie),
        json!({ "optionIds": [first_option_id] }),
    )
    .await;
    assert_eq!(vote_status, StatusCode::OK, "{vote_body}");
    assert_eq!(vote_body["discussionNumber"], 12);
    assert_eq!(vote_body["changed"], true);
    assert_eq!(vote_body["poll"]["totalVotes"], 1);
    assert_eq!(
        vote_body["poll"]["viewerVoteOptionIds"][0],
        first_option_id.to_string()
    );
    assert_eq!(vote_body["poll"]["options"][0]["votesCount"], 1);
    assert_eq!(vote_body["poll"]["options"][0]["percentage"], 100);

    let (idempotent_status, idempotent_body) = put_json(
        app.clone(),
        &vote_path,
        Some(&voter_cookie),
        json!({ "optionIds": [first_option_id] }),
    )
    .await;
    assert_eq!(idempotent_status, StatusCode::OK, "{idempotent_body}");
    assert_eq!(idempotent_body["changed"], false);
    assert_eq!(idempotent_body["poll"]["totalVotes"], 1);

    let (replace_status, replace_body) = put_json(
        app.clone(),
        &vote_path,
        Some(&voter_cookie),
        json!({ "optionIds": [second_option_id] }),
    )
    .await;
    assert_eq!(replace_status, StatusCode::OK, "{replace_body}");
    assert_eq!(replace_body["poll"]["totalVotes"], 1);
    assert_eq!(
        replace_body["poll"]["viewerVoteOptionIds"][0],
        second_option_id.to_string()
    );
    assert_eq!(replace_body["poll"]["options"][0]["votesCount"], 0);
    assert_eq!(replace_body["poll"]["options"][1]["votesCount"], 1);

    let active_vote_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_poll_votes WHERE poll_id = $1 AND user_id = $2 AND replaced_at IS NULL",
    )
    .bind(poll_id)
    .bind(voter.id)
    .fetch_one(&pool)
    .await
    .expect("active poll votes should count");
    assert_eq!(active_vote_count, 1);
    let replaced_vote_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_poll_votes WHERE poll_id = $1 AND user_id = $2 AND replaced_at IS NOT NULL",
    )
    .bind(poll_id)
    .bind(voter.id)
    .fetch_one(&pool)
    .await
    .expect("replaced poll votes should count");
    assert_eq!(replaced_vote_count, 1);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_activity_events WHERE discussion_id = $1 AND event_type = 'poll_voted'",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("poll vote events should count");
    assert_eq!(event_count, 2);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM audit_events WHERE actor_user_id = $1 AND event_type = 'repository.discussion_poll.vote'",
    )
    .bind(voter.id)
    .fetch_one(&pool)
    .await
    .expect("poll vote audit rows should count");
    assert_eq!(audit_count, 2);

    sqlx::query("UPDATE discussions SET locked = true WHERE id = $1")
        .bind(discussion_id)
        .execute(&pool)
        .await
        .expect("discussion should lock");
    let (locked_status, locked_body) = put_json(
        app,
        &vote_path,
        Some(&voter_cookie),
        json!({ "optionIds": [first_option_id] }),
    )
    .await;
    assert_eq!(locked_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(locked_body["error"]["code"], "validation_failed");
    assert!(!locked_body.to_string().contains("test-session-secret"));
}

#[tokio::test]
async fn repository_discussion_moderation_supports_pin_lock_state_and_category_contracts() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussion moderation scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-moderation-owner").await;
    let moderator = create_user(&pool, "discussion-moderation-moderator").await;
    let reader = create_user(&pool, "discussion-moderation-reader").await;
    let moderator_cookie = cookie_header(&pool, &config, &moderator).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussion-moderation-{}", Uuid::new_v4().simple()),
            description: Some("Discussion moderation contract".to_owned()),
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
        moderator.id,
        RepositoryRole::Triage,
        "direct",
    )
    .await
    .expect("moderator permission should grant");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");
    let general_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, position, format)
        VALUES ($1, 'general', 'General', '💬', 1, 'open_ended')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("general category should insert");
    let ideas_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, position, format)
        VALUES ($1, 'ideas', 'Ideas', '💡', 2, 'question_and_answer')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("ideas category should insert");
    let discussion_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussions (repository_id, category_id, number, title, body, author_user_id)
        VALUES ($1, $2, 1, 'Moderate me', 'Needs careful handling.', $3)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(general_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("discussion should insert");
    let needs_docs_label_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO labels (repository_id, name, color, description)
        VALUES ($1, 'needs-docs', 'b46838', 'Needs documentation')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("needs docs label should insert");
    let imports_label_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO labels (repository_id, name, color, description)
        VALUES ($1, 'imports', '8b5cf6', 'Import workflows')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("imports label should insert");
    sqlx::query("INSERT INTO discussion_labels (discussion_id, label_id) VALUES ($1, $2)")
        .bind(discussion_id)
        .bind(needs_docs_label_id)
        .execute(&pool)
        .await
        .expect("initial discussion label should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!("/api/repos/{owner_login}/{}/discussions/1", repository.name);

    let (reader_pin_status, reader_pin_body) = put_json(
        app.clone(),
        &format!("{base}/pin"),
        Some(&reader_cookie),
        json!({ "target": "global" }),
    )
    .await;
    assert_eq!(
        reader_pin_status,
        StatusCode::FORBIDDEN,
        "{reader_pin_body}"
    );
    assert!(!reader_pin_body.to_string().contains("test-session-secret"));

    let (pin_status, pin_body) = put_json(
        app.clone(),
        &format!("{base}/pin"),
        Some(&moderator_cookie),
        json!({
            "target": "global",
            "title": "Read this first",
            "body": "Pinned for maintainers and contributors."
        }),
    )
    .await;
    assert_eq!(pin_status, StatusCode::OK, "{pin_body}");
    assert_eq!(pin_body["discussion"]["number"], 1);
    let global_pin_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_pins WHERE discussion_id = $1 AND pin_scope = 'global' AND custom_title = 'Read this first'",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("global pin should count");
    assert_eq!(global_pin_count, 1);

    let (category_mismatch_status, category_mismatch_body) = put_json(
        app.clone(),
        &format!("{base}/pin"),
        Some(&moderator_cookie),
        json!({ "target": "category", "categorySlug": "ideas" }),
    )
    .await;
    assert_eq!(category_mismatch_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(category_mismatch_body["error"]["code"], "validation_failed");

    let (lock_status, lock_body) = put_json(
        app.clone(),
        &format!("{base}/lock"),
        Some(&moderator_cookie),
        json!({ "allowReactions": false }),
    )
    .await;
    assert_eq!(lock_status, StatusCode::OK, "{lock_body}");
    assert_eq!(lock_body["discussion"]["locked"], true);
    let (reaction_status, reaction_body) = put_json(
        app.clone(),
        &format!("{base}/reactions"),
        Some(&reader_cookie),
        json!({ "content": "heart" }),
    )
    .await;
    assert_eq!(reaction_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(reaction_body["error"]["message"]
        .as_str()
        .expect("reaction error")
        .contains("locked discussions"));

    let (unlock_status, unlock_body) = delete_json(
        app.clone(),
        &format!("{base}/lock"),
        Some(&moderator_cookie),
        json!({}),
    )
    .await;
    assert_eq!(unlock_status, StatusCode::OK, "{unlock_body}");
    assert_eq!(unlock_body["discussion"]["locked"], false);

    let (close_status, close_body) = put_json(
        app.clone(),
        &format!("{base}/state"),
        Some(&moderator_cookie),
        json!({ "state": "closed", "reason": "duplicate" }),
    )
    .await;
    assert_eq!(close_status, StatusCode::OK, "{close_body}");
    assert_eq!(close_body["discussion"]["state"], "closed");
    let closed_reason: Option<String> =
        sqlx::query_scalar("SELECT closed_reason FROM discussions WHERE id = $1")
            .bind(discussion_id)
            .fetch_one(&pool)
            .await
            .expect("closed reason should load");
    assert_eq!(closed_reason.as_deref(), Some("duplicate"));

    let (category_status, category_body) = patch_json(
        app.clone(),
        &format!("{base}/category"),
        Some(&moderator_cookie),
        json!({ "categorySlug": "ideas" }),
    )
    .await;
    assert_eq!(category_status, StatusCode::OK, "{category_body}");
    assert_eq!(category_body["category"]["slug"], "ideas");
    let moved_category_id: Uuid = sqlx::query_scalar(
        "SELECT category_id FROM discussions WHERE repository_id = $1 AND number = 1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("discussion category should load");
    assert_eq!(moved_category_id, ideas_id);

    let (labels_status, labels_body) = patch_json(
        app.clone(),
        &format!("{base}/metadata"),
        Some(&moderator_cookie),
        json!({ "labelIds": [imports_label_id] }),
    )
    .await;
    assert_eq!(labels_status, StatusCode::OK, "{labels_body}");
    assert_eq!(labels_body["labels"][0]["name"], "imports");
    assert_eq!(labels_body["sidebar"]["labelOptions"][0]["name"], "imports");
    let active_label_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_labels WHERE discussion_id = $1 AND label_id = $2",
    )
    .bind(discussion_id)
    .bind(imports_label_id)
    .fetch_one(&pool)
    .await
    .expect("active label should count");
    assert_eq!(active_label_count, 1);
    let removed_label_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_labels WHERE discussion_id = $1 AND label_id = $2",
    )
    .bind(discussion_id)
    .bind(needs_docs_label_id)
    .fetch_one(&pool)
    .await
    .expect("removed label should count");
    assert_eq!(removed_label_count, 0);
    let label_event_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM discussion_activity_events
        WHERE discussion_id = $1 AND event_type = 'labels_changed'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("label event should load");
    assert_eq!(label_event_payload["labelIds"], json!([imports_label_id]));
    assert_eq!(
        label_event_payload["removedLabelIds"],
        json!([needs_docs_label_id])
    );
    let label_audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM audit_events
        WHERE actor_user_id = $1
          AND target_type = 'repository_discussion'
          AND event_type = 'repository.discussion.labels.update'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(moderator.id)
    .fetch_one(&pool)
    .await
    .expect("label audit should load");
    assert_eq!(
        label_audit_metadata["addedLabelIds"],
        json!([imports_label_id])
    );

    let (unpin_status, unpin_body) = delete_json(
        app.clone(),
        &format!("{base}/pin"),
        Some(&moderator_cookie),
        json!({}),
    )
    .await;
    assert_eq!(unpin_status, StatusCode::OK, "{unpin_body}");
    let pin_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM discussion_pins WHERE discussion_id = $1")
            .bind(discussion_id)
            .fetch_one(&pool)
            .await
            .expect("pins should count");
    assert_eq!(pin_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM audit_events
        WHERE actor_user_id = $1
          AND target_type = 'repository_discussion'
          AND event_type IN (
              'repository.discussion.pin',
              'repository.discussion.lock',
              'repository.discussion.unlock',
              'repository.discussion.close',
              'repository.discussion.labels.update',
              'repository.discussion.unpin'
          )
        "#,
    )
    .bind(moderator.id)
    .fetch_one(&pool)
    .await
    .expect("audit rows should count");
    assert_eq!(audit_count, 6);
    let event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM discussion_activity_events
        WHERE discussion_id = $1
          AND event_type IN ('pinned', 'locked', 'unlocked', 'closed', 'category_changed', 'labels_changed', 'unpinned')
        "#,
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("activity rows should count");
    assert_eq!(event_count, 7);
    let notification_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM notifications WHERE subject_type = 'discussion' AND subject_id = $1",
    )
    .bind(discussion_id)
    .fetch_one(&pool)
    .await
    .expect("notifications should count");
    assert!(notification_count >= 2);
    assert!(!category_body.to_string().contains("test-session-secret"));
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
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position, accepts_answers, format)
        VALUES ($1, 'polls', 'Polls', '📊', 'Vote on options.', 2, false, 'poll')
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

    let poll_number = poll_body["discussionNumber"]
        .as_i64()
        .expect("poll number should exist");
    let direct_poll_detail = repository_discussion_detail_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        poll_number,
        RepositoryDiscussionDetailQuery {
            sort: None,
            page: None,
            page_size: None,
        },
    )
    .await
    .expect("poll detail domain should load")
    .expect("poll detail should exist");
    assert_eq!(
        direct_poll_detail.discussion.title,
        "Choose a default branch policy"
    );
    let (poll_detail_status, poll_detail_body) = get_json(
        app.clone(),
        &format!("{base}/{poll_number}"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(poll_detail_status, StatusCode::OK, "{poll_detail_body}");
    assert_eq!(
        poll_detail_body["discussion"]["title"],
        "Choose a default branch policy"
    );
    assert_eq!(
        poll_detail_body["poll"]["options"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
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

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("PATCH")
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

async fn put_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("PUT")
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

async fn delete_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("DELETE")
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
async fn repository_discussion_category_settings_support_admin_create_and_edit() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository discussion category admin scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-category-owner").await;
    let reader = create_user(&pool, "discussion-category-reader").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("discussion-category-admin-{}", Uuid::new_v4().simple()),
            description: Some("Discussion category admin contract".to_owned()),
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
    let section_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_category_sections (repository_id, name, position)
        VALUES ($1, 'Community', 1)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("section should insert");
    let general_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (
            repository_id, section_id, slug, name, emoji, description, position,
            format, accepts_answers, is_default
        )
        VALUES ($1, $2, 'general', 'General', '💬', 'Open-ended discussion.', 1, 'open_ended', false, true)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(section_id)
    .fetch_one(&pool)
    .await
    .expect("category should insert");
    sqlx::query(
        r#"
        INSERT INTO discussions (repository_id, category_id, number, title, body, author_user_id)
        VALUES ($1, $2, 1, 'Welcome', 'Introduce yourself.', $3)
        "#,
    )
    .bind(repository.id)
    .bind(general_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("discussion should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let path = format!(
        "/api/repos/{owner_login}/{}/settings/discussions/categories",
        repository.name
    );

    let (reader_status, reader_body) = get_json(app.clone(), &path, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::FORBIDDEN, "{reader_body}");
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let (get_status, get_body) = get_json(app.clone(), &path, Some(&owner_cookie)).await;
    assert_eq!(get_status, StatusCode::OK, "{get_body}");
    assert_eq!(get_body["viewer"]["canManage"], true);
    assert_eq!(get_body["categoryLimit"], 25);
    assert_eq!(get_body["remainingCategories"], 24);
    assert_eq!(get_body["sections"][0]["name"], "Community");
    assert_eq!(get_body["categories"][0]["slug"], "general");
    assert_eq!(get_body["categories"][0]["format"], "open_ended");
    assert_eq!(get_body["categories"][0]["count"], 1);

    let (create_status, create_body) = post_json(
        app.clone(),
        &path,
        Some(&owner_cookie),
        json!({
            "name": "Q&A",
            "emoji": "❓",
            "description": "Questions with accepted answers.",
            "format": "question_and_answer",
            "sectionId": section_id,
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK, "{create_body}");
    let created = create_body["categories"]
        .as_array()
        .expect("categories")
        .iter()
        .find(|category| category["slug"] == "q-a")
        .expect("created category should be returned");
    assert_eq!(created["acceptsAnswers"], true);
    assert_eq!(created["sectionName"], "Community");
    let category_id = created["id"].as_str().expect("category id");

    let (duplicate_status, duplicate_body) = post_json(
        app.clone(),
        &path,
        Some(&owner_cookie),
        json!({
            "name": "q&a",
            "emoji": "❓",
            "format": "question_and_answer",
        }),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(duplicate_body["error"]["code"], "validation_failed");

    let (update_status, update_body) = patch_json(
        app.clone(),
        &format!("{path}/{category_id}"),
        Some(&owner_cookie),
        json!({
            "name": "Announcements",
            "emoji": "📣",
            "description": "Maintainer updates.",
            "format": "announcement",
            "sectionId": null,
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{update_body}");
    let updated = update_body["categories"]
        .as_array()
        .expect("categories")
        .iter()
        .find(|category| category["id"] == category_id)
        .expect("updated category should be returned");
    assert_eq!(updated["name"], "Announcements");
    assert_eq!(updated["format"], "announcement");
    assert_eq!(updated["acceptsAnswers"], false);
    assert_eq!(updated["sectionId"], Value::Null);

    let audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM audit_events
        WHERE actor_user_id = $1
          AND target_type = 'repository_discussion_category'
          AND event_type IN ('repository.discussion_category.create', 'repository.discussion_category.update')
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("audit events should count");
    assert_eq!(audit_count, 2);

    let (missing_status, missing_body) = patch_json(
        app,
        &format!("{path}/{}", Uuid::new_v4()),
        Some(&owner_cookie),
        json!({ "name": "Missing" }),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error"]["code"], "not_found");
}

#[tokio::test]
async fn repository_discussion_category_settings_support_sections_order_and_delete_move() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping repository discussion category restructuring scenario; set TEST_DATABASE_URL"
        );
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "discussion-restructure-owner").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!(
                "discussion-category-restructure-{}",
                Uuid::new_v4().simple()
            ),
            description: Some("Discussion category restructuring contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    let general_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, position, format)
        VALUES ($1, 'general', 'General', '💬', 1, 'open_ended')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("general category should insert");
    let ideas_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, position, format)
        VALUES ($1, 'ideas', 'Ideas', '💡', 2, 'question_and_answer')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("ideas category should insert");
    sqlx::query(
        r#"
        INSERT INTO discussions (repository_id, category_id, number, title, body, author_user_id)
        VALUES ($1, $2, 1, 'Welcome', 'Introduce yourself.', $3)
        "#,
    )
    .bind(repository.id)
    .bind(general_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("discussion should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!(
        "/api/repos/{owner_login}/{}/settings/discussions",
        repository.name
    );

    let (section_status, section_body) = post_json(
        app.clone(),
        &format!("{base}/sections"),
        Some(&owner_cookie),
        json!({ "name": "Community" }),
    )
    .await;
    assert_eq!(section_status, StatusCode::OK, "{section_body}");
    let section_id = section_body["sections"][0]["id"]
        .as_str()
        .expect("section id");

    let (rename_status, rename_body) = patch_json(
        app.clone(),
        &format!("{base}/sections/{section_id}"),
        Some(&owner_cookie),
        json!({ "name": "Support" }),
    )
    .await;
    assert_eq!(rename_status, StatusCode::OK, "{rename_body}");
    assert_eq!(rename_body["sections"][0]["name"], "Support");

    let (order_status, order_body) = put_json(
        app.clone(),
        &format!("{base}/categories/order"),
        Some(&owner_cookie),
        json!({
            "items": [
                { "id": ideas_id, "sectionId": section_id, "position": 1 },
                { "id": general_id, "sectionId": null, "position": 2 }
            ]
        }),
    )
    .await;
    assert_eq!(order_status, StatusCode::OK, "{order_body}");
    let moved = order_body["categories"]
        .as_array()
        .expect("categories")
        .iter()
        .find(|category| category["id"] == ideas_id.to_string())
        .expect("ideas category");
    assert_eq!(moved["sectionName"], "Support");
    assert_eq!(moved["position"], 1);

    let (delete_missing_destination_status, delete_missing_destination_body) = delete_json(
        app.clone(),
        &format!("{base}/categories/{general_id}"),
        Some(&owner_cookie),
        json!({}),
    )
    .await;
    assert_eq!(
        delete_missing_destination_status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        delete_missing_destination_body["error"]["code"],
        "validation_failed"
    );

    let (delete_status, delete_body) = delete_json(
        app.clone(),
        &format!("{base}/categories/{general_id}"),
        Some(&owner_cookie),
        json!({ "moveToCategoryId": ideas_id }),
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK, "{delete_body}");
    assert_eq!(
        delete_body["categories"]
            .as_array()
            .expect("categories")
            .len(),
        1
    );
    let migrated_category_id: Uuid = sqlx::query_scalar(
        "SELECT category_id FROM discussions WHERE repository_id = $1 AND number = 1",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("discussion should remain");
    assert_eq!(migrated_category_id, ideas_id);

    let (last_delete_status, last_delete_body) = delete_json(
        app.clone(),
        &format!("{base}/categories/{ideas_id}"),
        Some(&owner_cookie),
        json!({}),
    )
    .await;
    assert_eq!(last_delete_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(last_delete_body["error"]["code"], "validation_failed");

    let audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM audit_events
        WHERE actor_user_id = $1
          AND event_type IN (
            'repository.discussion_category_section.create',
            'repository.discussion_category_section.update',
            'repository.discussion_category.reorder',
            'repository.discussion_category.delete'
          )
        "#,
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("audit events should count");
    assert_eq!(audit_count, 4);
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
    assert!(
        reader_body["labels"]
            .as_array()
            .expect("labels")
            .iter()
            .any(|label| label["name"] == "roadmap"),
        "roadmap label should be available in the select panel: {reader_body}"
    );
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
