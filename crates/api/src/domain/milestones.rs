use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    issues::{append_timeline_event, IssueState},
    markdown::{render_markdown, RenderMarkdownInput},
    permissions::RepositoryRole,
    repositories::{
        get_repository_by_owner_name, repository_permission_for_user, Repository,
        RepositoryVisibility,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum MilestonesError {
    #[error("repository was not found")]
    RepositoryNotFound,
    #[error("milestone was not found")]
    MilestoneNotFound,
    #[error("user does not have repository access")]
    RepositoryAccessDenied,
    #[error("archived repositories cannot change milestones")]
    ArchivedRepository,
    #[error("{0}")]
    Validation(String),
    #[error("milestone title already exists")]
    Conflict,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneListState {
    #[default]
    Open,
    Closed,
    All,
}

impl MilestoneListState {
    fn as_filter(&self) -> Option<&'static str> {
        match self {
            Self::Open => Some("open"),
            Self::Closed => Some("closed"),
            Self::All => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MilestoneSort {
    #[default]
    UpdatedDesc,
    DueDesc,
    DueAsc,
    CompleteAsc,
    CompleteDesc,
    AlphaAsc,
    AlphaDesc,
    IssuesDesc,
    IssuesAsc,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMilestonesQuery {
    pub state: MilestoneListState,
    pub sort: MilestoneSort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMilestoneMutation {
    pub title: String,
    pub description: Option<String>,
    pub due_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneViewer {
    pub permission: Option<String>,
    pub can_edit_milestones: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRepositorySummary {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub is_archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneProgress {
    pub open_count: i64,
    pub closed_count: i64,
    pub total_count: i64,
    pub percent_complete: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMilestoneSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub state: IssueState,
    pub due_on: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: MilestoneProgress,
    pub open_issues_href: String,
    pub closed_issues_href: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMilestonesView {
    #[serde(flatten)]
    pub envelope: ListEnvelope<RepositoryMilestoneSummary>,
    pub open_count: i64,
    pub closed_count: i64,
    pub filters: RepositoryMilestonesQuery,
    pub viewer: MilestoneViewer,
    pub repository: MilestoneRepositorySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneIssueItem {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub state: IssueState,
    pub is_pull_request: bool,
    pub href: String,
    pub comment_count: i64,
    pub label_names: Vec<String>,
    pub assignee_logins: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMilestoneDetail {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub description_html: String,
    pub state: IssueState,
    pub due_on: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: MilestoneProgress,
    pub items: Vec<MilestoneIssueItem>,
    pub viewer: MilestoneViewer,
    pub repository: MilestoneRepositorySummary,
    pub href: String,
}

pub async fn repository_milestones_for_actor_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
    query: RepositoryMilestonesQuery,
    page: i64,
    page_size: i64,
) -> Result<RepositoryMilestonesView, MilestonesError> {
    let repository = repository_for_optional_actor(pool, owner, repo, actor_user_id).await?;
    let viewer = milestone_viewer(pool, &repository, actor_user_id).await?;
    let offset = (page.saturating_sub(1)) * page_size;
    let state_filter = query.state.as_filter();

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM milestones WHERE repository_id = $1 AND ($2::text IS NULL OR state = $2)",
    )
    .bind(repository.id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;
    let open_count = milestone_state_count(pool, repository.id, "open").await?;
    let closed_count = milestone_state_count(pool, repository.id, "closed").await?;

    let rows = milestone_rows(
        pool,
        repository.id,
        state_filter,
        &query.sort,
        page_size,
        offset,
    )
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(summary_from_row(row, &repository)?);
    }

    Ok(RepositoryMilestonesView {
        envelope: ListEnvelope {
            items,
            total,
            page,
            page_size,
        },
        open_count,
        closed_count,
        filters: query,
        viewer,
        repository: repository_summary(&repository),
    })
}

pub async fn repository_milestone_detail_for_actor_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    milestone_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryMilestoneDetail, MilestonesError> {
    let repository = repository_for_optional_actor(pool, owner, repo, actor_user_id).await?;
    let viewer = milestone_viewer(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(&milestone_select_sql("milestones.id = $2"))
        .bind(repository.id)
        .bind(milestone_id)
        .fetch_optional(pool)
        .await?
        .ok_or(MilestonesError::MilestoneNotFound)?;
    let summary = summary_from_row(row, &repository)?;
    let description_html = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: None,
            markdown: summary.description.clone().unwrap_or_default(),
            enable_task_toggles: None,
        },
    )
    .await
    .map_err(|error| match error {
        super::markdown::MarkdownError::Sqlx(error) => MilestonesError::Sqlx(error),
        super::markdown::MarkdownError::TooLarge => {
            MilestonesError::Validation("milestone description is too large".to_owned())
        }
        super::markdown::MarkdownError::TaskNotFound => {
            MilestonesError::Validation("milestone task item was not found".to_owned())
        }
    })?
    .html;

    Ok(RepositoryMilestoneDetail {
        items: milestone_items(pool, &repository, milestone_id).await?,
        viewer,
        repository: repository_summary(&repository),
        id: summary.id,
        title: summary.title,
        description: summary.description,
        description_html,
        state: summary.state,
        due_on: summary.due_on,
        closed_at: summary.closed_at,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
        progress: summary.progress,
        href: summary.href,
    })
}

pub async fn create_repository_milestone_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Uuid,
    input: RepositoryMilestoneMutation,
) -> Result<RepositoryMilestoneDetail, MilestonesError> {
    let repository = repository_for_writer(pool, owner, repo, actor_user_id).await?;
    let input = validate_milestone_input(input)?;
    let row = sqlx::query(
        r#"
        INSERT INTO milestones (repository_id, title, description, due_on, created_by_user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.due_on)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await
    .map_err(map_unique_title)?;
    let milestone_id: Uuid = row.get("id");
    insert_milestone_event(
        pool,
        repository.id,
        Some(milestone_id),
        Some(actor_user_id),
        "created",
        json!({ "title": input.title, "dueOn": input.due_on }),
    )
    .await?;
    insert_audit_event(
        pool,
        actor_user_id,
        "repository.milestone.create",
        "milestone",
        milestone_id,
        json!({ "repositoryId": repository.id }),
    )
    .await?;
    repository_milestone_detail_for_actor_by_owner_name(
        pool,
        owner,
        repo,
        milestone_id,
        Some(actor_user_id),
    )
    .await
}

pub async fn update_repository_milestone_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    milestone_id: Uuid,
    actor_user_id: Uuid,
    input: RepositoryMilestoneMutation,
) -> Result<RepositoryMilestoneDetail, MilestonesError> {
    let repository = repository_for_writer(pool, owner, repo, actor_user_id).await?;
    ensure_milestone_belongs(pool, repository.id, milestone_id).await?;
    let input = validate_milestone_input(input)?;
    sqlx::query(
        r#"
        UPDATE milestones
        SET title = $3, description = $4, due_on = $5
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(milestone_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.due_on)
    .execute(pool)
    .await
    .map_err(map_unique_title)?;
    insert_milestone_event(
        pool,
        repository.id,
        Some(milestone_id),
        Some(actor_user_id),
        "edited",
        json!({ "title": input.title, "dueOn": input.due_on }),
    )
    .await?;
    insert_audit_event(
        pool,
        actor_user_id,
        "repository.milestone.update",
        "milestone",
        milestone_id,
        json!({ "repositoryId": repository.id }),
    )
    .await?;
    repository_milestone_detail_for_actor_by_owner_name(
        pool,
        owner,
        repo,
        milestone_id,
        Some(actor_user_id),
    )
    .await
}

pub async fn update_repository_milestone_state_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    milestone_id: Uuid,
    actor_user_id: Uuid,
    state: IssueState,
) -> Result<RepositoryMilestoneDetail, MilestonesError> {
    let repository = repository_for_writer(pool, owner, repo, actor_user_id).await?;
    ensure_milestone_belongs(pool, repository.id, milestone_id).await?;
    let closed_at = matches!(state, IssueState::Closed).then(Utc::now);
    sqlx::query(
        "UPDATE milestones SET state = $3, closed_at = $4 WHERE repository_id = $1 AND id = $2",
    )
    .bind(repository.id)
    .bind(milestone_id)
    .bind(state.as_str())
    .bind(closed_at)
    .execute(pool)
    .await?;
    let event_type = if matches!(state, IssueState::Closed) {
        "closed"
    } else {
        "reopened"
    };
    insert_milestone_event(
        pool,
        repository.id,
        Some(milestone_id),
        Some(actor_user_id),
        event_type,
        json!({ "state": state.as_str() }),
    )
    .await?;
    insert_audit_event(
        pool,
        actor_user_id,
        &format!("repository.milestone.{event_type}"),
        "milestone",
        milestone_id,
        json!({ "repositoryId": repository.id }),
    )
    .await?;
    repository_milestone_detail_for_actor_by_owner_name(
        pool,
        owner,
        repo,
        milestone_id,
        Some(actor_user_id),
    )
    .await
}

pub async fn delete_repository_milestone_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    milestone_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), MilestonesError> {
    let repository = repository_for_writer(pool, owner, repo, actor_user_id).await?;
    ensure_milestone_belongs(pool, repository.id, milestone_id).await?;
    let issue_rows = sqlx::query(
        r#"
        SELECT issues.id, pull_requests.id AS pull_request_id
        FROM issues
        LEFT JOIN pull_requests ON pull_requests.issue_id = issues.id
        WHERE issues.repository_id = $1 AND issues.milestone_id = $2
        "#,
    )
    .bind(repository.id)
    .bind(milestone_id)
    .fetch_all(pool)
    .await?;

    sqlx::query(
        "UPDATE issues SET milestone_id = NULL WHERE repository_id = $1 AND milestone_id = $2",
    )
    .bind(repository.id)
    .bind(milestone_id)
    .execute(pool)
    .await?;
    for row in issue_rows {
        let issue_id: Uuid = row.get("id");
        let pull_request_id: Option<Uuid> = row.get("pull_request_id");
        append_timeline_event(
            pool,
            repository.id,
            if pull_request_id.is_none() {
                Some(issue_id)
            } else {
                None
            },
            pull_request_id,
            Some(actor_user_id),
            "metadata_changed",
            json!({
                "milestoneId": serde_json::Value::Null,
                "removedMilestoneId": milestone_id,
            }),
        )
        .await
        .map_err(|error| match error {
            super::issues::CollaborationError::Sqlx(error) => MilestonesError::Sqlx(error),
            other => MilestonesError::Validation(other.to_string()),
        })?;
    }
    sqlx::query("DELETE FROM milestones WHERE repository_id = $1 AND id = $2")
        .bind(repository.id)
        .bind(milestone_id)
        .execute(pool)
        .await?;
    insert_milestone_event(
        pool,
        repository.id,
        None,
        Some(actor_user_id),
        "deleted",
        json!({ "milestoneId": milestone_id }),
    )
    .await?;
    insert_audit_event(
        pool,
        actor_user_id,
        "repository.milestone.delete",
        "milestone",
        milestone_id,
        json!({ "repositoryId": repository.id }),
    )
    .await?;
    Ok(())
}

async fn repository_for_optional_actor(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, MilestonesError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => MilestonesError::Sqlx(error),
            _ => MilestonesError::RepositoryNotFound,
        })?
        .ok_or(MilestonesError::RepositoryNotFound)?;
    match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, &repository, user_id, RepositoryRole::Read).await?;
        }
        None if repository.visibility == RepositoryVisibility::Public => {}
        None => return Err(MilestonesError::RepositoryNotFound),
    }
    Ok(repository)
}

async fn repository_for_writer(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Uuid,
) -> Result<Repository, MilestonesError> {
    let repository = repository_for_optional_actor(pool, owner, repo, Some(actor_user_id)).await?;
    if repository.is_archived {
        return Err(MilestonesError::ArchivedRepository);
    }
    repository_viewer_permission(pool, &repository, actor_user_id, RepositoryRole::Write).await?;
    Ok(repository)
}

async fn repository_viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Option<String>, MilestonesError> {
    let permission = repository_permission_for_user(pool, repository.id, user_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => MilestonesError::Sqlx(error),
            _ => MilestonesError::RepositoryAccessDenied,
        })?;
    let Some(permission) = permission else {
        if required_role == RepositoryRole::Read
            && repository.visibility == RepositoryVisibility::Public
        {
            return Ok(Some("read".to_owned()));
        }
        return Err(MilestonesError::RepositoryAccessDenied);
    };
    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Triage => permission.role >= RepositoryRole::Triage,
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Maintain => permission.role >= RepositoryRole::Maintain,
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };
    allowed
        .then(|| Some(permission.role.as_str().to_owned()))
        .ok_or(MilestonesError::RepositoryAccessDenied)
}

