use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    permissions::RepositoryRole,
    repositories::{
        get_repository, get_repository_by_owner_name, repository_permission_for_user, Repository,
        RepositoryError, RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Active,
    Disabled,
}

impl WorkflowState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for WorkflowState {
    type Error = AutomationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            other => Err(AutomationError::InvalidWorkflowState(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
    Cancelled,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for RunStatus {
    type Error = AutomationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(AutomationError::InvalidRunStatus(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
}

impl RunConclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::TimedOut => "timed_out",
        }
    }
}

impl TryFrom<&str> for RunConclusion {
    type Error = AutomationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "cancelled" => Ok(Self::Cancelled),
            "skipped" => Ok(Self::Skipped),
            "timed_out" => Ok(Self::TimedOut),
            other => Err(AutomationError::InvalidRunConclusion(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    Container,
    Npm,
    Maven,
    Generic,
}

impl PackageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Container => "container",
            Self::Npm => "npm",
            Self::Maven => "maven",
            Self::Generic => "generic",
        }
    }
}

impl TryFrom<&str> for PackageType {
    type Error = AutomationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "container" => Ok(Self::Container),
            "npm" => Ok(Self::Npm),
            "maven" => Ok(Self::Maven),
            "generic" => Ok(Self::Generic),
            other => Err(AutomationError::InvalidPackageType(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionsWorkflow {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub path: String,
    pub state: WorkflowState,
    pub trigger_events: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub workflow_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub run_number: i64,
    pub status: RunStatus,
    pub conclusion: Option<RunConclusion>,
    pub head_branch: String,
    pub head_sha: Option<String>,
    pub event: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsDashboard {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub workflows: Vec<ActionsWorkflowRailItem>,
    pub runs: ListEnvelope<ActionsRunListItem>,
    pub filters: ActionsRunFilters,
    pub filter_options: ActionsRunFilterOptions,
    pub empty_state: ActionsEmptyState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowDetail {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub workflow: ActionsWorkflowDetailWorkflow,
    pub workflows: Vec<ActionsWorkflowRailItem>,
    pub runs: ListEnvelope<ActionsRunListItem>,
    pub filters: ActionsRunFilters,
    pub filter_options: ActionsRunFilterOptions,
    pub refs: Vec<ActionsWorkflowRef>,
    pub empty_state: ActionsEmptyState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsDashboardRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowDetailWorkflow {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub state: WorkflowState,
    pub trigger_events: Vec<String>,
    pub source_branch: String,
    pub source_sha: Option<String>,
    pub source_blob_id: Option<Uuid>,
    pub source_href: String,
    pub dispatch: WorkflowDispatchSpec,
    pub yaml_parse_error: Option<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDispatchSpec {
    pub enabled: bool,
    pub inputs: Vec<WorkflowDispatchInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDispatchInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowRef {
    pub name: String,
    pub short_name: String,
    pub kind: String,
    pub sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowRailItem {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub state: WorkflowState,
    pub trigger_events: Vec<String>,
    pub pinned: bool,
    pub run_count: i64,
    pub latest_run: Option<ActionsWorkflowLatestRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowLatestRun {
    pub id: Uuid,
    pub run_number: i64,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunListItem {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_path: String,
    pub run_number: i64,
    pub display_title: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub status_category: String,
    pub event: String,
    pub actor: Option<ActionsActor>,
    pub head_branch: String,
    pub head_sha: Option<String>,
    pub short_sha: Option<String>,
    pub pull_request: Option<ActionsRunPullRequest>,
    pub commit_message: Option<String>,
    pub job_summary: ActionsJobSummary,
    pub duration_seconds: Option<i64>,
    pub is_live: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsActor {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunPullRequest {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobSummary {
    pub total: i64,
    pub queued: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub success: i64,
    pub failure: i64,
    pub skipped: i64,
    pub timed_out: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunFilters {
    pub q: Option<String>,
    pub workflow: Option<String>,
    pub event: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub actor: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunFilterOptions {
    pub workflows: Vec<ActionsFilterOption>,
    pub events: Vec<ActionsFilterOption>,
    pub statuses: Vec<ActionsFilterOption>,
    pub branches: Vec<ActionsFilterOption>,
    pub actors: Vec<ActionsFilterOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsFilterOption {
    pub value: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsEmptyState {
    pub has_workflows: bool,
    pub has_runs: bool,
    pub message: String,
    pub new_workflow_href: String,
}

#[derive(Debug, Clone, Default)]
pub struct ActionsDashboardQuery {
    pub q: Option<String>,
    pub workflow: Option<String>,
    pub event: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub actor: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ActionsWorkflowDetailQuery {
    pub q: Option<String>,
    pub event: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub actor: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordActionsRecentView {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub workflow: Option<String>,
    pub q: Option<String>,
    pub event: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub actor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRecentView {
    pub repository_id: Uuid,
    pub user_id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub filters: Value,
    pub viewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowJob {
    pub id: Uuid,
    pub run_id: Uuid,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<RunConclusion>,
    pub runner_label: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub job_id: Uuid,
    pub number: i32,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<RunConclusion>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub package_type: PackageType,
    pub visibility: String,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageVersion {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub manifest: Value,
    pub blob_key: Option<String>,
    pub size_bytes: Option<i64>,
    pub published_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflow {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub name: String,
    pub path: String,
    pub trigger_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowRun {
    pub workflow_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub head_branch: String,
    pub head_sha: Option<String>,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRun {
    pub status: RunStatus,
    pub conclusion: Option<RunConclusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowJob {
    pub run_id: Uuid,
    pub name: String,
    pub runner_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowStep {
    pub job_id: Uuid,
    pub number: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePackage {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub name: String,
    pub package_type: PackageType,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePackageVersion {
    pub package_id: Uuid,
    pub actor_user_id: Uuid,
    pub version: String,
    pub manifest: Value,
    pub blob_key: Option<String>,
    pub size_bytes: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum AutomationError {
    #[error("repository was not found")]
    RepositoryNotFound,
    #[error("user does not have repository access")]
    RepositoryAccessDenied,
    #[error("workflow was not found")]
    WorkflowNotFound,
    #[error("workflow run was not found")]
    WorkflowRunNotFound,
    #[error("workflow job was not found")]
    WorkflowJobNotFound,
    #[error("package was not found")]
    PackageNotFound,
    #[error("invalid workflow state `{0}`")]
    InvalidWorkflowState(String),
    #[error("invalid run status `{0}`")]
    InvalidRunStatus(String),
    #[error("invalid run conclusion `{0}`")]
    InvalidRunConclusion(String),
    #[error("invalid package type `{0}`")]
    InvalidPackageType(String),
    #[error("invalid actions filter `{0}`")]
    InvalidActionsFilter(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn actions_dashboard_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    query: ActionsDashboardQuery,
) -> Result<ActionsDashboard, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let viewer_permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let q = cleaned_filter(query.q);
    let workflow = cleaned_filter(query.workflow);
    let event = cleaned_filter(query.event);
    let status = cleaned_filter(query.status).map(normalize_actions_status);
    let branch = cleaned_filter(query.branch);
    let actor = cleaned_filter(query.actor);
    if let Some(status) = status.as_deref() {
        if !ACTIONS_STATUS_OPTIONS.contains(&status) {
            return Err(AutomationError::InvalidActionsFilter(format!(
                "unsupported status `{status}`"
            )));
        }
    }
    let offset = (page - 1) * page_size;
    let run_filters = ActionsRunFilterRefs {
        repository_id,
        q: q.as_deref(),
        workflow: workflow.as_deref(),
        event: event.as_deref(),
        status: status.as_deref(),
        branch: branch.as_deref(),
        actor: actor.as_deref(),
    };

    let workflows = actions_workflow_rail(pool, repository_id).await?;
    let total = actions_run_count(pool, run_filters).await?;
    let mut runs = actions_run_items(pool, run_filters, page_size, offset).await?;
    hydrate_job_summaries(pool, &mut runs).await?;
    let filter_options = actions_filter_options(pool, repository_id).await?;
    let has_workflows = !workflows.is_empty();
    let has_runs = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM workflow_runs WHERE repository_id = $1)",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;

    Ok(ActionsDashboard {
        repository: ActionsDashboardRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.clone(),
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission,
        workflows,
        runs: ListEnvelope {
            items: runs,
            total,
            page,
            page_size,
        },
        filters: ActionsRunFilters {
            q,
            workflow,
            event,
            status,
            branch,
            actor,
            page,
            page_size,
        },
        filter_options,
        empty_state: ActionsEmptyState {
            has_workflows,
            has_runs,
            message: if has_workflows {
                "No workflow runs match the current filters.".to_owned()
            } else {
                "This repository does not have any workflows yet.".to_owned()
            },
            new_workflow_href: format!(
                "/{}/{}/new/{}/.github/workflows",
                repository.owner_login, repository.name, repository.default_branch
            ),
        },
    })
}

pub async fn actions_workflow_detail_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    workflow_path: &str,
    query: ActionsWorkflowDetailQuery,
) -> Result<ActionsWorkflowDetail, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let viewer_permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let workflow = actions_workflow_detail_workflow(pool, &repository, workflow_path).await?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let q = cleaned_filter(query.q);
    let event = cleaned_filter(query.event);
    let status = cleaned_filter(query.status).map(normalize_actions_status);
    let branch = cleaned_filter(query.branch);
    let actor = cleaned_filter(query.actor);
    if let Some(status) = status.as_deref() {
        if !ACTIONS_STATUS_OPTIONS.contains(&status) {
            return Err(AutomationError::InvalidActionsFilter(format!(
                "unsupported status `{status}`"
            )));
        }
    }
    let offset = (page - 1) * page_size;
    let workflow_id_filter = workflow.id.to_string();
    let run_filters = ActionsRunFilterRefs {
        repository_id,
        q: q.as_deref(),
        workflow: Some(workflow_id_filter.as_str()),
        event: event.as_deref(),
        status: status.as_deref(),
        branch: branch.as_deref(),
        actor: actor.as_deref(),
    };

    let workflows = actions_workflow_rail(pool, repository_id).await?;
    let total = actions_run_count(pool, run_filters).await?;
    let mut runs = actions_run_items(pool, run_filters, page_size, offset).await?;
    hydrate_job_summaries(pool, &mut runs).await?;
    let mut filter_options =
        actions_filter_options_for_workflow(pool, repository_id, workflow.id).await?;
    filter_options.workflows = Vec::new();
    let refs = actions_workflow_refs(pool, repository_id).await?;
    let has_runs = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM workflow_runs WHERE repository_id = $1 AND workflow_id = $2)",
    )
    .bind(repository_id)
    .bind(workflow.id)
    .fetch_one(pool)
    .await?;

    Ok(ActionsWorkflowDetail {
        repository: ActionsDashboardRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.clone(),
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission,
        workflow,
        workflows,
        runs: ListEnvelope {
            items: runs,
            total,
            page,
            page_size,
        },
        filters: ActionsRunFilters {
            q,
            workflow: None,
            event,
            status,
            branch,
            actor,
            page,
            page_size,
        },
        filter_options,
        refs,
        empty_state: ActionsEmptyState {
            has_workflows: true,
            has_runs,
            message: if has_runs {
                "No runs for this workflow match the current filters.".to_owned()
            } else {
                "This workflow has not run yet.".to_owned()
            },
            new_workflow_href: format!(
                "/{}/{}/new/{}/.github/workflows",
                repository.owner_login, repository.name, repository.default_branch
            ),
        },
    })
}

pub async fn record_actions_recent_view(
    pool: &PgPool,
    input: RecordActionsRecentView,
) -> Result<ActionsRecentView, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Read,
    )
    .await?;
    let workflow_id = match cleaned_filter(input.workflow) {
        Some(workflow) => {
            Some(resolve_workflow_filter(pool, input.repository_id, &workflow).await?)
        }
        None => None,
    };
    let status = cleaned_filter(input.status).map(normalize_actions_status);
    if let Some(status) = status.as_deref() {
        if !ACTIONS_STATUS_OPTIONS.contains(&status) {
            return Err(AutomationError::InvalidActionsFilter(format!(
                "unsupported status `{status}`"
            )));
        }
    }
    let filters = json!({
        "q": cleaned_filter(input.q),
        "workflow": workflow_id.map(|id| id.to_string()),
        "event": cleaned_filter(input.event),
        "status": status,
        "branch": cleaned_filter(input.branch),
        "actor": cleaned_filter(input.actor),
    });
    let row = sqlx::query(
        r#"
        INSERT INTO actions_recent_views (repository_id, user_id, workflow_id, filters)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET workflow_id = EXCLUDED.workflow_id,
                      filters = EXCLUDED.filters,
                      viewed_at = now()
        RETURNING repository_id, user_id, workflow_id, filters, viewed_at
        "#,
    )
    .bind(input.repository_id)
    .bind(input.actor_user_id)
    .bind(workflow_id)
    .bind(filters)
    .fetch_one(pool)
    .await?;

    Ok(ActionsRecentView {
        repository_id: row.get("repository_id"),
        user_id: row.get("user_id"),
        workflow_id: row.get("workflow_id"),
        filters: row.get("filters"),
        viewed_at: row.get("viewed_at"),
    })
}

pub async fn create_workflow(
    pool: &PgPool,
    input: CreateWorkflow,
) -> Result<ActionsWorkflow, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let row = sqlx::query(
        r#"
        INSERT INTO actions_workflows (repository_id, name, path, trigger_events)
        VALUES ($1, $2, $3, $4)
        RETURNING id, repository_id, name, path, state, trigger_events, created_at, updated_at
        "#,
    )
    .bind(input.repository_id)
    .bind(&input.name)
    .bind(&input.path)
    .bind(&input.trigger_events)
    .fetch_one(pool)
    .await?;

    workflow_from_row(row)
}

pub async fn list_workflows(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<ActionsWorkflow>, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM actions_workflows WHERE repository_id = $1",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, name, path, state, trigger_events, created_at, updated_at
        FROM actions_workflows
        WHERE repository_id = $1
        ORDER BY lower(name), path
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(repository_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let items = rows
        .into_iter()
        .map(workflow_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_workflow_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ActionsWorkflow, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let workflow = get_workflow(pool, workflow_id).await?;
    if workflow.repository_id != repository_id {
        return Err(AutomationError::WorkflowNotFound);
    }
    Ok(workflow)
}

pub async fn create_workflow_run(
    pool: &PgPool,
    input: CreateWorkflowRun,
) -> Result<WorkflowRun, AutomationError> {
    let workflow = get_workflow(pool, input.workflow_id).await?;
    if let Some(actor_user_id) = input.actor_user_id {
        require_repository_role(
            pool,
            workflow.repository_id,
            actor_user_id,
            RepositoryRole::Write,
        )
        .await?;
    }
    let run_number = next_run_number(pool, input.workflow_id).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, head_branch, head_sha, event
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
                  head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(workflow.repository_id)
    .bind(input.workflow_id)
    .bind(input.actor_user_id)
    .bind(run_number)
    .bind(&input.head_branch)
    .bind(&input.head_sha)
    .bind(&input.event)
    .fetch_one(pool)
    .await?;

    workflow_run_from_row(row)
}

pub async fn list_workflow_runs(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Option<Uuid>,
    actor_user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<WorkflowRun>, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM workflow_runs
        WHERE repository_id = $1
          AND ($2::uuid IS NULL OR workflow_id = $2)
        "#,
    )
    .bind(repository_id)
    .bind(workflow_id)
    .fetch_one(pool)
    .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
               head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        FROM workflow_runs
        WHERE repository_id = $1
          AND ($2::uuid IS NULL OR workflow_id = $2)
        ORDER BY created_at DESC, run_number DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(repository_id)
    .bind(workflow_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let items = rows
        .into_iter()
        .map(workflow_run_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_workflow_run_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    run_id: Uuid,
    actor_user_id: Uuid,
) -> Result<WorkflowRun, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
               head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        FROM workflow_runs
        WHERE id = $1 AND repository_id = $2
        "#,
    )
    .bind(run_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowRunNotFound)?;

    workflow_run_from_row(row)
}

pub async fn transition_workflow_run(
    pool: &PgPool,
    run_id: Uuid,
    input: TransitionRun,
) -> Result<WorkflowRun, AutomationError> {
    let row = sqlx::query(
        r#"
        UPDATE workflow_runs
        SET status = $2,
            conclusion = $3,
            started_at = CASE WHEN $2 = 'in_progress' AND started_at IS NULL THEN now() ELSE started_at END,
            completed_at = CASE WHEN $2 IN ('completed', 'cancelled') THEN now() ELSE NULL END
        WHERE id = $1
        RETURNING id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
                  head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(run_id)
    .bind(input.status.as_str())
    .bind(input.conclusion.map(RunConclusion::as_str))
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowRunNotFound)?;

    workflow_run_from_row(row)
}

pub async fn create_workflow_job(
    pool: &PgPool,
    input: CreateWorkflowJob,
) -> Result<WorkflowJob, AutomationError> {
    run_repository_id(pool, input.run_id).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_jobs (run_id, name, runner_label)
        VALUES ($1, $2, $3)
        RETURNING id, run_id, name, status, conclusion, runner_label, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(input.run_id)
    .bind(&input.name)
    .bind(&input.runner_label)
    .fetch_one(pool)
    .await?;

    workflow_job_from_row(row)
}

pub async fn create_workflow_step(
    pool: &PgPool,
    input: CreateWorkflowStep,
) -> Result<WorkflowStep, AutomationError> {
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_steps (job_id, number, name)
        VALUES ($1, $2, $3)
        RETURNING id, job_id, number, name, status, conclusion, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(input.job_id)
    .bind(input.number)
    .bind(&input.name)
    .fetch_one(pool)
    .await?;

    workflow_step_from_row(row)
}

pub async fn create_package(
    pool: &PgPool,
    input: CreatePackage,
) -> Result<Package, AutomationError> {
    let repository = require_repository(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let row = sqlx::query(
        r#"
        INSERT INTO packages (
            repository_id,
            owner_user_id,
            owner_organization_id,
            name,
            package_type,
            visibility,
            created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, repository_id, name, package_type, visibility, created_by_user_id, created_at, updated_at
        "#,
    )
    .bind(input.repository_id)
    .bind(repository.owner_user_id)
    .bind(repository.owner_organization_id)
    .bind(&input.name)
    .bind(input.package_type.as_str())
    .bind(&input.visibility)
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;

    package_from_row(row)
}

pub async fn create_package_version(
    pool: &PgPool,
    input: CreatePackageVersion,
) -> Result<PackageVersion, AutomationError> {
    let repository_id = package_repository_id(pool, input.package_id).await?;
    require_repository_role(
        pool,
        repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let row = sqlx::query(
        r#"
        INSERT INTO package_versions (
            package_id, version, manifest, blob_key, size_bytes, published_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, package_id, version, manifest, blob_key, size_bytes, published_by_user_id, created_at
        "#,
    )
    .bind(input.package_id)
    .bind(&input.version)
    .bind(input.manifest)
    .bind(&input.blob_key)
    .bind(input.size_bytes)
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;

    Ok(package_version_from_row(row))
}

pub async fn list_packages(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Package>, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let total =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM packages WHERE repository_id = $1")
            .bind(repository_id)
            .fetch_one(pool)
            .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, name, package_type, visibility, created_by_user_id, created_at, updated_at
        FROM packages
        WHERE repository_id = $1
        ORDER BY lower(name), package_type
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(repository_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let items = rows
        .into_iter()
        .map(package_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_package_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    package_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Package, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, name, package_type, visibility, created_by_user_id, created_at, updated_at
        FROM packages
        WHERE id = $1 AND repository_id = $2
        "#,
    )
    .bind(package_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::PackageNotFound)?;

    package_from_row(row)
}

pub async fn list_package_versions(
    pool: &PgPool,
    repository_id: Uuid,
    package_id: Uuid,
    actor_user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<PackageVersion>, AutomationError> {
    get_package_for_actor(pool, repository_id, package_id, actor_user_id).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let total =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM package_versions WHERE package_id = $1")
            .bind(package_id)
            .fetch_one(pool)
            .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, package_id, version, manifest, blob_key, size_bytes, published_by_user_id, created_at
        FROM package_versions
        WHERE package_id = $1
        ORDER BY created_at DESC, lower(version)
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(package_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let items = rows.into_iter().map(package_version_from_row).collect();

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn repository_for_actor_by_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Uuid, AutomationError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    require_repository_role(pool, repository.id, actor_user_id, required_role).await?;
    Ok(repository.id)
}

pub async fn repository_for_optional_actor_by_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, AutomationError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    require_repository_read_for_viewer(pool, repository.id, actor_user_id).await
}

const ACTIONS_STATUS_OPTIONS: &[&str] = &[
    "action_required",
    "cancelled",
    "completed",
    "failure",
    "in_progress",
    "neutral",
    "queued",
    "skipped",
    "stale",
    "success",
    "timed_out",
    "waiting",
];

fn cleaned_filter(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_actions_status(value: String) -> String {
    value.trim().to_lowercase().replace([' ', '-'], "_")
}

async fn resolve_workflow_filter(
    pool: &PgPool,
    repository_id: Uuid,
    workflow: &str,
) -> Result<Uuid, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT id
        FROM actions_workflows
        WHERE repository_id = $1
          AND (
              id::text = $2
              OR lower(name) = lower($2)
              OR lower(path) = lower($2)
          )
        "#,
    )
    .bind(repository_id)
    .bind(workflow)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AutomationError::InvalidActionsFilter(format!("unknown workflow `{workflow}`"))
    })?;
    Ok(row.get("id"))
}

async fn require_repository_read_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, AutomationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;

    match actor_user_id {
        Some(user_id) => {
            if repository.visibility == RepositoryVisibility::Public {
                Ok(repository)
            } else {
                require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;
                Ok(repository)
            }
        }
        None if repository.visibility == RepositoryVisibility::Public => Ok(repository),
        None => Err(AutomationError::RepositoryAccessDenied),
    }
}

async fn viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<Option<String>, AutomationError> {
    let Some(user_id) = actor_user_id else {
        return Ok(
            (repository.visibility == RepositoryVisibility::Public).then(|| "read".to_owned())
        );
    };
    if repository.owner_user_id == Some(user_id) {
        return Ok(Some("owner".to_owned()));
    }
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(Some("read".to_owned()));
    }
    Ok(repository_permission_for_user(pool, repository.id, user_id)
        .await
        .map_err(map_repository_error)?
        .map(|permission| permission.role.as_str().to_owned()))
}

async fn actions_workflow_rail(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsWorkflowRailItem>, AutomationError> {
    let rows = sqlx::query(
        r#"
        WITH latest_runs AS (
            SELECT DISTINCT ON (workflow_id)
                   workflow_id, id, run_number, status, conclusion, created_at
            FROM workflow_runs
            WHERE repository_id = $1
            ORDER BY workflow_id, created_at DESC, run_number DESC
        )
        SELECT actions_workflows.id,
               actions_workflows.name,
               actions_workflows.path,
               actions_workflows.state,
               actions_workflows.trigger_events,
               actions_workflows.pinned_order,
               COALESCE(run_counts.run_count, 0)::bigint AS run_count,
               latest_runs.id AS latest_run_id,
               latest_runs.run_number AS latest_run_number,
               latest_runs.status AS latest_run_status,
               latest_runs.conclusion AS latest_run_conclusion,
               latest_runs.created_at AS latest_run_created_at
        FROM actions_workflows
        LEFT JOIN (
            SELECT workflow_id, count(*)::bigint AS run_count
            FROM workflow_runs
            WHERE repository_id = $1
            GROUP BY workflow_id
        ) run_counts ON run_counts.workflow_id = actions_workflows.id
        LEFT JOIN latest_runs ON latest_runs.workflow_id = actions_workflows.id
        WHERE actions_workflows.repository_id = $1
        ORDER BY actions_workflows.pinned_order NULLS LAST, lower(actions_workflows.name), actions_workflows.path
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let state: String = row.get("state");
            let latest_run_id: Option<Uuid> = row.get("latest_run_id");
            Ok(ActionsWorkflowRailItem {
                id: row.get("id"),
                name: row.get("name"),
                path: row.get("path"),
                state: WorkflowState::try_from(state.as_str())?,
                trigger_events: row.get("trigger_events"),
                pinned: row.get::<Option<i32>, _>("pinned_order").is_some(),
                run_count: row.get("run_count"),
                latest_run: latest_run_id.map(|id| ActionsWorkflowLatestRun {
                    id,
                    run_number: row.get("latest_run_number"),
                    status: row.get("latest_run_status"),
                    conclusion: row.get("latest_run_conclusion"),
                    created_at: row.get("latest_run_created_at"),
                }),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct ActionsRunFilterRefs<'a> {
    repository_id: Uuid,
    q: Option<&'a str>,
    workflow: Option<&'a str>,
    event: Option<&'a str>,
    status: Option<&'a str>,
    branch: Option<&'a str>,
    actor: Option<&'a str>,
}

async fn actions_run_count(
    pool: &PgPool,
    filters: ActionsRunFilterRefs<'_>,
) -> Result<i64, AutomationError> {
    let sql = format!(
        r#"
        SELECT count(*)::bigint
        FROM workflow_runs
        JOIN actions_workflows ON actions_workflows.id = workflow_runs.workflow_id
        LEFT JOIN users ON users.id = workflow_runs.actor_user_id
        LEFT JOIN commits ON commits.id = workflow_runs.commit_id
        WHERE
        {ACTIONS_RUN_FILTER_PREDICATE}
        "#
    );
    sqlx::query_scalar::<_, i64>(&sql)
        .bind(filters.repository_id)
        .bind(filters.q)
        .bind(filters.workflow)
        .bind(filters.event)
        .bind(filters.status)
        .bind(filters.branch)
        .bind(filters.actor)
        .fetch_one(pool)
        .await
        .map_err(AutomationError::Sqlx)
}

async fn actions_run_items(
    pool: &PgPool,
    filters: ActionsRunFilterRefs<'_>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActionsRunListItem>, AutomationError> {
    let sql = format!(
        r#"
        SELECT workflow_runs.id,
               workflow_runs.workflow_id,
               actions_workflows.name AS workflow_name,
               actions_workflows.path AS workflow_path,
               workflow_runs.run_number,
               COALESCE(workflow_runs.display_title, commits.message, actions_workflows.name) AS display_title,
               workflow_runs.status,
               workflow_runs.conclusion,
               workflow_runs.event,
               workflow_runs.actor_user_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               users.display_name AS actor_display_name,
               users.avatar_url AS actor_avatar_url,
               workflow_runs.head_branch,
               workflow_runs.head_sha,
               pull_requests.id AS pull_request_id,
               pull_requests.number AS pull_request_number,
               pull_requests.title AS pull_request_title,
               commits.message AS commit_message,
               workflow_runs.started_at,
               workflow_runs.completed_at,
               workflow_runs.created_at,
               workflow_runs.updated_at
        FROM workflow_runs
        JOIN actions_workflows ON actions_workflows.id = workflow_runs.workflow_id
        LEFT JOIN users ON users.id = workflow_runs.actor_user_id
        LEFT JOIN pull_requests ON pull_requests.id = workflow_runs.pull_request_id
        LEFT JOIN commits ON commits.id = workflow_runs.commit_id
        WHERE
        {ACTIONS_RUN_FILTER_PREDICATE}
        ORDER BY workflow_runs.created_at DESC, workflow_runs.run_number DESC
        LIMIT $8 OFFSET $9
        "#
    );
    let rows = sqlx::query(&sql)
        .bind(filters.repository_id)
        .bind(filters.q)
        .bind(filters.workflow)
        .bind(filters.event)
        .bind(filters.status)
        .bind(filters.branch)
        .bind(filters.actor)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(actions_run_list_item_from_row)
        .collect()
}

const ACTIONS_RUN_FILTER_PREDICATE: &str = r#"
        workflow_runs.repository_id = $1
        AND (
            $2::text IS NULL
            OR workflow_runs.display_title ILIKE '%' || $2 || '%'
            OR actions_workflows.name ILIKE '%' || $2 || '%'
            OR actions_workflows.path ILIKE '%' || $2 || '%'
            OR workflow_runs.head_branch ILIKE '%' || $2 || '%'
            OR workflow_runs.head_sha ILIKE '%' || $2 || '%'
            OR workflow_runs.run_number::text = $2
            OR commits.message ILIKE '%' || $2 || '%'
        )
        AND (
            $3::text IS NULL
            OR actions_workflows.id::text = $3
            OR lower(actions_workflows.name) = lower($3)
            OR lower(actions_workflows.path) = lower($3)
        )
        AND ($4::text IS NULL OR lower(workflow_runs.event) = lower($4))
        AND (
            $5::text IS NULL
            OR workflow_runs.status = $5
            OR workflow_runs.conclusion = $5
            OR (
                $5 = 'completed'
                AND workflow_runs.status = 'completed'
            )
        )
        AND ($6::text IS NULL OR lower(workflow_runs.head_branch) = lower($6))
        AND (
            $7::text IS NULL
            OR users.id::text = $7
            OR lower(COALESCE(NULLIF(users.username, ''), users.email)) = lower($7)
            OR users.email ILIKE '%' || $7 || '%'
            OR users.display_name ILIKE '%' || $7 || '%'
        )
"#;

fn actions_run_list_item_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ActionsRunListItem, AutomationError> {
    let status: String = row.get("status");
    let conclusion: Option<String> = row.get("conclusion");
    let actor_user_id: Option<Uuid> = row.get("actor_user_id");
    let pull_request_id: Option<Uuid> = row.get("pull_request_id");
    let head_sha: Option<String> = row.get("head_sha");
    let started_at: Option<DateTime<Utc>> = row.get("started_at");
    let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
    Ok(ActionsRunListItem {
        id: row.get("id"),
        workflow_id: row.get("workflow_id"),
        workflow_name: row.get("workflow_name"),
        workflow_path: row.get("workflow_path"),
        run_number: row.get("run_number"),
        display_title: row.get("display_title"),
        status_category: status_category(&status, conclusion.as_deref()),
        status,
        conclusion,
        event: row.get("event"),
        actor: actor_user_id.map(|id| ActionsActor {
            id,
            login: row.get("actor_login"),
            display_name: row.get("actor_display_name"),
            avatar_url: row.get("actor_avatar_url"),
        }),
        head_branch: row.get("head_branch"),
        short_sha: head_sha
            .as_deref()
            .map(|sha| sha.chars().take(7).collect::<String>()),
        head_sha,
        pull_request: pull_request_id.map(|id| ActionsRunPullRequest {
            id,
            number: row.get("pull_request_number"),
            title: row.get("pull_request_title"),
        }),
        commit_message: row.get("commit_message"),
        job_summary: ActionsJobSummary::default(),
        duration_seconds: match (started_at, completed_at) {
            (Some(started), Some(completed)) => Some((completed - started).num_seconds().max(0)),
            _ => None,
        },
        is_live: matches!(
            row.get::<String, _>("status").as_str(),
            "queued" | "in_progress"
        ),
        started_at,
        completed_at,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn status_category(status: &str, conclusion: Option<&str>) -> String {
    match (status, conclusion) {
        ("completed", Some(conclusion)) => conclusion.to_owned(),
        ("cancelled", _) => "cancelled".to_owned(),
        ("completed", None) => "completed".to_owned(),
        (status, _) => status.to_owned(),
    }
}

async fn hydrate_job_summaries(
    pool: &PgPool,
    runs: &mut [ActionsRunListItem],
) -> Result<(), AutomationError> {
    if runs.is_empty() {
        return Ok(());
    }
    let run_ids = runs.iter().map(|run| run.id).collect::<Vec<_>>();
    let rows = sqlx::query(
        r#"
        SELECT run_id,
               count(*)::bigint AS total,
               count(*) FILTER (WHERE status = 'queued')::bigint AS queued,
               count(*) FILTER (WHERE status = 'in_progress')::bigint AS in_progress,
               count(*) FILTER (WHERE status = 'completed')::bigint AS completed,
               count(*) FILTER (WHERE status = 'cancelled')::bigint AS cancelled,
               count(*) FILTER (WHERE conclusion = 'success')::bigint AS success,
               count(*) FILTER (WHERE conclusion = 'failure')::bigint AS failure,
               count(*) FILTER (WHERE conclusion = 'skipped')::bigint AS skipped,
               count(*) FILTER (WHERE conclusion = 'timed_out')::bigint AS timed_out
        FROM workflow_jobs
        WHERE run_id = ANY($1)
        GROUP BY run_id
        "#,
    )
    .bind(&run_ids)
    .fetch_all(pool)
    .await?;
    let mut summaries = HashMap::new();
    for row in rows {
        summaries.insert(
            row.get::<Uuid, _>("run_id"),
            ActionsJobSummary {
                total: row.get("total"),
                queued: row.get("queued"),
                in_progress: row.get("in_progress"),
                completed: row.get("completed"),
                cancelled: row.get("cancelled"),
                success: row.get("success"),
                failure: row.get("failure"),
                skipped: row.get("skipped"),
                timed_out: row.get("timed_out"),
            },
        );
    }
    for run in runs {
        if let Some(summary) = summaries.remove(&run.id) {
            run.job_summary = summary;
        }
    }
    Ok(())
}

async fn actions_filter_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<ActionsRunFilterOptions, AutomationError> {
    Ok(ActionsRunFilterOptions {
        workflows: workflow_filter_options(pool, repository_id).await?,
        events: distinct_run_options(pool, repository_id, "event").await?,
        statuses: status_filter_options(pool, repository_id).await?,
        branches: distinct_run_options(pool, repository_id, "head_branch").await?,
        actors: actor_filter_options(pool, repository_id).await?,
    })
}

async fn actions_filter_options_for_workflow(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Uuid,
) -> Result<ActionsRunFilterOptions, AutomationError> {
    Ok(ActionsRunFilterOptions {
        workflows: Vec::new(),
        events: distinct_run_options_for_workflow(pool, repository_id, workflow_id, "event")
            .await?,
        statuses: status_filter_options_for_workflow(pool, repository_id, workflow_id).await?,
        branches: distinct_run_options_for_workflow(
            pool,
            repository_id,
            workflow_id,
            "head_branch",
        )
        .await?,
        actors: actor_filter_options_for_workflow(pool, repository_id, workflow_id).await?,
    })
}

async fn workflow_filter_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT actions_workflows.id::text AS value,
               actions_workflows.name AS label,
               count(workflow_runs.id)::bigint AS count
        FROM actions_workflows
        LEFT JOIN workflow_runs ON workflow_runs.workflow_id = actions_workflows.id
        WHERE actions_workflows.repository_id = $1
        GROUP BY actions_workflows.id, actions_workflows.name, actions_workflows.pinned_order
        ORDER BY actions_workflows.pinned_order NULLS LAST, lower(actions_workflows.name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(filter_option_from_row).collect())
}

async fn distinct_run_options(
    pool: &PgPool,
    repository_id: Uuid,
    column: &str,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let sql = match column {
        "event" => {
            "SELECT event AS value, event AS label, count(*)::bigint AS count FROM workflow_runs WHERE repository_id = $1 GROUP BY event ORDER BY lower(event)"
        }
        "head_branch" => {
            "SELECT head_branch AS value, head_branch AS label, count(*)::bigint AS count FROM workflow_runs WHERE repository_id = $1 GROUP BY head_branch ORDER BY lower(head_branch)"
        }
        _ => unreachable!("unsupported filter column"),
    };
    let rows = sqlx::query(sql).bind(repository_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(filter_option_from_row).collect())
}

async fn distinct_run_options_for_workflow(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Uuid,
    column: &str,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let sql = match column {
        "event" => {
            "SELECT event AS value, event AS label, count(*)::bigint AS count FROM workflow_runs WHERE repository_id = $1 AND workflow_id = $2 GROUP BY event ORDER BY lower(event)"
        }
        "head_branch" => {
            "SELECT head_branch AS value, head_branch AS label, count(*)::bigint AS count FROM workflow_runs WHERE repository_id = $1 AND workflow_id = $2 GROUP BY head_branch ORDER BY lower(head_branch)"
        }
        _ => unreachable!("unsupported filter column"),
    };
    let rows = sqlx::query(sql)
        .bind(repository_id)
        .bind(workflow_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(filter_option_from_row).collect())
}

async fn status_filter_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT category AS value, category AS label, count(*)::bigint AS count
        FROM (
            SELECT CASE
                WHEN status = 'completed' AND conclusion IS NOT NULL THEN conclusion
                ELSE status
            END AS category
            FROM workflow_runs
            WHERE repository_id = $1
        ) categories
        GROUP BY category
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    let mut counts = rows
        .into_iter()
        .map(|row| (row.get::<String, _>("value"), row.get::<i64, _>("count")))
        .collect::<HashMap<_, _>>();

    Ok(ACTIONS_STATUS_OPTIONS
        .iter()
        .map(|status| ActionsFilterOption {
            value: (*status).to_owned(),
            label: status.replace('_', " "),
            count: counts.remove(*status).unwrap_or(0),
        })
        .collect())
}

async fn status_filter_options_for_workflow(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Uuid,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT category AS value, category AS label, count(*)::bigint AS count
        FROM (
            SELECT CASE
                WHEN status = 'completed' AND conclusion IS NOT NULL THEN conclusion
                ELSE status
            END AS category
            FROM workflow_runs
            WHERE repository_id = $1 AND workflow_id = $2
        ) categories
        GROUP BY category
        "#,
    )
    .bind(repository_id)
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    let mut counts = rows
        .into_iter()
        .map(|row| (row.get::<String, _>("value"), row.get::<i64, _>("count")))
        .collect::<HashMap<_, _>>();

    Ok(ACTIONS_STATUS_OPTIONS
        .iter()
        .map(|status| ActionsFilterOption {
            value: (*status).to_owned(),
            label: status.replace('_', " "),
            count: counts.remove(*status).unwrap_or(0),
        })
        .collect())
}

async fn actor_filter_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id::text AS value,
               COALESCE(NULLIF(users.username, ''), users.email) AS label,
               count(workflow_runs.id)::bigint AS count
        FROM workflow_runs
        JOIN users ON users.id = workflow_runs.actor_user_id
        WHERE workflow_runs.repository_id = $1
        GROUP BY users.id, users.username, users.email
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email))
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(filter_option_from_row).collect())
}

async fn actor_filter_options_for_workflow(
    pool: &PgPool,
    repository_id: Uuid,
    workflow_id: Uuid,
) -> Result<Vec<ActionsFilterOption>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id::text AS value,
               COALESCE(NULLIF(users.username, ''), users.email) AS label,
               count(workflow_runs.id)::bigint AS count
        FROM workflow_runs
        JOIN users ON users.id = workflow_runs.actor_user_id
        WHERE workflow_runs.repository_id = $1 AND workflow_runs.workflow_id = $2
        GROUP BY users.id, users.username, users.email
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email))
        "#,
    )
    .bind(repository_id)
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(filter_option_from_row).collect())
}

fn filter_option_from_row(row: sqlx::postgres::PgRow) -> ActionsFilterOption {
    ActionsFilterOption {
        value: row.get("value"),
        label: row.get("label"),
        count: row.get("count"),
    }
}

async fn actions_workflow_detail_workflow(
    pool: &PgPool,
    repository: &Repository,
    workflow_path: &str,
) -> Result<ActionsWorkflowDetailWorkflow, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT id, name, path, state, trigger_events, source_blob_id, source_sha,
               source_branch, yaml_parse_error, dispatch_inputs, dispatch_enabled
        FROM actions_workflows
        WHERE repository_id = $1 AND lower(path) = lower($2)
        "#,
    )
    .bind(repository.id)
    .bind(workflow_path)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowNotFound)?;

    let state: String = row.get("state");
    let path: String = row.get("path");
    let source_branch = row
        .get::<Option<String>, _>("source_branch")
        .unwrap_or_else(|| repository.default_branch.clone());
    let dispatch_inputs = workflow_dispatch_inputs_from_json(row.get("dispatch_inputs"))?;
    let yaml_parse_error: Option<String> = row.get("yaml_parse_error");
    Ok(ActionsWorkflowDetailWorkflow {
        id: row.get("id"),
        name: row.get("name"),
        path: path.clone(),
        state: WorkflowState::try_from(state.as_str())?,
        trigger_events: row.get("trigger_events"),
        source_branch: source_branch.clone(),
        source_sha: row.get("source_sha"),
        source_blob_id: row.get("source_blob_id"),
        source_href: format!(
            "/{}/{}/blob/{}/{}",
            repository.owner_login, repository.name, source_branch, path
        ),
        dispatch: WorkflowDispatchSpec {
            enabled: row.get("dispatch_enabled"),
            inputs: dispatch_inputs,
        },
        valid: yaml_parse_error.is_none(),
        yaml_parse_error,
    })
}

fn workflow_dispatch_inputs_from_json(
    value: Value,
) -> Result<Vec<WorkflowDispatchInput>, AutomationError> {
    serde_json::from_value(value).map_err(|error| {
        AutomationError::InvalidActionsFilter(format!("invalid workflow dispatch inputs: {error}"))
    })
}

async fn actions_workflow_refs(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsWorkflowRef>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               commits.oid AS sha
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.kind IN ('branch', 'tag')
        ORDER BY repository_git_refs.kind, lower(repository_git_refs.name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            let short_name = name
                .strip_prefix("refs/heads/")
                .or_else(|| name.strip_prefix("refs/tags/"))
                .unwrap_or(name.as_str())
                .to_owned();
            ActionsWorkflowRef {
                name,
                short_name,
                kind: row.get("kind"),
                sha: row.get("sha"),
            }
        })
        .collect())
}

async fn get_workflow(
    pool: &PgPool,
    workflow_id: Uuid,
) -> Result<ActionsWorkflow, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, name, path, state, trigger_events, created_at, updated_at
        FROM actions_workflows
        WHERE id = $1
        "#,
    )
    .bind(workflow_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowNotFound)?;

    workflow_from_row(row)
}

async fn next_run_number(pool: &PgPool, workflow_id: Uuid) -> Result<i64, AutomationError> {
    let number = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(max(run_number), 0) + 1 FROM workflow_runs WHERE workflow_id = $1",
    )
    .bind(workflow_id)
    .fetch_one(pool)
    .await?;
    Ok(number)
}

async fn run_repository_id(pool: &PgPool, run_id: Uuid) -> Result<Uuid, AutomationError> {
    sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM workflow_runs WHERE id = $1")
        .bind(run_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AutomationError::WorkflowRunNotFound)
}

async fn package_repository_id(pool: &PgPool, package_id: Uuid) -> Result<Uuid, AutomationError> {
    sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM packages WHERE id = $1")
        .bind(package_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AutomationError::PackageNotFound)
}

async fn require_repository(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Repository, AutomationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    require_repository_role(pool, repository_id, user_id, required_role).await?;
    Ok(repository)
}

async fn require_repository_role(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<(), AutomationError> {
    let permission = repository_permission_for_user(pool, repository_id, user_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryAccessDenied)?;

    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };

    if allowed {
        Ok(())
    } else {
        Err(AutomationError::RepositoryAccessDenied)
    }
}

fn map_repository_error(error: RepositoryError) -> AutomationError {
    match error {
        RepositoryError::Sqlx(error) => AutomationError::Sqlx(error),
        RepositoryError::NotFound => AutomationError::RepositoryNotFound,
        _ => AutomationError::RepositoryAccessDenied,
    }
}

fn workflow_from_row(row: sqlx::postgres::PgRow) -> Result<ActionsWorkflow, AutomationError> {
    let state: String = row.get("state");
    Ok(ActionsWorkflow {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        name: row.get("name"),
        path: row.get("path"),
        state: WorkflowState::try_from(state.as_str())?,
        trigger_events: row.get("trigger_events"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn workflow_run_from_row(row: sqlx::postgres::PgRow) -> Result<WorkflowRun, AutomationError> {
    let status: String = row.get("status");
    let conclusion: Option<String> = row.get("conclusion");
    Ok(WorkflowRun {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        workflow_id: row.get("workflow_id"),
        actor_user_id: row.get("actor_user_id"),
        run_number: row.get("run_number"),
        status: RunStatus::try_from(status.as_str())?,
        conclusion: conclusion
            .as_deref()
            .map(RunConclusion::try_from)
            .transpose()?,
        head_branch: row.get("head_branch"),
        head_sha: row.get("head_sha"),
        event: row.get("event"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn workflow_job_from_row(row: sqlx::postgres::PgRow) -> Result<WorkflowJob, AutomationError> {
    let status: String = row.get("status");
    let conclusion: Option<String> = row.get("conclusion");
    Ok(WorkflowJob {
        id: row.get("id"),
        run_id: row.get("run_id"),
        name: row.get("name"),
        status: RunStatus::try_from(status.as_str())?,
        conclusion: conclusion
            .as_deref()
            .map(RunConclusion::try_from)
            .transpose()?,
        runner_label: row.get("runner_label"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn workflow_step_from_row(row: sqlx::postgres::PgRow) -> Result<WorkflowStep, AutomationError> {
    let status: String = row.get("status");
    let conclusion: Option<String> = row.get("conclusion");
    Ok(WorkflowStep {
        id: row.get("id"),
        job_id: row.get("job_id"),
        number: row.get("number"),
        name: row.get("name"),
        status: RunStatus::try_from(status.as_str())?,
        conclusion: conclusion
            .as_deref()
            .map(RunConclusion::try_from)
            .transpose()?,
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn package_from_row(row: sqlx::postgres::PgRow) -> Result<Package, AutomationError> {
    let package_type: String = row.get("package_type");
    Ok(Package {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        name: row.get("name"),
        package_type: PackageType::try_from(package_type.as_str())?,
        visibility: row.get("visibility"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn package_version_from_row(row: sqlx::postgres::PgRow) -> PackageVersion {
    PackageVersion {
        id: row.get("id"),
        package_id: row.get("package_id"),
        version: row.get("version"),
        manifest: row.get("manifest"),
        blob_key: row.get("blob_key"),
        size_bytes: row.get("size_bytes"),
        published_by_user_id: row.get("published_by_user_id"),
        created_at: row.get("created_at"),
    }
}
