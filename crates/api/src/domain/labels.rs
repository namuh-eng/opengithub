use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    permissions::RepositoryRole,
    repositories::{
        get_repository_by_owner_name, repository_permission_for_user, Repository, RepositoryError,
        RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryLabelSort {
    Name,
    TotalIssueCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryLabelDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LabelViewer {
    pub authenticated: bool,
    pub role: Option<String>,
    pub can_read: bool,
    pub can_write: bool,
    pub can_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelCounts {
    pub open_issues: i64,
    pub open_pull_requests: i64,
    pub discussions: i64,
    pub total_issue_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub counts: RepositoryLabelCounts,
    pub issues_href: String,
    pub pull_requests_href: String,
    pub discussions_href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelsFilters {
    pub query: Option<String>,
    pub sort: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelsView {
    pub items: Vec<RepositoryLabelSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub filters: RepositoryLabelsFilters,
    pub viewer: LabelViewer,
    pub repository: RepositoryLabelRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelRepository {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub visibility: String,
    pub is_archived: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelMutationRequest {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLabelMutationResult {
    pub label: RepositoryLabelSummary,
    pub event_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryLabelsListQuery {
    pub query: Option<String>,
    pub sort: RepositoryLabelSort,
    pub direction: RepositoryLabelDirection,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum LabelsError {
    #[error("repository was not found")]
    RepositoryNotFound,
    #[error("user does not have repository access")]
    RepositoryAccessDenied,
    #[error("label was not found")]
    LabelNotFound,
    #[error("repository is archived")]
    ArchivedRepository,
    #[error("{0}")]
    Validation(String),
    #[error("label already exists")]
    Conflict,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn repository_labels_for_actor_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
    query: RepositoryLabelsListQuery,
) -> Result<RepositoryLabelsView, LabelsError> {
    let repository =
        repository_for_optional_actor(pool, owner, repo, actor_user_id, RepositoryRole::Read)
            .await?;
    let viewer = label_viewer(pool, &repository, actor_user_id).await?;
    let labels = label_summaries(
        pool,
        &repository,
        query.query.as_deref(),
        query.sort,
        query.direction,
    )
    .await?;
    let total = labels.len() as i64;
    let start = ((query.page - 1) * query.page_size).max(0) as usize;
    let end = (start + query.page_size as usize).min(labels.len());
    let items = if start >= labels.len() {
        Vec::new()
    } else {
        labels[start..end].to_vec()
    };

    Ok(RepositoryLabelsView {
        items,
        total,
        page: query.page,
        page_size: query.page_size,
        filters: RepositoryLabelsFilters {
            query: query.query,
            sort: query.sort.as_str().to_owned(),
            direction: query.direction.as_str().to_owned(),
        },
        viewer,
        repository: repository_summary(&repository),
    })
}

pub async fn create_repository_label_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Uuid,
    request: RepositoryLabelMutationRequest,
) -> Result<RepositoryLabelMutationResult, LabelsError> {
    let repository =
        repository_for_actor(pool, owner, repo, actor_user_id, RepositoryRole::Write).await?;
    ensure_not_archived(&repository)?;
    let input = validate_label_request(request, None)?;
    let row = sqlx::query(
        r#"
        INSERT INTO labels (repository_id, name, color, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&input.name)
    .bind(&input.color)
    .bind(&input.description)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_label_error)?;
    let label_id: Uuid = row.get("id");
    let event_id = insert_label_event(
        pool,
        repository.id,
        Some(label_id),
        actor_user_id,
        "label_created",
        None,
        Some(json!({ "name": input.name, "color": input.color, "description": input.description })),
    )
    .await?;
    let label = label_summary_by_id(pool, &repository, label_id).await?;
    Ok(RepositoryLabelMutationResult { label, event_id })
}

pub async fn update_repository_label_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    label_id: Uuid,
    actor_user_id: Uuid,
    request: RepositoryLabelMutationRequest,
) -> Result<RepositoryLabelMutationResult, LabelsError> {
    let repository =
        repository_for_actor(pool, owner, repo, actor_user_id, RepositoryRole::Write).await?;
    ensure_not_archived(&repository)?;
    let before = label_summary_by_id(pool, &repository, label_id).await?;
    let input = validate_label_request(request, Some(label_id))?;
    let updated = sqlx::query(
        r#"
        UPDATE labels
        SET name = $3, color = $4, description = $5
        WHERE id = $1 AND repository_id = $2
        RETURNING id
        "#,
    )
    .bind(label_id)
    .bind(repository.id)
    .bind(&input.name)
    .bind(&input.color)
    .bind(&input.description)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_label_error)?;
    if updated.is_none() {
        return Err(LabelsError::LabelNotFound);
    }
    let after = label_summary_by_id(pool, &repository, label_id).await?;
    let event_id = insert_label_event(
        pool,
        repository.id,
        Some(label_id),
        actor_user_id,
        "label_updated",
        Some(json!(&before)),
        Some(json!(&after)),
    )
    .await?;
    Ok(RepositoryLabelMutationResult {
        label: after,
        event_id,
    })
}

pub async fn delete_repository_label_by_owner_name(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    label_id: Uuid,
    actor_user_id: Uuid,
) -> Result<RepositoryLabelMutationResult, LabelsError> {
    let repository =
        repository_for_actor(pool, owner, repo, actor_user_id, RepositoryRole::Write).await?;
    ensure_not_archived(&repository)?;
    let before = label_summary_by_id(pool, &repository, label_id).await?;
    let deleted = sqlx::query("DELETE FROM labels WHERE id = $1 AND repository_id = $2")
        .bind(label_id)
        .bind(repository.id)
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(LabelsError::LabelNotFound);
    }
    let event_id = insert_label_event(
        pool,
        repository.id,
        None,
        actor_user_id,
        "label_deleted",
        Some(json!(&before)),
        None,
    )
    .await?;
    Ok(RepositoryLabelMutationResult {
        label: before,
        event_id,
    })
}

impl RepositoryLabelSort {
    pub fn parse(value: Option<&str>) -> Result<Self, LabelsError> {
        match value.unwrap_or("name") {
            "name" => Ok(Self::Name),
            "total_issue_count" | "totalIssueCount" | "count" => Ok(Self::TotalIssueCount),
            other => Err(LabelsError::Validation(format!(
                "unsupported label sort `{other}`"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::TotalIssueCount => "total_issue_count",
        }
    }
}

impl RepositoryLabelDirection {
    pub fn parse(value: Option<&str>) -> Result<Self, LabelsError> {
        match value.unwrap_or("asc") {
            "asc" | "ascending" => Ok(Self::Asc),
            "desc" | "descending" => Ok(Self::Desc),
            other => Err(LabelsError::Validation(format!(
                "unsupported label direction `{other}`"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

impl From<RepositoryLabelsView> for ListEnvelope<RepositoryLabelSummary> {
    fn from(view: RepositoryLabelsView) -> Self {
        Self {
            items: view.items,
            total: view.total,
            page: view.page,
            page_size: view.page_size,
        }
    }
}

#[derive(Debug)]
struct ValidatedLabelRequest {
    name: String,
    color: String,
    description: Option<String>,
}

async fn repository_for_optional_actor(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
    required_role: RepositoryRole,
) -> Result<Repository, LabelsError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await
        .map_err(map_repository_error)?
        .ok_or(LabelsError::RepositoryNotFound)?;
    match actor_user_id {
        Some(user_id) => {
            require_repository_role(pool, &repository, user_id, required_role).await?;
        }
        None if required_role == RepositoryRole::Read
            && repository.visibility == RepositoryVisibility::Public => {}
        None => return Err(LabelsError::RepositoryAccessDenied),
    }
    Ok(repository)
}

async fn repository_for_actor(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Repository, LabelsError> {
    repository_for_optional_actor(pool, owner, repo, Some(actor_user_id), required_role).await
}

async fn require_repository_role(
    pool: &PgPool,
    repository: &Repository,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<String, LabelsError> {
    let permission = repository_permission_for_user(pool, repository.id, user_id)
        .await
        .map_err(map_repository_error)?;
    let Some(permission) = permission else {
        if required_role == RepositoryRole::Read
            && repository.visibility == RepositoryVisibility::Public
        {
            return Ok("read".to_owned());
        }
        return Err(LabelsError::RepositoryAccessDenied);
    };
    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Triage => permission.role >= RepositoryRole::Triage,
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Maintain => permission.role >= RepositoryRole::Maintain,
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };
    if allowed {
        Ok(permission.role.as_str().to_owned())
    } else {
        Err(LabelsError::RepositoryAccessDenied)
    }
}

async fn label_viewer(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<LabelViewer, LabelsError> {
    let Some(user_id) = actor_user_id else {
        return Ok(LabelViewer {
            authenticated: false,
            role: None,
            can_read: repository.visibility == RepositoryVisibility::Public,
            can_write: false,
            can_admin: false,
        });
    };
    let role = require_repository_role(pool, repository, user_id, RepositoryRole::Read).await?;
    let parsed = RepositoryRole::try_from(role.as_str()).ok();
    Ok(LabelViewer {
        authenticated: true,
        role: Some(role),
        can_read: true,
        can_write: parsed.map(|role| role.can_write()).unwrap_or(false),
        can_admin: parsed.map(|role| role.can_admin()).unwrap_or(false),
    })
}

async fn label_summaries(
    pool: &PgPool,
    repository: &Repository,
    query: Option<&str>,
    sort: RepositoryLabelSort,
    direction: RepositoryLabelDirection,
) -> Result<Vec<RepositoryLabelSummary>, LabelsError> {
    let search = query.map(str::trim).filter(|value| !value.is_empty());
    let rows = sqlx::query(
        r#"
        WITH label_counts AS (
            SELECT labels.id AS label_id,
                   COUNT(DISTINCT issues.id) FILTER (
                       WHERE issues.state = 'open'
                         AND pull_requests.id IS NULL
                   ) AS open_issues,
                   COUNT(DISTINCT pull_requests.id) FILTER (
                       WHERE pull_requests.state = 'open'
                   ) AS open_pull_requests,
                   COUNT(DISTINCT discussions.id) AS discussions
            FROM labels
            LEFT JOIN issue_labels ON issue_labels.label_id = labels.id
            LEFT JOIN issues ON issues.id = issue_labels.issue_id
            LEFT JOIN pull_requests ON pull_requests.issue_id = issues.id
            LEFT JOIN discussion_labels ON discussion_labels.label_id = labels.id
            LEFT JOIN discussions ON discussions.id = discussion_labels.discussion_id
            WHERE labels.repository_id = $1
            GROUP BY labels.id
        )
        SELECT labels.id, labels.name, labels.color, labels.description, labels.is_default,
               labels.created_at, labels.updated_at,
               COALESCE(label_counts.open_issues, 0)::bigint AS open_issues,
               COALESCE(label_counts.open_pull_requests, 0)::bigint AS open_pull_requests,
               COALESCE(label_counts.discussions, 0)::bigint AS discussions
        FROM labels
        LEFT JOIN label_counts ON label_counts.label_id = labels.id
        WHERE labels.repository_id = $1
          AND (
              $2::text IS NULL
              OR labels.name ILIKE '%' || $2 || '%'
              OR COALESCE(labels.description, '') ILIKE '%' || $2 || '%'
          )
        "#,
    )
    .bind(repository.id)
    .bind(search)
    .fetch_all(pool)
    .await?;

    let mut labels = rows
        .into_iter()
        .map(|row| label_summary_from_row(repository, row))
        .collect::<Result<Vec<_>, _>>()?;
    labels.sort_by(|left, right| {
        let ordering = match sort {
            RepositoryLabelSort::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            RepositoryLabelSort::TotalIssueCount => left
                .counts
                .total_issue_count
                .cmp(&right.counts.total_issue_count)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())),
        };
        match direction {
            RepositoryLabelDirection::Asc => ordering,
            RepositoryLabelDirection::Desc => ordering.reverse(),
        }
    });
    Ok(labels)
}

async fn label_summary_by_id(
    pool: &PgPool,
    repository: &Repository,
    label_id: Uuid,
) -> Result<RepositoryLabelSummary, LabelsError> {
    let labels = label_summaries(
        pool,
        repository,
        None,
        RepositoryLabelSort::Name,
        RepositoryLabelDirection::Asc,
    )
    .await?;
    labels
        .into_iter()
        .find(|label| label.id == label_id)
        .ok_or(LabelsError::LabelNotFound)
}

fn label_summary_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
) -> Result<RepositoryLabelSummary, LabelsError> {
    let name: String = row.get("name");
    let encoded = query_escape(&name);
    let open_issues = row.get("open_issues");
    let open_pull_requests = row.get("open_pull_requests");
    let discussions = row.get("discussions");
    Ok(RepositoryLabelSummary {
        id: row.get("id"),
        name,
        color: row.get("color"),
        description: row.get("description"),
        is_default: row.get("is_default"),
        counts: RepositoryLabelCounts {
            open_issues,
            open_pull_requests,
            discussions,
            total_issue_count: open_issues + open_pull_requests,
        },
        issues_href: format!(
            "/{}/{}/issues?q=is%3Aopen%20label%3A{}",
            repository.owner_login, repository.name, encoded
        ),
        pull_requests_href: format!(
            "/{}/{}/pulls?q=is%3Aopen%20is%3Apr%20label%3A{}",
            repository.owner_login, repository.name, encoded
        ),
        discussions_href: format!(
            "/{}/{}/discussions?label={}",
            repository.owner_login, repository.name, encoded
        ),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn query_escape(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn validate_label_request(
    request: RepositoryLabelMutationRequest,
    _label_id: Option<Uuid>,
) -> Result<ValidatedLabelRequest, LabelsError> {
    let name = request.name.trim().to_owned();
    if name.is_empty() {
        return Err(LabelsError::Validation("label name is required".to_owned()));
    }
    if name.len() > 100 {
        return Err(LabelsError::Validation(
            "label name must be 100 characters or fewer".to_owned(),
        ));
    }
    let color = request
        .color
        .trim()
        .trim_start_matches('#')
        .to_ascii_lowercase();
    if color.len() != 6 || !color.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(LabelsError::Validation(
            "label color must be a six-character hex value".to_owned(),
        ));
    }
    let description = request.description.and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    if description.as_ref().is_some_and(|value| value.len() > 240) {
        return Err(LabelsError::Validation(
            "label description must be 240 characters or fewer".to_owned(),
        ));
    }
    Ok(ValidatedLabelRequest {
        name,
        color,
        description,
    })
}

fn ensure_not_archived(repository: &Repository) -> Result<(), LabelsError> {
    if repository.is_archived {
        Err(LabelsError::ArchivedRepository)
    } else {
        Ok(())
    }
}

async fn insert_label_event(
    pool: &PgPool,
    repository_id: Uuid,
    label_id: Option<Uuid>,
    actor_user_id: Uuid,
    event_type: &str,
    before_state: Option<serde_json::Value>,
    after_state: Option<serde_json::Value>,
) -> Result<Uuid, LabelsError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repository_label_events (
            repository_id, label_id, actor_user_id, event_type, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(label_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(before_state)
    .bind(after_state)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

fn repository_summary(repository: &Repository) -> RepositoryLabelRepository {
    RepositoryLabelRepository {
        id: repository.id,
        owner: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        is_archived: repository.is_archived,
    }
}

fn map_repository_error(error: RepositoryError) -> LabelsError {
    match error {
        RepositoryError::Sqlx(error) => LabelsError::Sqlx(error),
        RepositoryError::PermissionDenied | RepositoryError::OwnerPermissionDenied => {
            LabelsError::RepositoryAccessDenied
        }
        _ => LabelsError::RepositoryNotFound,
    }
}

fn map_sqlx_label_error(error: sqlx::Error) -> LabelsError {
    match &error {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            LabelsError::Conflict
        }
        _ => LabelsError::Sqlx(error),
    }
}
