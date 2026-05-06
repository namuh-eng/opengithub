use chrono::Utc;
use opengithub_api::domain::{
    identity::{upsert_user_by_email, User},
    milestones::{
        create_repository_milestone_by_owner_name, delete_repository_milestone_by_owner_name,
        repository_milestone_detail_for_actor_by_owner_name,
        repository_milestones_for_actor_by_owner_name, update_repository_milestone_by_owner_name,
        update_repository_milestone_state_by_owner_name, MilestoneListState, MilestoneSort,
        RepositoryMilestoneMutation, RepositoryMilestonesQuery,
    },
    permissions::RepositoryRole,
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

    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping repository milestones scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_milestone_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.milestones') IS NOT NULL
               AND to_regclass('public.milestone_events') IS NOT NULL
               AND to_regclass('public.milestone_item_order') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_milestone_tables {
            eprintln!("skipping repository milestones scenario; migration failed: {error}");
            return None;
        }
    }
    Some(pool)
}

#[tokio::test]
async fn repository_milestones_contract_lists_mutates_and_clears_associations() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository milestones contract; set TEST_DATABASE_URL");
        return;
    };

    let owner = create_user(&pool, "milestones-owner").await;
    let writer = create_user(&pool, "milestones-writer").await;
    let reader = create_user(&pool, "milestones-reader").await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("milestones-{}", Uuid::new_v4().simple()),
            description: Some("Milestones contract".to_owned()),
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

    let milestone = create_repository_milestone_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        writer.id,
        RepositoryMilestoneMutation {
            title: "Beta launch".to_owned(),
            description: Some("Ship **core** planning.".to_owned()),
            due_on: Some(Utc::now()),
        },
    )
    .await
    .expect("milestone should create");
    assert_eq!(milestone.title, "Beta launch");
    assert!(milestone.viewer.can_edit_milestones);

    let open_issue_id = insert_issue(
        &pool,
        repository.id,
        owner.id,
        1,
        "Open scope",
        "open",
        milestone.id,
    )
    .await;
    let closed_issue_id = insert_issue(
        &pool,
        repository.id,
        owner.id,
        2,
        "Closed scope",
        "closed",
        milestone.id,
    )
    .await;
    let pr_issue_id = insert_issue(
        &pool,
        repository.id,
        owner.id,
        3,
        "PR scope",
        "open",
        milestone.id,
    )
    .await;
    let pull_request_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, body, author_user_id, head_ref, base_ref
        )
        VALUES ($1, $2, 3, 'PR scope', 'Pull request body', $3, 'feature', 'main')
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(pr_issue_id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("pull request should insert");

    let list = repository_milestones_for_actor_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        Some(reader.id),
        RepositoryMilestonesQuery {
            state: MilestoneListState::Open,
            sort: MilestoneSort::IssuesDesc,
        },
        1,
        30,
    )
    .await
    .expect("milestones should list");
    assert_eq!(list.envelope.items.len(), 1);
    assert_eq!(list.envelope.items[0].progress.open_count, 2);
    assert_eq!(list.envelope.items[0].progress.closed_count, 1);
    assert_eq!(list.envelope.items[0].progress.percent_complete, 33);
    assert!(!list.viewer.can_edit_milestones);

    let detail = repository_milestone_detail_for_actor_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        milestone.id,
        Some(reader.id),
    )
    .await
    .expect("milestone detail should load");
    assert!(detail.description_html.contains("<strong>core</strong>"));
    assert_eq!(detail.items.len(), 3);
    assert!(detail.items.iter().any(|item| item.is_pull_request));

    let updated = update_repository_milestone_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        milestone.id,
        writer.id,
        RepositoryMilestoneMutation {
            title: "Beta launch v2".to_owned(),
            description: Some("Updated description".to_owned()),
            due_on: None,
        },
    )
    .await
    .expect("milestone should update");
    assert_eq!(updated.title, "Beta launch v2");

    let closed = update_repository_milestone_state_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        milestone.id,
        writer.id,
        opengithub_api::domain::issues::IssueState::Closed,
    )
    .await
    .expect("milestone should close");
    assert_eq!(closed.state.as_str(), "closed");

    delete_repository_milestone_by_owner_name(
        &pool,
        &repository.owner_login,
        &repository.name,
        milestone.id,
        writer.id,
    )
    .await
    .expect("milestone should delete");
    let remaining = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM issues WHERE id = ANY($1) AND milestone_id IS NOT NULL",
    )
    .bind(vec![open_issue_id, closed_issue_id, pr_issue_id])
    .fetch_one(&pool)
    .await
    .expect("association count should load");
    assert_eq!(remaining, 0);
    let timeline_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM timeline_events WHERE repository_id = $1 AND event_type = 'metadata_changed'",
    )
    .bind(repository.id)
    .fetch_one(&pool)
    .await
    .expect("timeline count should load");
    assert!(timeline_count >= 3);
    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM audit_events WHERE actor_user_id = $1 AND target_id = $2",
    )
    .bind(writer.id)
    .bind(milestone.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert!(audit_count >= 4);
    assert!(pull_request_id != Uuid::nil());
}

async fn create_user(pool: &PgPool, username: &str) -> User {
    upsert_user_by_email(
        pool,
        &format!("{username}-{}@example.com", Uuid::new_v4().simple()),
        Some(username),
        None,
    )
    .await
    .expect("user should upsert")
}

async fn insert_issue(
    pool: &PgPool,
    repository_id: Uuid,
    author_user_id: Uuid,
    number: i64,
    title: &str,
    state: &str,
    milestone_id: Uuid,
) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id, milestone_id)
        VALUES ($1, $2, $3, 'body', $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .bind(title)
    .bind(state)
    .bind(author_user_id)
    .bind(milestone_id)
    .fetch_one(pool)
    .await
    .expect("issue should insert")
}