async fn milestone_viewer(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<MilestoneViewer, MilestonesError> {
    let permission = match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, repository, user_id, RepositoryRole::Read).await?
        }
        None => {
            if repository.visibility == RepositoryVisibility::Public {
                Some("read".to_owned())
            } else {
                None
            }
        }
    };
    let can_edit_milestones = permission
        .as_deref()
        .and_then(|value| RepositoryRole::try_from(value).ok())
        .is_some_and(RepositoryRole::can_write)
        && !repository.is_archived;
    Ok(MilestoneViewer {
        permission,
        can_edit_milestones,
    })
}

fn validate_milestone_input(
    input: RepositoryMilestoneMutation,
) -> Result<RepositoryMilestoneMutation, MilestonesError> {
    let title = input.title.trim().to_owned();
    if title.is_empty() {
        return Err(MilestonesError::Validation(
            "milestone title is required".to_owned(),
        ));
    }
    if title.chars().count() > 120 {
        return Err(MilestonesError::Validation(
            "milestone title must be 120 characters or fewer".to_owned(),
        ));
    }
    let description = input
        .description
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if description
        .as_deref()
        .is_some_and(|value| value.chars().count() > 16_384)
    {
        return Err(MilestonesError::Validation(
            "milestone description must be 16384 characters or fewer".to_owned(),
        ));
    }
    Ok(RepositoryMilestoneMutation {
        title,
        description,
        due_on: input.due_on,
    })
}

