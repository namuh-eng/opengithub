use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use crate::api_types::ListEnvelope;
use crate::jobs::{enqueue_job, JobLeaseError};

use super::{
    actions_secrets::{
        actions_secret_redaction_values, mask_actions_secret_values,
        resolve_actions_runtime_context, ActionsRuntimeResolutionDiagnostics,
        ActionsRuntimeResolutionRequest,
    },
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
pub struct ActionsRunDetail {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub workflow: ActionsRunDetailWorkflow,
    pub run: ActionsRunListItem,
    pub runtime_policy: ActionsRuntimeResolutionDiagnostics,
    pub attempts: Vec<ActionsRunAttempt>,
    pub jobs: Vec<ActionsRunJobDetail>,
    pub annotations: Vec<ActionsRunAnnotation>,
    pub artifacts: Vec<ActionsRunArtifact>,
    pub action_state: ActionsRunActionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogDetail {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub workflow: ActionsRunDetailWorkflow,
    pub run: ActionsRunListItem,
    pub jobs: Vec<ActionsRunJobDetail>,
    pub job: ActionsRunJobDetail,
    pub steps: Vec<ActionsJobLogStep>,
    pub annotations: Vec<ActionsRunAnnotation>,
    pub log_state: ActionsJobLogState,
    pub search: ActionsJobLogSearch,
    pub options: ActionsJobLogOptions,
    pub download_href: String,
    pub run_archive_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogStep {
    pub id: Option<Uuid>,
    pub number: i32,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub duration_seconds: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub lines: ListEnvelope<ActionsJobLogLine>,
    pub match_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogState {
    pub available: bool,
    pub status: u16,
    pub reason: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_live: bool,
    pub next_cursor: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogSearch {
    pub query: Option<String>,
    pub total_matches: i64,
    pub selected_match: Option<i64>,
    pub matches: Vec<ActionsJobLogSearchMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogSearchMatch {
    pub line_number: i32,
    pub step_id: Option<Uuid>,
    pub step_number: i32,
    pub anchor: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogOptions {
    pub show_timestamps: bool,
    pub raw_logs: bool,
    pub wrap_lines: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunDetailWorkflow {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub state: WorkflowState,
    pub source_branch: String,
    pub source_sha: Option<String>,
    pub source_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunAttempt {
    pub id: Option<Uuid>,
    pub attempt_number: i32,
    pub status: String,
    pub conclusion: Option<String>,
    pub trigger_kind: String,
    pub actor: Option<ActionsActor>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunJobDetail {
    pub id: Uuid,
    pub name: String,
    pub group_name: Option<String>,
    pub attempt_number: i32,
    pub status: String,
    pub conclusion: Option<String>,
    pub runner_label: Option<String>,
    pub duration_seconds: Option<i64>,
    pub log_available: bool,
    pub log_deleted_at: Option<DateTime<Utc>>,
    pub steps: Vec<ActionsRunStepDetail>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunStepDetail {
    pub id: Uuid,
    pub number: i32,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub duration_seconds: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunAnnotation {
    pub id: Uuid,
    pub job_id: Option<Uuid>,
    pub step_id: Option<Uuid>,
    pub level: String,
    pub path: Option<String>,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub title: Option<String>,
    pub message: String,
    pub raw_details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunArtifact {
    pub id: Uuid,
    pub name: String,
    pub digest: Option<String>,
    pub size_bytes: i64,
    pub retention_days: i32,
    pub expired_at: Option<DateTime<Utc>>,
    pub download_available: bool,
    pub delete_available: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsArtifactUpload {
    pub name: String,
    pub size_bytes: i64,
    pub digest: Option<String>,
    pub storage_key: Option<String>,
    pub retention_days: Option<i32>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsDependencyCache {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub key: String,
    pub version: String,
    pub scope: String,
    #[serde(skip_serializing)]
    pub storage_key: String,
    pub size_bytes: i64,
    pub last_used_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsDependencyCaches {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub caches: ListEnvelope<ActionsDependencyCache>,
    pub total_size_bytes: i64,
    pub limit_bytes: i64,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsCacheReserve {
    pub key: String,
    pub version: String,
    pub size_bytes: i64,
    pub scope: Option<String>,
    pub storage_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLog {
    pub job: ActionsJobLogJob,
    pub lines: Vec<ActionsJobLogLine>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub query: Option<String>,
    pub download_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogJob {
    pub id: Uuid,
    pub run_id: Uuid,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub log_deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsJobLogLine {
    pub line_number: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub content: String,
    pub anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsArtifactDownload {
    pub artifact_id: Uuid,
    pub name: String,
    pub filename: String,
    pub download_url: String,
    pub storage_key: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunLogArchive {
    pub run_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActionsLogPreferences {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub show_timestamps: bool,
    pub raw_logs: bool,
    pub wrap_lines: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunActionState {
    pub can_rerun: bool,
    pub can_rerun_failed: bool,
    pub can_cancel: bool,
    pub can_delete_logs: bool,
    pub disabled_reason: Option<String>,
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
    pub yaml_parsed_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Default)]
pub struct ActionsJobLogDetailQuery {
    pub q: Option<String>,
    pub selected_match: Option<i64>,
    pub show_timestamps: Option<bool>,
    pub raw_logs: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchWorkflowRun {
    pub repository_id: Uuid,
    pub workflow_path: String,
    pub actor_user_id: Uuid,
    pub ref_name: String,
    pub inputs: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunRerunMode {
    All,
    Failed,
    Job,
}

impl WorkflowRunRerunMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::All => "rerun_all",
            Self::Failed => "rerun_failed",
            Self::Job => "rerun_job",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerunWorkflowRun {
    pub repository_id: Uuid,
    pub run_id: Uuid,
    pub actor_user_id: Uuid,
    pub mode: WorkflowRunRerunMode,
    pub job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutateWorkflowRun {
    pub repository_id: Uuid,
    pub run_id: Uuid,
    pub actor_user_id: Uuid,
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
pub struct TriggerWorkflowsForPush {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub ref_name: String,
    pub head_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerWorkflowsForPullRequest {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub pull_request_id: Uuid,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerWorkflowsForSchedule {
    pub repository_id: Uuid,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PushTriggerResult {
    pub scanned_workflows: usize,
    pub triggered_runs: Vec<WorkflowRun>,
    pub skipped_workflows: Vec<PushTriggerSkip>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PushTriggerSkip {
    pub path: String,
    pub reason: String,
}

pub type WorkflowTriggerResult = PushTriggerResult;
pub type WorkflowTriggerSkip = PushTriggerSkip;

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
#[serde(rename_all = "camelCase")]
pub struct ActionsRunner {
    pub id: Uuid,
    pub name: String,
    pub labels: Vec<String>,
    pub status: String,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub busy_since: Option<DateTime<Utc>>,
    pub current_job: Option<ActionsRunnerJob>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunnerJob {
    pub run_id: Uuid,
    pub job_id: Uuid,
    pub job_name: String,
    pub run_number: i64,
    pub workflow_name: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunnerQueue {
    pub queued_jobs: i64,
    pub busy_runners: i64,
    pub online_runners: i64,
    pub offline_runners: i64,
    pub concurrency_limit: i32,
    pub cancel_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsWorkflowPermissions {
    pub github_token_permission: String,
    pub allow_pull_request_approval: bool,
    pub github_token_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryActionsRunnerSettings {
    pub repository: ActionsDashboardRepository,
    pub viewer_permission: Option<String>,
    pub can_manage_runners: bool,
    pub runners: Vec<ActionsRunner>,
    pub queue: ActionsRunnerQueue,
    pub workflow_permissions: ActionsWorkflowPermissions,
    pub setup: ActionsRunnerSetup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRunnerSetup {
    pub registration_token: Option<String>,
    pub docker_command: Option<String>,
    pub expires_in_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActionsRunner {
    pub name: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActionsRunnerSettings {
    pub concurrency_limit: i32,
    pub cancel_in_progress: bool,
    pub github_token_permission: String,
    pub allow_pull_request_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerHeartbeat {
    pub runner_id: Uuid,
    pub runner_token: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendWorkflowJobLogChunk {
    pub job_id: Uuid,
    pub runner_token: String,
    pub step_id: Option<Uuid>,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub finalize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowJobLogChunkResult {
    pub job_id: Uuid,
    pub run_id: Uuid,
    pub s3_key: String,
    pub bytes_written: i64,
    pub appended_lines: i64,
    pub next_cursor: Option<i32>,
    pub finalized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowJobLogStream {
    pub job: ActionsJobLogJob,
    pub lines: Vec<ActionsJobLogLine>,
    pub next_cursor: Option<i32>,
    pub finalized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerScheduleResult {
    pub assigned: Vec<ActionsRunnerJob>,
    pub queued_jobs: i64,
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
    #[error("workflow logs are unavailable")]
    WorkflowLogsUnavailable,
    #[error("workflow artifact was not found")]
    WorkflowArtifactNotFound,
    #[error("workflow artifact download is unavailable")]
    WorkflowArtifactUnavailable,
    #[error("workflow cache was not found")]
    WorkflowCacheNotFound,
    #[error("invalid actions storage request: {0}")]
    InvalidActionsStorage(String),
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
    #[error("workflow dispatch is not available: {0}")]
    WorkflowDispatchDisabled(String),
    #[error("invalid workflow dispatch: {0}")]
    InvalidWorkflowDispatch(String),
    #[error("workflow run action is unavailable: {0}")]
    WorkflowRunActionUnavailable(String),
    #[error(transparent)]
    ActionsSecrets(#[from] super::actions_secrets::ActionsSecretsError),
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
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

pub async fn actions_run_detail_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    run_id: Uuid,
) -> Result<ActionsRunDetail, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let viewer_permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let mut runs = actions_run_items_by_run(pool, repository_id, run_id).await?;
    hydrate_job_summaries(pool, &mut runs).await?;
    let run = runs
        .into_iter()
        .next()
        .ok_or(AutomationError::WorkflowRunNotFound)?;
    let workflow = actions_run_detail_workflow(pool, &repository, run.workflow_id).await?;
    let attempts = actions_run_attempts(pool, &run).await?;
    let jobs = actions_run_jobs(pool, run.id).await?;
    let redaction_values = actions_secret_redaction_values(pool, repository.id).await?;
    let mut annotations = actions_run_annotations(pool, run.id).await?;
    mask_actions_annotations(&mut annotations, &redaction_values);
    let artifacts = actions_run_artifacts(pool, run.id).await?;
    let runtime_policy = resolve_actions_runtime_context(
        pool,
        ActionsRuntimeResolutionRequest {
            repository_id: repository.id,
            event: run.event.clone(),
            fork_pull_request: run.pull_request.is_some() && run.event == "pull_request",
            environment: None,
            environment_approved: false,
            explicit_secret_names: None,
        },
    )
    .await?
    .diagnostics;
    let action_state = actions_run_action_state(
        &run,
        &run.job_summary,
        viewer_permission.as_deref(),
        repository.is_archived,
    );

    Ok(ActionsRunDetail {
        repository: ActionsDashboardRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.clone(),
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission,
        workflow,
        run,
        runtime_policy,
        attempts,
        jobs,
        annotations,
        artifacts,
        action_state,
    })
}

pub async fn actions_job_log_detail_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    run_id: Uuid,
    job_id: Uuid,
    query: ActionsJobLogDetailQuery,
) -> Result<ActionsJobLogDetail, AutomationError> {
    let detail = actions_run_detail_for_viewer(pool, repository_id, actor_user_id, run_id).await?;
    let job = detail
        .jobs
        .iter()
        .find(|job| job.id == job_id)
        .cloned()
        .ok_or(AutomationError::WorkflowJobNotFound)?;
    let q = cleaned_filter(query.q);
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 500);
    let offset = (page - 1) * page_size;
    let like_query = q.as_ref().map(|value| format!("%{}%", escape_like(value)));
    let options = actions_log_options(
        pool,
        repository_id,
        actor_user_id,
        query.show_timestamps,
        query.raw_logs,
    )
    .await?;
    let annotations = detail
        .annotations
        .iter()
        .filter(|annotation| annotation.job_id == Some(job_id))
        .cloned()
        .collect::<Vec<_>>();
    let log_available = job.log_available;
    let deleted_at = job.log_deleted_at;
    let redaction_values = actions_secret_redaction_values(pool, repository_id).await?;

    let total_matches = if log_available {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM workflow_job_log_lines
            WHERE job_id = $1
              AND ($2::text IS NULL OR content ILIKE $2 ESCAPE '\')
            "#,
        )
        .bind(job_id)
        .bind(like_query.as_deref())
        .fetch_one(pool)
        .await?
    } else {
        0
    };
    let latest_line = if log_available {
        sqlx::query_scalar::<_, Option<i32>>(
            "SELECT max(line_number) FROM workflow_job_log_lines WHERE job_id = $1",
        )
        .bind(job_id)
        .fetch_one(pool)
        .await?
    } else {
        None
    };
    let line_rows = if log_available {
        sqlx::query(
            r#"
            SELECT line_number, timestamp, content, step_id
            FROM workflow_job_log_lines
            WHERE job_id = $1
              AND ($2::text IS NULL OR content ILIKE $2 ESCAPE '\')
            ORDER BY line_number
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(job_id)
        .bind(like_query.as_deref())
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };
    let count_rows = if log_available {
        sqlx::query(
            r#"
            SELECT step_id, count(*)::bigint AS total
            FROM workflow_job_log_lines
            WHERE job_id = $1
              AND ($2::text IS NULL OR content ILIKE $2 ESCAPE '\')
            GROUP BY step_id
            "#,
        )
        .bind(job_id)
        .bind(like_query.as_deref())
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };
    let mut totals_by_step = HashMap::<Option<Uuid>, i64>::new();
    for row in count_rows {
        totals_by_step.insert(row.get("step_id"), row.get("total"));
    }

    let mut lines_by_step = HashMap::<Option<Uuid>, Vec<ActionsJobLogLine>>::new();
    let mut matches = Vec::new();
    for row in line_rows {
        let step_id: Option<Uuid> = row.get("step_id");
        let line_number: i32 = row.get("line_number");
        let content: String =
            mask_actions_secret_values(&row.get::<String, _>("content"), &redaction_values);
        let anchor = format!("L{line_number}");
        if q.is_some() {
            matches.push(ActionsJobLogSearchMatch {
                line_number,
                step_id,
                step_number: step_number_for_log_step(&job.steps, step_id),
                anchor: anchor.clone(),
                preview: content.chars().take(180).collect(),
            });
        }
        lines_by_step
            .entry(step_id)
            .or_default()
            .push(ActionsJobLogLine {
                line_number,
                timestamp: row.get("timestamp"),
                content,
                anchor,
            });
    }

    let mut steps = job
        .steps
        .iter()
        .map(|step| {
            let key = Some(step.id);
            let lines = lines_by_step.remove(&key).unwrap_or_default();
            let total = totals_by_step.get(&key).copied().unwrap_or(0);
            ActionsJobLogStep {
                id: Some(step.id),
                number: step.number,
                name: step.name.clone(),
                status: step.status.clone(),
                conclusion: step.conclusion.clone(),
                duration_seconds: step.duration_seconds,
                started_at: step.started_at,
                completed_at: step.completed_at,
                lines: ListEnvelope {
                    items: lines,
                    total,
                    page,
                    page_size,
                },
                match_count: total,
            }
        })
        .collect::<Vec<_>>();
    let unassigned_lines = lines_by_step.remove(&None).unwrap_or_default();
    let unassigned_total = totals_by_step.get(&None).copied().unwrap_or(0);
    if unassigned_total > 0 || job.steps.is_empty() {
        steps.insert(
            0,
            ActionsJobLogStep {
                id: None,
                number: 0,
                name: "Job log".to_owned(),
                status: job.status.clone(),
                conclusion: job.conclusion.clone(),
                duration_seconds: job.duration_seconds,
                started_at: job.started_at,
                completed_at: job.completed_at,
                lines: ListEnvelope {
                    items: unassigned_lines,
                    total: unassigned_total,
                    page,
                    page_size,
                },
                match_count: unassigned_total,
            },
        );
    }

    Ok(ActionsJobLogDetail {
        repository: detail.repository.clone(),
        viewer_permission: detail.viewer_permission.clone(),
        workflow: detail.workflow.clone(),
        run: detail.run.clone(),
        jobs: detail.jobs.clone(),
        job,
        steps,
        annotations,
        log_state: ActionsJobLogState {
            available: log_available,
            status: if log_available { 200 } else { 410 },
            reason: (!log_available).then(|| "workflow logs are unavailable".to_owned()),
            deleted_at,
            is_live: matches!(detail.run.status.as_str(), "queued" | "in_progress"),
            next_cursor: latest_line,
        },
        search: ActionsJobLogSearch {
            query: q,
            total_matches,
            selected_match: query.selected_match.filter(|value| *value > 0),
            matches,
        },
        options,
        download_href: format!(
            "/api/repos/{}/{}/actions/jobs/{}/logs/download",
            detail.repository.owner_login, detail.repository.name, job_id
        ),
        run_archive_href: format!(
            "/api/repos/{}/{}/actions/runs/{}/logs/archive",
            detail.repository.owner_login, detail.repository.name, run_id
        ),
    })
}

pub async fn update_actions_log_preferences_for_viewer(
    pool: &PgPool,
    input: UpdateActionsLogPreferences,
) -> Result<ActionsJobLogOptions, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Read,
    )
    .await?;
    let row = sqlx::query(
        r#"
        INSERT INTO actions_log_preferences (
            repository_id, user_id, show_timestamps, raw_logs, wrap_lines
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET
            show_timestamps = EXCLUDED.show_timestamps,
            raw_logs = EXCLUDED.raw_logs,
            wrap_lines = EXCLUDED.wrap_lines
        RETURNING show_timestamps, raw_logs, wrap_lines
        "#,
    )
    .bind(input.repository_id)
    .bind(input.actor_user_id)
    .bind(input.show_timestamps)
    .bind(input.raw_logs)
    .bind(input.wrap_lines)
    .fetch_one(pool)
    .await?;

    Ok(ActionsJobLogOptions {
        show_timestamps: row.get("show_timestamps"),
        raw_logs: row.get("raw_logs"),
        wrap_lines: row.get("wrap_lines"),
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

pub async fn trigger_workflows_for_push(
    pool: &PgPool,
    input: TriggerWorkflowsForPush,
) -> Result<PushTriggerResult, AutomationError> {
    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    require_repository_role(
        pool,
        repository.id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;

    let pushed_ref = normalize_pushed_ref(&input.ref_name)?;
    let workflow_files = workflow_files_for_ref(pool, repository.id, &input.ref_name).await?;
    let changed_paths = changed_paths_for_commit(pool, repository.id, &input.head_sha).await?;
    let mut triggered_runs = Vec::new();
    let mut skipped_workflows = Vec::new();

    for file in &workflow_files {
        let parsed = match parse_workflow_file(&file.content) {
            Ok(parsed) => parsed,
            Err(error) => {
                upsert_discovered_workflow(
                    pool,
                    &repository,
                    file,
                    &pushed_ref.short_name,
                    DiscoveredWorkflow {
                        name: workflow_name_from_path(&file.path),
                        trigger_events: Vec::new(),
                        dispatch_enabled: false,
                        dispatch_inputs: Vec::new(),
                        yaml_parse_error: Some(sanitize_yaml_parse_error(error.to_string())),
                    },
                )
                .await?;
                skipped_workflows.push(PushTriggerSkip {
                    path: file.path.clone(),
                    reason: "invalid_yaml".to_owned(),
                });
                continue;
            }
        };

        let workflow = upsert_discovered_workflow(
            pool,
            &repository,
            file,
            &pushed_ref.short_name,
            parsed.discovered.clone(),
        )
        .await?;
        if workflow.state != WorkflowState::Active {
            skipped_workflows.push(PushTriggerSkip {
                path: file.path.clone(),
                reason: "disabled".to_owned(),
            });
            continue;
        }
        let Some(ref push_config) = parsed.push else {
            skipped_workflows.push(PushTriggerSkip {
                path: file.path.clone(),
                reason: "push_not_configured".to_owned(),
            });
            continue;
        };
        if !push_config.matches_ref(&pushed_ref) {
            skipped_workflows.push(PushTriggerSkip {
                path: file.path.clone(),
                reason: "ref_filter".to_owned(),
            });
            continue;
        }
        if !push_config.matches_paths(&changed_paths) {
            skipped_workflows.push(PushTriggerSkip {
                path: file.path.clone(),
                reason: "path_filter".to_owned(),
            });
            continue;
        }

        let run = create_push_workflow_run(
            pool,
            &repository,
            &workflow,
            &parsed,
            &input,
            &pushed_ref,
            &changed_paths,
        )
        .await?;
        triggered_runs.push(run);
    }

    Ok(PushTriggerResult {
        scanned_workflows: workflow_files.len(),
        triggered_runs,
        skipped_workflows,
    })
}

pub async fn trigger_workflows_for_pull_request(
    pool: &PgPool,
    input: TriggerWorkflowsForPullRequest,
) -> Result<WorkflowTriggerResult, AutomationError> {
    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    require_repository_role(
        pool,
        repository.id,
        input.actor_user_id,
        RepositoryRole::Read,
    )
    .await?;

    let pull_request = pull_request_trigger_context(pool, repository.id, input.pull_request_id)
        .await?
        .ok_or(AutomationError::WorkflowRunNotFound)?;
    let base_ref_name = format!("refs/heads/{}", pull_request.base_ref);
    let workflow_files = workflow_files_for_ref(pool, repository.id, &base_ref_name).await?;
    let changed_paths =
        changed_paths_for_pull_request(pool, repository.id, input.pull_request_id).await?;
    let mut triggered_runs = Vec::new();
    let mut skipped_workflows = Vec::new();

    for file in &workflow_files {
        let parsed = match parse_workflow_file(&file.content) {
            Ok(parsed) => parsed,
            Err(error) => {
                upsert_discovered_workflow(
                    pool,
                    &repository,
                    file,
                    &pull_request.base_ref,
                    DiscoveredWorkflow {
                        name: workflow_name_from_path(&file.path),
                        trigger_events: Vec::new(),
                        dispatch_enabled: false,
                        dispatch_inputs: Vec::new(),
                        yaml_parse_error: Some(sanitize_yaml_parse_error(error.to_string())),
                    },
                )
                .await?;
                skipped_workflows.push(WorkflowTriggerSkip {
                    path: file.path.clone(),
                    reason: "invalid_yaml".to_owned(),
                });
                continue;
            }
        };

        let workflow = upsert_discovered_workflow(
            pool,
            &repository,
            file,
            &pull_request.base_ref,
            parsed.discovered.clone(),
        )
        .await?;
        if workflow.state != WorkflowState::Active {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "disabled".to_owned(),
            });
            continue;
        }
        let Some(ref pull_request_config) = parsed.pull_request else {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "pull_request_not_configured".to_owned(),
            });
            continue;
        };
        if !pull_request_config.matches_branch(&pull_request.base_ref) {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "branch_filter".to_owned(),
            });
            continue;
        }
        if !pull_request_config.matches_paths(&changed_paths) {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "path_filter".to_owned(),
            });
            continue;
        }

        let run = create_event_workflow_run(
            pool,
            EventWorkflowRunInput {
                repository: &repository,
                workflow: &workflow,
                parsed: &parsed,
                actor_user_id: Some(input.actor_user_id),
                head_branch: &pull_request.head_ref,
                head_sha: pull_request.head_sha.as_deref(),
                event: "pull_request",
                display_title: format!(
                    "{} #{}: {}",
                    input.action, pull_request.number, pull_request.title
                ),
                event_payload: json!({
                    "action": input.action,
                    "number": pull_request.number,
                    "pullRequestId": pull_request.id,
                    "baseRef": pull_request.base_ref,
                    "headRef": pull_request.head_ref,
                    "headSha": pull_request.head_sha,
                    "workflowPath": workflow.path,
                    "changedPaths": changed_paths,
                    "source": "pull_request",
                }),
                queue: "actions.workflow_pull_request",
                lease_prefix: "workflow-pull-request",
                pull_request_id: Some(pull_request.id),
                fork_pull_request: pull_request.is_fork,
            },
        )
        .await?;
        triggered_runs.push(run);
    }

    Ok(WorkflowTriggerResult {
        scanned_workflows: workflow_files.len(),
        triggered_runs,
        skipped_workflows,
    })
}

pub async fn trigger_workflows_for_schedule(
    pool: &PgPool,
    input: TriggerWorkflowsForSchedule,
) -> Result<WorkflowTriggerResult, AutomationError> {
    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    let default_ref_name = format!("refs/heads/{}", repository.default_branch);
    let workflow_files = workflow_files_for_ref(pool, repository.id, &default_ref_name).await?;
    let head_sha = head_sha_for_ref(pool, repository.id, &default_ref_name).await?;
    let mut triggered_runs = Vec::new();
    let mut skipped_workflows = Vec::new();

    for file in &workflow_files {
        let parsed = match parse_workflow_file(&file.content) {
            Ok(parsed) => parsed,
            Err(error) => {
                upsert_discovered_workflow(
                    pool,
                    &repository,
                    file,
                    &repository.default_branch,
                    DiscoveredWorkflow {
                        name: workflow_name_from_path(&file.path),
                        trigger_events: Vec::new(),
                        dispatch_enabled: false,
                        dispatch_inputs: Vec::new(),
                        yaml_parse_error: Some(sanitize_yaml_parse_error(error.to_string())),
                    },
                )
                .await?;
                skipped_workflows.push(WorkflowTriggerSkip {
                    path: file.path.clone(),
                    reason: "invalid_yaml".to_owned(),
                });
                continue;
            }
        };

        let workflow = upsert_discovered_workflow(
            pool,
            &repository,
            file,
            &repository.default_branch,
            parsed.discovered.clone(),
        )
        .await?;
        if workflow.state != WorkflowState::Active {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "disabled".to_owned(),
            });
            continue;
        }
        let Some(ref schedule_config) = parsed.schedule else {
            skipped_workflows.push(WorkflowTriggerSkip {
                path: file.path.clone(),
                reason: "schedule_not_configured".to_owned(),
            });
            continue;
        };
        if let Some(expected) = input.schedule.as_deref() {
            if !schedule_config.matches_schedule(expected) {
                skipped_workflows.push(WorkflowTriggerSkip {
                    path: file.path.clone(),
                    reason: "schedule_filter".to_owned(),
                });
                continue;
            }
        }

        let run = create_event_workflow_run(
            pool,
            EventWorkflowRunInput {
                repository: &repository,
                workflow: &workflow,
                parsed: &parsed,
                actor_user_id: None,
                head_branch: &repository.default_branch,
                head_sha: head_sha.as_deref(),
                event: "schedule",
                display_title: format!("{} scheduled", workflow.name),
                event_payload: json!({
                    "schedule": input.schedule,
                    "ref": default_ref_name,
                    "headBranch": repository.default_branch,
                    "headSha": head_sha,
                    "workflowPath": workflow.path,
                    "source": "scheduler",
                }),
                queue: "actions.workflow_schedule",
                lease_prefix: "workflow-schedule",
                pull_request_id: None,
                fork_pull_request: false,
            },
        )
        .await?;
        triggered_runs.push(run);
    }

    Ok(WorkflowTriggerResult {
        scanned_workflows: workflow_files.len(),
        triggered_runs,
        skipped_workflows,
    })
}

pub async fn dispatch_workflow_run(
    pool: &PgPool,
    input: DispatchWorkflowRun,
) -> Result<ActionsRunListItem, AutomationError> {
    let repository = require_repository(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let workflow =
        actions_workflow_detail_workflow(pool, &repository, &input.workflow_path).await?;
    if !workflow.valid {
        return Err(AutomationError::WorkflowDispatchDisabled(
            "the workflow YAML is invalid".to_owned(),
        ));
    }
    if workflow.state != WorkflowState::Active {
        return Err(AutomationError::WorkflowDispatchDisabled(
            "the workflow is disabled".to_owned(),
        ));
    }
    if !workflow.dispatch.enabled {
        return Err(AutomationError::WorkflowDispatchDisabled(
            "workflow_dispatch is not configured".to_owned(),
        ));
    }
    if workflow.source_branch != repository.default_branch {
        return Err(AutomationError::WorkflowDispatchDisabled(format!(
            "workflow_dispatch is only available from the default branch `{}`",
            repository.default_branch
        )));
    }

    let dispatch_inputs = validate_dispatch_inputs(&workflow.dispatch.inputs, input.inputs)?;
    let resolved_ref = resolve_workflow_dispatch_ref(pool, repository.id, &input.ref_name).await?;
    let run_number = next_run_number(pool, workflow.id).await?;
    let display_title = format!("Run {} manually", workflow.name);
    let runtime_context = resolve_actions_runtime_context(
        pool,
        ActionsRuntimeResolutionRequest {
            repository_id: repository.id,
            event: "workflow_dispatch".to_owned(),
            fork_pull_request: false,
            environment: None,
            environment_approved: false,
            explicit_secret_names: None,
        },
    )
    .await?;
    let event_payload = json!({
        "workflowPath": workflow.path,
        "ref": resolved_ref.name,
        "headBranch": resolved_ref.short_name,
        "headSha": resolved_ref.sha,
        "inputs": dispatch_inputs.clone(),
        "runtimePolicy": runtime_context.diagnostics,
    });

    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, head_branch,
            head_sha, event, display_title, event_payload
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'workflow_dispatch', $7, $8)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(workflow.id)
    .bind(input.actor_user_id)
    .bind(run_number)
    .bind(&resolved_ref.short_name)
    .bind(&resolved_ref.sha)
    .bind(&display_title)
    .bind(&event_payload)
    .fetch_one(&mut *transaction)
    .await?;
    let run_id: Uuid = row.get("id");

    sqlx::query(
        r#"
        INSERT INTO workflow_jobs (run_id, name, runner_label)
        VALUES ($1, 'workflow dispatch', 'ubuntu-latest')
        "#,
    )
    .bind(run_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    enqueue_job(
        pool,
        "actions.workflow_dispatch",
        &format!("workflow-dispatch:{run_id}"),
        json!({
            "repositoryId": repository.id,
            "workflowId": workflow.id,
            "workflowPath": workflow.path,
            "runId": run_id,
            "runNumber": run_number,
            "actorUserId": input.actor_user_id,
            "ref": resolved_ref.name,
            "headBranch": resolved_ref.short_name,
            "headSha": resolved_ref.sha,
            "inputs": dispatch_inputs,
            "runtimePolicy": runtime_context.diagnostics,
        }),
    )
    .await?;

    let workflow_id_filter = workflow.id.to_string();
    let mut runs = actions_run_items(
        pool,
        ActionsRunFilterRefs {
            repository_id: repository.id,
            q: None,
            workflow: Some(workflow_id_filter.as_str()),
            event: None,
            status: None,
            branch: None,
            actor: None,
        },
        1,
        0,
    )
    .await?;
    hydrate_job_summaries(pool, &mut runs).await?;
    runs.into_iter()
        .find(|run| run.id == run_id)
        .ok_or(AutomationError::WorkflowRunNotFound)
}

pub async fn rerun_workflow_run(
    pool: &PgPool,
    input: RerunWorkflowRun,
) -> Result<ActionsRunDetail, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let run =
        get_workflow_run_for_actor(pool, input.repository_id, input.run_id, input.actor_user_id)
            .await?;
    if !matches!(run.status, RunStatus::Completed | RunStatus::Cancelled) {
        return Err(AutomationError::WorkflowRunActionUnavailable(
            "only completed or cancelled runs can be re-run".to_owned(),
        ));
    }

    let latest_attempt = latest_attempt_number(pool, input.run_id).await?;
    if latest_attempt >= 10 {
        return Err(AutomationError::WorkflowRunActionUnavailable(
            "workflow run reached the re-run attempt limit".to_owned(),
        ));
    }
    let source_jobs = rerun_source_jobs(
        pool,
        input.run_id,
        latest_attempt,
        &input.mode,
        input.job_id,
    )
    .await?;
    if source_jobs.is_empty() {
        return Err(AutomationError::WorkflowRunActionUnavailable(
            "no jobs are eligible for this re-run".to_owned(),
        ));
    }

    let next_attempt = latest_attempt + 1;
    let trigger_kind = input.mode.as_str();
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_run_attempts (
            run_id, attempt_number, status, triggered_by_user_id, trigger_kind
        )
        VALUES ($1, $2, 'queued', $3, $4)
        "#,
    )
    .bind(input.run_id)
    .bind(next_attempt)
    .bind(input.actor_user_id)
    .bind(trigger_kind)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE workflow_runs
        SET status = 'queued',
            conclusion = NULL,
            actor_user_id = $2,
            started_at = NULL,
            completed_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(input.run_id)
    .bind(input.actor_user_id)
    .execute(&mut *tx)
    .await?;

    for source in &source_jobs {
        let new_job_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO workflow_jobs (
                id, run_id, name, status, conclusion, runner_label, attempt_number, group_name
            )
            VALUES ($1, $2, $3, 'queued', NULL, $4, $5, $6)
            "#,
        )
        .bind(new_job_id)
        .bind(input.run_id)
        .bind(&source.name)
        .bind(&source.runner_label)
        .bind(next_attempt)
        .bind(&source.group_name)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO workflow_steps (job_id, number, name, status, conclusion)
            SELECT $1, number, name, 'queued', NULL
            FROM workflow_steps
            WHERE job_id = $2
            ORDER BY number
            "#,
        )
        .bind(new_job_id)
        .bind(source.id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'workflow_run.rerun', 'workflow_run', $2, $3)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(input.run_id.to_string())
    .bind(json!({
        "repositoryId": input.repository_id,
        "attemptNumber": next_attempt,
        "mode": trigger_kind,
        "jobId": input.job_id,
        "jobCount": source_jobs.len(),
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    enqueue_job(
        pool,
        "actions.workflow_rerun",
        &format!("workflow-rerun:{}:{next_attempt}", input.run_id),
        json!({
            "repositoryId": input.repository_id,
            "workflowId": run.workflow_id,
            "runId": input.run_id,
            "attemptNumber": next_attempt,
            "actorUserId": input.actor_user_id,
            "mode": trigger_kind,
            "jobIds": source_jobs.iter().map(|job| job.id).collect::<Vec<_>>(),
        }),
    )
    .await?;

    actions_run_detail_for_viewer(
        pool,
        input.repository_id,
        Some(input.actor_user_id),
        input.run_id,
    )
    .await
}

pub async fn cancel_workflow_run(
    pool: &PgPool,
    input: MutateWorkflowRun,
) -> Result<ActionsRunDetail, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let run =
        get_workflow_run_for_actor(pool, input.repository_id, input.run_id, input.actor_user_id)
            .await?;
    if !matches!(run.status, RunStatus::Queued | RunStatus::InProgress) {
        return Err(AutomationError::WorkflowRunActionUnavailable(
            "only queued or in-progress runs can be cancelled".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE workflow_runs
        SET status = 'cancelled',
            conclusion = 'cancelled',
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'cancelled',
            conclusion = 'cancelled',
            completed_at = COALESCE(completed_at, now())
        WHERE run_id = $1 AND status IN ('queued', 'in_progress')
        "#,
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_run_attempts
        SET status = 'cancelled',
            conclusion = 'cancelled',
            completed_at = COALESCE(completed_at, now())
        WHERE run_id = $1 AND status IN ('queued', 'in_progress')
        "#,
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'workflow_run.cancelled', 'workflow_run', $2, $3)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(input.run_id.to_string())
    .bind(json!({ "repositoryId": input.repository_id }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    actions_run_detail_for_viewer(
        pool,
        input.repository_id,
        Some(input.actor_user_id),
        input.run_id,
    )
    .await
}

pub async fn delete_workflow_run_logs(
    pool: &PgPool,
    input: MutateWorkflowRun,
) -> Result<ActionsRunDetail, AutomationError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let run =
        get_workflow_run_for_actor(pool, input.repository_id, input.run_id, input.actor_user_id)
            .await?;
    if !matches!(run.status, RunStatus::Completed | RunStatus::Cancelled) {
        return Err(AutomationError::WorkflowRunActionUnavailable(
            "logs can only be deleted after a run reaches a terminal state".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE workflow_jobs SET log_deleted_at = COALESCE(log_deleted_at, now()) WHERE run_id = $1",
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        DELETE FROM workflow_job_log_lines
        WHERE job_id IN (SELECT id FROM workflow_jobs WHERE run_id = $1)
        "#,
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE job_logs
        SET bytes_written = 0,
            finalized_at = COALESCE(finalized_at, now())
        WHERE run_id = $1
        "#,
    )
    .bind(input.run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'workflow_run.logs_deleted', 'workflow_run', $2, $3)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(input.run_id.to_string())
    .bind(json!({ "repositoryId": input.repository_id }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    actions_run_detail_for_viewer(
        pool,
        input.repository_id,
        Some(input.actor_user_id),
        input.run_id,
    )
    .await
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

pub async fn repository_for_runner_by_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
) -> Result<Uuid, AutomationError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await
        .map_err(map_repository_error)?
        .ok_or(AutomationError::RepositoryNotFound)?;
    Ok(repository.id)
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
            if repository.owner_user_id == Some(user_id)
                || repository.visibility == RepositoryVisibility::Public
            {
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

async fn actions_run_items_by_run(
    pool: &PgPool,
    repository_id: Uuid,
    run_id: Uuid,
) -> Result<Vec<ActionsRunListItem>, AutomationError> {
    let rows = sqlx::query(
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
        WHERE workflow_runs.repository_id = $1 AND workflow_runs.id = $2
        "#
    )
    .bind(repository_id)
    .bind(run_id)
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

async fn actions_run_detail_workflow(
    pool: &PgPool,
    repository: &Repository,
    workflow_id: Uuid,
) -> Result<ActionsRunDetailWorkflow, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT id, name, path, state, source_branch, source_sha
        FROM actions_workflows
        WHERE id = $1 AND repository_id = $2
        "#,
    )
    .bind(workflow_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowNotFound)?;
    let state: String = row.get("state");
    let path: String = row.get("path");
    let source_branch = row
        .get::<Option<String>, _>("source_branch")
        .unwrap_or_else(|| repository.default_branch.clone());

    Ok(ActionsRunDetailWorkflow {
        id: row.get("id"),
        name: row.get("name"),
        path: path.clone(),
        state: WorkflowState::try_from(state.as_str())?,
        source_branch: source_branch.clone(),
        source_sha: row.get("source_sha"),
        source_href: format!(
            "/{}/{}/blob/{}/{}",
            repository.owner_login, repository.name, source_branch, path
        ),
    })
}

async fn actions_run_attempts(
    pool: &PgPool,
    run: &ActionsRunListItem,
) -> Result<Vec<ActionsRunAttempt>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT workflow_run_attempts.id,
               workflow_run_attempts.attempt_number,
               workflow_run_attempts.status,
               workflow_run_attempts.conclusion,
               workflow_run_attempts.trigger_kind,
               workflow_run_attempts.triggered_by_user_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               users.display_name AS actor_display_name,
               users.avatar_url AS actor_avatar_url,
               workflow_run_attempts.started_at,
               workflow_run_attempts.completed_at,
               workflow_run_attempts.created_at
        FROM workflow_run_attempts
        LEFT JOIN users ON users.id = workflow_run_attempts.triggered_by_user_id
        WHERE workflow_run_attempts.run_id = $1
        ORDER BY workflow_run_attempts.attempt_number
        "#,
    )
    .bind(run.id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(vec![ActionsRunAttempt {
            id: None,
            attempt_number: 1,
            status: run.status.clone(),
            conclusion: run.conclusion.clone(),
            trigger_kind: "initial".to_owned(),
            actor: run.actor.clone(),
            started_at: run.started_at,
            completed_at: run.completed_at,
            created_at: run.created_at,
        }]);
    }

    rows.into_iter()
        .map(|row| {
            let actor_user_id: Option<Uuid> = row.get("triggered_by_user_id");
            Ok(ActionsRunAttempt {
                id: row.get("id"),
                attempt_number: row.get("attempt_number"),
                status: row.get("status"),
                conclusion: row.get("conclusion"),
                trigger_kind: row.get("trigger_kind"),
                actor: actor_user_id.map(|id| ActionsActor {
                    id,
                    login: row.get("actor_login"),
                    display_name: row.get("actor_display_name"),
                    avatar_url: row.get("actor_avatar_url"),
                }),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                created_at: row.get("created_at"),
            })
        })
        .collect()
}

async fn actions_run_jobs(
    pool: &PgPool,
    run_id: Uuid,
) -> Result<Vec<ActionsRunJobDetail>, AutomationError> {
    let job_rows = sqlx::query(
        r#"
        SELECT id, name, group_name, attempt_number, status, conclusion, runner_label,
               log_storage_key, log_deleted_at, started_at, completed_at, created_at, updated_at
        FROM workflow_jobs
        WHERE run_id = $1
        ORDER BY attempt_number, COALESCE(group_name, ''), created_at, name
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    let job_ids = job_rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let mut steps_by_job: HashMap<Uuid, Vec<ActionsRunStepDetail>> = HashMap::new();
    if !job_ids.is_empty() {
        let step_rows = sqlx::query(
            r#"
            SELECT id, job_id, number, name, status, conclusion, started_at, completed_at
            FROM workflow_steps
            WHERE job_id = ANY($1)
            ORDER BY job_id, number
            "#,
        )
        .bind(&job_ids)
        .fetch_all(pool)
        .await?;
        for row in step_rows {
            let job_id = row.get::<Uuid, _>("job_id");
            let started_at: Option<DateTime<Utc>> = row.get("started_at");
            let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
            steps_by_job
                .entry(job_id)
                .or_default()
                .push(ActionsRunStepDetail {
                    id: row.get("id"),
                    number: row.get("number"),
                    name: row.get("name"),
                    status: row.get("status"),
                    conclusion: row.get("conclusion"),
                    duration_seconds: duration_seconds(started_at, completed_at),
                    started_at,
                    completed_at,
                });
        }
    }

    Ok(job_rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let started_at: Option<DateTime<Utc>> = row.get("started_at");
            let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
            let log_deleted_at: Option<DateTime<Utc>> = row.get("log_deleted_at");
            let log_storage_key: Option<String> = row.get("log_storage_key");
            ActionsRunJobDetail {
                id,
                name: row.get("name"),
                group_name: row.get("group_name"),
                attempt_number: row.get("attempt_number"),
                status: row.get("status"),
                conclusion: row.get("conclusion"),
                runner_label: row.get("runner_label"),
                duration_seconds: duration_seconds(started_at, completed_at),
                log_available: log_storage_key.is_some() && log_deleted_at.is_none(),
                log_deleted_at,
                steps: steps_by_job.remove(&id).unwrap_or_default(),
                started_at,
                completed_at,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn actions_run_annotations(
    pool: &PgPool,
    run_id: Uuid,
) -> Result<Vec<ActionsRunAnnotation>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, job_id, step_id, annotation_level, path, start_line, end_line,
               title, message, raw_details, created_at
        FROM workflow_annotations
        WHERE run_id = $1
        ORDER BY created_at, path, start_line
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ActionsRunAnnotation {
            id: row.get("id"),
            job_id: row.get("job_id"),
            step_id: row.get("step_id"),
            level: row.get("annotation_level"),
            path: row.get("path"),
            start_line: row.get("start_line"),
            end_line: row.get("end_line"),
            title: row.get("title"),
            message: row.get("message"),
            raw_details: row.get("raw_details"),
            created_at: row.get("created_at"),
        })
        .collect())
}

async fn actions_run_artifacts(
    pool: &PgPool,
    run_id: Uuid,
) -> Result<Vec<ActionsRunArtifact>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, digest, size_bytes, storage_key, retention_days, expired_at,
               deleted_at, created_at, updated_at
        FROM workflow_artifacts
        WHERE run_id = $1 AND deleted_at IS NULL
        ORDER BY lower(name)
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    let now = Utc::now();

    Ok(rows
        .into_iter()
        .map(|row| {
            let storage_key: Option<String> = row.get("storage_key");
            let expired_at: Option<DateTime<Utc>> = row.get("expired_at");
            let deleted_at: Option<DateTime<Utc>> = row.get("deleted_at");
            ActionsRunArtifact {
                id: row.get("id"),
                name: row.get("name"),
                digest: row.get("digest"),
                size_bytes: row.get("size_bytes"),
                retention_days: row.get("retention_days"),
                expired_at,
                download_available: storage_key.is_some()
                    && deleted_at.is_none()
                    && expired_at.map(|value| value > now).unwrap_or(true),
                delete_available: deleted_at.is_none(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

pub async fn workflow_job_logs_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    job_id: Uuid,
    query: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<ActionsJobLog, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let job = workflow_job_for_repository(pool, repository_id, job_id).await?;
    if job.log_deleted_at.is_some() {
        return Err(AutomationError::WorkflowLogsUnavailable);
    }

    let page = page.max(1);
    let page_size = page_size.clamp(1, 500);
    let offset = (page - 1) * page_size;
    let query = cleaned_filter(query);
    let like_query = query
        .as_ref()
        .map(|value| format!("%{}%", escape_like(value)));
    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM workflow_job_log_lines
        WHERE job_id = $1
          AND ($2::text IS NULL OR content ILIKE $2 ESCAPE '\')
        "#,
    )
    .bind(job_id)
    .bind(like_query.as_deref())
    .fetch_one(pool)
    .await?;
    let rows = sqlx::query(
        r#"
        SELECT line_number, timestamp, content
        FROM workflow_job_log_lines
        WHERE job_id = $1
          AND ($2::text IS NULL OR content ILIKE $2 ESCAPE '\')
        ORDER BY line_number
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(job_id)
    .bind(like_query.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let redaction_values = actions_secret_redaction_values(pool, repository_id).await?;
    let lines = rows
        .into_iter()
        .map(|row| {
            let line_number: i32 = row.get("line_number");
            ActionsJobLogLine {
                line_number,
                timestamp: row.get("timestamp"),
                content: mask_actions_secret_values(
                    &row.get::<String, _>("content"),
                    &redaction_values,
                ),
                anchor: format!("L{line_number}"),
            }
        })
        .collect();

    Ok(ActionsJobLog {
        job,
        lines,
        total,
        page,
        page_size,
        query,
        download_href: format!(
            "/api/repos/{}/{}/actions/jobs/{}/logs/download",
            repository.owner_login, repository.name, job_id
        ),
    })
}

pub async fn workflow_job_log_stream_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    job_id: Uuid,
    after: Option<i32>,
) -> Result<WorkflowJobLogStream, AutomationError> {
    require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let job = workflow_job_for_repository(pool, repository_id, job_id).await?;
    if job.log_deleted_at.is_some() {
        return Err(AutomationError::WorkflowLogsUnavailable);
    }
    let after = after.unwrap_or(0).max(0);
    let rows = sqlx::query(
        r#"
        SELECT line_number, timestamp, content
        FROM workflow_job_log_lines
        WHERE job_id = $1 AND line_number > $2
        ORDER BY line_number
        LIMIT 500
        "#,
    )
    .bind(job_id)
    .bind(after)
    .fetch_all(pool)
    .await?;
    let redaction_values = actions_secret_redaction_values(pool, repository_id).await?;
    let lines = rows
        .into_iter()
        .map(|row| {
            let line_number: i32 = row.get("line_number");
            ActionsJobLogLine {
                line_number,
                timestamp: row.get("timestamp"),
                content: mask_actions_secret_values(
                    &row.get::<String, _>("content"),
                    &redaction_values,
                ),
                anchor: format!("L{line_number}"),
            }
        })
        .collect::<Vec<_>>();
    let next_cursor = lines.last().map(|line| line.line_number).or(Some(after));
    let finalized_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT finalized_at FROM job_logs WHERE job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(WorkflowJobLogStream {
        job,
        lines,
        next_cursor,
        finalized_at,
    })
}

pub async fn workflow_job_log_download_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    job_id: Uuid,
) -> Result<(String, String), AutomationError> {
    let log =
        workflow_job_logs_for_viewer(pool, repository_id, actor_user_id, job_id, None, 1, 500)
            .await?;
    if log.total > log.lines.len() as i64 {
        let rows = sqlx::query(
            r#"
            SELECT timestamp, content
            FROM workflow_job_log_lines
            WHERE job_id = $1
            ORDER BY line_number
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;
        let redaction_values = actions_secret_redaction_values(pool, repository_id).await?;
        let body = rows
            .into_iter()
            .map(|row| {
                let content =
                    mask_actions_secret_values(&row.get::<String, _>("content"), &redaction_values);
                format_log_line(row.get("timestamp"), content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok((format!("{}.log", safe_filename(&log.job.name)), body));
    }

    let body = log
        .lines
        .iter()
        .map(|line| format_log_line(line.timestamp, line.content.clone()))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((format!("{}.log", safe_filename(&log.job.name)), body))
}

pub async fn actions_runner_settings_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<RepositoryActionsRunnerSettings, AutomationError> {
    let repository =
        require_repository(pool, repository_id, actor_user_id, RepositoryRole::Admin).await?;
    ensure_runner_settings(pool, repository_id, actor_user_id).await?;
    mark_timed_out_runners(pool, repository_id).await?;
    let viewer_permission = viewer_permission(pool, &repository, Some(actor_user_id)).await?;
    let runners = actions_runners(pool, repository_id).await?;
    let queue = actions_runner_queue(pool, repository_id).await?;
    let workflow_permissions = actions_workflow_permissions(pool, repository_id).await?;
    let token = sqlx::query_scalar::<_, String>(
        r#"
        SELECT registration_token
        FROM actions_runners
        WHERE repository_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| format!("ogr_{}", Uuid::new_v4().simple()));

    Ok(RepositoryActionsRunnerSettings {
        repository: ActionsDashboardRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.clone(),
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission,
        can_manage_runners: true,
        runners,
        queue,
        workflow_permissions,
        setup: ActionsRunnerSetup {
            registration_token: Some(token.clone()),
            docker_command: Some(format!(
                "docker run --rm -e OPENGITHUB_RUNNER_TOKEN={} opengithub/runner:latest --repo {}/{}",
                token, repository.owner_login, repository.name
            )),
            expires_in_minutes: 60,
        },
    })
}

pub async fn create_actions_runner(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    input: CreateActionsRunner,
) -> Result<RepositoryActionsRunnerSettings, AutomationError> {
    require_repository(pool, repository_id, actor_user_id, RepositoryRole::Admin).await?;
    let name = input.name.trim();
    let labels = normalize_runner_labels(input.labels);
    if name.is_empty() || labels.is_empty() {
        return Err(AutomationError::InvalidWorkflowDispatch(
            "runner name and at least one label are required".to_owned(),
        ));
    }
    let token = format!("ogr_{}", Uuid::new_v4().simple());
    sqlx::query(
        r#"
        INSERT INTO actions_runners (repository_id, name, labels, status, registration_token, created_by_user_id)
        VALUES ($1, $2, $3, 'offline', $4, $5)
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .bind(json!(labels))
    .bind(token)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    actions_runner_settings_for_viewer(pool, repository_id, actor_user_id).await
}

pub async fn update_actions_runner_settings(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    input: UpdateActionsRunnerSettings,
) -> Result<RepositoryActionsRunnerSettings, AutomationError> {
    require_repository(pool, repository_id, actor_user_id, RepositoryRole::Admin).await?;
    if !(1..=64).contains(&input.concurrency_limit) {
        return Err(AutomationError::InvalidWorkflowDispatch(
            "concurrency limit must be between 1 and 64".to_owned(),
        ));
    }
    let github_token_permission =
        normalize_github_token_permission(&input.github_token_permission)?;
    sqlx::query(
        r#"
        INSERT INTO actions_runner_settings
            (
                repository_id, concurrency_limit, cancel_in_progress,
                workflow_token_permission, allow_actions_approve_pull_requests,
                updated_by_user_id
            )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (repository_id) DO UPDATE
        SET concurrency_limit = EXCLUDED.concurrency_limit,
            cancel_in_progress = EXCLUDED.cancel_in_progress,
            workflow_token_permission = EXCLUDED.workflow_token_permission,
            allow_actions_approve_pull_requests = EXCLUDED.allow_actions_approve_pull_requests,
            updated_by_user_id = EXCLUDED.updated_by_user_id
        "#,
    )
    .bind(repository_id)
    .bind(input.concurrency_limit)
    .bind(input.cancel_in_progress)
    .bind(&github_token_permission)
    .bind(input.allow_pull_request_approval)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    actions_runner_settings_for_viewer(pool, repository_id, actor_user_id).await
}

pub async fn record_runner_heartbeat(
    pool: &PgPool,
    repository_id: Uuid,
    input: RunnerHeartbeat,
) -> Result<ActionsRunner, AutomationError> {
    let status = match input.status.as_str() {
        "online" | "busy" | "offline" => input.status,
        other => {
            return Err(AutomationError::InvalidWorkflowDispatch(format!(
                "unsupported runner status `{other}`"
            )))
        }
    };
    let updated = sqlx::query_scalar::<_, bool>(
        r#"
        UPDATE actions_runners
        SET status = $3,
            last_heartbeat = CASE WHEN $3 = 'offline' THEN last_heartbeat ELSE now() END,
            busy_since = CASE WHEN $3 = 'busy' THEN COALESCE(busy_since, now()) ELSE NULL END
        WHERE repository_id = $1 AND id = $2
          AND registration_token = $4
          AND length(trim($4)) > 0
        RETURNING true
        "#,
    )
    .bind(repository_id)
    .bind(input.runner_id)
    .bind(&status)
    .bind(input.runner_token.trim())
    .fetch_optional(pool)
    .await?
    .unwrap_or(false);
    if !updated {
        return Err(AutomationError::WorkflowJobNotFound);
    }
    actions_runners(pool, repository_id)
        .await?
        .into_iter()
        .find(|runner| runner.id == input.runner_id)
        .ok_or(AutomationError::WorkflowJobNotFound)
}

pub async fn append_workflow_job_log_chunk(
    pool: &PgPool,
    input: AppendWorkflowJobLogChunk,
) -> Result<WorkflowJobLogChunkResult, AutomationError> {
    let content = input.content.replace("\r\n", "\n").replace('\r', "\n");
    if content.is_empty() && !input.finalize {
        return Err(AutomationError::InvalidWorkflowDispatch(
            "log chunk content is required unless finalizing".to_owned(),
        ));
    }
    let job_row = sqlx::query(
        r#"
        SELECT workflow_jobs.id,
               workflow_jobs.run_id,
               workflow_jobs.name,
               workflow_jobs.runner_id,
               workflow_runs.repository_id,
               actions_runners.registration_token
        FROM workflow_jobs
        JOIN workflow_runs ON workflow_runs.id = workflow_jobs.run_id
        LEFT JOIN actions_runners ON actions_runners.id = workflow_jobs.runner_id
        WHERE workflow_jobs.id = $1
        "#,
    )
    .bind(input.job_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowJobNotFound)?;
    let runner_id: Option<Uuid> = job_row.get("runner_id");
    let registration_token: Option<String> = job_row.get("registration_token");
    if runner_id.is_none()
        || registration_token.as_deref() != Some(input.runner_token.trim())
        || input.runner_token.trim().is_empty()
    {
        return Err(AutomationError::RepositoryAccessDenied);
    }
    if let Some(step_id) = input.step_id {
        let step_belongs = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM workflow_steps WHERE id = $1 AND job_id = $2)",
        )
        .bind(step_id)
        .bind(input.job_id)
        .fetch_one(pool)
        .await?;
        if !step_belongs {
            return Err(AutomationError::WorkflowJobNotFound);
        }
    }

    let run_id: Uuid = job_row.get("run_id");
    let job_name: String = job_row.get("name");
    let s3_key = format!("actions/{run_id}/{}/log.txt", safe_filename(&job_name));
    let current_max = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max(line_number) FROM workflow_job_log_lines WHERE job_id = $1",
    )
    .bind(input.job_id)
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    let lines = content
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO workflow_job_log_lines (job_id, step_id, line_number, timestamp, content)
            VALUES ($1, $2, $3, COALESCE($4, now()), $5)
            ON CONFLICT (job_id, line_number) DO UPDATE
            SET timestamp = EXCLUDED.timestamp,
                content = EXCLUDED.content,
                step_id = EXCLUDED.step_id
            "#,
        )
        .bind(input.job_id)
        .bind(input.step_id)
        .bind(current_max + index as i32 + 1)
        .bind(input.timestamp)
        .bind(line)
        .execute(pool)
        .await?;
        if let Some(command) = parse_workflow_annotation_command(line) {
            sqlx::query(
                r#"
                INSERT INTO workflow_annotations (
                    run_id, job_id, step_id, annotation_level, path, start_line, end_line,
                    title, message, raw_details
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(run_id)
            .bind(input.job_id)
            .bind(input.step_id)
            .bind(command.level)
            .bind(command.path)
            .bind(command.start_line)
            .bind(command.end_line)
            .bind(command.title)
            .bind(command.message)
            .bind(line)
            .execute(pool)
            .await?;
        }
    }
    let byte_delta = content.len() as i64;
    let metadata = sqlx::query(
        r#"
        INSERT INTO job_logs (job_id, run_id, job_name, s3_key, bytes_written, finalized_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $6 THEN now() ELSE NULL END)
        ON CONFLICT (job_id) DO UPDATE
        SET job_name = EXCLUDED.job_name,
            s3_key = EXCLUDED.s3_key,
            bytes_written = job_logs.bytes_written + EXCLUDED.bytes_written,
            finalized_at = CASE
                WHEN $6 THEN COALESCE(job_logs.finalized_at, now())
                ELSE job_logs.finalized_at
            END
        RETURNING bytes_written, finalized_at
        "#,
    )
    .bind(input.job_id)
    .bind(run_id)
    .bind(&job_name)
    .bind(&s3_key)
    .bind(byte_delta)
    .bind(input.finalize)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET log_storage_key = $2,
            log_deleted_at = NULL,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(input.job_id)
    .bind(&s3_key)
    .execute(pool)
    .await?;
    if input.finalize {
        sqlx::query(
            r#"
            UPDATE workflow_job_assignments
            SET status = CASE WHEN status = 'in_progress' THEN 'completed' ELSE status END
            WHERE job_id = $1
            "#,
        )
        .bind(input.job_id)
        .execute(pool)
        .await?;
        if let Some(runner_id) = runner_id {
            sqlx::query(
                r#"
                UPDATE actions_runners
                SET status = 'online', busy_since = NULL, last_heartbeat = now()
                WHERE id = $1 AND status = 'busy'
                "#,
            )
            .bind(runner_id)
            .execute(pool)
            .await?;
        }
    }
    let next_cursor = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max(line_number) FROM workflow_job_log_lines WHERE job_id = $1",
    )
    .bind(input.job_id)
    .fetch_one(pool)
    .await?;

    Ok(WorkflowJobLogChunkResult {
        job_id: input.job_id,
        run_id,
        s3_key,
        bytes_written: metadata.get("bytes_written"),
        appended_lines: lines.len() as i64,
        next_cursor,
        finalized_at: metadata.get("finalized_at"),
    })
}

struct ParsedWorkflowAnnotation {
    level: &'static str,
    path: Option<String>,
    start_line: Option<i32>,
    end_line: Option<i32>,
    title: Option<String>,
    message: String,
}

fn parse_workflow_annotation_command(line: &str) -> Option<ParsedWorkflowAnnotation> {
    let (kind, rest) = line.strip_prefix("::")?.split_once("::")?;
    let level = if kind.starts_with("error") {
        "failure"
    } else if kind.starts_with("warning") {
        "warning"
    } else if kind.starts_with("notice") {
        "notice"
    } else {
        return None;
    };
    let (metadata, message) = kind
        .split_once(' ')
        .map(|(_, metadata)| (metadata, rest))
        .unwrap_or(("", rest));
    let mut path = None;
    let mut start_line = None;
    let mut end_line = None;
    let mut title = None;
    for part in metadata.split(',').filter(|part| !part.is_empty()) {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        match key {
            "file" => path = Some(value.to_owned()),
            "line" => start_line = value.parse::<i32>().ok(),
            "endLine" => end_line = value.parse::<i32>().ok(),
            "title" => title = Some(value.to_owned()),
            _ => {}
        }
    }
    Some(ParsedWorkflowAnnotation {
        level,
        path,
        start_line,
        end_line,
        title,
        message: message.to_owned(),
    })
}

pub async fn schedule_queued_action_jobs(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<RunnerScheduleResult, AutomationError> {
    require_repository(pool, repository_id, actor_user_id, RepositoryRole::Admin).await?;
    ensure_runner_settings(pool, repository_id, actor_user_id).await?;
    mark_timed_out_runners(pool, repository_id).await?;
    let queue = actions_runner_queue(pool, repository_id).await?;
    let available_slots = (queue.concurrency_limit as i64 - queue.busy_runners).max(0);
    if available_slots == 0 {
        return Ok(RunnerScheduleResult {
            assigned: Vec::new(),
            queued_jobs: queue.queued_jobs,
        });
    }

    let jobs = sqlx::query(
        r#"
        SELECT workflow_jobs.id AS job_id, workflow_jobs.run_id, workflow_jobs.name AS job_name,
               workflow_jobs.runner_label, workflow_runs.run_number, actions_workflows.name AS workflow_name
        FROM workflow_jobs
        JOIN workflow_runs ON workflow_runs.id = workflow_jobs.run_id
        JOIN actions_workflows ON actions_workflows.id = workflow_runs.workflow_id
        WHERE workflow_runs.repository_id = $1
          AND workflow_jobs.status = 'queued'
        ORDER BY workflow_jobs.created_at
        LIMIT $2
        "#,
    )
    .bind(repository_id)
    .bind(available_slots)
    .fetch_all(pool)
    .await?;

    let mut assigned = Vec::new();
    for job in jobs {
        let runner = find_available_runner_for_label(
            pool,
            repository_id,
            job.get::<Option<String>, _>("runner_label"),
        )
        .await?;
        let Some(runner_id) = runner else { continue };
        let job_id: Uuid = job.get("job_id");
        let run_id: Uuid = job.get("run_id");
        let job_name: String = job.get("job_name");
        let row = sqlx::query(
            r#"
            WITH updated_job AS (
                UPDATE workflow_jobs
                SET status = 'in_progress', runner_id = $1, assigned_at = now(), started_at = COALESCE(started_at, now())
                WHERE id = $2 AND status = 'queued'
                RETURNING id
            ), updated_run AS (
                UPDATE workflow_runs
                SET status = 'in_progress', started_at = COALESCE(started_at, now())
                WHERE id = $3 AND status = 'queued'
                RETURNING id
            ), updated_runner AS (
                UPDATE actions_runners
                SET status = 'busy', busy_since = COALESCE(busy_since, now()), last_heartbeat = COALESCE(last_heartbeat, now())
                WHERE id = $1
                RETURNING id
            )
            INSERT INTO workflow_job_assignments (run_id, job_id, job_name, runner_id)
            SELECT $3, $2, $4, $1
            WHERE EXISTS (SELECT 1 FROM updated_job)
            ON CONFLICT (job_id) DO NOTHING
            RETURNING started_at
            "#,
        )
        .bind(runner_id)
        .bind(job_id)
        .bind(run_id)
        .bind(&job_name)
        .fetch_optional(pool)
        .await?;
        if let Some(row) = row {
            assigned.push(ActionsRunnerJob {
                run_id,
                job_id,
                job_name,
                run_number: job.get("run_number"),
                workflow_name: job.get("workflow_name"),
                started_at: row.get("started_at"),
            });
        }
    }
    let queued_jobs = actions_runner_queue(pool, repository_id).await?.queued_jobs;
    Ok(RunnerScheduleResult {
        assigned,
        queued_jobs,
    })
}

async fn ensure_runner_settings(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), AutomationError> {
    sqlx::query(
        r#"
        INSERT INTO actions_runner_settings (repository_id, updated_by_user_id)
        VALUES ($1, $2)
        ON CONFLICT (repository_id) DO NOTHING
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_timed_out_runners(pool: &PgPool, repository_id: Uuid) -> Result<(), AutomationError> {
    sqlx::query(
        r#"
        UPDATE actions_runners
        SET status = 'offline', busy_since = NULL
        WHERE repository_id = $1
          AND status IN ('online', 'busy')
          AND last_heartbeat IS NOT NULL
          AND last_heartbeat < now() - interval '5 minutes'
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_jobs
        SET status = 'completed',
            conclusion = 'timed_out',
            completed_at = COALESCE(completed_at, now())
        WHERE runner_id IN (
            SELECT id FROM actions_runners WHERE repository_id = $1 AND status = 'offline'
        )
          AND status = 'in_progress'
          AND assigned_at < now() - interval '5 minutes'
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE workflow_job_assignments
        SET status = 'timed_out'
        WHERE runner_id IN (
            SELECT id FROM actions_runners WHERE repository_id = $1 AND status = 'offline'
        )
          AND status = 'in_progress'
          AND started_at < now() - interval '5 minutes'
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn actions_runners(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsRunner>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT actions_runners.id, actions_runners.name, actions_runners.labels,
               actions_runners.status, actions_runners.last_heartbeat,
               actions_runners.busy_since, actions_runners.created_at,
               actions_runners.updated_at,
               workflow_job_assignments.run_id, workflow_job_assignments.job_id,
               workflow_job_assignments.job_name, workflow_job_assignments.started_at,
               workflow_runs.run_number, actions_workflows.name AS workflow_name
        FROM actions_runners
        LEFT JOIN workflow_job_assignments
          ON workflow_job_assignments.runner_id = actions_runners.id
         AND workflow_job_assignments.status = 'in_progress'
        LEFT JOIN workflow_runs ON workflow_runs.id = workflow_job_assignments.run_id
        LEFT JOIN actions_workflows ON actions_workflows.id = workflow_runs.workflow_id
        WHERE actions_runners.repository_id = $1
        ORDER BY actions_runners.created_at DESC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let labels_value: Value = row.get("labels");
            let labels = labels_value
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            let current_job = row
                .get::<Option<Uuid>, _>("job_id")
                .map(|job_id| ActionsRunnerJob {
                    run_id: row.get("run_id"),
                    job_id,
                    job_name: row.get("job_name"),
                    run_number: row.get("run_number"),
                    workflow_name: row.get("workflow_name"),
                    started_at: row.get("started_at"),
                });
            ActionsRunner {
                id: row.get("id"),
                name: row.get("name"),
                labels,
                status: row.get("status"),
                last_heartbeat: row.get("last_heartbeat"),
                busy_since: row.get("busy_since"),
                current_job,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn actions_runner_queue(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<ActionsRunnerQueue, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT count(*) FROM workflow_jobs
             JOIN workflow_runs ON workflow_runs.id = workflow_jobs.run_id
             WHERE workflow_runs.repository_id = $1 AND workflow_jobs.status = 'queued')::bigint AS queued_jobs,
            (SELECT count(*) FROM actions_runners WHERE repository_id = $1 AND status = 'busy')::bigint AS busy_runners,
            (SELECT count(*) FROM actions_runners WHERE repository_id = $1 AND status IN ('online', 'busy'))::bigint AS online_runners,
            (SELECT count(*) FROM actions_runners WHERE repository_id = $1 AND status = 'offline')::bigint AS offline_runners,
            actions_runner_settings.concurrency_limit,
            actions_runner_settings.cancel_in_progress
        FROM actions_runner_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;

    Ok(ActionsRunnerQueue {
        queued_jobs: row.get("queued_jobs"),
        busy_runners: row.get("busy_runners"),
        online_runners: row.get("online_runners"),
        offline_runners: row.get("offline_runners"),
        concurrency_limit: row.get("concurrency_limit"),
        cancel_in_progress: row.get("cancel_in_progress"),
    })
}

async fn actions_workflow_permissions(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<ActionsWorkflowPermissions, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT workflow_token_permission, allow_actions_approve_pull_requests
        FROM actions_runner_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;
    let permission = row
        .as_ref()
        .map(|row| row.get::<String, _>("workflow_token_permission"))
        .unwrap_or_else(|| "read".to_owned());
    let allow_pull_request_approval = row
        .as_ref()
        .map(|row| row.get::<bool, _>("allow_actions_approve_pull_requests"))
        .unwrap_or(false);
    Ok(ActionsWorkflowPermissions {
        github_token_scopes: github_token_scopes(&permission, allow_pull_request_approval),
        github_token_permission: permission,
        allow_pull_request_approval,
    })
}

fn normalize_github_token_permission(value: &str) -> Result<String, AutomationError> {
    match value.trim() {
        "read" | "read-only" => Ok("read".to_owned()),
        "write" | "read-write" => Ok("write".to_owned()),
        _ => Err(AutomationError::InvalidWorkflowDispatch(
            "GITHUB_TOKEN permission must be read or write".to_owned(),
        )),
    }
}

fn github_token_scopes(permission: &str, allow_pull_request_approval: bool) -> Vec<String> {
    let mut scopes = vec![
        "contents:read".to_owned(),
        "metadata:read".to_owned(),
        "packages:read".to_owned(),
    ];
    if permission == "write" {
        scopes.extend([
            "actions:write".to_owned(),
            "checks:write".to_owned(),
            "contents:write".to_owned(),
            "issues:write".to_owned(),
            "packages:write".to_owned(),
            "pull-requests:write".to_owned(),
        ]);
        if allow_pull_request_approval {
            scopes.push("pull-requests:approve".to_owned());
        }
    }
    scopes
}

async fn find_available_runner_for_label(
    pool: &PgPool,
    repository_id: Uuid,
    runner_label: Option<String>,
) -> Result<Option<Uuid>, AutomationError> {
    let labels = normalize_runner_labels(vec![
        "self-hosted".to_owned(),
        runner_label.unwrap_or_else(|| "ubuntu-latest".to_owned()),
    ]);
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM actions_runners
        WHERE repository_id = $1
          AND status = 'online'
          AND labels ?& $2
        ORDER BY last_heartbeat DESC NULLS LAST, created_at
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(&labels)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

fn normalize_runner_labels(labels: Vec<String>) -> Vec<String> {
    let mut normalized = labels
        .into_iter()
        .flat_map(|label| {
            label
                .split(',')
                .map(|part| part.trim().to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

pub async fn workflow_run_log_archive_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    run_id: Uuid,
) -> Result<ActionsRunLogArchive, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let run = actions_run_items_by_run(pool, repository_id, run_id)
        .await?
        .into_iter()
        .next()
        .ok_or(AutomationError::WorkflowRunNotFound)?;
    let jobs = actions_run_jobs(pool, run_id).await?;
    if jobs.iter().all(|job| !job.log_available) {
        return Err(AutomationError::WorkflowLogsUnavailable);
    }

    let job_ids = jobs
        .iter()
        .filter(|job| job.log_available)
        .map(|job| job.id)
        .collect::<Vec<_>>();
    let rows = sqlx::query(
        r#"
        SELECT workflow_jobs.id AS job_id,
               workflow_jobs.name AS job_name,
               workflow_job_log_lines.timestamp,
               workflow_job_log_lines.content
        FROM workflow_jobs
        JOIN workflow_job_log_lines ON workflow_job_log_lines.job_id = workflow_jobs.id
        WHERE workflow_jobs.id = ANY($1)
        ORDER BY workflow_jobs.created_at, workflow_jobs.name, workflow_job_log_lines.line_number
        "#,
    )
    .bind(&job_ids)
    .fetch_all(pool)
    .await?;
    let redaction_values = actions_secret_redaction_values(pool, repository_id).await?;

    let mut body = format!(
        "opengithub workflow log archive\nrepository: {}/{}\nrun: #{}\n\n",
        repository.owner_login, repository.name, run.run_number
    );
    let mut current_job: Option<Uuid> = None;
    for row in rows {
        let job_id: Uuid = row.get("job_id");
        if current_job != Some(job_id) {
            current_job = Some(job_id);
            body.push_str(&format!("\n== {} ==\n", row.get::<String, _>("job_name")));
        }
        let content =
            mask_actions_secret_values(&row.get::<String, _>("content"), &redaction_values);
        body.push_str(&format_log_line(row.get("timestamp"), content));
        body.push('\n');
    }

    Ok(ActionsRunLogArchive {
        run_id,
        filename: format!("run-{}-logs.txt", run.run_number),
        content_type: "text/plain; charset=utf-8".to_owned(),
        body,
    })
}

pub async fn workflow_artifact_download_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    artifact_id: Uuid,
) -> Result<ActionsArtifactDownload, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        SELECT workflow_artifacts.id, workflow_artifacts.name, workflow_artifacts.storage_key,
               workflow_artifacts.expired_at
        FROM workflow_artifacts
        JOIN workflow_runs ON workflow_runs.id = workflow_artifacts.run_id
        WHERE workflow_artifacts.id = $1 AND workflow_runs.repository_id = $2
        "#,
    )
    .bind(artifact_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowArtifactNotFound)?;
    let storage_key: Option<String> = row.get("storage_key");
    let expired_at: Option<DateTime<Utc>> = row.get("expired_at");
    if storage_key.is_none() || expired_at.map(|value| value <= Utc::now()).unwrap_or(false) {
        return Err(AutomationError::WorkflowArtifactUnavailable);
    }
    let name: String = row.get("name");
    let filename = format!("{}.zip", safe_filename(&name));
    Ok(ActionsArtifactDownload {
        artifact_id,
        name,
        filename,
        download_url: format!(
            "/api/repos/{}/{}/actions/artifacts/{}/download?token=dev-local",
            repository.owner_login, repository.name, artifact_id
        ),
        storage_key: storage_key.unwrap_or_default(),
        expires_at: Utc::now() + chrono::Duration::minutes(10),
    })
}

pub async fn upload_workflow_artifact_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    run_id: Uuid,
    actor_user_id: Uuid,
    request: ActionsArtifactUpload,
) -> Result<ActionsRunArtifact, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Write).await?;
    let run_exists = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM workflow_runs WHERE id = $1 AND repository_id = $2",
    )
    .bind(run_id)
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    if run_exists == 0 {
        return Err(AutomationError::WorkflowRunNotFound);
    }

    let name = non_blank(request.name, "artifact name")?;
    let retention_days = request.retention_days.unwrap_or(90).clamp(1, 90);
    if request.size_bytes < 0 {
        return Err(AutomationError::InvalidActionsStorage(
            "artifact size must be non-negative".to_owned(),
        ));
    }
    let storage_key = request.storage_key.unwrap_or_else(|| {
        format!(
            "actions/artifacts/{}/{}/{}.zip",
            repository_id,
            run_id,
            safe_filename(&name)
        )
    });
    let content_type = request
        .content_type
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "application/zip".to_owned());
    let expired_at = Utc::now() + chrono::Duration::days(i64::from(retention_days));

    let row = sqlx::query(
        r#"
        INSERT INTO workflow_artifacts (
            run_id, name, digest, size_bytes, storage_key, content_type, retention_days, expired_at, deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL)
        ON CONFLICT (run_id, lower(name))
        DO UPDATE SET digest = EXCLUDED.digest,
                      size_bytes = EXCLUDED.size_bytes,
                      storage_key = EXCLUDED.storage_key,
                      content_type = EXCLUDED.content_type,
                      retention_days = EXCLUDED.retention_days,
                      expired_at = EXCLUDED.expired_at,
                      deleted_at = NULL
        RETURNING id, name, digest, size_bytes, retention_days, expired_at, created_at, updated_at
        "#,
    )
    .bind(run_id)
    .bind(&name)
    .bind(request.digest)
    .bind(request.size_bytes)
    .bind(storage_key)
    .bind(content_type)
    .bind(retention_days)
    .bind(expired_at)
    .fetch_one(pool)
    .await?;

    Ok(ActionsRunArtifact {
        id: row.get("id"),
        name: row.get("name"),
        digest: row.get("digest"),
        size_bytes: row.get("size_bytes"),
        retention_days: row.get("retention_days"),
        expired_at: row.get("expired_at"),
        download_available: true,
        delete_available: true,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn delete_workflow_artifact_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    artifact_id: Uuid,
) -> Result<ActionsRunArtifact, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Write).await?;
    let row = sqlx::query(
        r#"
        UPDATE workflow_artifacts
        SET deleted_at = now()
        FROM workflow_runs
        WHERE workflow_artifacts.run_id = workflow_runs.id
          AND workflow_runs.repository_id = $1
          AND workflow_artifacts.id = $2
          AND workflow_artifacts.deleted_at IS NULL
        RETURNING workflow_artifacts.id, workflow_artifacts.name, workflow_artifacts.digest,
                  workflow_artifacts.size_bytes, workflow_artifacts.retention_days,
                  workflow_artifacts.expired_at, workflow_artifacts.created_at,
                  workflow_artifacts.updated_at
        "#,
    )
    .bind(repository_id)
    .bind(artifact_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowArtifactNotFound)?;

    Ok(ActionsRunArtifact {
        id: row.get("id"),
        name: row.get("name"),
        digest: row.get("digest"),
        size_bytes: row.get("size_bytes"),
        retention_days: row.get("retention_days"),
        expired_at: row.get("expired_at"),
        download_available: false,
        delete_available: false,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn actions_dependency_caches_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    page: i64,
    page_size: i64,
) -> Result<ActionsDependencyCaches, AutomationError> {
    let repository = require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM workflow_dependency_caches WHERE repository_id = $1 AND deleted_at IS NULL",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let total_size_bytes = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(sum(size_bytes), 0)::bigint FROM workflow_dependency_caches WHERE repository_id = $1 AND deleted_at IS NULL",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, cache_key, version, scope, storage_key, size_bytes, last_used_at, created_at, updated_at
        FROM workflow_dependency_caches
        WHERE repository_id = $1 AND deleted_at IS NULL
        ORDER BY last_used_at DESC, cache_key ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(repository_id)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(pool)
    .await?;
    let can_delete = match actor_user_id {
        Some(user_id) if repository.owner_user_id == Some(user_id) => true,
        Some(user_id) => repository_permission_for_user(pool, repository_id, user_id)
            .await
            .map_err(map_repository_error)?
            .map(|permission| permission.role.can_write())
            .unwrap_or(false),
        None => false,
    };

    Ok(ActionsDependencyCaches {
        repository: ActionsDashboardRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.clone(),
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission: viewer_permission(pool, &repository, actor_user_id).await?,
        caches: ListEnvelope {
            items: rows
                .into_iter()
                .map(|row| ActionsDependencyCache {
                    id: row.get("id"),
                    repository_id: row.get("repository_id"),
                    key: row.get("cache_key"),
                    version: row.get("version"),
                    scope: row.get("scope"),
                    storage_key: row.get("storage_key"),
                    size_bytes: row.get("size_bytes"),
                    last_used_at: row.get("last_used_at"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
                .collect(),
            total,
            page,
            page_size,
        },
        total_size_bytes,
        limit_bytes: 10 * 1024 * 1024 * 1024,
        can_delete,
    })
}

pub async fn reserve_actions_cache_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    request: ActionsCacheReserve,
) -> Result<ActionsDependencyCache, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Write).await?;
    let key = non_blank(request.key, "cache key")?;
    let version = non_blank(request.version, "cache version")?;
    if request.size_bytes < 0 {
        return Err(AutomationError::InvalidActionsStorage(
            "cache size must be non-negative".to_owned(),
        ));
    }
    let scope = request
        .scope
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "refs/heads/main".to_owned());
    let storage_key = request.storage_key.unwrap_or_else(|| {
        format!(
            "actions/caches/{}/{}/{}",
            repository_id,
            safe_filename(&key),
            safe_filename(&version)
        )
    });
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_dependency_caches (
            repository_id, cache_key, version, scope, storage_key, size_bytes, last_used_at, deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, now(), NULL)
        ON CONFLICT (repository_id, cache_key, version)
        WHERE deleted_at IS NULL
        DO UPDATE SET scope = EXCLUDED.scope,
                      storage_key = EXCLUDED.storage_key,
                      size_bytes = EXCLUDED.size_bytes,
                      last_used_at = now()
        RETURNING id, repository_id, cache_key, version, scope, storage_key, size_bytes, last_used_at, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(&key)
    .bind(&version)
    .bind(&scope)
    .bind(storage_key)
    .bind(request.size_bytes)
    .fetch_one(pool)
    .await?;

    evict_actions_caches_to_limit(pool, repository_id, 10 * 1024 * 1024 * 1024).await?;

    Ok(ActionsDependencyCache {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        key: row.get("cache_key"),
        version: row.get("version"),
        scope: row.get("scope"),
        storage_key: row.get("storage_key"),
        size_bytes: row.get("size_bytes"),
        last_used_at: row.get("last_used_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn delete_actions_cache_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    cache_id: Uuid,
) -> Result<ActionsDependencyCache, AutomationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Write).await?;
    let row = sqlx::query(
        r#"
        UPDATE workflow_dependency_caches
        SET deleted_at = now()
        WHERE repository_id = $1 AND id = $2 AND deleted_at IS NULL
        RETURNING id, repository_id, cache_key, version, scope, storage_key, size_bytes, last_used_at, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(cache_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowCacheNotFound)?;
    Ok(ActionsDependencyCache {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        key: row.get("cache_key"),
        version: row.get("version"),
        scope: row.get("scope"),
        storage_key: row.get("storage_key"),
        size_bytes: row.get("size_bytes"),
        last_used_at: row.get("last_used_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn evict_actions_caches_to_limit(
    pool: &PgPool,
    repository_id: Uuid,
    limit_bytes: i64,
) -> Result<(), AutomationError> {
    sqlx::query(
        r#"
        WITH ranked AS (
            SELECT id,
                   sum(size_bytes) OVER (ORDER BY last_used_at DESC, created_at DESC) AS running_size
            FROM workflow_dependency_caches
            WHERE repository_id = $1 AND deleted_at IS NULL
        )
        UPDATE workflow_dependency_caches
        SET deleted_at = now()
        WHERE id IN (SELECT id FROM ranked WHERE running_size > $2)
        "#,
    )
    .bind(repository_id)
    .bind(limit_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

fn non_blank(value: String, label: &str) -> Result<String, AutomationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AutomationError::InvalidActionsStorage(format!(
            "{label} cannot be blank"
        )));
    }
    Ok(trimmed.to_owned())
}

async fn workflow_job_for_repository(
    pool: &PgPool,
    repository_id: Uuid,
    job_id: Uuid,
) -> Result<ActionsJobLogJob, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT workflow_jobs.id, workflow_jobs.run_id, workflow_jobs.name, workflow_jobs.status,
               workflow_jobs.conclusion, workflow_jobs.log_deleted_at
        FROM workflow_jobs
        JOIN workflow_runs ON workflow_runs.id = workflow_jobs.run_id
        WHERE workflow_jobs.id = $1 AND workflow_runs.repository_id = $2
        "#,
    )
    .bind(job_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AutomationError::WorkflowJobNotFound)?;

    Ok(ActionsJobLogJob {
        id: row.get("id"),
        run_id: row.get("run_id"),
        name: row.get("name"),
        status: row.get("status"),
        conclusion: row.get("conclusion"),
        log_deleted_at: row.get("log_deleted_at"),
    })
}

async fn actions_log_options(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    show_timestamps_override: Option<bool>,
    raw_logs_override: Option<bool>,
) -> Result<ActionsJobLogOptions, AutomationError> {
    let stored = if let Some(user_id) = actor_user_id {
        sqlx::query(
            r#"
            SELECT show_timestamps, raw_logs, wrap_lines
            FROM actions_log_preferences
            WHERE repository_id = $1 AND user_id = $2
            "#,
        )
        .bind(repository_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .map(|row| ActionsJobLogOptions {
            show_timestamps: row.get("show_timestamps"),
            raw_logs: row.get("raw_logs"),
            wrap_lines: row.get("wrap_lines"),
        })
    } else {
        None
    };
    let mut options = stored.unwrap_or(ActionsJobLogOptions {
        show_timestamps: true,
        raw_logs: false,
        wrap_lines: false,
    });
    if let Some(show_timestamps) = show_timestamps_override {
        options.show_timestamps = show_timestamps;
    }
    if let Some(raw_logs) = raw_logs_override {
        options.raw_logs = raw_logs;
    }
    Ok(options)
}

fn step_number_for_log_step(steps: &[ActionsRunStepDetail], step_id: Option<Uuid>) -> i32 {
    let Some(step_id) = step_id else {
        return 0;
    };
    steps
        .iter()
        .find(|step| step.id == step_id)
        .map(|step| step.number)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
struct RerunSourceJob {
    id: Uuid,
    name: String,
    runner_label: Option<String>,
    group_name: Option<String>,
}

async fn latest_attempt_number(pool: &PgPool, run_id: Uuid) -> Result<i32, AutomationError> {
    let attempt = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max(attempt_number) FROM workflow_run_attempts WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;
    if let Some(attempt) = attempt {
        return Ok(attempt.max(1));
    }
    let job_attempt = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max(attempt_number) FROM workflow_jobs WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;
    Ok(job_attempt.unwrap_or(1).max(1))
}

async fn rerun_source_jobs(
    pool: &PgPool,
    run_id: Uuid,
    latest_attempt: i32,
    mode: &WorkflowRunRerunMode,
    job_id: Option<Uuid>,
) -> Result<Vec<RerunSourceJob>, AutomationError> {
    let rows = match mode {
        WorkflowRunRerunMode::All => {
            sqlx::query(
                r#"
                SELECT id, name, runner_label, group_name
                FROM workflow_jobs
                WHERE run_id = $1 AND attempt_number = $2
                ORDER BY created_at, name
                "#,
            )
            .bind(run_id)
            .bind(latest_attempt)
            .fetch_all(pool)
            .await?
        }
        WorkflowRunRerunMode::Failed => {
            sqlx::query(
                r#"
                SELECT id, name, runner_label, group_name
                FROM workflow_jobs
                WHERE run_id = $1
                  AND attempt_number = $2
                  AND conclusion IN ('failure', 'timed_out')
                ORDER BY created_at, name
                "#,
            )
            .bind(run_id)
            .bind(latest_attempt)
            .fetch_all(pool)
            .await?
        }
        WorkflowRunRerunMode::Job => {
            let Some(job_id) = job_id else {
                return Err(AutomationError::WorkflowRunActionUnavailable(
                    "jobId is required for job-specific re-runs".to_owned(),
                ));
            };
            sqlx::query(
                r#"
                SELECT id, name, runner_label, group_name
                FROM workflow_jobs
                WHERE run_id = $1 AND id = $2
                "#,
            )
            .bind(run_id)
            .bind(job_id)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| RerunSourceJob {
            id: row.get("id"),
            name: row.get("name"),
            runner_label: row.get("runner_label"),
            group_name: row.get("group_name"),
        })
        .collect())
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

fn format_log_line(timestamp: Option<DateTime<Utc>>, content: String) -> String {
    match timestamp {
        Some(timestamp) => format!("{} {content}", timestamp.to_rfc3339()),
        None => content,
    }
}

fn mask_actions_annotations(annotations: &mut [ActionsRunAnnotation], redaction_values: &[String]) {
    for annotation in annotations {
        annotation.message = mask_actions_secret_values(&annotation.message, redaction_values);
        annotation.raw_details = annotation
            .raw_details
            .as_deref()
            .map(|details| mask_actions_secret_values(details, redaction_values));
    }
}

fn safe_filename(value: &str) -> String {
    let mut filename = String::new();
    let mut previous_was_separator = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '.' | '_') {
            filename.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator {
            filename.push('-');
            previous_was_separator = true;
        }
    }

    let filename = filename.trim_matches('-').to_owned();
    if filename.is_empty() {
        "download".to_owned()
    } else {
        filename
    }
}

fn actions_run_action_state(
    run: &ActionsRunListItem,
    job_summary: &ActionsJobSummary,
    viewer_permission: Option<&str>,
    repository_archived: bool,
) -> ActionsRunActionState {
    let can_write = matches!(viewer_permission, Some("owner" | "admin" | "write"));
    let disabled_reason = if !can_write {
        Some("write permission is required for workflow run actions".to_owned())
    } else if repository_archived {
        Some("archived repositories cannot mutate workflow runs".to_owned())
    } else {
        None
    };
    let can_mutate = disabled_reason.is_none();
    let is_live = matches!(run.status.as_str(), "queued" | "in_progress");
    let is_terminal = matches!(run.status.as_str(), "completed" | "cancelled");

    ActionsRunActionState {
        can_rerun: can_mutate && is_terminal,
        can_rerun_failed: can_mutate && is_terminal && job_summary.failure > 0,
        can_cancel: can_mutate && is_live,
        can_delete_logs: can_mutate && is_terminal,
        disabled_reason,
    }
}

fn duration_seconds(
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
) -> Option<i64> {
    match (started_at, completed_at) {
        (Some(started), Some(completed)) => Some((completed - started).num_seconds().max(0)),
        _ => None,
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
               source_branch, yaml_parse_error, dispatch_inputs, dispatch_enabled, updated_at
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
    let yaml_parse_error: Option<String> = row
        .get::<Option<String>, _>("yaml_parse_error")
        .map(sanitize_yaml_parse_error);
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
        yaml_parsed_at: row.get("updated_at"),
    })
}

fn sanitize_yaml_parse_error(error: String) -> String {
    let first_line = error
        .lines()
        .find(|line| {
            let trimmed = line.trim().to_ascii_lowercase();
            !trimmed.is_empty()
                && !trimmed.contains("stack backtrace")
                && !trimmed.starts_with("at ")
                && !trimmed.contains("panicked at")
        })
        .unwrap_or("Workflow YAML could not be parsed.")
        .trim();

    first_line.chars().take(240).collect()
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

fn validate_dispatch_inputs(
    specs: &[WorkflowDispatchInput],
    raw_inputs: HashMap<String, Value>,
) -> Result<Value, AutomationError> {
    if raw_inputs.len() > 25 {
        return Err(AutomationError::InvalidWorkflowDispatch(
            "workflow_dispatch supports at most 25 inputs".to_owned(),
        ));
    }

    let mut normalized = serde_json::Map::new();
    for spec in specs {
        let value = raw_inputs
            .get(&spec.name)
            .cloned()
            .or_else(|| spec.default.clone().map(Value::String));
        let Some(value) = value else {
            if spec.required {
                return Err(AutomationError::InvalidWorkflowDispatch(format!(
                    "input `{}` is required",
                    spec.name
                )));
            }
            continue;
        };

        let normalized_value = match spec.input_type.as_str() {
            "boolean" => match value {
                Value::Bool(value) => Value::Bool(value),
                Value::String(value) if value.eq_ignore_ascii_case("true") => Value::Bool(true),
                Value::String(value) if value.eq_ignore_ascii_case("false") => Value::Bool(false),
                _ => {
                    return Err(AutomationError::InvalidWorkflowDispatch(format!(
                        "input `{}` must be a boolean",
                        spec.name
                    )));
                }
            },
            "choice" => {
                let value = string_dispatch_input(&spec.name, value)?;
                if !spec.options.iter().any(|option| option == &value) {
                    return Err(AutomationError::InvalidWorkflowDispatch(format!(
                        "input `{}` must be one of: {}",
                        spec.name,
                        spec.options.join(", ")
                    )));
                }
                Value::String(value)
            }
            "number" => {
                let value = string_dispatch_input(&spec.name, value)?;
                if value.parse::<f64>().is_err() {
                    return Err(AutomationError::InvalidWorkflowDispatch(format!(
                        "input `{}` must be numeric",
                        spec.name
                    )));
                }
                Value::String(value)
            }
            _ => Value::String(string_dispatch_input(&spec.name, value)?),
        };

        if normalized_value.to_string().len() > 2048 {
            return Err(AutomationError::InvalidWorkflowDispatch(format!(
                "input `{}` is too large",
                spec.name
            )));
        }
        normalized.insert(spec.name.clone(), normalized_value);
    }

    for key in raw_inputs.keys() {
        if !specs.iter().any(|spec| spec.name == *key) {
            return Err(AutomationError::InvalidWorkflowDispatch(format!(
                "input `{key}` is not defined for this workflow"
            )));
        }
    }

    Ok(Value::Object(normalized))
}

fn string_dispatch_input(name: &str, value: Value) -> Result<String, AutomationError> {
    match value {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        _ => Err(AutomationError::InvalidWorkflowDispatch(format!(
            "input `{name}` must be a scalar value"
        ))),
    }
}

#[derive(Debug, Clone)]
struct WorkflowSourceFile {
    path: String,
    content: String,
    oid: String,
    blob_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
struct ParsedWorkflow {
    discovered: DiscoveredWorkflow,
    push: Option<PushWorkflowConfig>,
    pull_request: Option<PullRequestWorkflowConfig>,
    schedule: Option<ScheduleWorkflowConfig>,
    jobs: Vec<WorkflowJobPlan>,
    concurrency_group: Option<String>,
}

#[derive(Debug, Clone)]
struct DiscoveredWorkflow {
    name: String,
    trigger_events: Vec<String>,
    dispatch_enabled: bool,
    dispatch_inputs: Vec<WorkflowDispatchInput>,
    yaml_parse_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct PushWorkflowConfig {
    branches: Vec<String>,
    branches_ignore: Vec<String>,
    tags: Vec<String>,
    tags_ignore: Vec<String>,
    paths: Vec<String>,
    paths_ignore: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PullRequestWorkflowConfig {
    branches: Vec<String>,
    branches_ignore: Vec<String>,
    paths: Vec<String>,
    paths_ignore: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct ScheduleWorkflowConfig {
    schedules: Vec<String>,
}

#[derive(Debug, Clone)]
struct PushedRef {
    name: String,
    short_name: String,
    kind: String,
}

#[derive(Debug, Clone)]
struct WorkflowJobPlan {
    name: String,
    runner_label: Option<String>,
    steps: Vec<String>,
    matrix: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct PullRequestTriggerContext {
    id: Uuid,
    number: i64,
    title: String,
    base_ref: String,
    head_ref: String,
    head_sha: Option<String>,
    is_fork: bool,
}

struct EventWorkflowRunInput<'a> {
    repository: &'a Repository,
    workflow: &'a ActionsWorkflow,
    parsed: &'a ParsedWorkflow,
    actor_user_id: Option<Uuid>,
    head_branch: &'a str,
    head_sha: Option<&'a str>,
    event: &'a str,
    display_title: String,
    event_payload: Value,
    queue: &'a str,
    lease_prefix: &'a str,
    pull_request_id: Option<Uuid>,
    fork_pull_request: bool,
}

impl PushWorkflowConfig {
    fn matches_ref(&self, pushed_ref: &PushedRef) -> bool {
        match pushed_ref.kind.as_str() {
            "branch" => {
                patterns_allow(&self.branches, &pushed_ref.short_name)
                    && !patterns_match_any(&self.branches_ignore, &pushed_ref.short_name)
            }
            "tag" => {
                patterns_allow(&self.tags, &pushed_ref.short_name)
                    && !patterns_match_any(&self.tags_ignore, &pushed_ref.short_name)
            }
            _ => false,
        }
    }

    fn matches_paths(&self, changed_paths: &[String]) -> bool {
        if changed_paths.is_empty() {
            return self.paths.is_empty();
        }
        let included = if self.paths.is_empty() {
            true
        } else {
            changed_paths
                .iter()
                .any(|path| patterns_match_any(&self.paths, path))
        };
        if !included {
            return false;
        }
        if self.paths_ignore.is_empty() {
            return true;
        }
        changed_paths
            .iter()
            .any(|path| !patterns_match_any(&self.paths_ignore, path))
    }
}

impl PullRequestWorkflowConfig {
    fn matches_branch(&self, base_ref: &str) -> bool {
        patterns_allow(&self.branches, base_ref)
            && !patterns_match_any(&self.branches_ignore, base_ref)
    }

    fn matches_paths(&self, changed_paths: &[String]) -> bool {
        if changed_paths.is_empty() {
            return self.paths.is_empty();
        }
        let included = if self.paths.is_empty() {
            true
        } else {
            changed_paths
                .iter()
                .any(|path| patterns_match_any(&self.paths, path))
        };
        if !included {
            return false;
        }
        if self.paths_ignore.is_empty() {
            return true;
        }
        changed_paths
            .iter()
            .any(|path| !patterns_match_any(&self.paths_ignore, path))
    }
}

impl ScheduleWorkflowConfig {
    fn matches_schedule(&self, schedule: &str) -> bool {
        self.schedules.is_empty() || self.schedules.iter().any(|item| item == schedule)
    }
}

async fn workflow_files_for_ref(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
) -> Result<Vec<WorkflowSourceFile>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT repository_files.path,
               repository_files.content,
               repository_files.oid,
               git_objects.id AS blob_id
        FROM repository_git_refs
        JOIN repository_files
          ON repository_files.commit_id = repository_git_refs.target_commit_id
         AND repository_files.repository_id = repository_git_refs.repository_id
        LEFT JOIN git_objects
          ON git_objects.repository_id = repository_files.repository_id
         AND git_objects.oid = repository_files.oid
         AND git_objects.object_type = 'blob'
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name = $2
          AND lower(repository_files.path) LIKE '.github/workflows/%'
          AND (
              lower(repository_files.path) LIKE '%.yml'
              OR lower(repository_files.path) LIKE '%.yaml'
          )
        ORDER BY lower(repository_files.path)
        "#,
    )
    .bind(repository_id)
    .bind(ref_name)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| WorkflowSourceFile {
            path: row.get("path"),
            content: row.get("content"),
            oid: row.get("oid"),
            blob_id: row.get("blob_id"),
        })
        .collect())
}

async fn changed_paths_for_commit(
    pool: &PgPool,
    repository_id: Uuid,
    head_sha: &str,
) -> Result<Vec<String>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT repository_files.path
        FROM commits
        JOIN repository_files ON repository_files.commit_id = commits.id
        WHERE commits.repository_id = $1 AND commits.oid = $2
        ORDER BY lower(repository_files.path)
        "#,
    )
    .bind(repository_id)
    .bind(head_sha)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| row.get("path")).collect())
}

async fn changed_paths_for_pull_request(
    pool: &PgPool,
    repository_id: Uuid,
    pull_request_id: Uuid,
) -> Result<Vec<String>, AutomationError> {
    let rows = sqlx::query(
        r#"
        SELECT pull_request_files.path
        FROM pull_request_files
        JOIN pull_requests ON pull_requests.id = pull_request_files.pull_request_id
        WHERE pull_requests.repository_id = $1 AND pull_requests.id = $2
        ORDER BY lower(pull_request_files.path)
        "#,
    )
    .bind(repository_id)
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| row.get("path")).collect())
}

async fn pull_request_trigger_context(
    pool: &PgPool,
    repository_id: Uuid,
    pull_request_id: Uuid,
) -> Result<Option<PullRequestTriggerContext>, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT pull_requests.id, pull_requests.number, pull_requests.title,
               pull_requests.base_ref, pull_requests.head_ref,
               pull_requests.head_repository_id,
               pull_requests.base_repository_id,
               commits.oid AS head_sha
        FROM pull_requests
        LEFT JOIN pull_request_commits
          ON pull_request_commits.pull_request_id = pull_requests.id
        LEFT JOIN commits
          ON commits.id = pull_request_commits.commit_id
        WHERE pull_requests.repository_id = $1 AND pull_requests.id = $2
        ORDER BY pull_request_commits.position DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(pull_request_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let head_repository_id = row.get::<Option<Uuid>, _>("head_repository_id");
        let base_repository_id = row.get::<Option<Uuid>, _>("base_repository_id");
        PullRequestTriggerContext {
            id: row.get("id"),
            number: row.get("number"),
            title: row.get("title"),
            base_ref: row.get("base_ref"),
            head_ref: row.get("head_ref"),
            head_sha: row.get("head_sha"),
            is_fork: head_repository_id.is_some()
                && base_repository_id.is_some()
                && head_repository_id != base_repository_id,
        }
    }))
}

async fn head_sha_for_ref(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
) -> Result<Option<String>, AutomationError> {
    let row = sqlx::query(
        r#"
        SELECT commits.oid
        FROM repository_git_refs
        JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name = $2
        "#,
    )
    .bind(repository_id)
    .bind(ref_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| row.get("oid")))
}

async fn upsert_discovered_workflow(
    pool: &PgPool,
    repository: &Repository,
    file: &WorkflowSourceFile,
    source_branch: &str,
    workflow: DiscoveredWorkflow,
) -> Result<ActionsWorkflow, AutomationError> {
    let row = sqlx::query(
        r#"
        INSERT INTO actions_workflows (
            repository_id, name, path, trigger_events, source_blob_id, source_sha,
            source_branch, yaml_parse_error, dispatch_enabled, dispatch_inputs
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET name = EXCLUDED.name,
                      trigger_events = EXCLUDED.trigger_events,
                      source_blob_id = EXCLUDED.source_blob_id,
                      source_sha = EXCLUDED.source_sha,
                      source_branch = EXCLUDED.source_branch,
                      yaml_parse_error = EXCLUDED.yaml_parse_error,
                      dispatch_enabled = EXCLUDED.dispatch_enabled,
                      dispatch_inputs = EXCLUDED.dispatch_inputs
        RETURNING id, repository_id, name, path, state, trigger_events, created_at, updated_at
        "#,
    )
    .bind(repository.id)
    .bind(workflow.name)
    .bind(&file.path)
    .bind(workflow.trigger_events)
    .bind(file.blob_id)
    .bind(&file.oid)
    .bind(source_branch)
    .bind(workflow.yaml_parse_error)
    .bind(workflow.dispatch_enabled)
    .bind(serde_json::to_value(workflow.dispatch_inputs).unwrap_or_else(|_| json!([])))
    .fetch_one(pool)
    .await?;
    workflow_from_row(row)
}

async fn create_push_workflow_run(
    pool: &PgPool,
    repository: &Repository,
    workflow: &ActionsWorkflow,
    parsed: &ParsedWorkflow,
    input: &TriggerWorkflowsForPush,
    pushed_ref: &PushedRef,
    changed_paths: &[String],
) -> Result<WorkflowRun, AutomationError> {
    let run_number = next_run_number(pool, workflow.id).await?;
    let display_title = format!(
        "{} pushed to {}",
        repository.owner_login, pushed_ref.short_name
    );
    let runtime_context = resolve_actions_runtime_context(
        pool,
        ActionsRuntimeResolutionRequest {
            repository_id: repository.id,
            event: "push".to_owned(),
            fork_pull_request: false,
            environment: None,
            environment_approved: false,
            explicit_secret_names: None,
        },
    )
    .await?;
    let event_payload = json!({
        "ref": pushed_ref.name,
        "headBranch": if pushed_ref.kind == "branch" { Some(pushed_ref.short_name.clone()) } else { None },
        "headTag": if pushed_ref.kind == "tag" { Some(pushed_ref.short_name.clone()) } else { None },
        "headSha": input.head_sha,
        "workflowPath": workflow.path,
        "changedPaths": changed_paths,
        "source": "git_receive_pack",
        "runtimePolicy": runtime_context.diagnostics,
    });
    let workflow_matrix = json!({
        "jobCount": parsed.jobs.len(),
        "jobs": parsed.jobs.iter().map(|job| {
            json!({
                "name": job.name,
                "matrix": job.matrix,
            })
        }).collect::<Vec<_>>(),
    });

    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, head_branch,
            head_sha, event, display_title, event_payload, concurrency_group, workflow_matrix
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'push', $7, $8, $9, $10)
        RETURNING id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
                  head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(repository.id)
    .bind(workflow.id)
    .bind(input.actor_user_id)
    .bind(run_number)
    .bind(&pushed_ref.short_name)
    .bind(&input.head_sha)
    .bind(&display_title)
    .bind(&event_payload)
    .bind(&parsed.concurrency_group)
    .bind(&workflow_matrix)
    .fetch_one(&mut *tx)
    .await?;
    let run = workflow_run_from_row(row)?;

    let jobs = if parsed.jobs.is_empty() {
        vec![WorkflowJobPlan {
            name: workflow.name.clone(),
            runner_label: Some("ubuntu-latest".to_owned()),
            steps: vec!["Run workflow".to_owned()],
            matrix: BTreeMap::new(),
        }]
    } else {
        parsed.jobs.clone()
    };

    for job in jobs {
        let job_row = sqlx::query(
            r#"
            INSERT INTO workflow_jobs (run_id, name, runner_label, group_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(run.id)
        .bind(&job.name)
        .bind(&job.runner_label)
        .bind(&parsed.concurrency_group)
        .fetch_one(&mut *tx)
        .await?;
        let job_id: Uuid = job_row.get("id");
        let steps = if job.steps.is_empty() {
            vec!["Run job".to_owned()]
        } else {
            job.steps
        };
        for (index, step) in steps.into_iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO workflow_steps (job_id, number, name)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(job_id)
            .bind((index + 1) as i32)
            .bind(step)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    enqueue_job(
        pool,
        "actions.workflow_push",
        &format!("workflow-push:{}:{}", workflow.id, run.id),
        json!({
            "repositoryId": repository.id,
            "workflowId": workflow.id,
            "workflowPath": workflow.path,
            "runId": run.id,
            "runNumber": run.run_number,
            "actorUserId": input.actor_user_id,
            "ref": pushed_ref.name,
            "headBranch": pushed_ref.short_name,
            "headSha": input.head_sha,
            "concurrencyGroup": parsed.concurrency_group,
            "eventPayload": event_payload,
            "matrix": workflow_matrix,
        }),
    )
    .await?;

    Ok(run)
}

async fn create_event_workflow_run(
    pool: &PgPool,
    input: EventWorkflowRunInput<'_>,
) -> Result<WorkflowRun, AutomationError> {
    let run_number = next_run_number(pool, input.workflow.id).await?;
    let runtime_context = resolve_actions_runtime_context(
        pool,
        ActionsRuntimeResolutionRequest {
            repository_id: input.repository.id,
            event: input.event.to_owned(),
            fork_pull_request: input.fork_pull_request,
            environment: None,
            environment_approved: false,
            explicit_secret_names: None,
        },
    )
    .await?;
    let mut event_payload = input.event_payload;
    if let Value::Object(ref mut payload) = event_payload {
        payload.insert(
            "runtimePolicy".to_owned(),
            serde_json::to_value(&runtime_context.diagnostics).unwrap_or_else(|_| json!({})),
        );
    }
    let workflow_matrix = workflow_matrix_payload(input.parsed);

    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO workflow_runs (
            repository_id, workflow_id, actor_user_id, run_number, head_branch,
            head_sha, event, display_title, event_payload, concurrency_group,
            workflow_matrix, pull_request_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, repository_id, workflow_id, actor_user_id, run_number, status, conclusion,
                  head_branch, head_sha, event, started_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(input.repository.id)
    .bind(input.workflow.id)
    .bind(input.actor_user_id)
    .bind(run_number)
    .bind(input.head_branch)
    .bind(input.head_sha)
    .bind(input.event)
    .bind(&input.display_title)
    .bind(&event_payload)
    .bind(&input.parsed.concurrency_group)
    .bind(&workflow_matrix)
    .bind(input.pull_request_id)
    .fetch_one(&mut *tx)
    .await?;
    let run = workflow_run_from_row(row)?;

    let jobs = if input.parsed.jobs.is_empty() {
        vec![WorkflowJobPlan {
            name: input.workflow.name.clone(),
            runner_label: Some("ubuntu-latest".to_owned()),
            steps: vec!["Run workflow".to_owned()],
            matrix: BTreeMap::new(),
        }]
    } else {
        input.parsed.jobs.clone()
    };
    for job in jobs {
        let job_row = sqlx::query(
            r#"
            INSERT INTO workflow_jobs (run_id, name, runner_label, group_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(run.id)
        .bind(&job.name)
        .bind(&job.runner_label)
        .bind(&input.parsed.concurrency_group)
        .fetch_one(&mut *tx)
        .await?;
        let job_id: Uuid = job_row.get("id");
        let steps = if job.steps.is_empty() {
            vec!["Run job".to_owned()]
        } else {
            job.steps
        };
        for (index, step) in steps.into_iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO workflow_steps (job_id, number, name)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(job_id)
            .bind((index + 1) as i32)
            .bind(step)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    enqueue_job(
        pool,
        input.queue,
        &format!("{}:{}:{}", input.lease_prefix, input.workflow.id, run.id),
        json!({
            "repositoryId": input.repository.id,
            "workflowId": input.workflow.id,
            "workflowPath": input.workflow.path,
            "runId": run.id,
            "runNumber": run.run_number,
            "actorUserId": input.actor_user_id,
            "event": input.event,
            "headBranch": input.head_branch,
            "headSha": input.head_sha,
            "concurrencyGroup": input.parsed.concurrency_group,
            "eventPayload": event_payload,
            "matrix": workflow_matrix,
        }),
    )
    .await?;

    Ok(run)
}

fn workflow_matrix_payload(parsed: &ParsedWorkflow) -> Value {
    json!({
        "jobCount": parsed.jobs.len(),
        "jobs": parsed.jobs.iter().map(|job| {
            json!({
                "name": job.name,
                "matrix": job.matrix,
            })
        }).collect::<Vec<_>>(),
    })
}

fn parse_workflow_file(source: &str) -> Result<ParsedWorkflow, serde_yaml::Error> {
    let document: serde_yaml::Value = serde_yaml::from_str(source)?;
    let name = yaml_get(&document, "name")
        .and_then(yaml_scalar_string)
        .unwrap_or_else(|| "Workflow".to_owned());
    let on = yaml_get(&document, "on");
    let trigger_events = workflow_trigger_events(on);
    let push = push_workflow_config(on);
    let pull_request = pull_request_workflow_config(on);
    let schedule = schedule_workflow_config(on);
    let dispatch_inputs = workflow_dispatch_inputs(on);
    let jobs = workflow_job_plans(yaml_get(&document, "jobs"));
    let concurrency_group = yaml_get(&document, "concurrency").and_then(concurrency_group);

    Ok(ParsedWorkflow {
        discovered: DiscoveredWorkflow {
            name,
            trigger_events,
            dispatch_enabled: !dispatch_inputs.is_empty()
                || workflow_trigger_events(on)
                    .iter()
                    .any(|event| event == "workflow_dispatch"),
            dispatch_inputs,
            yaml_parse_error: None,
        },
        push,
        pull_request,
        schedule,
        jobs,
        concurrency_group,
    })
}

fn workflow_trigger_events(on: Option<&serde_yaml::Value>) -> Vec<String> {
    let Some(on) = on else {
        return Vec::new();
    };
    match on {
        serde_yaml::Value::String(event) => vec![event.clone()],
        serde_yaml::Value::Sequence(events) => {
            events.iter().filter_map(yaml_scalar_string).collect()
        }
        serde_yaml::Value::Mapping(mapping) => mapping
            .keys()
            .filter_map(yaml_key_string)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    }
}

fn push_workflow_config(on: Option<&serde_yaml::Value>) -> Option<PushWorkflowConfig> {
    let on = on?;
    match on {
        serde_yaml::Value::String(event) if event == "push" => Some(PushWorkflowConfig::default()),
        serde_yaml::Value::Sequence(events)
            if events
                .iter()
                .filter_map(yaml_scalar_string)
                .any(|event| event == "push") =>
        {
            Some(PushWorkflowConfig::default())
        }
        serde_yaml::Value::Mapping(mapping) => {
            let push = mapping.iter().find_map(|(key, value)| {
                if yaml_key_string(key).as_deref() == Some("push") {
                    Some(value)
                } else {
                    None
                }
            })?;
            let mut config = PushWorkflowConfig::default();
            if let serde_yaml::Value::Mapping(push_mapping) = push {
                config.branches = yaml_string_list(mapping_get(push_mapping, "branches"));
                config.branches_ignore =
                    yaml_string_list(mapping_get(push_mapping, "branches-ignore"));
                config.tags = yaml_string_list(mapping_get(push_mapping, "tags"));
                config.tags_ignore = yaml_string_list(mapping_get(push_mapping, "tags-ignore"));
                config.paths = yaml_string_list(mapping_get(push_mapping, "paths"));
                config.paths_ignore = yaml_string_list(mapping_get(push_mapping, "paths-ignore"));
            }
            Some(config)
        }
        _ => None,
    }
}

fn pull_request_workflow_config(
    on: Option<&serde_yaml::Value>,
) -> Option<PullRequestWorkflowConfig> {
    let on = on?;
    match on {
        serde_yaml::Value::String(event) if event == "pull_request" => {
            Some(PullRequestWorkflowConfig::default())
        }
        serde_yaml::Value::Sequence(events)
            if events
                .iter()
                .filter_map(yaml_scalar_string)
                .any(|event| event == "pull_request") =>
        {
            Some(PullRequestWorkflowConfig::default())
        }
        serde_yaml::Value::Mapping(mapping) => {
            let pull_request = mapping.iter().find_map(|(key, value)| {
                if yaml_key_string(key).as_deref() == Some("pull_request") {
                    Some(value)
                } else {
                    None
                }
            })?;
            let mut config = PullRequestWorkflowConfig::default();
            if let serde_yaml::Value::Mapping(pull_mapping) = pull_request {
                config.branches = yaml_string_list(mapping_get(pull_mapping, "branches"));
                config.branches_ignore =
                    yaml_string_list(mapping_get(pull_mapping, "branches-ignore"));
                config.paths = yaml_string_list(mapping_get(pull_mapping, "paths"));
                config.paths_ignore = yaml_string_list(mapping_get(pull_mapping, "paths-ignore"));
            }
            Some(config)
        }
        _ => None,
    }
}

fn schedule_workflow_config(on: Option<&serde_yaml::Value>) -> Option<ScheduleWorkflowConfig> {
    let on = on?;
    match on {
        serde_yaml::Value::String(event) if event == "schedule" => {
            Some(ScheduleWorkflowConfig::default())
        }
        serde_yaml::Value::Sequence(events)
            if events
                .iter()
                .filter_map(yaml_scalar_string)
                .any(|event| event == "schedule") =>
        {
            Some(ScheduleWorkflowConfig::default())
        }
        serde_yaml::Value::Mapping(mapping) => {
            let schedule = mapping.iter().find_map(|(key, value)| {
                if yaml_key_string(key).as_deref() == Some("schedule") {
                    Some(value)
                } else {
                    None
                }
            })?;
            let schedules = match schedule {
                serde_yaml::Value::Sequence(entries) => entries
                    .iter()
                    .filter_map(|entry| {
                        entry
                            .as_mapping()
                            .and_then(|mapping| mapping_get(mapping, "cron"))
                            .and_then(yaml_scalar_string)
                    })
                    .collect(),
                _ => Vec::new(),
            };
            Some(ScheduleWorkflowConfig { schedules })
        }
        _ => None,
    }
}

fn workflow_dispatch_inputs(on: Option<&serde_yaml::Value>) -> Vec<WorkflowDispatchInput> {
    let Some(serde_yaml::Value::Mapping(on_mapping)) = on else {
        return Vec::new();
    };
    let Some(serde_yaml::Value::Mapping(dispatch_mapping)) =
        mapping_get(on_mapping, "workflow_dispatch")
    else {
        return Vec::new();
    };
    let Some(serde_yaml::Value::Mapping(inputs_mapping)) = mapping_get(dispatch_mapping, "inputs")
    else {
        return Vec::new();
    };

    inputs_mapping
        .iter()
        .filter_map(|(key, value)| {
            let name = yaml_key_string(key)?;
            let input_mapping = match value {
                serde_yaml::Value::Mapping(mapping) => Some(mapping),
                _ => None,
            };
            let description = input_mapping
                .and_then(|mapping| mapping_get(mapping, "description"))
                .and_then(yaml_scalar_string);
            let required = input_mapping
                .and_then(|mapping| mapping_get(mapping, "required"))
                .and_then(yaml_bool)
                .unwrap_or(false);
            let default_value = input_mapping
                .and_then(|mapping| mapping_get(mapping, "default"))
                .and_then(yaml_scalar_string);
            let input_type = input_mapping
                .and_then(|mapping| mapping_get(mapping, "type"))
                .and_then(yaml_scalar_string)
                .unwrap_or_else(|| "string".to_owned());
            let options = input_mapping
                .and_then(|mapping| mapping_get(mapping, "options"))
                .map(|value| yaml_string_list(Some(value)))
                .unwrap_or_default();
            Some(WorkflowDispatchInput {
                name,
                label: description.clone().unwrap_or_else(|| input_type.clone()),
                description,
                required,
                default: default_value,
                input_type,
                options,
            })
        })
        .collect()
}

fn workflow_job_plans(jobs: Option<&serde_yaml::Value>) -> Vec<WorkflowJobPlan> {
    let Some(serde_yaml::Value::Mapping(jobs)) = jobs else {
        return Vec::new();
    };
    let mut plans = Vec::new();
    for (key, value) in jobs {
        let job_key = yaml_key_string(key).unwrap_or_else(|| "job".to_owned());
        let Some(mapping) = value.as_mapping() else {
            plans.push(WorkflowJobPlan {
                name: job_key,
                runner_label: None,
                steps: Vec::new(),
                matrix: BTreeMap::new(),
            });
            continue;
        };
        let base_name = mapping_get(mapping, "name")
            .and_then(yaml_scalar_string)
            .unwrap_or(job_key);
        let runner_label = mapping_get(mapping, "runs-on").and_then(yaml_scalar_or_first_string);
        let steps = workflow_step_names(mapping_get(mapping, "steps"));
        let matrices = matrix_combinations(
            mapping_get(mapping, "strategy")
                .and_then(|strategy| strategy.as_mapping())
                .and_then(|strategy| mapping_get(strategy, "matrix")),
        );
        for matrix in matrices {
            let name = if matrix.is_empty() {
                base_name.clone()
            } else {
                let suffix = matrix
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{base_name} ({suffix})")
            };
            plans.push(WorkflowJobPlan {
                name,
                runner_label: runner_label.clone(),
                steps: steps.clone(),
                matrix,
            });
        }
    }
    plans
}

fn workflow_step_names(steps: Option<&serde_yaml::Value>) -> Vec<String> {
    let Some(serde_yaml::Value::Sequence(steps)) = steps else {
        return Vec::new();
    };
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            step.as_mapping()
                .and_then(|mapping| mapping_get(mapping, "name"))
                .and_then(yaml_scalar_string)
                .or_else(|| {
                    step.as_mapping()
                        .and_then(|mapping| mapping_get(mapping, "uses"))
                        .and_then(yaml_scalar_string)
                })
                .or_else(|| {
                    step.as_mapping()
                        .and_then(|mapping| mapping_get(mapping, "run"))
                        .map(|_| "Run command".to_owned())
                })
                .unwrap_or_else(|| format!("Step {}", index + 1))
        })
        .collect()
}

fn matrix_combinations(matrix: Option<&serde_yaml::Value>) -> Vec<BTreeMap<String, String>> {
    let Some(serde_yaml::Value::Mapping(matrix)) = matrix else {
        return vec![BTreeMap::new()];
    };
    let mut dimensions = Vec::new();
    for (key, value) in matrix {
        let Some(name) = yaml_key_string(key) else {
            continue;
        };
        if name == "include" || name == "exclude" {
            continue;
        }
        let values = yaml_string_list(Some(value));
        if !values.is_empty() {
            dimensions.push((name, values));
        }
    }
    if dimensions.is_empty() {
        return vec![BTreeMap::new()];
    }
    let mut combinations = vec![BTreeMap::new()];
    for (name, values) in dimensions {
        let mut next = Vec::new();
        for combination in &combinations {
            for value in &values {
                let mut clone = combination.clone();
                clone.insert(name.clone(), value.clone());
                next.push(clone);
            }
        }
        combinations = next;
    }
    combinations
}

fn normalize_pushed_ref(ref_name: &str) -> Result<PushedRef, AutomationError> {
    if let Some(short_name) = ref_name.strip_prefix("refs/heads/") {
        Ok(PushedRef {
            name: ref_name.to_owned(),
            short_name: short_name.to_owned(),
            kind: "branch".to_owned(),
        })
    } else if let Some(short_name) = ref_name.strip_prefix("refs/tags/") {
        Ok(PushedRef {
            name: ref_name.to_owned(),
            short_name: short_name.to_owned(),
            kind: "tag".to_owned(),
        })
    } else {
        Err(AutomationError::InvalidWorkflowDispatch(format!(
            "unsupported pushed ref `{ref_name}`"
        )))
    }
}

fn workflow_name_from_path(path: &str) -> String {
    path.rsplit('/')
        .next()
        .unwrap_or("Workflow")
        .trim_end_matches(".yaml")
        .trim_end_matches(".yml")
        .replace(['_', '-'], " ")
}

fn yaml_get<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping_get(mapping, key))
}

fn mapping_get<'a>(mapping: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    mapping.iter().find_map(|(candidate, value)| {
        if yaml_key_string(candidate).as_deref() == Some(key) {
            Some(value)
        } else {
            None
        }
    })
}

fn yaml_key_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(value) => Some(value.clone()),
        serde_yaml::Value::Bool(true) => Some("on".to_owned()),
        _ => None,
    }
}

fn yaml_scalar_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(value) => Some(value.clone()),
        serde_yaml::Value::Number(value) => Some(value.to_string()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn yaml_scalar_or_first_string(value: &serde_yaml::Value) -> Option<String> {
    yaml_scalar_string(value).or_else(|| match value {
        serde_yaml::Value::Sequence(items) => items.first().and_then(yaml_scalar_string),
        _ => None,
    })
}

fn yaml_bool(value: &serde_yaml::Value) -> Option<bool> {
    match value {
        serde_yaml::Value::Bool(value) => Some(*value),
        serde_yaml::Value::String(value) if value.eq_ignore_ascii_case("true") => Some(true),
        serde_yaml::Value::String(value) if value.eq_ignore_ascii_case("false") => Some(false),
        _ => None,
    }
}

fn yaml_string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    match value {
        Some(serde_yaml::Value::Sequence(values)) => {
            values.iter().filter_map(yaml_scalar_string).collect()
        }
        Some(value) => yaml_scalar_string(value).into_iter().collect(),
        None => Vec::new(),
    }
}

fn concurrency_group(value: &serde_yaml::Value) -> Option<String> {
    yaml_scalar_string(value).or_else(|| {
        value
            .as_mapping()
            .and_then(|mapping| mapping_get(mapping, "group"))
            .and_then(yaml_scalar_string)
    })
}

fn patterns_allow(patterns: &[String], value: &str) -> bool {
    patterns.is_empty() || patterns_match_any(patterns, value)
}

fn patterns_match_any(patterns: &[String], value: &str) -> bool {
    patterns.iter().any(|pattern| glob_match(pattern, value))
}

fn glob_match(pattern: &str, value: &str) -> bool {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                regex.push_str(".*");
            }
            '*' => regex.push_str("[^/]*"),
            '?' => regex.push_str("[^/]"),
            _ => regex.push_str(&regex::escape(&ch.to_string())),
        }
    }
    regex.push('$');
    Regex::new(&regex)
        .map(|compiled| compiled.is_match(value))
        .unwrap_or(false)
}

async fn resolve_workflow_dispatch_ref(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
) -> Result<ActionsWorkflowRef, AutomationError> {
    let cleaned = ref_name.trim();
    if cleaned.is_empty() {
        return Err(AutomationError::InvalidWorkflowDispatch(
            "ref is required".to_owned(),
        ));
    }
    let rows = actions_workflow_refs(pool, repository_id).await?;
    rows.into_iter()
        .find(|item| {
            item.name.eq_ignore_ascii_case(cleaned) || item.short_name.eq_ignore_ascii_case(cleaned)
        })
        .ok_or_else(|| AutomationError::InvalidWorkflowDispatch(format!("unknown ref `{cleaned}`")))
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
        .map_err(map_repository_error)?;

    let Some(permission) = permission else {
        if required_role == RepositoryRole::Read {
            let repository = get_repository(pool, repository_id)
                .await
                .map_err(map_repository_error)?
                .ok_or(AutomationError::RepositoryNotFound)?;
            if repository.visibility == RepositoryVisibility::Public {
                return Ok(());
            }
        }
        return Err(AutomationError::RepositoryAccessDenied);
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
