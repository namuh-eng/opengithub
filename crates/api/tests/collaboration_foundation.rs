use opengithub_api::domain::{
    identity::upsert_user_by_email,
    issues::{
        add_issue_comment, add_issue_reaction, create_issue, ensure_default_labels, issue_timeline,
        list_issues, update_issue_state, CollaborationError, CreateComment, CreateIssue,
        IssueState, ReactionContent, UpdateIssueState,
    },
    permissions::RepositoryRole,
    pulls::{
        create_pull_request, get_pull_request, list_pull_requests, pull_request_timeline,
        update_pull_request_state, CreatePullRequest, PullRequestState, UpdatePullRequestState,
    },
    repositories::{
        create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
        RepositoryVisibility,
    },
};
use sqlx::PgPool;
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
async fn issues_comments_reactions_filters_and_permissions_round_trip() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres collaboration scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        &pool,
        &format!("issue-owner-{unique}@opengithub.local"),
        Some("Issue Owner"),
        None,
    )
    .await
    .expect("owner should upsert");
    let reader = upsert_user_by_email(
        &pool,
        &format!("issue-reader-{unique}@opengithub.local"),
        Some("Issue Reader"),
        None,
    )
    .await
    .expect("reader should upsert");
    let outsider = upsert_user_by_email(
        &pool,
        &format!("issue-outsider-{unique}@opengithub.local"),
        Some("Issue Outsider"),
        None,
    )
    .await
    .expect("outsider should upsert");

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("issues-{unique}"),
            description: None,
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

    let defaults = ensure_default_labels(&pool, repository.id)
        .await
        .expect("default labels should upsert");
    assert!(defaults.iter().any(|label| label.name == "bug"));
    assert!(defaults.iter().all(|label| label.is_default));

    let issue = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Issue list loses filters".to_owned(),
            body: Some("Changing status should preserve assignee filters.".to_owned()),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![defaults[0].id],
            assignee_user_ids: vec![owner.id],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("issue should create");
    assert_eq!(issue.number, 1);
    assert_eq!(issue.state, IssueState::Open);

    let unauthorized = create_issue(
        &pool,
        CreateIssue {
            repository_id: repository.id,
            actor_user_id: outsider.id,
            title: "Unauthorized".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await;
    assert!(
        matches!(
            unauthorized,
            Err(CollaborationError::RepositoryAccessDenied)
        ),
        "users without repository access must not create issues"
    );

    let open = list_issues(
        &pool,
        repository.id,
        owner.id,
        Some(IssueState::Open),
        1,
        10,
    )
    .await
    .expect("open issues should list");
    assert_eq!(open.total, 1);
    assert_eq!(open.items[0].id, issue.id);

    let comment = add_issue_comment(
        &pool,
        issue.id,
        CreateComment {
            actor_user_id: owner.id,
            body: "Confirmed with a reproduction.".to_owned(),
        },
    )
    .await
    .expect("owner should comment");
    assert_eq!(comment.issue_id, Some(issue.id));

    let reaction = add_issue_reaction(&pool, issue.id, reader.id, ReactionContent::ThumbsUp)
        .await
        .expect("readers should react");
    assert_eq!(reaction.content, ReactionContent::ThumbsUp);

    let closed = update_issue_state(
        &pool,
        issue.id,
        UpdateIssueState {
            actor_user_id: owner.id,
            state: IssueState::Closed,
        },
    )
    .await
    .expect("issue should close");
    assert_eq!(closed.state, IssueState::Closed);
    assert_eq!(closed.closed_by_user_id, Some(owner.id));

    let no_open = list_issues(
        &pool,
        repository.id,
        owner.id,
        Some(IssueState::Open),
        1,
        10,
    )
    .await
    .expect("open issues should list after close");
    assert_eq!(no_open.total, 0);
    let closed_list = list_issues(
        &pool,
        repository.id,
        owner.id,
        Some(IssueState::Closed),
        1,
        10,
    )
    .await
    .expect("closed issues should list");
    assert_eq!(closed_list.total, 1);

    let events = issue_timeline(&pool, issue.id, Some(owner.id))
        .await
        .expect("timeline should load");
    let event_types = events
        .into_iter()
        .map(|event| event.event_type)
        .collect::<Vec<_>>();
    assert_eq!(event_types, vec!["opened", "commented", "closed"]);
}

#[tokio::test]
async fn pull_requests_share_issue_numbers_and_timeline_state() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres collaboration scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        &pool,
        &format!("pr-owner-{unique}@opengithub.local"),
        Some("PR Owner"),
        None,
    )
    .await
    .expect("owner should upsert");

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("pulls-{unique}"),
            description: None,
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
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
            title: "Keep the numbering shared".to_owned(),
            body: None,
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: None,
            label_ids: vec![],
            assignee_user_ids: vec![],
            attachments: Vec::new(),
        },
    )
    .await
    .expect("issue should create first");

    let detail = create_pull_request(
        &pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id: owner.id,
            title: "Add collaboration routes".to_owned(),
            body: Some("Implements issues and pulls foundation.".to_owned()),
            head_ref: "feature/collaboration".to_owned(),
            base_ref: "main".to_owned(),
            head_repository_id: None,
            is_draft: false,
            label_ids: vec![],
            milestone_id: None,
            assignee_user_ids: vec![],
            reviewer_user_ids: vec![],
            template_slug: None,
        },
    )
    .await
    .expect("pull request should create");
    assert_eq!(issue.number, 1);
    assert_eq!(detail.pull_request.number, 2);
    assert_eq!(detail.issue.number, 2);
    assert_eq!(detail.pull_request.issue_id, detail.issue.id);

    let open = list_pull_requests(
        &pool,
        repository.id,
        owner.id,
        Some(PullRequestState::Open),
        1,
        10,
    )
    .await
    .expect("open pulls should list");
    assert_eq!(open.total, 1);
    assert_eq!(open.items[0].number, 2);

    let fetched = get_pull_request(&pool, repository.id, 2, owner.id)
        .await
        .expect("pull request should fetch by number");
    assert_eq!(fetched.pull_request.head_ref, "feature/collaboration");

    sqlx::query(
        r#"
        INSERT INTO pull_request_files (pull_request_id, path, status, additions, deletions, byte_size)
        VALUES ($1, 'src/collaboration.rs', 'modified', 6, 1, 128)
        "#,
    )
    .bind(detail.pull_request.id)
    .execute(&pool)
    .await
    .expect("pull request should have diff metadata before merge");

    let merged = update_pull_request_state(
        &pool,
        detail.pull_request.id,
        UpdatePullRequestState {
            actor_user_id: owner.id,
            state: PullRequestState::Merged,
            merge_commit_id: None,
        },
    )
    .await
    .expect("pull request should merge");
    assert_eq!(merged.state, PullRequestState::Merged);
    assert_eq!(merged.merged_by_user_id, Some(owner.id));

    let events = pull_request_timeline(&pool, detail.pull_request.id, Some(owner.id))
        .await
        .expect("pull timeline should load");
    let event_types = events
        .into_iter()
        .map(|event| event.event_type)
        .collect::<Vec<_>>();
    assert_eq!(event_types, vec!["opened", "merged"]);
}