async fn ensure_milestone_belongs(
    pool: &PgPool,
    repository_id: Uuid,
    milestone_id: Uuid,
) -> Result<(), MilestonesError> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM milestones WHERE repository_id = $1 AND id = $2)",
    )
    .bind(repository_id)
    .bind(milestone_id)
    .fetch_one(pool)
    .await?;
    exists
        .then_some(())
        .ok_or(MilestonesError::MilestoneNotFound)
}

async fn milestone_state_count(
    pool: &PgPool,
    repository_id: Uuid,
    state: &str,
) -> Result<i64, MilestonesError> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM milestones WHERE repository_id = $1 AND state = $2",
    )
    .bind(repository_id)
    .bind(state)
    .fetch_one(pool)
    .await
    .map_err(MilestonesError::Sqlx)
}

async fn milestone_rows(
    pool: &PgPool,
    repository_id: Uuid,
    state_filter: Option<&str>,
    sort: &MilestoneSort,
    limit: i64,
    offset: i64,
) -> Result<Vec<sqlx::postgres::PgRow>, MilestonesError> {
    let order_by = match sort {
        MilestoneSort::UpdatedDesc => "milestones.updated_at DESC, milestones.id DESC",
        MilestoneSort::DueDesc => "milestones.due_on DESC NULLS LAST, milestones.updated_at DESC",
        MilestoneSort::DueAsc => "milestones.due_on ASC NULLS LAST, milestones.updated_at DESC",
        MilestoneSort::CompleteAsc => "percent_complete ASC, milestones.updated_at DESC",
        MilestoneSort::CompleteDesc => "percent_complete DESC, milestones.updated_at DESC",
        MilestoneSort::AlphaAsc => "lower(milestones.title) ASC",
        MilestoneSort::AlphaDesc => "lower(milestones.title) DESC",
        MilestoneSort::IssuesDesc => "total_count DESC, milestones.updated_at DESC",
        MilestoneSort::IssuesAsc => "total_count ASC, milestones.updated_at DESC",
    };
    let sql = format!(
        "{} AND ($2::text IS NULL OR milestones.state = $2) ORDER BY {order_by} LIMIT $3 OFFSET $4",
        milestone_select_sql("milestones.repository_id = $1")
    );
    sqlx::query(&sql)
        .bind(repository_id)
        .bind(state_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(MilestonesError::Sqlx)
}

fn milestone_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT milestones.id, milestones.title, milestones.description, milestones.state,
               milestones.due_on, milestones.closed_at, milestones.created_at, milestones.updated_at,
               COALESCE(count(issues.id) FILTER (WHERE issues.state = 'open'), 0)::bigint AS open_count,
               COALESCE(count(issues.id) FILTER (WHERE issues.state = 'closed'), 0)::bigint AS closed_count,
               COALESCE(count(issues.id), 0)::bigint AS total_count,
               CASE WHEN count(issues.id) = 0 THEN 0
                    ELSE floor((count(issues.id) FILTER (WHERE issues.state = 'closed')::numeric / count(issues.id)::numeric) * 100)::bigint
               END AS percent_complete
        FROM milestones
        LEFT JOIN issues ON issues.milestone_id = milestones.id
        WHERE {where_clause}
        GROUP BY milestones.id
        "#
    )
}

fn summary_from_row(
    row: sqlx::postgres::PgRow,
    repository: &Repository,
) -> Result<RepositoryMilestoneSummary, MilestonesError> {
    let id: Uuid = row.get("id");
    let title: String = row.get("title");
    let state = IssueState::try_from(row.get::<String, _>("state").as_str())
        .map_err(|_| MilestonesError::Validation("invalid milestone state".to_owned()))?;
    Ok(RepositoryMilestoneSummary {
        id,
        title: title.clone(),
        description: row.get("description"),
        state,
        due_on: row.get("due_on"),
        closed_at: row.get("closed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        progress: MilestoneProgress {
            open_count: row.get("open_count"),
            closed_count: row.get("closed_count"),
            total_count: row.get("total_count"),
            percent_complete: row.get("percent_complete"),
        },
        open_issues_href: format!(
            "/{}/{}/issues?q=milestone:%22{}%22+state:open",
            repository.owner_login, repository.name, title
        ),
        closed_issues_href: format!(
            "/{}/{}/issues?q=milestone:%22{}%22+state:closed",
            repository.owner_login, repository.name, title
        ),
        href: format!(
            "/{}/{}/milestones/{}",
            repository.owner_login, repository.name, id
        ),
    })
}

async fn milestone_items(
    pool: &PgPool,
    repository: &Repository,
    milestone_id: Uuid,
) -> Result<Vec<MilestoneIssueItem>, MilestonesError> {
    let rows = sqlx::query(
        r#"
        SELECT issues.id, issues.number, issues.title, issues.state, issues.updated_at,
               pull_requests.id AS pull_request_id,
               COALESCE(count(DISTINCT comments.id), 0)::bigint AS comment_count,
               COALESCE(array_remove(array_agg(DISTINCT labels.name ORDER BY labels.name), NULL), ARRAY[]::text[]) AS label_names,
               COALESCE(array_remove(array_agg(DISTINCT users.username ORDER BY users.username), NULL), ARRAY[]::text[]) AS assignee_logins
        FROM issues
        LEFT JOIN pull_requests ON pull_requests.issue_id = issues.id
        LEFT JOIN comments ON comments.issue_id = issues.id
        LEFT JOIN issue_labels ON issue_labels.issue_id = issues.id
        LEFT JOIN labels ON labels.id = issue_labels.label_id
        LEFT JOIN issue_assignees ON issue_assignees.issue_id = issues.id
        LEFT JOIN users ON users.id = issue_assignees.user_id
        WHERE issues.repository_id = $1 AND issues.milestone_id = $2
        GROUP BY issues.id, pull_requests.id
        ORDER BY issues.state ASC, issues.updated_at DESC, issues.number DESC
        "#
    )
    .bind(repository.id)
    .bind(milestone_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let number: i64 = row.get("number");
            let is_pull_request = row.get::<Option<Uuid>, _>("pull_request_id").is_some();
            MilestoneIssueItem {
                id: row.get("id"),
                number,
                title: row.get("title"),
                state: IssueState::try_from(row.get::<String, _>("state").as_str())
                    .unwrap_or_default(),
                is_pull_request,
                href: format!(
                    "/{}/{}/{}/{}",
                    repository.owner_login,
                    repository.name,
                    if is_pull_request { "pull" } else { "issues" },
                    number
                ),
                comment_count: row.get("comment_count"),
                label_names: row.get("label_names"),
                assignee_logins: row.get("assignee_logins"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

fn repository_summary(repository: &Repository) -> MilestoneRepositorySummary {
    MilestoneRepositorySummary {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.clone(),
        is_archived: repository.is_archived,
    }
}

async fn insert_milestone_event(
    pool: &PgPool,
    repository_id: Uuid,
    milestone_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
    event_type: &str,
    metadata: serde_json::Value,
) -> Result<(), MilestonesError> {
    sqlx::query(
        "INSERT INTO milestone_events (repository_id, milestone_id, actor_user_id, event_type, metadata) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(repository_id)
    .bind(milestone_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_audit_event(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    target_type: &str,
    target_id: Uuid,
    metadata: serde_json::Value,
) -> Result<(), MilestonesError> {
    sqlx::query(
        "INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_type)
    .bind(target_id)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

fn map_unique_title(error: sqlx::Error) -> MilestonesError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.constraint() == Some("milestones_repo_title_lower_unique") {
            return MilestonesError::Conflict;
        }
    }
    MilestonesError::Sqlx(error)
}
