use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::{normalize_pagination, ListEnvelope};

use super::repositories::{
    can_read_repository, get_repository_by_owner_name, repository_permission_for_user,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<ProjectRow>,
    pub scope: ProjectListScopeSummary,
    pub filters: ProjectListFilters,
    pub counts: ProjectCounts,
    pub templates: ListEnvelope<ProjectTemplateRow>,
    pub viewer_permissions: ProjectListPermissions,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListScopeSummary {
    pub kind: String,
    pub login: String,
    pub repository: Option<ProjectRepositoryScopeSummary>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRepositoryScopeSummary {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRow {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub visibility: String,
    pub href: String,
    pub workspace_href: String,
    pub owner: String,
    pub is_template: bool,
    pub default_repository: Option<ProjectRepositoryScopeSummary>,
    pub linked_repositories_count: i64,
    pub status: Option<ProjectStatusSummary>,
    pub counts: ProjectItemCounts,
    pub viewer_role: Option<String>,
    pub viewer_can_copy: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplateRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub project_title: String,
    pub project_href: String,
    pub is_public: bool,
    pub viewer_can_copy: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCounts {
    pub open: i64,
    pub closed: i64,
    pub templates: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemCounts {
    pub total: i64,
    pub open: i64,
    pub closed: i64,
    pub draft: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusSummary {
    pub status: String,
    pub label: String,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListFilters {
    pub query: Option<String>,
    pub state: String,
    pub tab: String,
    pub sort: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_create: bool,
    pub can_copy: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectListQuery<'a> {
    pub query: Option<&'a str>,
    pub state: Option<&'a str>,
    pub tab: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectWorkspaceQuery<'a> {
    pub view: Option<&'a str>,
    pub query: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub group: Option<&'a str>,
    pub slice: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectItemsArchivedQuery<'a> {
    pub item_type: Option<&'a str>,
    pub query: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectViewStateRequest {
    pub query: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub slice: Option<String>,
    #[serde(default)]
    pub hidden_field_ids: Vec<Uuid>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectViewLayoutRequest {
    pub layout: String,
    pub column_field_id: Option<Uuid>,
    pub swimlane_field_id: Option<Uuid>,
    pub start_field_id: Option<Uuid>,
    pub target_field_id: Option<Uuid>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRoadmapSettingsRequest {
    pub start_field_id: Uuid,
    pub target_field_id: Uuid,
    #[serde(default)]
    pub marker_field_ids: Vec<Uuid>,
    pub zoom: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldCreateRequest {
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldUpdateRequest {
    pub name: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldDeleteRequest {
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldOptionCreateRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldOptionUpdateRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldOptionReorderRequest {
    pub option_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIterationSettingsRequest {
    pub start_date: NaiveDate,
    pub duration: i64,
    pub duration_unit: String,
    pub generated_iterations: Option<i64>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIterationCreateRequest {
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub duration_days: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIterationUpdateRequest {
    pub name: String,
    pub start_date: NaiveDate,
    pub duration_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIterationBreakCreateRequest {
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub duration_days: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemFieldValueRequest {
    pub value: Value,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemAddRequest {
    pub item_type: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub url: Option<String>,
    pub issue_id: Option<Uuid>,
    pub pull_request_id: Option<Uuid>,
    pub position_after_item_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemsBulkAddRequest {
    #[serde(default)]
    pub items: Vec<ProjectItemAddRequest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemPositionRequest {
    pub before_item_id: Option<Uuid>,
    pub after_item_id: Option<Uuid>,
    pub group_field_id: Option<Uuid>,
    pub group_value: Option<Value>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldSettings {
    pub project: ProjectWorkspaceProject,
    pub fields: Vec<ProjectFieldSettingsField>,
    pub limits: ProjectFieldSettingsLimits,
    pub viewer_permissions: ProjectFieldSettingsPermissions,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldSettingsField {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub position: i64,
    pub settings: Value,
    pub built_in: bool,
    pub editable: bool,
    pub deletable: bool,
    pub usage_count: i64,
    pub options: Vec<ProjectFieldOption>,
    pub iterations: Vec<ProjectIteration>,
    pub breaks: Vec<ProjectIterationBreak>,
    pub cache_version: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldOption {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub position: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIteration {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub duration_days: i64,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIterationBreak {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub duration_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldSettingsLimits {
    pub max_fields: i64,
    pub used_fields: i64,
    pub remaining_fields: i64,
    pub max_options_per_field: i64,
    pub max_iterations_per_field: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFieldSettingsPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_create_fields: bool,
    pub can_rename_fields: bool,
    pub can_delete_fields: bool,
    pub can_manage_options: bool,
    pub can_manage_iterations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowSettings {
    pub project: ProjectWorkspaceProject,
    pub workflows: Vec<ProjectWorkflowDefinition>,
    pub eligible_fields: Vec<ProjectWorkflowEligibleField>,
    pub repository_targets: Vec<ProjectWorkflowRepositoryTarget>,
    pub recent_logs: Vec<ProjectWorkflowExecutionLog>,
    pub viewer_permissions: ProjectWorkflowSettingsPermissions,
    pub automation_actor: String,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowDefinition {
    pub id: Uuid,
    pub workflow_key: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub trigger_event: String,
    pub configuration: Value,
    pub rules: Vec<ProjectWorkflowRule>,
    pub repository_target_ids: Vec<Uuid>,
    pub actor_label: String,
    pub source: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_run_status: Option<String>,
    pub last_run_message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowRule {
    pub id: Uuid,
    pub rule_type: String,
    pub configuration: Value,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowEligibleField {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub options: Vec<ProjectFieldOption>,
    pub supports_status_target: bool,
    pub supports_archive_criteria: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowRepositoryTarget {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub href: String,
    pub visibility: String,
    pub permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowExecutionLog {
    pub id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub item_id: Option<Uuid>,
    pub actor: Option<ProjectWorkspaceUser>,
    pub source: String,
    pub event_type: String,
    pub status: String,
    pub message: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowSettingsPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_manage_workflows: bool,
    pub can_view_logs: bool,
    pub can_select_repositories: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkflowUpdateRequest {
    pub enabled: Option<bool>,
    pub condition: Option<String>,
    pub status_field_id: Option<Uuid>,
    pub status_option_id: Option<Uuid>,
    pub repository_target_ids: Option<Vec<Uuid>>,
    pub archive_after_days: Option<i64>,
    pub close_on_status: Option<bool>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsUpdateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub readme: Option<String>,
    pub visibility: Option<String>,
    pub default_repository_id: Option<Uuid>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRepositoryLinkRequest {
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusUpdateRequest {
    pub status: String,
    pub body: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplateUpdateRequest {
    pub is_template: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAccessGrantCreateRequest {
    pub target_type: String,
    pub target_id: Uuid,
    pub role: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAccessGrantUpdateRequest {
    pub role: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAccessGrantDeleteRequest {
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub project: ProjectWorkspaceProject,
    pub general: ProjectSettingsGeneral,
    pub policy: ProjectSettingsPolicy,
    pub repositories: Vec<ProjectSettingsRepositoryLink>,
    pub access_grants: Vec<ProjectSettingsAccessGrant>,
    pub team_grants: Vec<ProjectSettingsTeamGrant>,
    pub eligible_users: Vec<ProjectWorkspaceUser>,
    pub eligible_teams: Vec<ProjectSettingsTeamOption>,
    pub status_updates: Vec<ProjectSettingsStatusUpdate>,
    pub template: ProjectSettingsTemplate,
    pub danger_state: ProjectSettingsDangerState,
    pub viewer_permissions: ProjectSettingsPermissions,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsGeneral {
    pub title: String,
    pub description: Option<String>,
    pub readme: Option<String>,
    pub visibility: String,
    pub default_repository_id: Option<Uuid>,
    pub created_by: Option<ProjectWorkspaceUser>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub readme_revision_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsPolicy {
    pub owner_kind: String,
    pub organization_id: Option<Uuid>,
    pub projects_enabled: bool,
    pub base_permission: Option<String>,
    pub visibility_changes_allowed: bool,
    pub visibility_locked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsRepositoryLink {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub href: String,
    pub visibility: String,
    pub link_type: String,
    pub is_default: bool,
    pub viewer_permission: Option<String>,
    pub linked_by: Option<ProjectWorkspaceUser>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsAccessGrant {
    pub id: Uuid,
    pub user: ProjectWorkspaceUser,
    pub role: String,
    pub source: String,
    pub inherited: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsTeamGrant {
    pub id: Uuid,
    pub team: ProjectSettingsTeamOption,
    pub role: String,
    pub member_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsTeamOption {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsStatusUpdate {
    pub id: Uuid,
    pub status: String,
    pub label: String,
    pub body: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub author: Option<ProjectWorkspaceUser>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsTemplate {
    pub is_template: bool,
    pub template_id: Option<Uuid>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsDangerState {
    pub state: String,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<ProjectWorkspaceUser>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<ProjectWorkspaceUser>,
    pub delete_confirmation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettingsPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_edit_general: bool,
    pub can_change_visibility: bool,
    pub can_link_repositories: bool,
    pub can_publish_status: bool,
    pub can_manage_template: bool,
    pub can_manage_access: bool,
    pub can_close: bool,
    pub can_reopen: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAutomationInvocationRequest {
    pub source: String,
    pub item_id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub actions_workflow_run_id: Option<Uuid>,
    pub idempotency_key: String,
    pub field_updates: Vec<ProjectAutomationFieldUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAutomationFieldUpdate {
    pub field_id: Uuid,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAutomationInvocationResponse {
    pub project_id: Uuid,
    pub item_id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub source: String,
    pub status: String,
    pub message: String,
    pub applied_updates: Vec<ProjectAutomationAppliedUpdate>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAutomationAppliedUpdate {
    pub field_id: Uuid,
    pub field_name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAutomationEvent {
    ItemAdded,
    IssueClosed,
    IssueReopened,
    PullRequestClosed,
    PullRequestMerged,
}

impl ProjectAutomationEvent {
    fn as_str(self) -> &'static str {
        match self {
            Self::ItemAdded => "item_added",
            Self::IssueClosed => "issue_closed",
            Self::IssueReopened => "issue_reopened",
            Self::PullRequestClosed => "pull_request_closed",
            Self::PullRequestMerged => "pull_request_merged",
        }
    }

    fn workflow_events(self) -> &'static [&'static str] {
        match self {
            Self::ItemAdded => &["item_added"],
            Self::IssueClosed | Self::PullRequestClosed => &["item_closed"],
            Self::IssueReopened => &["item_reopened"],
            Self::PullRequestMerged => &["pull_request_merged", "item_closed"],
        }
    }

    fn state(self) -> &'static str {
        match self {
            Self::IssueClosed | Self::PullRequestClosed | Self::PullRequestMerged => "closed",
            Self::ItemAdded | Self::IssueReopened => "open",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectAutomationInput {
    pub actor_user_id: Uuid,
    pub repository_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub pull_request_id: Option<Uuid>,
    pub event: ProjectAutomationEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspace {
    pub project: ProjectWorkspaceProject,
    pub selected_view: ProjectWorkspaceView,
    pub views: Vec<ProjectWorkspaceView>,
    pub layout_choices: Vec<ProjectWorkspaceLayoutChoice>,
    pub fields: Vec<ProjectWorkspaceField>,
    pub board_config: Option<ProjectWorkspaceBoardConfig>,
    pub roadmap_config: Option<ProjectWorkspaceRoadmapConfig>,
    #[serde(flatten)]
    pub items: ListEnvelope<ProjectWorkspaceItem>,
    pub groups: Vec<ProjectWorkspaceGroup>,
    pub slices: Vec<ProjectWorkspaceSlice>,
    pub filters: ProjectWorkspaceFilters,
    pub unsaved_view: ProjectWorkspaceUnsavedState,
    pub viewer_permissions: ProjectWorkspacePermissions,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceProject {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub visibility: String,
    pub owner: String,
    pub href: String,
    pub workspace_href: String,
    pub viewer_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceView {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub layout: String,
    pub href: String,
    pub configuration: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceLayoutChoice {
    pub layout: String,
    pub label: String,
    pub keyboard_hint: String,
    pub active: bool,
    pub enabled: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceField {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub position: i64,
    pub settings: Value,
    pub hidden: bool,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceBoardConfig {
    pub column_field: Option<ProjectWorkspaceLayoutField>,
    pub swimlane_field: Option<ProjectWorkspaceLayoutField>,
    pub eligible_column_fields: Vec<ProjectWorkspaceLayoutField>,
    pub eligible_swimlane_fields: Vec<ProjectWorkspaceLayoutField>,
    pub columns: Vec<ProjectWorkspaceBoardColumn>,
    pub empty_columns_visible: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceRoadmapConfig {
    pub start_date_field: Option<ProjectWorkspaceLayoutField>,
    pub target_date_field: Option<ProjectWorkspaceLayoutField>,
    pub marker_fields: Vec<ProjectWorkspaceLayoutField>,
    pub eligible_date_fields: Vec<ProjectWorkspaceLayoutField>,
    pub eligible_marker_fields: Vec<ProjectWorkspaceLayoutField>,
    pub zoom: String,
    pub zoom_options: Vec<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceLayoutField {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceBoardColumn {
    pub key: String,
    pub label: String,
    pub field_id: Uuid,
    pub count: i64,
    pub item_limit: Option<i64>,
    pub over_limit: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceItem {
    pub id: Uuid,
    pub item_type: String,
    pub position: String,
    pub title: String,
    pub body: Option<String>,
    pub state: Option<String>,
    pub number: Option<i64>,
    pub href: Option<String>,
    pub repository: Option<ProjectRepositoryScopeSummary>,
    pub field_values: Vec<ProjectWorkspaceFieldValue>,
    pub labels: Vec<ProjectWorkspaceLabel>,
    pub assignees: Vec<ProjectWorkspaceUser>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceFieldValue {
    pub field_id: Uuid,
    pub value: Value,
    pub display_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceLabel {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceUser {
    pub id: Uuid,
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceGroup {
    pub key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceSlice {
    pub key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceFilters {
    pub query: Option<String>,
    pub sort: String,
    pub group: Option<String>,
    pub slice: Option<String>,
    pub tokens: Vec<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceUnsavedState {
    pub active: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspacePermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_edit: bool,
    pub can_manage_views: bool,
    pub can_change_layout: bool,
    pub can_add_items: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemDetail {
    pub project: ProjectWorkspaceProject,
    pub item: ProjectWorkspaceItem,
    pub source: Option<ProjectItemSourceSummary>,
    pub activity: Vec<ProjectItemActivity>,
    pub comments: Vec<ProjectItemComment>,
    pub archive: ProjectItemArchiveState,
    pub draft: Option<ProjectDraftIssueMetadata>,
    pub viewer_permissions: ProjectItemDetailPermissions,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemSourceSummary {
    pub source_type: String,
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub href: String,
    pub repository: ProjectRepositoryScopeSummary,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub sync_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemActivity {
    pub id: Uuid,
    pub event_type: String,
    pub actor: Option<ProjectWorkspaceUser>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemComment {
    pub id: Uuid,
    pub author: ProjectWorkspaceUser,
    pub body: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemArchiveState {
    pub archived: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by: Option<ProjectWorkspaceUser>,
    pub restored_at: Option<DateTime<Utc>>,
    pub restored_by: Option<ProjectWorkspaceUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDraftIssueMetadata {
    pub editable: bool,
    pub edit_version: DateTime<Utc>,
    pub repository_notifications_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDraftUpdateRequest {
    pub title: String,
    pub body: Option<String>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemCommentCreateRequest {
    pub body: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemCommentUpdateRequest {
    pub body: String,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConversionTargets {
    pub project: ProjectWorkspaceProject,
    pub repositories: Vec<ProjectConversionRepository>,
    pub viewer_permissions: ProjectConversionPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConversionRepository {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub href: String,
    pub labels: Vec<ProjectWorkspaceLabel>,
    pub assignees: Vec<ProjectWorkspaceUser>,
    pub milestones: Vec<ProjectConversionMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConversionMilestone {
    pub id: Uuid,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConversionPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_convert: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDraftConvertRequest {
    pub repository_id: Uuid,
    #[serde(default)]
    pub label_ids: Vec<Uuid>,
    #[serde(default)]
    pub assignee_user_ids: Vec<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectItemDetailPermissions {
    pub authenticated: bool,
    pub viewer_role: Option<String>,
    pub can_edit: bool,
    pub can_comment: bool,
    pub can_convert: bool,
    pub can_archive: bool,
    pub can_restore: bool,
    pub can_remove: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchivedItem {
    pub item: ProjectWorkspaceItem,
    pub source: Option<ProjectItemSourceSummary>,
    pub archived_at: DateTime<Utc>,
    pub archived_by: Option<ProjectWorkspaceUser>,
    pub viewer_permissions: ProjectItemDetailPermissions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyProjectRequest {
    pub title: String,
    #[serde(default)]
    pub include_draft_issues: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CopiedProject {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub href: String,
    pub workspace_href: String,
    pub owner: String,
    pub copied_views: i64,
    pub copied_fields: i64,
    pub copied_workflows: i64,
    pub copied_draft_items: i64,
}

#[derive(Debug, Clone)]
enum ProjectScope {
    User {
        id: Uuid,
        login: String,
    },
    Organization {
        id: Uuid,
        login: String,
        viewer_role: Option<String>,
        projects_enabled: bool,
    },
    Repository {
        id: Uuid,
        owner_login: String,
        name: String,
        full_name: String,
        viewer_role: Option<String>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectsError {
    #[error("project list was not found")]
    NotFound,
    #[error("project list is not visible to this viewer")]
    Forbidden,
    #[error("invalid project list filter: {0}")]
    InvalidFilter(String),
    #[error("invalid project mutation: {0}")]
    Validation(String),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
    #[error("repository error")]
    Repository(#[from] super::repositories::RepositoryError),
}

pub async fn copy_project_for_actor(
    pool: &PgPool,
    source_project_id: Uuid,
    actor_user_id: Uuid,
    request: CopyProjectRequest,
) -> Result<CopiedProject, ProjectsError> {
    let title = request.title.trim().chars().take(160).collect::<String>();
    if title.is_empty() {
        return Err(ProjectsError::Validation(
            "Project title is required.".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let source = sqlx::query(
        r#"
        SELECT
          projects.id,
          projects.owner_user_id,
          projects.owner_organization_id,
          projects.number,
          projects.title,
          projects.short_description,
          projects.readme,
          projects.visibility,
          projects.default_repository_id,
          projects.created_by_user_id,
          COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
          COALESCE(organization_policy_settings.projects_enabled, true) AS projects_enabled,
          organization_memberships.role AS organization_role,
          organization_policy_settings.projects_base_permission AS organization_base_role,
          project_permissions.role AS project_role
        FROM projects
        LEFT JOIN users owner_user ON owner_user.id = projects.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = projects.owner_organization_id
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = projects.owner_organization_id
        LEFT JOIN organization_memberships
          ON organization_memberships.organization_id = projects.owner_organization_id
         AND organization_memberships.user_id = $2
        LEFT JOIN project_permissions
          ON project_permissions.project_id = projects.id
         AND project_permissions.user_id = $2
        WHERE projects.id = $1
        FOR UPDATE
        "#,
    )
    .bind(source_project_id)
    .bind(actor_user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ProjectsError::NotFound)?;

    let owner_user_id: Option<Uuid> = source.try_get("owner_user_id")?;
    let owner_organization_id: Option<Uuid> = source.try_get("owner_organization_id")?;
    let projects_enabled: bool = source.try_get("projects_enabled")?;
    if !projects_enabled {
        return Err(ProjectsError::Forbidden);
    }
    let project_role: Option<String> = source.try_get("project_role")?;
    let org_role: Option<String> = source.try_get("organization_role")?;
    let org_base_role: Option<String> = source.try_get("organization_base_role")?;
    let can_copy = owner_user_id == Some(actor_user_id)
        || project_role.as_deref().is_some_and(can_write_project_role)
        || org_role
            .as_deref()
            .is_some_and(|role| matches!(role, "owner" | "admin"))
        || org_base_role.as_deref().is_some_and(can_write_project_role);
    if !can_copy {
        return Err(ProjectsError::Forbidden);
    }

    let next_number: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(max(number), 0) + 1
        FROM projects
        WHERE (($1::uuid IS NOT NULL AND owner_user_id = $1)
            OR ($2::uuid IS NOT NULL AND owner_organization_id = $2))
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_organization_id)
    .fetch_one(&mut *tx)
    .await?;

    let new_project_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO projects (
          owner_user_id, owner_organization_id, number, title, short_description,
          readme, visibility, default_repository_id, created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_organization_id)
    .bind(next_number)
    .bind(&title)
    .bind(source.try_get::<Option<String>, _>("short_description")?)
    .bind(source.try_get::<Option<String>, _>("readme")?)
    .bind(source.try_get::<String, _>("visibility")?)
    .bind(source.try_get::<Option<Uuid>, _>("default_repository_id")?)
    .bind(actor_user_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO project_repositories (project_id, repository_id, link_type)
        SELECT $2, repository_id, link_type
        FROM project_repositories
        WHERE project_id = $1
        ON CONFLICT (project_id, repository_id) DO NOTHING
        "#,
    )
    .bind(source_project_id)
    .bind(new_project_id)
    .execute(&mut *tx)
    .await?;

    let copied_views = sqlx::query_scalar::<_, i64>(
        r#"
        WITH inserted AS (
          INSERT INTO project_views (project_id, name, layout, position, configuration)
          SELECT $2, name, layout, position, configuration
          FROM project_views
          WHERE project_id = $1
          RETURNING 1
        )
        SELECT count(*)::bigint FROM inserted
        "#,
    )
    .bind(source_project_id)
    .bind(new_project_id)
    .fetch_one(&mut *tx)
    .await?;
    let copied_fields = sqlx::query_scalar::<_, i64>(
        r#"
        WITH inserted AS (
          INSERT INTO project_fields (project_id, name, field_type, position, settings)
          SELECT $2, name, field_type, position, settings
          FROM project_fields
          WHERE project_id = $1
          RETURNING 1
        )
        SELECT count(*)::bigint FROM inserted
        "#,
    )
    .bind(source_project_id)
    .bind(new_project_id)
    .fetch_one(&mut *tx)
    .await?;
    let copied_workflows = sqlx::query_scalar::<_, i64>(
        r#"
        WITH inserted AS (
          INSERT INTO project_workflows (project_id, name, enabled, trigger_event, configuration)
          SELECT $2, name, enabled, trigger_event, configuration
          FROM project_workflows
          WHERE project_id = $1
          RETURNING 1
        )
        SELECT count(*)::bigint FROM inserted
        "#,
    )
    .bind(source_project_id)
    .bind(new_project_id)
    .fetch_one(&mut *tx)
    .await?;
    let copied_draft_items = if request.include_draft_issues {
        sqlx::query_scalar::<_, i64>(
            r#"
            WITH inserted AS (
              INSERT INTO project_items (project_id, item_type, title, body, position)
              SELECT $2, item_type, title, body, position
              FROM project_items
              WHERE project_id = $1
                AND item_type = 'draft_issue'
                AND archived_at IS NULL
              RETURNING 1
            )
            SELECT count(*)::bigint FROM inserted
            "#,
        )
        .bind(source_project_id)
        .bind(new_project_id)
        .fetch_one(&mut *tx)
        .await?
    } else {
        0
    };

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.copy', 'project', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(new_project_id)
    .bind(json!({
        "sourceProjectId": source_project_id,
        "includeDraftIssues": request.include_draft_issues,
        "copiedViews": copied_views,
        "copiedFields": copied_fields,
        "copiedWorkflows": copied_workflows,
        "copiedDraftItems": copied_draft_items
    }))
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO project_recent_visits (project_id, user_id, reason, metadata)
        VALUES ($1, $2, 'copy', $3)
        ON CONFLICT (project_id, user_id, reason)
        DO UPDATE SET viewed_at = now(), metadata = EXCLUDED.metadata
        "#,
    )
    .bind(new_project_id)
    .bind(actor_user_id)
    .bind(json!({ "sourceProjectId": source_project_id }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    let owner: String = source
        .try_get::<Option<String>, _>("owner_login")?
        .unwrap_or_else(|| "unknown".to_owned());
    Ok(CopiedProject {
        id: new_project_id,
        number: next_number,
        title,
        href: format!("/{owner}/projects/{next_number}"),
        workspace_href: format!("/{owner}/projects/{next_number}/views/1"),
        owner,
        copied_views,
        copied_fields,
        copied_workflows,
        copied_draft_items,
    })
}

pub async fn user_projects(
    pool: &PgPool,
    username: &str,
    viewer_user_id: Option<Uuid>,
    query: ProjectListQuery<'_>,
) -> Result<ProjectList, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT id, COALESCE(NULLIF(username, ''), email) AS login
        FROM users
        WHERE lower(COALESCE(NULLIF(username, ''), email)) = lower($1)
           OR lower(email) = lower($1)
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;

    let scope = ProjectScope::User {
        id: row.try_get("id")?,
        login: row.try_get("login")?,
    };
    projects_for_scope(pool, scope, viewer_user_id, query).await
}

pub async fn project_workspace(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
    query: ProjectWorkspaceQuery<'_>,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = workspace_project_row(pool, project_id, viewer_user_id).await?;
    if project.visibility != "public" && project.viewer_role.is_none() {
        return if viewer_user_id.is_some() {
            Err(ProjectsError::Forbidden)
        } else {
            Err(ProjectsError::NotFound)
        };
    }

    let viewer_role = project.viewer_role.clone();
    let views = workspace_views(pool, project_id, &project.owner, project.number).await?;
    let selected_view = select_workspace_view(&views, query.view)?;
    let fields = workspace_fields(pool, project_id, &selected_view).await?;
    let filters = normalize_workspace_filters(query, &selected_view, &fields)?;
    let unsaved_view = workspace_unsaved_state(&filters, &selected_view);
    let mut items = workspace_items(pool, project_id, viewer_user_id, &fields).await?;
    apply_workspace_filters(&mut items, &filters);
    sort_workspace_items(&mut items, &filters.sort);
    let groups = workspace_groups(&items, filters.group.as_deref(), &fields);
    let slices = workspace_slices(&items, filters.slice.as_deref(), &fields);
    let total = items.len() as i64;
    let can_edit = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    let layout_choices = workspace_layout_choices(&selected_view, can_edit, &fields);
    let board_config = workspace_board_config(pool, &selected_view, &fields, &items).await?;
    let roadmap_config = workspace_roadmap_config(pool, &selected_view, &fields).await?;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let page_items = items
        .into_iter()
        .skip(offset)
        .take(filters.page_size as usize)
        .collect();

    Ok(ProjectWorkspace {
        project,
        selected_view,
        views,
        layout_choices,
        fields,
        board_config: Some(board_config),
        roadmap_config: Some(roadmap_config),
        items: ListEnvelope {
            items: page_items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        groups,
        slices,
        filters,
        unsaved_view,
        viewer_permissions: ProjectWorkspacePermissions {
            authenticated: viewer_user_id.is_some(),
            viewer_role,
            can_edit,
            can_manage_views: can_edit,
            can_change_layout: can_edit,
            can_add_items: can_edit,
        },
        unavailable_reason: None,
    })
}

pub async fn project_item_detail(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectItemDetail, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, viewer_user_id).await?;
    let fields = workspace_fields_for_detail(pool, project_id).await?;
    let mut items =
        project_items_for_detail(pool, project_id, viewer_user_id, Some(item_id), false).await?;
    if items.is_empty() {
        items =
            project_items_for_detail(pool, project_id, viewer_user_id, Some(item_id), true).await?;
    }
    let item = items.pop().ok_or(ProjectsError::NotFound)?;
    let values = workspace_field_values(pool, &[item.id]).await?;
    let labels = workspace_labels(pool, &[item.id]).await?;
    let assignees = workspace_assignees(pool, &[item.id]).await?;
    let item = workspace_item_from_row(item.row, &fields, &values, &labels, &assignees)?;
    let source = project_item_source(pool, item.id).await?;
    let archive = project_item_archive_state(pool, item.id).await?;
    let activity = project_item_activity(pool, item.id).await?;
    let comments = project_item_comments(pool, item.id).await?;
    let can_edit = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    let is_draft = item.item_type == "draft_issue";
    let item_updated_at = item.updated_at;
    let archived = archive.archived;
    let permissions = project_item_permissions(
        viewer_user_id,
        project.viewer_role.clone(),
        can_edit,
        is_draft,
        archived,
    );
    Ok(ProjectItemDetail {
        project: project.clone(),
        item,
        source,
        activity,
        comments,
        archive,
        draft: is_draft.then_some(ProjectDraftIssueMetadata {
            editable: can_edit && !archived,
            edit_version: item_updated_at,
            repository_notifications_enabled: false,
        }),
        viewer_permissions: permissions,
        unavailable_reason: None,
    })
}

pub async fn project_items_archived(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
    query: ProjectItemsArchivedQuery<'_>,
) -> Result<ListEnvelope<ProjectArchivedItem>, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, viewer_user_id).await?;
    let filters = normalize_archived_item_filters(query)?;
    let fields = workspace_fields_for_detail(pool, project_id).await?;
    let mut rows = project_items_for_detail(pool, project_id, viewer_user_id, None, true).await?;
    if let Some(item_type) = filters.item_type.as_deref() {
        rows.retain(|row| row.item_type == item_type);
    }
    if let Some(query) = filters.query.as_deref() {
        let normalized = query.to_lowercase();
        rows.retain(|row| row.search_title.to_lowercase().contains(&normalized));
    }
    let total = rows.len() as i64;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let page_rows = rows
        .into_iter()
        .skip(offset)
        .take(filters.page_size as usize)
        .collect::<Vec<_>>();
    let item_ids = page_rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let values = workspace_field_values(pool, &item_ids).await?;
    let labels = workspace_labels(pool, &item_ids).await?;
    let assignees = workspace_assignees(pool, &item_ids).await?;
    let can_edit = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    let mut archived_items = Vec::new();
    for row in page_rows {
        let archived_at = row.archived_at.ok_or_else(|| {
            ProjectsError::Validation("Archived item is missing archive metadata.".to_owned())
        })?;
        let archived_by = row.archived_by;
        let item = workspace_item_from_row(row.row, &fields, &values, &labels, &assignees)?;
        let source = project_item_source(pool, item.id).await?;
        archived_items.push(ProjectArchivedItem {
            viewer_permissions: project_item_permissions(
                viewer_user_id,
                project.viewer_role.clone(),
                can_edit,
                item.item_type == "draft_issue",
                true,
            ),
            item,
            source,
            archived_at,
            archived_by,
        });
    }
    Ok(ListEnvelope {
        items: archived_items,
        total,
        page: filters.page,
        page_size: filters.page_size,
    })
}

pub async fn project_field_settings(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let project = workspace_project_row(pool, project_id, viewer_user_id).await?;
    if project.visibility != "public" && project.viewer_role.is_none() {
        return if viewer_user_id.is_some() {
            Err(ProjectsError::Forbidden)
        } else {
            Err(ProjectsError::NotFound)
        };
    }

    let viewer_role = project.viewer_role.clone();
    let can_manage = viewer_role.as_deref().is_some_and(can_write_project_role);
    let fields = field_settings_fields(pool, project_id).await?;
    let used_fields = fields.len() as i64;
    let max_fields = 50;

    Ok(ProjectFieldSettings {
        project,
        fields,
        limits: ProjectFieldSettingsLimits {
            max_fields,
            used_fields,
            remaining_fields: (max_fields - used_fields).max(0),
            max_options_per_field: 50,
            max_iterations_per_field: 100,
        },
        viewer_permissions: ProjectFieldSettingsPermissions {
            authenticated: viewer_user_id.is_some(),
            viewer_role,
            can_create_fields: can_manage,
            can_rename_fields: can_manage,
            can_delete_fields: can_manage,
            can_manage_options: can_manage,
            can_manage_iterations: can_manage,
        },
        unavailable_reason: None,
    })
}

pub async fn project_workflow_settings(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectWorkflowSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, viewer_user_id).await?;
    ensure_default_project_workflows(pool, project_id).await?;

    let viewer_role = project.viewer_role.clone();
    let can_manage = viewer_role.as_deref().is_some_and(can_write_project_role);
    let workflows = project_workflow_definitions(pool, project_id).await?;
    let eligible_fields = project_workflow_eligible_fields(pool, project_id).await?;
    let repository_targets =
        project_workflow_repository_targets(pool, project_id, viewer_user_id).await?;
    let recent_logs = project_workflow_execution_logs(pool, project_id).await?;

    Ok(ProjectWorkflowSettings {
        project,
        workflows,
        eligible_fields,
        repository_targets,
        recent_logs,
        viewer_permissions: ProjectWorkflowSettingsPermissions {
            authenticated: viewer_user_id.is_some(),
            viewer_role,
            can_manage_workflows: can_manage,
            can_view_logs: viewer_user_id.is_some(),
            can_select_repositories: can_manage,
        },
        automation_actor: "@opengithub-project-automation".to_owned(),
        unavailable_reason: None,
    })
}

pub async fn project_settings(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, viewer_user_id).await?;
    let general = project_settings_general(pool, project_id).await?;
    let policy = project_settings_policy(pool, project_id).await?;
    let repositories = project_settings_repositories(pool, project_id, viewer_user_id).await?;
    let access_grants = project_settings_access_grants(pool, project_id).await?;
    let team_grants = project_settings_team_grants(pool, project_id).await?;
    let eligible_users = project_settings_eligible_users(pool, project_id).await?;
    let eligible_teams = project_settings_eligible_teams(pool, project_id).await?;
    let status_updates = project_settings_status_updates(pool, project_id).await?;
    let template = project_settings_template(pool, project_id, general.title.clone()).await?;
    let danger_state = project_settings_danger_state(pool, project_id, &general.title).await?;

    let viewer_role = project.viewer_role.clone();
    let can_write = viewer_role.as_deref().is_some_and(can_write_project_role);
    let can_admin = viewer_role
        .as_deref()
        .is_some_and(|role| matches!(role, "owner" | "admin"));
    let visibility_changes_allowed = policy.visibility_changes_allowed;
    let projects_enabled = policy.projects_enabled;
    let closed = project.state == "closed";
    let deleted = danger_state.deleted_at.is_some();

    Ok(ProjectSettings {
        project,
        general,
        policy,
        repositories,
        access_grants,
        team_grants,
        eligible_users,
        eligible_teams,
        status_updates,
        template,
        danger_state,
        viewer_permissions: ProjectSettingsPermissions {
            authenticated: viewer_user_id.is_some(),
            viewer_role,
            can_edit_general: can_write && !closed && !deleted,
            can_change_visibility: can_admin && visibility_changes_allowed && !closed && !deleted,
            can_link_repositories: can_write && !closed && !deleted,
            can_publish_status: can_write && !deleted,
            can_manage_template: can_admin && !closed && !deleted,
            can_manage_access: can_admin && !closed && !deleted,
            can_close: can_admin && !closed && !deleted,
            can_reopen: can_admin && closed && !deleted,
            can_delete: can_admin && !deleted,
        },
        unavailable_reason: (!projects_enabled)
            .then_some("Organization policy has disabled Projects.".to_owned()),
    })
}

pub async fn update_project_settings_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectSettingsUpdateRequest,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    let can_write = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_write || project.state == "closed" {
        return Err(ProjectsError::Forbidden);
    }
    let current = project_settings_general(pool, project_id).await?;
    if let Some(expected) = request.expected_updated_at {
        if current.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project settings changed since they were loaded. Refresh before saving."
                    .to_owned(),
            ));
        }
    }

    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProjectsError::Validation("Project title is required.".to_owned()))?;
    if title.len() > 120 {
        return Err(ProjectsError::Validation(
            "Project title must be 120 characters or fewer.".to_owned(),
        ));
    }
    let description = request
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if description.is_some_and(|value| value.len() > 280) {
        return Err(ProjectsError::Validation(
            "Project description must be 280 characters or fewer.".to_owned(),
        ));
    }
    let readme = request
        .readme
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if readme.is_some_and(|value| value.len() > 20_000) {
        return Err(ProjectsError::Validation(
            "Project README must be 20000 characters or fewer.".to_owned(),
        ));
    }
    let visibility = request
        .visibility
        .as_deref()
        .unwrap_or(current.visibility.as_str());
    if !matches!(visibility, "public" | "private") {
        return Err(ProjectsError::Validation(
            "Project visibility must be public or private.".to_owned(),
        ));
    }
    if visibility != current.visibility {
        let policy = project_settings_policy(pool, project_id).await?;
        let can_admin = project
            .viewer_role
            .as_deref()
            .is_some_and(|role| matches!(role, "owner" | "admin"));
        if !can_admin || !policy.visibility_changes_allowed {
            return Err(ProjectsError::Forbidden);
        }
    }
    if let Some(repository_id) = request.default_repository_id {
        ensure_project_repository_write(pool, project_id, repository_id, actor_user_id).await?;
    }

    sqlx::query(
        r#"
        UPDATE projects
        SET title = $2,
            short_description = $3,
            readme = $4,
            visibility = $5,
            default_repository_id = $6,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(readme)
    .bind(visibility)
    .bind(request.default_repository_id)
    .execute(pool)
    .await?;

    if readme != current.readme.as_deref() {
        sqlx::query(
            r#"
            INSERT INTO project_readme_revisions (project_id, author_user_id, body)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(project_id)
        .bind(actor_user_id)
        .bind(readme)
        .execute(pool)
        .await?;
    }
    audit_project_settings_change(
        pool,
        actor_user_id,
        "project.settings.update",
        project_id,
        json!({
            "title": title,
            "visibility": visibility,
            "defaultRepositoryId": request.default_repository_id,
            "readmeChanged": readme != current.readme.as_deref()
        }),
    )
    .await?;
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn link_project_repository_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    repository_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectRepositoryLinkRequest,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role)
        || project.state == "closed"
    {
        return Err(ProjectsError::Forbidden);
    }
    let current = project_settings_general(pool, project_id).await?;
    if let Some(expected) = request.expected_updated_at {
        if current.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project settings changed since they were loaded. Refresh before linking repositories."
                    .to_owned(),
            ));
        }
    }
    ensure_repository_write_for_actor(pool, repository_id, actor_user_id).await?;
    ensure_repository_link_allowed_for_project(pool, project_id, repository_id).await?;
    sqlx::query(
        r#"
        INSERT INTO project_repositories (project_id, repository_id, link_type, linked_by_user_id)
        VALUES ($1, $2, 'linked', $3)
        ON CONFLICT (project_id, repository_id) DO UPDATE
        SET linked_by_user_id = EXCLUDED.linked_by_user_id,
            updated_at = now()
        "#,
    )
    .bind(project_id)
    .bind(repository_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    audit_project_settings_change(
        pool,
        actor_user_id,
        "project.repository.link",
        project_id,
        json!({ "repositoryId": repository_id }),
    )
    .await?;
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn unlink_project_repository_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    repository_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectRepositoryLinkRequest,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role)
        || project.state == "closed"
    {
        return Err(ProjectsError::Forbidden);
    }
    let current = project_settings_general(pool, project_id).await?;
    if let Some(expected) = request.expected_updated_at {
        if current.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project settings changed since they were loaded. Refresh before unlinking repositories."
                    .to_owned(),
            ));
        }
    }
    sqlx::query(
        r#"
        DELETE FROM project_repositories
        WHERE project_id = $1 AND repository_id = $2
        "#,
    )
    .bind(project_id)
    .bind(repository_id)
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE projects SET default_repository_id = NULL WHERE id = $1 AND default_repository_id = $2",
    )
    .bind(project_id)
    .bind(repository_id)
    .execute(pool)
    .await?;
    audit_project_settings_change(
        pool,
        actor_user_id,
        "project.repository.unlink",
        project_id,
        json!({ "repositoryId": repository_id }),
    )
    .await?;
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn create_project_status_update_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectStatusUpdateRequest,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role)
        || project.state == "closed"
    {
        return Err(ProjectsError::Forbidden);
    }
    if !matches!(
        request.status.as_str(),
        "on_track" | "at_risk" | "off_track" | "complete"
    ) {
        return Err(ProjectsError::Validation(
            "Project status must be on_track, at_risk, off_track, or complete.".to_owned(),
        ));
    }
    let body = request
        .body
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if body.is_some_and(|value| value.len() > 4000) {
        return Err(ProjectsError::Validation(
            "Project status message must be 4000 characters or fewer.".to_owned(),
        ));
    }
    if let (Some(start), Some(target)) = (request.start_date, request.target_date) {
        if target < start {
            return Err(ProjectsError::Validation(
                "Project status target date cannot be before the start date.".to_owned(),
            ));
        }
    }
    sqlx::query(
        r#"
        INSERT INTO project_status_updates
            (project_id, author_user_id, status, body, start_date, target_date)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(project_id)
    .bind(actor_user_id)
    .bind(&request.status)
    .bind(body)
    .bind(request.start_date)
    .bind(request.target_date)
    .execute(pool)
    .await?;
    audit_project_settings_change(
        pool,
        actor_user_id,
        "project.status_update.create",
        project_id,
        json!({ "status": request.status }),
    )
    .await?;
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_template_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectTemplateUpdateRequest,
) -> Result<ProjectSettings, ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(|role| matches!(role, "owner" | "admin"))
        || project.state == "closed"
    {
        return Err(ProjectsError::Forbidden);
    }
    let current = project_settings_general(pool, project_id).await?;
    if let Some(expected) = request.expected_updated_at {
        if current.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project settings changed since they were loaded. Refresh before saving template settings."
                    .to_owned(),
            ));
        }
    }
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(current.title.as_str());
    let description = request
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let is_public = request.is_public.unwrap_or(false);

    sqlx::query("UPDATE projects SET is_template = $2, updated_at = now() WHERE id = $1")
        .bind(project_id)
        .bind(request.is_template)
        .execute(pool)
        .await?;
    if request.is_template {
        sqlx::query(
            r#"
            INSERT INTO project_templates (project_id, title, description, is_public)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_id) DO UPDATE
            SET title = EXCLUDED.title,
                description = EXCLUDED.description,
                is_public = EXCLUDED.is_public
            "#,
        )
        .bind(project_id)
        .bind(title)
        .bind(description)
        .bind(is_public)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("DELETE FROM project_templates WHERE project_id = $1")
            .bind(project_id)
            .execute(pool)
            .await?;
    }
    audit_project_settings_change(
        pool,
        actor_user_id,
        "project.template.update",
        project_id,
        json!({ "isTemplate": request.is_template, "isPublic": is_public }),
    )
    .await?;
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn create_project_access_grant_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectAccessGrantCreateRequest,
) -> Result<ProjectSettings, ProjectsError> {
    ensure_can_manage_project_access(pool, project_id, actor_user_id, request.expected_updated_at)
        .await?;
    let role = normalize_project_access_role(&request.role)?;
    match request.target_type.as_str() {
        "user" => {
            ensure_project_user_grant_allowed(pool, project_id, request.target_id).await?;
            sqlx::query(
                r#"
                INSERT INTO project_permissions (project_id, user_id, role, source)
                VALUES ($1, $2, $3, 'direct')
                ON CONFLICT (project_id, user_id) DO UPDATE
                SET role = EXCLUDED.role,
                    source = 'direct',
                    updated_at = now()
                "#,
            )
            .bind(project_id)
            .bind(request.target_id)
            .bind(role)
            .execute(pool)
            .await?;
            audit_project_settings_change(
                pool,
                actor_user_id,
                "project.access.user.grant",
                project_id,
                json!({ "userId": request.target_id, "role": role }),
            )
            .await?;
        }
        "team" => {
            ensure_project_team_grant_allowed(pool, project_id, request.target_id).await?;
            sqlx::query(
                r#"
                INSERT INTO project_team_permissions (project_id, team_id, role, created_by_user_id)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (project_id, team_id) DO UPDATE
                SET role = EXCLUDED.role,
                    updated_at = now()
                "#,
            )
            .bind(project_id)
            .bind(request.target_id)
            .bind(role)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
            audit_project_settings_change(
                pool,
                actor_user_id,
                "project.access.team.grant",
                project_id,
                json!({ "teamId": request.target_id, "role": role }),
            )
            .await?;
        }
        _ => {
            return Err(ProjectsError::Validation(
                "Project access targetType must be user or team.".to_owned(),
            ));
        }
    }
    project_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_access_grant_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    grant_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectAccessGrantUpdateRequest,
) -> Result<ProjectSettings, ProjectsError> {
    ensure_can_manage_project_access(pool, project_id, actor_user_id, request.expected_updated_at)
        .await?;
    let role = normalize_project_access_role(&request.role)?;
    if let Some(user_id) = project_access_grant_user_id(pool, project_id, grant_id).await? {
        if role != "admin" {
            ensure_not_last_project_admin(pool, project_id, Some(grant_id)).await?;
        }
        sqlx::query(
            "UPDATE project_permissions SET role = $3, source = 'direct', updated_at = now() WHERE project_id = $1 AND id = $2",
        )
        .bind(project_id)
        .bind(grant_id)
        .bind(role)
        .execute(pool)
        .await?;
        audit_project_settings_change(
            pool,
            actor_user_id,
            "project.access.user.update",
            project_id,
            json!({ "grantId": grant_id, "userId": user_id, "role": role }),
        )
        .await?;
        return project_settings(pool, project_id, Some(actor_user_id)).await;
    }
    if let Some(team_id) = project_access_grant_team_id(pool, project_id, grant_id).await? {
        sqlx::query(
            "UPDATE project_team_permissions SET role = $3, updated_at = now() WHERE project_id = $1 AND id = $2",
        )
        .bind(project_id)
        .bind(grant_id)
        .bind(role)
        .execute(pool)
        .await?;
        audit_project_settings_change(
            pool,
            actor_user_id,
            "project.access.team.update",
            project_id,
            json!({ "grantId": grant_id, "teamId": team_id, "role": role }),
        )
        .await?;
        return project_settings(pool, project_id, Some(actor_user_id)).await;
    }
    Err(ProjectsError::NotFound)
}

pub async fn delete_project_access_grant_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    grant_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectAccessGrantDeleteRequest,
) -> Result<ProjectSettings, ProjectsError> {
    ensure_can_manage_project_access(pool, project_id, actor_user_id, request.expected_updated_at)
        .await?;
    if let Some(user_id) = project_access_grant_user_id(pool, project_id, grant_id).await? {
        ensure_not_last_project_admin(pool, project_id, Some(grant_id)).await?;
        sqlx::query("DELETE FROM project_permissions WHERE project_id = $1 AND id = $2")
            .bind(project_id)
            .bind(grant_id)
            .execute(pool)
            .await?;
        audit_project_settings_change(
            pool,
            actor_user_id,
            "project.access.user.remove",
            project_id,
            json!({ "grantId": grant_id, "userId": user_id }),
        )
        .await?;
        return project_settings(pool, project_id, Some(actor_user_id)).await;
    }
    if let Some(team_id) = project_access_grant_team_id(pool, project_id, grant_id).await? {
        sqlx::query("DELETE FROM project_team_permissions WHERE project_id = $1 AND id = $2")
            .bind(project_id)
            .bind(grant_id)
            .execute(pool)
            .await?;
        audit_project_settings_change(
            pool,
            actor_user_id,
            "project.access.team.remove",
            project_id,
            json!({ "grantId": grant_id, "teamId": team_id }),
        )
        .await?;
        return project_settings(pool, project_id, Some(actor_user_id)).await;
    }
    Err(ProjectsError::NotFound)
}

pub async fn update_project_workflow_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    workflow_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectWorkflowUpdateRequest,
) -> Result<ProjectWorkflowSettings, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let workflow = sqlx::query(
        r#"
        SELECT id, workflow_key, trigger_event, configuration, updated_at
        FROM project_workflows
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(workflow_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;

    let workflow_key: String = workflow.get("workflow_key");
    let trigger_event: String = workflow.get("trigger_event");
    let current_configuration: Value = workflow.get("configuration");
    let updated_at: DateTime<Utc> = workflow.get("updated_at");
    if let Some(expected) = request.expected_updated_at {
        if updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project workflow changed since it was loaded. Refresh before saving.".to_owned(),
            ));
        }
    }

    let condition = request
        .condition
        .as_deref()
        .unwrap_or_else(|| {
            current_configuration
                .get("condition")
                .and_then(Value::as_str)
                .unwrap_or("")
        })
        .trim()
        .chars()
        .take(240)
        .collect::<String>();
    if request
        .condition
        .as_ref()
        .is_some_and(|value| value.len() > 240)
    {
        return Err(ProjectsError::Validation(
            "Workflow condition must be 240 characters or fewer.".to_owned(),
        ));
    }

    let mut target = current_configuration
        .get("target")
        .cloned()
        .unwrap_or(Value::Null);
    if let Some(field_id) = request.status_field_id {
        let option_id = request.status_option_id.ok_or_else(|| {
            ProjectsError::Validation("A target status option is required.".to_owned())
        })?;
        validate_project_workflow_status_target(pool, project_id, field_id, option_id).await?;
        target = json!({ "fieldId": field_id, "optionId": option_id });
    }

    let archive_after_days = request.archive_after_days.or_else(|| {
        current_configuration
            .get("archiveAfterDays")
            .and_then(Value::as_i64)
    });
    if let Some(days) = archive_after_days {
        if !(1..=365).contains(&days) {
            return Err(ProjectsError::Validation(
                "Archive criteria must be between 1 and 365 days.".to_owned(),
            ));
        }
    }

    let close_on_status = request.close_on_status.or_else(|| {
        current_configuration
            .get("closeOnStatus")
            .and_then(Value::as_bool)
    });

    if let Some(repository_ids) = &request.repository_target_ids {
        if repository_ids.len() > 50 {
            return Err(ProjectsError::Validation(
                "Workflow repository target selection is limited to 50 repositories.".to_owned(),
            ));
        }
        for repository_id in repository_ids {
            let linked = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM (
                  SELECT default_repository_id AS repository_id
                  FROM projects
                  WHERE id = $1 AND default_repository_id IS NOT NULL
                  UNION
                  SELECT repository_id
                  FROM project_repositories
                  WHERE project_id = $1
                ) linked
                WHERE repository_id = $2
                "#,
            )
            .bind(project_id)
            .bind(repository_id)
            .fetch_one(pool)
            .await?;
            if linked == 0 {
                return Err(ProjectsError::Validation(
                    "Workflow repository targets must be linked to this project.".to_owned(),
                ));
            }
            let permission =
                repository_permission_for_user(pool, *repository_id, actor_user_id).await?;
            if !permission
                .as_ref()
                .is_some_and(|permission| permission.role.can_write())
            {
                return Err(ProjectsError::Validation(
                    "Workflow repository targets require repository write access.".to_owned(),
                ));
            }
        }
    }

    let configuration = json!({
        "condition": condition,
        "target": target,
        "archiveAfterDays": archive_after_days,
        "closeOnStatus": close_on_status.unwrap_or(false),
    });

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE project_workflows
        SET enabled = COALESCE($3, enabled),
            configuration = $4,
            source = 'ui',
            actor_label = '@opengithub-project-automation',
            last_run_at = now(),
            last_run_status = 'success',
            last_run_message = 'Workflow configuration saved.',
            updated_at = now()
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(workflow_id)
    .bind(request.enabled)
    .bind(&configuration)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE project_workflow_rules
        SET configuration = $2, updated_at = now()
        WHERE project_workflow_id = $1 AND rule_type = 'default_condition'
        "#,
    )
    .bind(workflow_id)
    .bind(json!({
        "condition": configuration.get("condition").cloned().unwrap_or(Value::Null),
        "target": configuration.get("target").cloned().unwrap_or(Value::Null),
        "archiveAfterDays": configuration.get("archiveAfterDays").cloned().unwrap_or(Value::Null),
        "closeOnStatus": configuration.get("closeOnStatus").cloned().unwrap_or(Value::Null),
    }))
    .execute(&mut *tx)
    .await?;

    if let Some(repository_ids) = request.repository_target_ids {
        sqlx::query(
            "DELETE FROM project_workflow_repository_targets WHERE project_workflow_id = $1",
        )
        .bind(workflow_id)
        .execute(&mut *tx)
        .await?;
        for repository_id in repository_ids {
            sqlx::query(
                r#"
                INSERT INTO project_workflow_repository_targets (project_workflow_id, repository_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(workflow_id)
            .bind(repository_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query(
        r#"
        INSERT INTO workflow_execution_logs
          (project_id, project_workflow_id, actor_user_id, source, event_type, status, message, metadata)
        VALUES ($1, $2, $3, 'ui', $4, 'success', 'Workflow configuration saved.', $5)
        "#,
    )
    .bind(project_id)
    .bind(workflow_id)
    .bind(actor_user_id)
    .bind(&trigger_event)
    .bind(json!({
        "workflowKey": workflow_key,
        "enabled": request.enabled,
        "configuration": configuration,
    }))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.workflow.update', 'project_workflow', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(workflow_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "workflowKey": workflow_key,
        "enabled": request.enabled,
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    project_workflow_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn run_project_item_automation(
    pool: &PgPool,
    input: ProjectAutomationInput,
) -> Result<(), ProjectsError> {
    let item_rows = project_automation_items(pool, &input).await?;
    for item in item_rows {
        ensure_default_project_workflows(pool, item.project_id).await?;
        run_project_item_workflows(pool, &item, &input).await?;
        run_project_auto_archive(pool, item.project_id, input.actor_user_id).await?;
    }
    Ok(())
}

pub async fn invoke_project_automation_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectAutomationInvocationRequest,
) -> Result<ProjectAutomationInvocationResponse, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role)
    {
        return Err(ProjectsError::Forbidden);
    }

    let source = normalize_project_automation_source(&request.source)?;
    let idempotency_key = normalize_project_automation_idempotency_key(&request.idempotency_key)?;
    let workflow = resolve_project_invocation_workflow(
        pool,
        project_id,
        request.workflow_id,
        request.workflow_key.as_deref(),
    )
    .await?;
    let item = project_invocation_item(pool, project_id, request.item_id).await?;

    if workflow_log_exists(pool, project_id, &idempotency_key).await? {
        return Ok(ProjectAutomationInvocationResponse {
            project_id,
            item_id: request.item_id,
            workflow_id: workflow.as_ref().map(|workflow| workflow.id),
            workflow_key: workflow
                .as_ref()
                .map(|workflow| workflow.workflow_key.clone()),
            source,
            status: "skipped".to_owned(),
            message: "Automation invocation was already applied.".to_owned(),
            applied_updates: Vec::new(),
            idempotency_key,
        });
    }

    if let Some(run_id) = request.actions_workflow_run_id {
        validate_actions_workflow_run_for_invocation(pool, run_id, actor_user_id).await?;
    }

    if let Some(repository_id) = item.repository_id {
        let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
        if !permission
            .as_ref()
            .is_some_and(|permission| permission.role.can_write())
        {
            record_workflow_execution(
                pool,
                &WorkflowExecutionRecord {
                    project_id,
                    workflow_id: workflow.as_ref().map(|workflow| workflow.id),
                    item_id: Some(request.item_id),
                    actor_user_id: Some(actor_user_id),
                    source: &source,
                    event_type: "automation_invocation",
                    status: "skipped",
                    message: "Actor cannot update the linked repository item.",
                    metadata: json!({
                        "workflowKey": workflow.as_ref().map(|workflow| workflow.workflow_key.clone()),
                        "idempotencyKey": idempotency_key,
                        "repositoryId": repository_id,
                        "source": source,
                        "actionsWorkflowRunId": request.actions_workflow_run_id,
                    }),
                },
            )
            .await?;
            return Ok(ProjectAutomationInvocationResponse {
                project_id,
                item_id: request.item_id,
                workflow_id: workflow.as_ref().map(|workflow| workflow.id),
                workflow_key: workflow
                    .as_ref()
                    .map(|workflow| workflow.workflow_key.clone()),
                source,
                status: "skipped".to_owned(),
                message: "Actor cannot update the linked repository item.".to_owned(),
                applied_updates: Vec::new(),
                idempotency_key,
            });
        }
    }

    if request.field_updates.is_empty() || request.field_updates.len() > 10 {
        return Err(ProjectsError::Validation(
            "Automation invocations must include between 1 and 10 field updates.".to_owned(),
        ));
    }

    let mut applied_updates = Vec::with_capacity(request.field_updates.len());
    for update in &request.field_updates {
        let field = project_invocation_field(pool, project_id, update.field_id).await?;
        let value = normalize_project_automation_field_value(pool, &field, &update.value).await?;
        sqlx::query(
            r#"
            INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_item_id, project_field_id)
            DO UPDATE SET value = EXCLUDED.value,
                          updated_by_user_id = EXCLUDED.updated_by_user_id,
                          updated_at = now()
            "#,
        )
        .bind(request.item_id)
        .bind(update.field_id)
        .bind(&value)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
        applied_updates.push(ProjectAutomationAppliedUpdate {
            field_id: update.field_id,
            field_name: field.name,
            value,
        });
    }

    record_project_item_event(
        pool,
        project_id,
        request.item_id,
        actor_user_id,
        "project.workflow.invoke",
        json!({
            "workflowId": workflow.as_ref().map(|workflow| workflow.id),
            "workflowKey": workflow.as_ref().map(|workflow| workflow.workflow_key.clone()),
            "source": source,
            "actionsWorkflowRunId": request.actions_workflow_run_id,
            "fieldUpdates": applied_updates,
            "actor": "@opengithub-project-automation",
        }),
    )
    .await?;
    record_workflow_execution(
        pool,
        &WorkflowExecutionRecord {
            project_id,
            workflow_id: workflow.as_ref().map(|workflow| workflow.id),
            item_id: Some(request.item_id),
            actor_user_id: Some(actor_user_id),
            source: &source,
            event_type: "automation_invocation",
            status: "success",
            message: "Automation invocation updated the project item.",
            metadata: json!({
                "workflowKey": workflow.as_ref().map(|workflow| workflow.workflow_key.clone()),
                "idempotencyKey": idempotency_key,
                "actionsWorkflowRunId": request.actions_workflow_run_id,
                "source": source,
                "fieldUpdates": applied_updates,
                "itemType": item.item_type,
            }),
        },
    )
    .await?;
    if let Some(workflow) = &workflow {
        sqlx::query(
            r#"
            UPDATE project_workflows
            SET last_run_at = now(),
                last_run_status = 'success',
                last_run_message = 'Automation invocation updated the project item.',
                source = $2,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(workflow.id)
        .bind(&source)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.workflow.invoke', 'project_item', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(request.item_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "workflowId": workflow.as_ref().map(|workflow| workflow.id),
        "workflowKey": workflow.as_ref().map(|workflow| workflow.workflow_key.clone()),
        "source": source,
        "actionsWorkflowRunId": request.actions_workflow_run_id,
        "fieldUpdates": applied_updates,
    }))
    .execute(pool)
    .await?;

    Ok(ProjectAutomationInvocationResponse {
        project_id,
        item_id: request.item_id,
        workflow_id: workflow.as_ref().map(|workflow| workflow.id),
        workflow_key: workflow
            .as_ref()
            .map(|workflow| workflow.workflow_key.clone()),
        source,
        status: "success".to_owned(),
        message: "Automation invocation updated the project item.".to_owned(),
        applied_updates,
        idempotency_key,
    })
}

pub async fn create_project_field_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldCreateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let name = normalize_project_field_name(&request.name)?;
    let field_type = normalize_custom_project_field_type(&request.field_type)?;
    let fields = field_settings_fields(pool, project_id).await?;
    if fields.len() >= 50 {
        return Err(ProjectsError::Validation(
            "Project field limit has been reached.".to_owned(),
        ));
    }
    ensure_unique_project_field_name(pool, project_id, None, &name).await?;
    let position = fields.iter().map(|field| field.position).max().unwrap_or(0) + 1;

    let field_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_fields (project_id, name, field_type, position, settings)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(&name)
    .bind(&field_type)
    .bind(position as i32)
    .bind(default_project_field_settings(&field_type))
    .fetch_one(pool)
    .await?;

    if field_type == "iteration" {
        seed_default_project_iterations(pool, field_id, Utc::now().date_naive(), 14, 3).await?;
    }

    invalidate_project_view_caches(pool, project_id).await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.field.create', 'project_field', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(field_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "fieldName": name,
        "fieldType": field_type,
    }))
    .execute(pool)
    .await?;

    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_field_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldUpdateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let field = project_field_admin_target(pool, project_id, field_id).await?;
    if is_builtin_project_field(&field.field_type) {
        return Err(ProjectsError::Validation(
            "Built-in project fields cannot be renamed.".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if field.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project field changed since it was loaded. Refresh before saving.".to_owned(),
            ));
        }
    }

    let name = normalize_project_field_name(&request.name)?;
    ensure_unique_project_field_name(pool, project_id, Some(field_id), &name).await?;
    sqlx::query(
        r#"
        UPDATE project_fields
        SET name = $3, cache_version = cache_version + 1, updated_at = now()
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .bind(&name)
    .execute(pool)
    .await?;
    invalidate_project_view_caches(pool, project_id).await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.field.rename', 'project_field', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(field_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "previousName": field.name,
        "fieldName": name,
        "fieldType": field.field_type,
    }))
    .execute(pool)
    .await?;

    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn delete_project_field_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldDeleteRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let field = project_field_admin_target(pool, project_id, field_id).await?;
    if is_builtin_project_field(&field.field_type) {
        return Err(ProjectsError::Validation(
            "Built-in project fields cannot be deleted.".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if field.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project field changed since it was loaded. Refresh before deleting.".to_owned(),
            ));
        }
    }

    let affected_item_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT project_items.id
        FROM project_item_field_values
        JOIN project_items ON project_items.id = project_item_field_values.project_item_id
        WHERE project_items.project_id = $1
          AND project_item_field_values.project_field_id = $2
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .fetch_all(pool)
    .await?;

    sqlx::query("DELETE FROM project_item_field_values WHERE project_field_id = $1")
        .bind(field_id)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        UPDATE project_fields
        SET deleted_at = now(), cache_version = cache_version + 1, updated_at = now()
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .execute(pool)
    .await?;
    invalidate_project_view_caches(pool, project_id).await?;

    for item_id in &affected_item_ids {
        sqlx::query(
            r#"
            INSERT INTO project_item_events (project_id, project_item_id, actor_user_id, event_type, metadata)
            VALUES ($1, $2, $3, 'project.field_value.delete', $4)
            "#,
        )
        .bind(project_id)
        .bind(item_id)
        .bind(actor_user_id)
        .bind(json!({
            "fieldId": field_id,
            "fieldName": field.name,
            "fieldType": field.field_type,
        }))
        .execute(pool)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.field.delete', 'project_field', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(field_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "fieldName": field.name,
        "fieldType": field.field_type,
        "removedValues": affected_item_ids.len(),
    }))
    .execute(pool)
    .await?;

    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn create_project_field_option_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldOptionCreateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = option_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let name = normalize_project_option_name(&request.name)?;
    let color = normalize_project_option_color(request.color.as_deref())?;
    let description = normalize_project_option_description(request.description.as_deref());
    ensure_unique_project_option_name(pool, field_id, None, &name).await?;

    let option_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_field_options WHERE project_field_id = $1",
    )
    .bind(field_id)
    .fetch_one(pool)
    .await?;
    if option_count >= 50 {
        return Err(ProjectsError::Validation(
            "Project option limit has been reached for this field.".to_owned(),
        ));
    }
    let position = next_project_option_position(pool, field_id).await?;
    let option_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_field_options (project_field_id, name, color, position, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(field_id)
    .bind(&name)
    .bind(&color)
    .bind(position as i32)
    .bind(&description)
    .fetch_one(pool)
    .await?;

    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_option_change(
        pool,
        actor_user_id,
        "project.field_option.create",
        option_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "optionName": name,
            "color": color,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_field_option_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    option_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldOptionUpdateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = option_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let option = project_option_admin_target(pool, field_id, option_id).await?;
    let name = normalize_project_option_name(&request.name)?;
    let color = normalize_project_option_color(request.color.as_deref())?;
    let description = normalize_project_option_description(request.description.as_deref());
    ensure_unique_project_option_name(pool, field_id, Some(option_id), &name).await?;

    sqlx::query(
        r#"
        UPDATE project_field_options
        SET name = $3, color = $4, description = $5
        WHERE project_field_id = $1 AND id = $2
        "#,
    )
    .bind(field_id)
    .bind(option_id)
    .bind(&name)
    .bind(&color)
    .bind(&description)
    .execute(pool)
    .await?;

    if option.name != name {
        sqlx::query(
            r#"
            UPDATE project_item_field_values
            SET value = to_jsonb($3::text), updated_by_user_id = $4, updated_at = now()
            WHERE project_field_id = $1 AND value = to_jsonb($2::text)
            "#,
        )
        .bind(field_id)
        .bind(&option.name)
        .bind(&name)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
        sqlx::query(
            r#"
            UPDATE project_board_column_settings
            SET option_key = $4, label = CASE WHEN label = $3 THEN $4 ELSE label END, updated_at = now()
            WHERE project_field_id = $1 AND option_key = $2
            "#,
        )
        .bind(field_id)
        .bind(&option.name)
        .bind(&option.name)
        .bind(&name)
        .execute(pool)
        .await?;
    }

    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_option_change(
        pool,
        actor_user_id,
        "project.field_option.update",
        option_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "previousName": option.name,
            "optionName": name,
            "color": color,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn reorder_project_field_options_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectFieldOptionReorderRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    option_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let existing: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM project_field_options WHERE project_field_id = $1 ORDER BY position, created_at",
    )
    .bind(field_id)
    .fetch_all(pool)
    .await?;
    if existing.len() != request.option_ids.len()
        || !existing.iter().all(|id| request.option_ids.contains(id))
    {
        return Err(ProjectsError::Validation(
            "Option reorder must include every option exactly once.".to_owned(),
        ));
    }
    for (index, option_id) in request.option_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE project_field_options SET position = $3 WHERE project_field_id = $1 AND id = $2",
        )
        .bind(field_id)
        .bind(option_id)
        .bind((index + 1) as i32)
        .execute(pool)
        .await?;
    }
    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_option_change(
        pool,
        actor_user_id,
        "project.field_option.reorder",
        field_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "optionIds": request.option_ids,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn delete_project_field_option_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    option_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = option_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let option = project_option_admin_target(pool, field_id, option_id).await?;
    let affected_item_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT project_items.id
        FROM project_item_field_values
        JOIN project_items ON project_items.id = project_item_field_values.project_item_id
        WHERE project_items.project_id = $1
          AND project_item_field_values.project_field_id = $2
          AND project_item_field_values.value = to_jsonb($3::text)
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .bind(&option.name)
    .fetch_all(pool)
    .await?;

    sqlx::query(
        "DELETE FROM project_item_field_values WHERE project_field_id = $1 AND value = to_jsonb($2::text)",
    )
    .bind(field_id)
    .bind(&option.name)
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM project_board_column_settings WHERE project_field_id = $1 AND option_key = $2",
    )
    .bind(field_id)
    .bind(&option.name)
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM project_field_options WHERE project_field_id = $1 AND id = $2")
        .bind(field_id)
        .bind(option_id)
        .execute(pool)
        .await?;

    for item_id in &affected_item_ids {
        sqlx::query(
            r#"
            INSERT INTO project_item_events (project_id, project_item_id, actor_user_id, event_type, metadata)
            VALUES ($1, $2, $3, 'project.field_option.delete', $4)
            "#,
        )
        .bind(project_id)
        .bind(item_id)
        .bind(actor_user_id)
        .bind(json!({
            "fieldId": field_id,
            "fieldName": field.name,
            "optionId": option_id,
            "optionName": option.name,
        }))
        .execute(pool)
        .await?;
    }

    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_option_change(
        pool,
        actor_user_id,
        "project.field_option.delete",
        option_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "optionName": option.name,
            "removedValues": affected_item_ids.len(),
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_iteration_settings_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectIterationSettingsRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = iteration_admin_target(pool, project_id, field_id, actor_user_id).await?;
    if let Some(expected) = request.expected_updated_at {
        if field.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project field changed since it was loaded. Refresh before saving iterations."
                    .to_owned(),
            ));
        }
    }
    let duration_days = normalize_iteration_duration(request.duration, &request.duration_unit)?;
    let generated = request.generated_iterations.unwrap_or(3).clamp(1, 100);
    sqlx::query("DELETE FROM project_iterations WHERE project_field_id = $1")
        .bind(field_id)
        .execute(pool)
        .await?;
    seed_default_project_iterations(pool, field_id, request.start_date, duration_days, generated)
        .await?;
    let settings = json!({
        "startDate": request.start_date,
        "duration": request.duration,
        "durationUnit": request.duration_unit.trim().to_ascii_lowercase(),
        "generatedIterations": generated,
    });
    sqlx::query(
        "UPDATE project_fields SET settings = $3, cache_version = cache_version + 1, updated_at = now() WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(field_id)
    .bind(settings)
    .execute(pool)
    .await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_iteration_change(
        pool,
        actor_user_id,
        "project.iteration_settings.update",
        field_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "startDate": request.start_date,
            "durationDays": duration_days,
            "generatedIterations": generated,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn create_project_iteration_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectIterationCreateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = iteration_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM project_iterations WHERE project_field_id = $1",
    )
    .bind(field_id)
    .fetch_one(pool)
    .await?;
    if count >= 100 {
        return Err(ProjectsError::Validation(
            "Project iteration limit has been reached for this field.".to_owned(),
        ));
    }
    let duration_days = request.duration_days.unwrap_or_else(|| {
        field
            .settings
            .get("duration")
            .and_then(Value::as_i64)
            .zip(field.settings.get("durationUnit").and_then(Value::as_str))
            .and_then(|(duration, unit)| normalize_iteration_duration(duration, unit).ok())
            .unwrap_or(14)
    });
    normalize_iteration_duration_days(duration_days)?;
    let start_date = match request.start_date {
        Some(date) => date,
        None => next_iteration_start_date(pool, field_id, duration_days).await?,
    };
    let name = normalize_iteration_name(
        request
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Iteration {}", count + 1)),
    )?;
    ensure_iteration_range_available(pool, field_id, None, start_date, duration_days).await?;
    let position = next_iteration_position(pool, field_id).await?;
    let iteration_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_iterations (project_field_id, name, start_date, duration_days, position)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(field_id)
    .bind(&name)
    .bind(start_date)
    .bind(duration_days as i32)
    .bind(position as i32)
    .fetch_one(pool)
    .await?;
    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_iteration_change(
        pool,
        actor_user_id,
        "project.iteration.create",
        iteration_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "iterationName": name,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_iteration_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    iteration_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectIterationUpdateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = iteration_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let name = normalize_iteration_name(request.name)?;
    normalize_iteration_duration_days(request.duration_days)?;
    ensure_iteration_exists(pool, field_id, iteration_id).await?;
    ensure_iteration_range_available(
        pool,
        field_id,
        Some(iteration_id),
        request.start_date,
        request.duration_days,
    )
    .await?;
    sqlx::query(
        "UPDATE project_iterations SET name = $3, start_date = $4, duration_days = $5 WHERE project_field_id = $1 AND id = $2",
    )
    .bind(field_id)
    .bind(iteration_id)
    .bind(&name)
    .bind(request.start_date)
    .bind(request.duration_days as i32)
    .execute(pool)
    .await?;
    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_iteration_change(
        pool,
        actor_user_id,
        "project.iteration.update",
        iteration_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "iterationName": name,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn create_project_iteration_break_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectIterationBreakCreateRequest,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = iteration_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let duration_days = request.duration_days.unwrap_or(1);
    normalize_iteration_duration_days(duration_days)?;
    ensure_iteration_range_available(pool, field_id, None, request.start_date, duration_days)
        .await?;
    let name = normalize_iteration_name(request.name.unwrap_or_else(|| "Break".to_owned()))?;
    let break_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_iteration_breaks (project_field_id, name, start_date, duration_days)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(field_id)
    .bind(&name)
    .bind(request.start_date)
    .bind(duration_days as i32)
    .fetch_one(pool)
    .await?;
    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_iteration_change(
        pool,
        actor_user_id,
        "project.iteration_break.create",
        break_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "breakName": name,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn delete_project_iteration_break_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    break_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectFieldSettings, ProjectsError> {
    let field = iteration_admin_target(pool, project_id, field_id, actor_user_id).await?;
    let deleted =
        sqlx::query("DELETE FROM project_iteration_breaks WHERE project_field_id = $1 AND id = $2")
            .bind(field_id)
            .bind(break_id)
            .execute(pool)
            .await?
            .rows_affected();
    if deleted == 0 {
        return Err(ProjectsError::NotFound);
    }
    touch_project_field(pool, project_id, field_id).await?;
    invalidate_project_view_caches(pool, project_id).await?;
    audit_project_iteration_change(
        pool,
        actor_user_id,
        "project.iteration_break.delete",
        break_id,
        json!({
            "projectId": project_id,
            "fieldId": field_id,
            "fieldName": field.name,
        }),
    )
    .await?;
    project_field_settings(pool, project_id, Some(actor_user_id)).await
}

pub async fn update_project_view_state_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    view_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectViewStateRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let views = workspace_views(pool, project_id, &project.owner, project.number).await?;
    let selected_view = views
        .iter()
        .find(|view| view.id == view_id)
        .cloned()
        .ok_or_else(|| {
            ProjectsError::InvalidFilter("view must reference an existing project view".to_owned())
        })?;
    if selected_view.layout != "table" {
        return Err(ProjectsError::InvalidFilter(
            "selected view must use the table layout".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if selected_view.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project view changed since it was loaded. Refresh before saving.".to_owned(),
            ));
        }
    }

    let fields = workspace_fields(pool, project_id, &selected_view).await?;
    let state = validate_project_view_state_request(&request, &fields)?;
    let mut configuration = selected_view.configuration.clone();
    if !configuration.is_object() {
        configuration = json!({});
    }
    configuration["query"] = state
        .query
        .as_ref()
        .map_or(Value::Null, |value| json!(value));
    configuration["sort"] = json!(state.sort);
    configuration["group"] = state
        .group
        .as_ref()
        .map_or(Value::Null, |value| json!(value));
    configuration["slice"] = state
        .slice
        .as_ref()
        .map_or(Value::Null, |value| json!(value));
    configuration["hiddenFieldIds"] = json!(state.hidden_field_ids);

    sqlx::query(
        r#"
        UPDATE project_views
        SET configuration = $3, updated_at = now()
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(view_id)
    .bind(&configuration)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.view_state.update', 'project_view', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(view_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "query": state.query,
        "sort": state.sort,
        "group": state.group,
        "slice": state.slice,
        "hiddenFieldIds": state.hidden_field_ids,
    }))
    .execute(pool)
    .await?;

    project_workspace(
        pool,
        project_id,
        Some(actor_user_id),
        ProjectWorkspaceQuery {
            view: Some(&view_id.to_string()),
            query: None,
            sort: None,
            group: None,
            slice: None,
            page: Some(1),
            page_size: None,
        },
    )
    .await
}

pub async fn update_project_view_layout_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    view_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectViewLayoutRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let views = workspace_views(pool, project_id, &project.owner, project.number).await?;
    let selected_view = views
        .iter()
        .find(|view| view.id == view_id)
        .cloned()
        .ok_or_else(|| {
            ProjectsError::InvalidFilter("view must reference an existing project view".to_owned())
        })?;
    if let Some(expected) = request.expected_updated_at {
        if selected_view.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project view changed since it was loaded. Refresh before changing layout."
                    .to_owned(),
            ));
        }
    }

    let fields = workspace_fields(pool, project_id, &selected_view).await?;
    let layout = validate_project_view_layout_request(&request, &fields)?;
    let mut configuration = selected_view.configuration.clone();
    if !configuration.is_object() {
        configuration = json!({});
    }
    if let Some(column_field_id) = layout.column_field_id {
        configuration["columnFieldId"] = json!(column_field_id.to_string());
    }
    if let Some(swimlane_field_id) = layout.swimlane_field_id {
        configuration["swimlaneFieldId"] = json!(swimlane_field_id.to_string());
    } else if layout.layout != "board" {
        configuration["swimlaneFieldId"] = Value::Null;
    }
    if let Some(start_field_id) = layout.start_field_id {
        configuration["startFieldId"] = json!(start_field_id.to_string());
    }
    if let Some(target_field_id) = layout.target_field_id {
        configuration["targetFieldId"] = json!(target_field_id.to_string());
    }

    sqlx::query(
        r#"
        UPDATE project_views
        SET layout = $3, configuration = $4, updated_at = now()
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(view_id)
    .bind(&layout.layout)
    .bind(&configuration)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.view_layout.update', 'project_view', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(view_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "layout": layout.layout,
        "columnFieldId": layout.column_field_id,
        "swimlaneFieldId": layout.swimlane_field_id,
        "startFieldId": layout.start_field_id,
        "targetFieldId": layout.target_field_id,
    }))
    .execute(pool)
    .await?;

    project_workspace(
        pool,
        project_id,
        Some(actor_user_id),
        ProjectWorkspaceQuery {
            view: Some(&view_id.to_string()),
            query: None,
            sort: None,
            group: None,
            slice: None,
            page: Some(1),
            page_size: None,
        },
    )
    .await
}

pub async fn update_project_roadmap_settings_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    view_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectRoadmapSettingsRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }

    let views = workspace_views(pool, project_id, &project.owner, project.number).await?;
    let selected_view = views
        .iter()
        .find(|view| view.id == view_id)
        .cloned()
        .ok_or_else(|| {
            ProjectsError::InvalidFilter("view must reference an existing project view".to_owned())
        })?;
    if selected_view.layout != "roadmap" {
        return Err(ProjectsError::InvalidFilter(
            "selected view must use the roadmap layout".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if selected_view.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project view changed since it was loaded. Refresh before saving roadmap settings."
                    .to_owned(),
            ));
        }
    }

    let fields = workspace_fields(pool, project_id, &selected_view).await?;
    let settings = validate_project_roadmap_settings_request(&request, &fields)?;
    let mut configuration = selected_view.configuration.clone();
    if !configuration.is_object() {
        configuration = json!({});
    }
    configuration["startFieldId"] = json!(settings.start_field_id.to_string());
    configuration["targetFieldId"] = json!(settings.target_field_id.to_string());
    configuration["markerFieldIds"] = json!(settings
        .marker_field_ids
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>());
    configuration["zoom"] = json!(settings.zoom);

    sqlx::query(
        r#"
        INSERT INTO project_roadmap_settings
          (project_view_id, start_field_id, target_field_id, marker_field_ids, zoom)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (project_view_id) DO UPDATE
        SET start_field_id = EXCLUDED.start_field_id,
            target_field_id = EXCLUDED.target_field_id,
            marker_field_ids = EXCLUDED.marker_field_ids,
            zoom = EXCLUDED.zoom,
            updated_at = now()
        "#,
    )
    .bind(view_id)
    .bind(settings.start_field_id)
    .bind(settings.target_field_id)
    .bind(&settings.marker_field_ids)
    .bind(&settings.zoom)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE project_views
        SET configuration = $3, updated_at = now()
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(view_id)
    .bind(&configuration)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.roadmap_settings.update', 'project_view', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(view_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "startFieldId": settings.start_field_id,
        "targetFieldId": settings.target_field_id,
        "markerFieldIds": settings.marker_field_ids,
        "zoom": settings.zoom,
    }))
    .execute(pool)
    .await?;

    project_workspace(
        pool,
        project_id,
        Some(actor_user_id),
        ProjectWorkspaceQuery {
            view: Some(&view_id.to_string()),
            query: None,
            sort: None,
            group: None,
            slice: None,
            page: Some(1),
            page_size: None,
        },
    )
    .await
}

pub async fn update_project_item_field_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemFieldValueRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let project_can_write = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !project_can_write {
        return Err(ProjectsError::Forbidden);
    }

    let field = workspace_field(pool, project_id, field_id).await?;
    if !field.editable {
        return Err(ProjectsError::Validation(
            "Project field is not editable from the table workspace".to_owned(),
        ));
    }

    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Archived project items cannot be edited".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if item.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project item changed since it was loaded. Refresh before editing.".to_owned(),
            ));
        }
    }

    if let (true, Some(repository_id)) = (is_linked_native_field(&field), item.repository_id) {
        let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
        if !permission.is_some_and(|permission| permission.role.can_write()) {
            return Err(ProjectsError::Forbidden);
        }
    }

    let normalized = normalize_project_field_value(&field, &request.value)?;
    apply_project_field_value(pool, &item, &field, &normalized, actor_user_id).await?;

    sqlx::query(
        r#"
        INSERT INTO project_item_events (project_id, project_item_id, actor_user_id, event_type, metadata)
        VALUES ($1, $2, $3, 'project.item_field.update', $4)
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(actor_user_id)
    .bind(json!({
        "fieldId": field_id,
        "fieldName": field.name,
        "fieldType": field.field_type,
        "value": normalized,
    }))
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'project.item_field.update', 'project_item', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(item_id.to_string())
    .bind(json!({
        "projectId": project_id,
        "fieldId": field_id,
        "fieldName": field.name,
        "itemType": item.item_type,
    }))
    .execute(pool)
    .await?;

    if let Some(repository_id) = item.repository_id {
        sqlx::query(
            r#"
            INSERT INTO timeline_events (repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata)
            VALUES ($1, $2, $3, $4, 'project_field_updated', $5)
            "#,
        )
        .bind(repository_id)
        .bind(item.issue_id)
        .bind(item.pull_request_id)
        .bind(actor_user_id)
        .bind(json!({
            "projectId": project_id,
            "projectItemId": item_id,
            "fieldId": field_id,
            "fieldName": field.name,
            "value": normalized,
        }))
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, repository_id, subject_type, subject_id, title, reason)
            SELECT issues.author_user_id, $2, 'project_item', $3, $4, 'project_field_update'
            FROM issues
            WHERE issues.id = $1 AND issues.author_user_id <> $5
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(item.issue_id.or(item.pull_request_issue_id))
        .bind(repository_id)
        .bind(item_id)
        .bind(format!("Project field {} was updated", field.name))
        .bind(actor_user_id)
        .execute(pool)
        .await?;
    }

    project_workspace(
        pool,
        project_id,
        Some(actor_user_id),
        ProjectWorkspaceQuery {
            view: None,
            query: None,
            sort: None,
            group: None,
            slice: None,
            page: Some(1),
            page_size: None,
        },
    )
    .await
}

pub async fn add_project_item_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemAddRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = writable_workspace_project(pool, project_id, actor_user_id).await?;
    let created_item_id = create_project_item(pool, project_id, actor_user_id, request).await?;
    record_project_item_event(
        pool,
        project_id,
        created_item_id,
        actor_user_id,
        "project.item.add",
        json!({ "source": "workspace_add_row" }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.add",
        created_item_id,
        json!({ "projectId": project_id, "projectTitle": project.title }),
    )
    .await?;
    if let Some(input) = project_automation_input_for_item(
        pool,
        created_item_id,
        actor_user_id,
        ProjectAutomationEvent::ItemAdded,
    )
    .await?
    {
        run_project_item_automation(pool, input).await?;
    }
    project_workspace_after_item_mutation(pool, project_id, actor_user_id).await
}

pub async fn bulk_add_project_items_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemsBulkAddRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    let project = writable_workspace_project(pool, project_id, actor_user_id).await?;
    if request.items.is_empty() {
        return Err(ProjectsError::Validation(
            "At least one project item is required".to_owned(),
        ));
    }
    if request.items.len() > 50 {
        return Err(ProjectsError::Validation(
            "Bulk add supports at most 50 items".to_owned(),
        ));
    }
    let mut created = Vec::new();
    for item in request.items {
        let item_id = create_project_item(pool, project_id, actor_user_id, item).await?;
        record_project_item_event(
            pool,
            project_id,
            item_id,
            actor_user_id,
            "project.item.add",
            json!({ "source": "workspace_bulk_add" }),
        )
        .await?;
        if let Some(input) = project_automation_input_for_item(
            pool,
            item_id,
            actor_user_id,
            ProjectAutomationEvent::ItemAdded,
        )
        .await?
        {
            run_project_item_automation(pool, input).await?;
        }
        created.push(item_id);
    }
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.bulk_add",
        project_id,
        json!({
            "projectId": project_id,
            "projectTitle": project.title,
            "createdItemIds": created,
        }),
    )
    .await?;
    project_workspace_after_item_mutation(pool, project_id, actor_user_id).await
}

pub async fn update_project_draft_item_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectDraftUpdateRequest,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item =
        draft_item_edit_target(pool, project_id, item_id, request.expected_updated_at).await?;
    let title = normalize_draft_title(&request.title)?;
    let body = normalize_draft_body(request.body.as_deref())?;

    sqlx::query(
        r#"
        UPDATE project_items
        SET title = $2, body = $3, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(item.id)
    .bind(&title)
    .bind(&body)
    .execute(pool)
    .await?;

    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.draft.update",
        json!({
            "title": title,
            "bodyUpdated": body.is_some(),
            "repositoryNotificationsEnabled": false,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.draft.update",
        item_id,
        json!({
            "projectId": project_id,
            "repositoryNotificationsEnabled": false,
        }),
    )
    .await?;

    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn create_project_item_comment_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemCommentCreateRequest,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    draft_item_edit_target(pool, project_id, item_id, request.expected_updated_at).await?;
    let body = normalize_project_item_comment_body(&request.body)?;

    let comment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_item_comments (project_id, project_item_id, author_user_id, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(actor_user_id)
    .bind(&body)
    .fetch_one(pool)
    .await?;

    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.draft_comment.create",
        json!({
            "commentId": comment_id,
            "repositoryNotificationsEnabled": false,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.draft_comment.create",
        comment_id,
        json!({ "projectId": project_id, "projectItemId": item_id }),
    )
    .await?;

    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn update_project_item_comment_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    comment_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemCommentUpdateRequest,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    draft_item_edit_target(pool, project_id, item_id, request.expected_updated_at).await?;
    let body = normalize_project_item_comment_body(&request.body)?;
    ensure_project_item_comment(pool, project_id, item_id, comment_id).await?;

    sqlx::query(
        r#"
        UPDATE project_item_comments
        SET body = $4, is_deleted = false, updated_at = now()
        WHERE project_id = $1 AND project_item_id = $2 AND id = $3
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(comment_id)
    .bind(&body)
    .execute(pool)
    .await?;

    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.draft_comment.update",
        json!({
            "commentId": comment_id,
            "repositoryNotificationsEnabled": false,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.draft_comment.update",
        comment_id,
        json!({ "projectId": project_id, "projectItemId": item_id }),
    )
    .await?;

    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn delete_project_item_comment_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    comment_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    draft_item_edit_target(pool, project_id, item_id, None).await?;
    ensure_project_item_comment(pool, project_id, item_id, comment_id).await?;

    sqlx::query(
        r#"
        UPDATE project_item_comments
        SET is_deleted = true, body = '[deleted]', updated_at = now()
        WHERE project_id = $1 AND project_item_id = $2 AND id = $3
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(comment_id)
    .execute(pool)
    .await?;

    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.draft_comment.delete",
        json!({
            "commentId": comment_id,
            "repositoryNotificationsEnabled": false,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.draft_comment.delete",
        comment_id,
        json!({ "projectId": project_id, "projectItemId": item_id }),
    )
    .await?;

    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn project_conversion_targets_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectConversionTargets, ProjectsError> {
    let project = writable_workspace_project(pool, project_id, actor_user_id).await?;
    let repositories = writable_project_repositories(pool, project_id, actor_user_id).await?;
    Ok(ProjectConversionTargets {
        project: project.clone(),
        repositories,
        viewer_permissions: ProjectConversionPermissions {
            authenticated: true,
            viewer_role: project.viewer_role,
            can_convert: true,
        },
    })
}

pub async fn convert_project_draft_to_issue_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectDraftConvertRequest,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.item_type != "draft_issue" {
        if item.issue_id.is_some() {
            return project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await;
        }
        return Err(ProjectsError::Validation(
            "Only draft project items can be converted to issues.".to_owned(),
        ));
    }
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Archived project items cannot be converted.".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if item.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project item changed since it was loaded. Refresh before converting.".to_owned(),
            ));
        }
    }
    ensure_project_repository_write(pool, project_id, request.repository_id, actor_user_id).await?;
    validate_conversion_labels(pool, request.repository_id, &request.label_ids).await?;
    validate_conversion_assignees(pool, request.repository_id, &request.assignee_user_ids).await?;
    validate_conversion_milestone(pool, request.repository_id, request.milestone_id).await?;

    let draft = sqlx::query(
        "SELECT title, body FROM project_items WHERE project_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;
    let title = normalize_draft_title(&draft.get::<String, _>("title"))?;
    let body = normalize_draft_body(draft.get::<Option<String>, _>("body").as_deref())?;
    let issue_number = next_issue_number(pool, request.repository_id).await?;
    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id, milestone_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(request.repository_id)
    .bind(issue_number)
    .bind(&title)
    .bind(&body)
    .bind(actor_user_id)
    .bind(request.milestone_id)
    .fetch_one(pool)
    .await?;

    for label_id in &request.label_ids {
        sqlx::query(
            "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(issue_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }
    for assignee_user_id in &request.assignee_user_ids {
        sqlx::query(
            r#"
            INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(issue_id)
        .bind(assignee_user_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        r#"
        UPDATE project_items
        SET item_type = 'issue',
            issue_id = $3,
            title = NULL,
            body = NULL,
            source_synced_at = now(),
            source_sync_version = source_sync_version + 1,
            updated_at = now()
        WHERE project_id = $1 AND id = $2
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(issue_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO timeline_events (repository_id, issue_id, actor_user_id, event_type, metadata)
        VALUES ($1, $2, $3, 'converted_from_project_draft', $4)
        "#,
    )
    .bind(request.repository_id)
    .bind(issue_id)
    .bind(actor_user_id)
    .bind(json!({ "projectId": project_id, "projectItemId": item_id }))
    .execute(pool)
    .await?;
    for assignee_user_id in &request.assignee_user_ids {
        if *assignee_user_id == actor_user_id {
            continue;
        }
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, repository_id, subject_type, subject_id, title, reason)
            VALUES ($1, $2, 'issue', $3, $4, 'assigned')
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(assignee_user_id)
        .bind(request.repository_id)
        .bind(issue_id)
        .bind(format!("You were assigned to {title}"))
        .execute(pool)
        .await?;
    }
    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.draft.convert_to_issue",
        json!({
            "issueId": issue_id,
            "issueNumber": issue_number,
            "repositoryId": request.repository_id,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.draft.convert_to_issue",
        item_id,
        json!({
            "projectId": project_id,
            "issueId": issue_id,
            "repositoryId": request.repository_id,
        }),
    )
    .await?;

    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn update_project_item_position_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemPositionRequest,
) -> Result<ProjectWorkspace, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Archived project items cannot be reordered".to_owned(),
        ));
    }
    if let Some(expected) = request.expected_updated_at {
        if item.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project item changed since it was loaded. Refresh before reordering.".to_owned(),
            ));
        }
    }

    let position = next_project_item_position(
        pool,
        project_id,
        request.after_item_id,
        request.before_item_id,
    )
    .await?;
    sqlx::query("UPDATE project_items SET position = $2, updated_at = now() WHERE id = $1")
        .bind(item_id)
        .bind(position)
        .execute(pool)
        .await?;

    if let Some(group_field_id) = request.group_field_id {
        let field = workspace_field(pool, project_id, group_field_id).await?;
        if !field.editable {
            return Err(ProjectsError::Validation(
                "Grouped rows can only move into editable fields".to_owned(),
            ));
        }
        let value = request.group_value.unwrap_or(Value::Null);
        let normalized = normalize_project_field_value(&field, &value)?;
        apply_project_field_value(pool, &item, &field, &normalized, actor_user_id).await?;
    }

    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.item.reorder",
        json!({
            "beforeItemId": request.before_item_id,
            "afterItemId": request.after_item_id,
            "groupFieldId": request.group_field_id,
        }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.reorder",
        item_id,
        json!({ "projectId": project_id, "position": position.to_string() }),
    )
    .await?;
    project_workspace_after_item_mutation(pool, project_id, actor_user_id).await
}

pub async fn remove_project_item_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectWorkspace, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Project item is already removed".to_owned(),
        ));
    }
    sqlx::query(
        r#"
        UPDATE project_items
        SET archived_at = now(),
            archived_by_user_id = $2,
            restored_at = NULL,
            restored_by_user_id = NULL,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(item_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.item.remove",
        json!({ "itemType": item.item_type }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.remove",
        item_id,
        json!({ "projectId": project_id }),
    )
    .await?;
    project_workspace_after_item_mutation(pool, project_id, actor_user_id).await
}

pub async fn archive_project_item_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Project item is already archived".to_owned(),
        ));
    }
    sqlx::query(
        r#"
        UPDATE project_items
        SET archived_at = now(),
            archived_by_user_id = $2,
            restored_at = NULL,
            restored_by_user_id = NULL,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(item_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.item.archive",
        json!({ "itemType": item.item_type }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.archive",
        item_id,
        json!({ "projectId": project_id }),
    )
    .await?;
    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn restore_project_item_for_actor(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectItemDetail, ProjectsError> {
    writable_workspace_project(pool, project_id, actor_user_id).await?;
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.archived_at.is_none() {
        return Err(ProjectsError::Validation(
            "Project item is not archived".to_owned(),
        ));
    }
    let next_position: f64 = sqlx::query_scalar(
        "SELECT COALESCE(max(position)::float8, 0) + 1 FROM project_items WHERE project_id = $1 AND archived_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE project_items
        SET archived_at = NULL,
            restored_at = now(),
            restored_by_user_id = $2,
            position = $3,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(item_id)
    .bind(actor_user_id)
    .bind(next_position)
    .execute(pool)
    .await?;
    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.item.restore",
        json!({ "itemType": item.item_type, "position": next_position.to_string() }),
    )
    .await?;
    record_project_audit(
        pool,
        actor_user_id,
        "project.item.restore",
        item_id,
        json!({ "projectId": project_id }),
    )
    .await?;
    project_item_detail(pool, project_id, item_id, Some(actor_user_id)).await
}

pub async fn organization_projects(
    pool: &PgPool,
    org: &str,
    viewer_user_id: Option<Uuid>,
    query: ProjectListQuery<'_>,
) -> Result<ProjectList, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT
          organizations.id,
          organizations.slug,
          organization_policy_settings.projects_base_permission,
          COALESCE(organization_policy_settings.projects_enabled, true) AS projects_enabled,
          organization_memberships.role AS viewer_role
        FROM organizations
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = organizations.id
        LEFT JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
         AND organization_memberships.user_id = $2
        WHERE lower(organizations.slug) = lower($1)
        "#,
    )
    .bind(org)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;

    let membership_role: Option<String> = row.try_get("viewer_role")?;
    let base_role: Option<String> = row.try_get("projects_base_permission")?;
    let viewer_role = membership_role.or(base_role.filter(|role| role != "none"));
    let projects_enabled: bool = row.try_get("projects_enabled")?;
    let scope = ProjectScope::Organization {
        id: row.try_get("id")?,
        login: row.try_get("slug")?,
        viewer_role,
        projects_enabled,
    };
    projects_for_scope(pool, scope, viewer_user_id, query).await
}

pub async fn repository_projects(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    viewer_user_id: Option<Uuid>,
    query: ProjectListQuery<'_>,
) -> Result<ProjectList, ProjectsError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await?
        .ok_or(ProjectsError::NotFound)?;
    if repository.visibility != super::repositories::RepositoryVisibility::Public {
        let Some(actor) = viewer_user_id else {
            return Err(ProjectsError::NotFound);
        };
        if !can_read_repository(pool, &repository, actor).await? {
            return Err(ProjectsError::NotFound);
        }
    }

    let viewer_role = match viewer_user_id {
        Some(user_id) => repository_permission_for_user(pool, repository.id, user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned()),
        None => None,
    };

    let scope = ProjectScope::Repository {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        full_name: format!("{}/{}", repository.owner_login, repository.name),
        viewer_role,
    };
    projects_for_scope(pool, scope, viewer_user_id, query).await
}

async fn projects_for_scope(
    pool: &PgPool,
    scope: ProjectScope,
    viewer_user_id: Option<Uuid>,
    query: ProjectListQuery<'_>,
) -> Result<ProjectList, ProjectsError> {
    let filters = normalize_project_filters(query)?;
    let rows = visible_project_rows(pool, &scope, viewer_user_id).await?;
    let mut projects = rows;
    apply_project_filters(&mut projects, &filters);
    sort_projects(&mut projects, &filters.sort);

    let counts = project_counts(&projects);
    let total = if filters.tab == "templates" {
        projects
            .iter()
            .filter(|project| project.is_template)
            .count() as i64
    } else {
        projects.len() as i64
    };
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let limit = filters.page_size as usize;
    let items = if filters.tab == "templates" {
        Vec::new()
    } else {
        projects
            .iter()
            .filter(|project| project.state == filters.state)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    };
    let templates_all = template_rows(&projects);
    let templates = templates_all
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let template_total = projects
        .iter()
        .filter(|project| project.is_template)
        .count() as i64;
    let permissions = permissions_for_scope(&scope, viewer_user_id);

    Ok(ProjectList {
        envelope: ListEnvelope {
            items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        scope: scope_summary(&scope),
        filters,
        counts,
        templates: ListEnvelope {
            items: templates,
            total: template_total,
            page: normalize_pagination(query.page, query.page_size).page,
            page_size: normalize_pagination(query.page, query.page_size).page_size,
        },
        viewer_permissions: permissions,
        unavailable_reason: unavailable_reason_for_scope(&scope),
    })
}

fn normalize_project_filters(
    query: ProjectListQuery<'_>,
) -> Result<ProjectListFilters, ProjectsError> {
    let pagination = normalize_pagination(query.page, query.page_size);
    let state = query.state.unwrap_or("open").trim().to_ascii_lowercase();
    if !matches!(state.as_str(), "open" | "closed") {
        return Err(ProjectsError::InvalidFilter(
            "state must be open or closed".to_owned(),
        ));
    }
    let tab = query.tab.unwrap_or("projects").trim().to_ascii_lowercase();
    if !matches!(tab.as_str(), "projects" | "templates") {
        return Err(ProjectsError::InvalidFilter(
            "tab must be projects or templates".to_owned(),
        ));
    }
    let sort = query
        .sort
        .unwrap_or("recently_updated")
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        sort.as_str(),
        "recently_updated" | "name_asc" | "name_desc" | "created_asc" | "created_desc"
    ) {
        return Err(ProjectsError::InvalidFilter(
            "sort must be recently_updated, name_asc, name_desc, created_asc, or created_desc"
                .to_owned(),
        ));
    }
    let query = query
        .query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(200).collect::<String>());

    Ok(ProjectListFilters {
        query,
        state,
        tab,
        sort,
        page: pagination.page,
        page_size: pagination.page_size,
    })
}

async fn visible_project_rows(
    pool: &PgPool,
    scope: &ProjectScope,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<ProjectRow>, ProjectsError> {
    let (owner_user_id, owner_organization_id, repository_id) = match scope {
        ProjectScope::User { id, .. } => (Some(*id), None, None),
        ProjectScope::Organization { id, .. } => (None, Some(*id), None),
        ProjectScope::Repository { id, .. } => (None, None, Some(*id)),
    };

    let rows = sqlx::query(
        r#"
        WITH latest_status AS (
            SELECT DISTINCT ON (project_id)
                   project_id, status, body, created_at
            FROM project_status_updates
            ORDER BY project_id, created_at DESC
        ),
        item_counts AS (
            SELECT
              project_id,
              count(*) FILTER (WHERE archived_at IS NULL) AS total_count,
              count(*) FILTER (WHERE archived_at IS NULL AND item_type = 'draft_issue') AS draft_count
            FROM project_items
            GROUP BY project_id
        ),
        repo_links AS (
            SELECT project_id, count(*) AS linked_count
            FROM project_repositories
            GROUP BY project_id
        ),
        viewer_roles AS (
            SELECT project_id, role
            FROM project_permissions
            WHERE user_id = $4
        )
        SELECT
          projects.id,
          projects.number,
          projects.title,
          projects.short_description,
          projects.state,
          projects.visibility,
          projects.is_template,
          projects.created_at,
          projects.updated_at,
          projects.closed_at,
          owner_user.username AS owner_username,
          owner_user.email AS owner_email,
          owner_org.slug AS owner_org_slug,
          default_repositories.id AS default_repository_id,
          COALESCE(NULLIF(default_owner.username, ''), default_owner.email, default_org.slug) AS default_repository_owner,
          default_repositories.name AS default_repository_name,
          latest_status.status AS status,
          latest_status.body AS status_body,
          latest_status.created_at AS status_created_at,
          COALESCE(item_counts.total_count, 0) AS items_total,
          COALESCE(item_counts.draft_count, 0) AS items_draft,
          COALESCE(repo_links.linked_count, 0) AS linked_repositories_count,
          viewer_roles.role AS viewer_role
        FROM projects
        LEFT JOIN users owner_user ON owner_user.id = projects.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = projects.owner_organization_id
        LEFT JOIN repositories default_repositories ON default_repositories.id = projects.default_repository_id
        LEFT JOIN users default_owner ON default_owner.id = default_repositories.owner_user_id
        LEFT JOIN organizations default_org ON default_org.id = default_repositories.owner_organization_id
        LEFT JOIN latest_status ON latest_status.project_id = projects.id
        LEFT JOIN item_counts ON item_counts.project_id = projects.id
        LEFT JOIN repo_links ON repo_links.project_id = projects.id
        LEFT JOIN viewer_roles ON viewer_roles.project_id = projects.id
        WHERE (
            ($1::uuid IS NOT NULL AND projects.owner_user_id = $1)
            OR ($2::uuid IS NOT NULL AND projects.owner_organization_id = $2)
            OR ($3::uuid IS NOT NULL AND (
                projects.default_repository_id = $3
                OR EXISTS (
                    SELECT 1 FROM project_repositories
                    WHERE project_repositories.project_id = projects.id
                      AND project_repositories.repository_id = $3
                )
            ))
        )
          AND (
            projects.visibility = 'public'
            OR projects.owner_user_id = $4
            OR viewer_roles.role IS NOT NULL
            OR EXISTS (
                SELECT 1
                FROM organization_memberships
                WHERE organization_memberships.organization_id = projects.owner_organization_id
                  AND organization_memberships.user_id = $4
            )
          )
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_organization_id)
    .bind(repository_id)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(project_from_row).collect()
}

fn project_from_row(row: sqlx::postgres::PgRow) -> Result<ProjectRow, ProjectsError> {
    let owner = row
        .try_get::<Option<String>, _>("owner_username")?
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            row.try_get::<Option<String>, _>("owner_email")
                .ok()
                .flatten()
        })
        .or_else(|| {
            row.try_get::<Option<String>, _>("owner_org_slug")
                .ok()
                .flatten()
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let number: i64 = row.try_get("number")?;
    let id: Uuid = row.try_get("id")?;
    let default_repository = row
        .try_get::<Option<Uuid>, _>("default_repository_id")?
        .map(|repo_id| {
            let repo_owner = row
                .try_get::<Option<String>, _>("default_repository_owner")
                .ok()
                .flatten()
                .unwrap_or_else(|| owner.clone());
            let repo_name = row
                .try_get::<Option<String>, _>("default_repository_name")
                .ok()
                .flatten()
                .unwrap_or_default();
            ProjectRepositoryScopeSummary {
                id: repo_id,
                owner: repo_owner.clone(),
                name: repo_name.clone(),
                full_name: format!("{repo_owner}/{repo_name}"),
                href: format!("/{repo_owner}/{repo_name}"),
            }
        });
    let status = row
        .try_get::<Option<String>, _>("status")?
        .map(|status| ProjectStatusSummary {
            label: status_label(&status),
            status,
            body: row.try_get("status_body").ok().flatten(),
            created_at: row
                .try_get("status_created_at")
                .unwrap_or_else(|_| row.try_get("updated_at").expect("updated_at")),
        });
    let viewer_role: Option<String> = row.try_get("viewer_role")?;
    let state: String = row.try_get("state")?;
    let is_template: bool = row.try_get("is_template")?;

    Ok(ProjectRow {
        id,
        number,
        title: row.try_get("title")?,
        description: row.try_get("short_description")?,
        state: state.clone(),
        visibility: row.try_get("visibility")?,
        href: format!("/{owner}/projects/{number}"),
        workspace_href: format!("/{owner}/projects/{number}/views/1"),
        owner,
        is_template,
        default_repository,
        linked_repositories_count: row.try_get("linked_repositories_count")?,
        status,
        counts: ProjectItemCounts {
            total: row.try_get("items_total")?,
            open: if state == "open" {
                row.try_get("items_total")?
            } else {
                0
            },
            closed: if state == "closed" {
                row.try_get("items_total")?
            } else {
                0
            },
            draft: row.try_get("items_draft")?,
        },
        viewer_can_copy: is_template || viewer_role.as_deref().is_some_and(can_write_project_role),
        viewer_role,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        closed_at: row.try_get("closed_at")?,
    })
}

async fn workspace_project_row(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectWorkspaceProject, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT
          projects.id, projects.number, projects.title, projects.short_description,
          projects.state, projects.visibility,
          projects.owner_user_id = $2 AS viewer_is_owner,
          COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
          project_permissions.role AS project_role,
          organization_memberships.role AS organization_role,
          organization_policy_settings.projects_base_permission AS organization_base_role
        FROM projects
        LEFT JOIN users owner_user ON owner_user.id = projects.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = projects.owner_organization_id
        LEFT JOIN project_permissions
          ON project_permissions.project_id = projects.id
         AND project_permissions.user_id = $2
        LEFT JOIN organization_memberships
          ON organization_memberships.organization_id = projects.owner_organization_id
         AND organization_memberships.user_id = $2
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = projects.owner_organization_id
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;
    let owner = row
        .try_get::<Option<String>, _>("owner_login")?
        .unwrap_or_else(|| "unknown".to_owned());
    let number: i64 = row.try_get("number")?;
    let viewer_role = workspace_role_from_row(&row)?;
    Ok(ProjectWorkspaceProject {
        id: project_id,
        number,
        title: row.try_get("title")?,
        description: row.try_get("short_description")?,
        state: row.try_get("state")?,
        visibility: row.try_get("visibility")?,
        href: format!("/{owner}/projects/{number}"),
        workspace_href: format!("/{owner}/projects/{number}/views/1"),
        owner,
        viewer_role,
    })
}

fn workspace_role_from_row(row: &sqlx::postgres::PgRow) -> Result<Option<String>, ProjectsError> {
    let viewer_is_owner: bool = row.try_get("viewer_is_owner").unwrap_or(false);
    if viewer_is_owner {
        return Ok(Some("admin".to_owned()));
    }
    let project_role: Option<String> = row.try_get("project_role")?;
    let org_role: Option<String> = row.try_get("organization_role")?;
    let org_base_role: Option<String> = row.try_get("organization_base_role")?;
    Ok(project_role
        .or(org_role)
        .or(org_base_role.filter(|role| role != "none")))
}

async fn project_settings_general(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<ProjectSettingsGeneral, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT projects.title,
               projects.short_description,
               projects.readme,
               projects.visibility,
               projects.default_repository_id,
               projects.created_at,
               projects.updated_at,
               creator.id AS creator_id,
               COALESCE(NULLIF(creator.username, ''), creator.email) AS creator_login,
               creator.avatar_url AS creator_avatar_url,
               COALESCE(readme_revisions.revision_count, 0) AS readme_revision_count
        FROM projects
        LEFT JOIN users creator ON creator.id = projects.created_by_user_id
        LEFT JOIN (
            SELECT project_id, count(*) AS revision_count
            FROM project_readme_revisions
            GROUP BY project_id
        ) readme_revisions ON readme_revisions.project_id = projects.id
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let created_by = row
        .try_get::<Option<Uuid>, _>("creator_id")?
        .map(|id| ProjectWorkspaceUser {
            id,
            login: row
                .try_get::<Option<String>, _>("creator_login")
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_owned()),
            avatar_url: row.get("creator_avatar_url"),
        });

    Ok(ProjectSettingsGeneral {
        title: row.get("title"),
        description: row.get("short_description"),
        readme: row.get("readme"),
        visibility: row.get("visibility"),
        default_repository_id: row.get("default_repository_id"),
        created_by,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        readme_revision_count: row.get("readme_revision_count"),
    })
}

async fn project_settings_policy(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<ProjectSettingsPolicy, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT projects.owner_organization_id,
               COALESCE(organization_policy_settings.projects_enabled, true) AS projects_enabled,
               organization_policy_settings.projects_base_permission,
               COALESCE(organization_policy_settings.members_can_change_repository_visibility, true) AS visibility_changes_allowed
        FROM projects
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = projects.owner_organization_id
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let organization_id: Option<Uuid> = row.get("owner_organization_id");
    let visibility_changes_allowed: bool = row.get("visibility_changes_allowed");
    Ok(ProjectSettingsPolicy {
        owner_kind: if organization_id.is_some() {
            "organization".to_owned()
        } else {
            "user".to_owned()
        },
        organization_id,
        projects_enabled: row.get("projects_enabled"),
        base_permission: row.get("projects_base_permission"),
        visibility_changes_allowed,
        visibility_locked_reason: (!visibility_changes_allowed)
            .then_some("Organization policy prevents project visibility changes.".to_owned()),
    })
}

async fn project_settings_repositories(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<ProjectSettingsRepositoryLink>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_repositories.id,
               repositories.id AS repository_id,
               repositories.name,
               repositories.visibility,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
               project_repositories.link_type,
               projects.default_repository_id = repositories.id AS is_default,
               project_repositories.created_at,
               project_repositories.updated_at,
               linked_by.id AS linked_by_id,
               COALESCE(NULLIF(linked_by.username, ''), linked_by.email) AS linked_by_login,
               linked_by.avatar_url AS linked_by_avatar_url
        FROM (
            SELECT id, project_id, repository_id, link_type, created_at, updated_at, linked_by_user_id
            FROM project_repositories
            WHERE project_id = $1
            UNION
            SELECT gen_random_uuid(), id, default_repository_id, 'default', created_at, updated_at, created_by_user_id
            FROM projects
            WHERE id = $1 AND default_repository_id IS NOT NULL
        ) project_repositories
        JOIN projects ON projects.id = project_repositories.project_id
        JOIN repositories ON repositories.id = project_repositories.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        LEFT JOIN users linked_by ON linked_by.id = project_repositories.linked_by_user_id
        ORDER BY is_default DESC, owner_login, repositories.name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let mut links = Vec::new();
    for row in rows {
        let repository_id: Uuid = row.get("repository_id");
        let visibility: String = row.get("visibility");
        let viewer_permission = match viewer_user_id {
            Some(user_id) => repository_permission_for_user(pool, repository_id, user_id)
                .await?
                .map(|permission| permission.role.as_str().to_owned()),
            None => None,
        };
        if visibility != "public" && viewer_permission.is_none() {
            continue;
        }
        let owner = row
            .try_get::<Option<String>, _>("owner_login")?
            .unwrap_or_else(|| "unknown".to_owned());
        let name: String = row.get("name");
        let linked_by =
            row.try_get::<Option<Uuid>, _>("linked_by_id")?
                .map(|id| ProjectWorkspaceUser {
                    id,
                    login: row
                        .try_get::<Option<String>, _>("linked_by_login")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "unknown".to_owned()),
                    avatar_url: row.get("linked_by_avatar_url"),
                });
        links.push(ProjectSettingsRepositoryLink {
            id: row.get("id"),
            repository_id,
            owner: owner.clone(),
            name: name.clone(),
            full_name: format!("{owner}/{name}"),
            href: format!("/{owner}/{name}"),
            visibility,
            link_type: row.get("link_type"),
            is_default: row.get("is_default"),
            viewer_permission,
            linked_by,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(links)
}

async fn project_settings_access_grants(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectSettingsAccessGrant>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_permissions.id,
               project_permissions.role,
               project_permissions.source,
               project_permissions.updated_at,
               users.id AS user_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM project_permissions
        JOIN users ON users.id = project_permissions.user_id
        WHERE project_permissions.project_id = $1
        ORDER BY project_permissions.source, login
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let source: String = row.get("source");
            ProjectSettingsAccessGrant {
                id: row.get("id"),
                user: ProjectWorkspaceUser {
                    id: row.get("user_id"),
                    login: row.get("login"),
                    avatar_url: row.get("avatar_url"),
                },
                role: row.get("role"),
                inherited: source != "direct",
                source,
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn project_settings_team_grants(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectSettingsTeamGrant>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_team_permissions.id,
               project_team_permissions.role,
               project_team_permissions.updated_at,
               teams.id AS team_id,
               teams.slug,
               teams.name,
               organizations.slug AS org_slug,
               COALESCE(member_counts.member_count, 0) AS member_count
        FROM project_team_permissions
        JOIN teams ON teams.id = project_team_permissions.team_id
        JOIN organizations ON organizations.id = teams.organization_id
        LEFT JOIN (
            SELECT team_id, count(*) AS member_count
            FROM team_memberships
            GROUP BY team_id
        ) member_counts ON member_counts.team_id = teams.id
        WHERE project_team_permissions.project_id = $1
        ORDER BY teams.name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let org_slug: String = row.get("org_slug");
            let slug: String = row.get("slug");
            ProjectSettingsTeamGrant {
                id: row.get("id"),
                team: ProjectSettingsTeamOption {
                    id: row.get("team_id"),
                    slug: slug.clone(),
                    name: row.get("name"),
                    href: format!("/orgs/{org_slug}/teams/{slug}"),
                },
                role: row.get("role"),
                member_count: row.get("member_count"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn project_settings_eligible_users(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectWorkspaceUser>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM projects
        JOIN users ON users.id = projects.owner_user_id
        WHERE projects.id = $1
        UNION
        SELECT DISTINCT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM projects
        JOIN organization_memberships
          ON organization_memberships.organization_id = projects.owner_organization_id
        JOIN users ON users.id = organization_memberships.user_id
        WHERE projects.id = $1
        ORDER BY login
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectWorkspaceUser {
            id: row.get("id"),
            login: row.get("login"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

async fn project_settings_eligible_teams(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectSettingsTeamOption>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT teams.id, teams.slug, teams.name, organizations.slug AS org_slug
        FROM projects
        JOIN teams ON teams.organization_id = projects.owner_organization_id
        JOIN organizations ON organizations.id = teams.organization_id
        WHERE projects.id = $1
        ORDER BY teams.name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let org_slug: String = row.get("org_slug");
            let slug: String = row.get("slug");
            ProjectSettingsTeamOption {
                id: row.get("id"),
                slug: slug.clone(),
                name: row.get("name"),
                href: format!("/orgs/{org_slug}/teams/{slug}"),
            }
        })
        .collect())
}

fn normalize_project_access_role(role: &str) -> Result<&str, ProjectsError> {
    match role {
        "read" | "write" | "admin" => Ok(role),
        _ => Err(ProjectsError::Validation(
            "Project access role must be read, write, or admin.".to_owned(),
        )),
    }
}

async fn ensure_can_manage_project_access(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    expected_updated_at: Option<DateTime<Utc>>,
) -> Result<(), ProjectsError> {
    let project = visible_workspace_project(pool, project_id, Some(actor_user_id)).await?;
    if project.state == "closed"
        || !project
            .viewer_role
            .as_deref()
            .is_some_and(|role| matches!(role, "owner" | "admin"))
    {
        return Err(ProjectsError::Forbidden);
    }
    if let Some(expected) = expected_updated_at {
        let current = project_settings_general(pool, project_id).await?;
        if current.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project settings changed since they were loaded. Refresh before changing access."
                    .to_owned(),
            ));
        }
    }
    Ok(())
}

async fn ensure_project_user_grant_allowed(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<(), ProjectsError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
          SELECT 1
          FROM projects
          WHERE projects.id = $1
            AND projects.owner_user_id = $2
          UNION
          SELECT 1
          FROM projects
          JOIN organization_memberships
            ON organization_memberships.organization_id = projects.owner_organization_id
           AND organization_memberships.user_id = $2
          WHERE projects.id = $1
        )
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "Project access can only be granted to the owner or organization members.".to_owned(),
        ))
    }
}

async fn ensure_project_team_grant_allowed(
    pool: &PgPool,
    project_id: Uuid,
    team_id: Uuid,
) -> Result<(), ProjectsError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
          SELECT 1
          FROM projects
          JOIN teams ON teams.organization_id = projects.owner_organization_id
          WHERE projects.id = $1 AND teams.id = $2
        )
        "#,
    )
    .bind(project_id)
    .bind(team_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "Project team grants must target a team in the owning organization.".to_owned(),
        ))
    }
}

async fn project_access_grant_user_id(
    pool: &PgPool,
    project_id: Uuid,
    grant_id: Uuid,
) -> Result<Option<Uuid>, ProjectsError> {
    sqlx::query_scalar("SELECT user_id FROM project_permissions WHERE project_id = $1 AND id = $2")
        .bind(project_id)
        .bind(grant_id)
        .fetch_optional(pool)
        .await
        .map_err(ProjectsError::Sqlx)
}

async fn project_access_grant_team_id(
    pool: &PgPool,
    project_id: Uuid,
    grant_id: Uuid,
) -> Result<Option<Uuid>, ProjectsError> {
    sqlx::query_scalar(
        "SELECT team_id FROM project_team_permissions WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(grant_id)
    .fetch_optional(pool)
    .await
    .map_err(ProjectsError::Sqlx)
}

async fn ensure_not_last_project_admin(
    pool: &PgPool,
    project_id: Uuid,
    excluding_grant_id: Option<Uuid>,
) -> Result<(), ProjectsError> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT
          (CASE WHEN projects.owner_user_id IS NOT NULL THEN 1 ELSE 0 END)
          + COALESCE((
              SELECT count(*)
              FROM organization_memberships
              WHERE organization_memberships.organization_id = projects.owner_organization_id
                AND organization_memberships.role = 'owner'
            ), 0)
          + COALESCE((
              SELECT count(*)
              FROM project_permissions
              WHERE project_permissions.project_id = projects.id
                AND project_permissions.role = 'admin'
                AND ($2::uuid IS NULL OR project_permissions.id <> $2)
            ), 0)
        FROM projects
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .bind(excluding_grant_id)
    .fetch_one(pool)
    .await?;
    if count > 0 {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "At least one project admin or owner must remain.".to_owned(),
        ))
    }
}

async fn project_settings_status_updates(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectSettingsStatusUpdate>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_status_updates.id,
               project_status_updates.status,
               project_status_updates.body,
               project_status_updates.start_date,
               project_status_updates.target_date,
               project_status_updates.created_at,
               users.id AS author_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url
        FROM project_status_updates
        LEFT JOIN users ON users.id = project_status_updates.author_user_id
        WHERE project_status_updates.project_id = $1
        ORDER BY project_status_updates.created_at DESC
        LIMIT 10
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let status: String = row.get("status");
            let author = row
                .try_get::<Option<Uuid>, _>("author_id")
                .ok()
                .flatten()
                .map(|id| ProjectWorkspaceUser {
                    id,
                    login: row
                        .try_get::<Option<String>, _>("author_login")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "unknown".to_owned()),
                    avatar_url: row.get("author_avatar_url"),
                });
            ProjectSettingsStatusUpdate {
                id: row.get("id"),
                label: status_label(&status),
                status,
                body: row.get("body"),
                start_date: row.get("start_date"),
                target_date: row.get("target_date"),
                author,
                created_at: row.get("created_at"),
            }
        })
        .collect())
}

async fn project_settings_template(
    pool: &PgPool,
    project_id: Uuid,
    fallback_title: String,
) -> Result<ProjectSettingsTemplate, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT projects.is_template,
               project_templates.id AS template_id,
               project_templates.title,
               project_templates.description,
               project_templates.is_public,
               project_templates.created_at
        FROM projects
        LEFT JOIN project_templates ON project_templates.project_id = projects.id
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    let is_template: bool = row.get("is_template");
    Ok(ProjectSettingsTemplate {
        is_template,
        template_id: row.get("template_id"),
        title: row
            .try_get::<Option<String>, _>("title")
            .ok()
            .flatten()
            .or_else(|| is_template.then_some(fallback_title)),
        description: row.get("description"),
        is_public: row
            .try_get::<Option<bool>, _>("is_public")
            .ok()
            .flatten()
            .unwrap_or(false),
        created_at: row.get("created_at"),
    })
}

async fn project_settings_danger_state(
    pool: &PgPool,
    project_id: Uuid,
    title: &str,
) -> Result<ProjectSettingsDangerState, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT projects.state,
               projects.closed_at,
               projects.deleted_at,
               closed_by.id AS closed_by_id,
               COALESCE(NULLIF(closed_by.username, ''), closed_by.email) AS closed_by_login,
               closed_by.avatar_url AS closed_by_avatar_url,
               deleted_by.id AS deleted_by_id,
               COALESCE(NULLIF(deleted_by.username, ''), deleted_by.email) AS deleted_by_login,
               deleted_by.avatar_url AS deleted_by_avatar_url
        FROM projects
        LEFT JOIN users closed_by ON closed_by.id = projects.closed_by_user_id
        LEFT JOIN users deleted_by ON deleted_by.id = projects.deleted_by_user_id
        WHERE projects.id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    let closed_by = row
        .try_get::<Option<Uuid>, _>("closed_by_id")
        .ok()
        .flatten()
        .map(|id| ProjectWorkspaceUser {
            id,
            login: row
                .try_get::<Option<String>, _>("closed_by_login")
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_owned()),
            avatar_url: row.get("closed_by_avatar_url"),
        });
    let deleted_by = row
        .try_get::<Option<Uuid>, _>("deleted_by_id")
        .ok()
        .flatten()
        .map(|id| ProjectWorkspaceUser {
            id,
            login: row
                .try_get::<Option<String>, _>("deleted_by_login")
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_owned()),
            avatar_url: row.get("deleted_by_avatar_url"),
        });
    Ok(ProjectSettingsDangerState {
        state: row.get("state"),
        closed_at: row.get("closed_at"),
        closed_by,
        deleted_at: row.get("deleted_at"),
        deleted_by,
        delete_confirmation: title.to_owned(),
    })
}

async fn workspace_views(
    pool: &PgPool,
    project_id: Uuid,
    owner: &str,
    project_number: i64,
) -> Result<Vec<ProjectWorkspaceView>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, layout, position, configuration, updated_at
        FROM project_views
        WHERE project_id = $1
        ORDER BY position, created_at
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let position: i32 = row.get("position");
            ProjectWorkspaceView {
                id: row.get("id"),
                number: i64::from(position),
                name: row.get("name"),
                layout: row.get("layout"),
                href: format!("/{owner}/projects/{project_number}/views/{position}"),
                configuration: row.get("configuration"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

fn select_workspace_view(
    views: &[ProjectWorkspaceView],
    requested: Option<&str>,
) -> Result<ProjectWorkspaceView, ProjectsError> {
    if views.is_empty() {
        return Err(ProjectsError::NotFound);
    }
    let requested = requested.unwrap_or("1").trim();
    let view = if let Ok(position) = requested.parse::<i64>() {
        views.iter().find(|view| view.number == position)
    } else if let Ok(id) = Uuid::parse_str(requested) {
        views.iter().find(|view| view.id == id)
    } else {
        None
    };
    view.cloned().ok_or_else(|| {
        ProjectsError::InvalidFilter("view must reference an existing project view".to_owned())
    })
}

async fn workspace_fields(
    pool: &PgPool,
    project_id: Uuid,
    selected_view: &ProjectWorkspaceView,
) -> Result<Vec<ProjectWorkspaceField>, ProjectsError> {
    let hidden = selected_view
        .configuration
        .get("hiddenFieldIds")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(|value| Uuid::parse_str(value).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let rows = sqlx::query(
        r#"
        SELECT id, name, field_type, position, settings
        FROM project_fields
        WHERE project_id = $1
          AND deleted_at IS NULL
        ORDER BY position, created_at
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let field_type: String = row.get("field_type");
            ProjectWorkspaceField {
                id,
                name: row.get("name"),
                field_type: field_type.clone(),
                position: i64::from(row.get::<i32, _>("position")),
                settings: row.get("settings"),
                hidden: hidden.contains(&id),
                editable: !matches!(field_type.as_str(), "repository"),
            }
        })
        .collect())
}

async fn workspace_field(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
) -> Result<ProjectWorkspaceField, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT id, name, field_type, position, settings
        FROM project_fields
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ProjectsError::InvalidFilter("field must reference a project field".to_owned())
    })?;
    let field_type: String = row.get("field_type");
    Ok(ProjectWorkspaceField {
        id: row.get("id"),
        name: row.get("name"),
        field_type: field_type.clone(),
        position: i64::from(row.get::<i32, _>("position")),
        settings: row.get("settings"),
        hidden: false,
        editable: !matches!(field_type.as_str(), "repository"),
    })
}

async fn field_settings_fields(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectFieldSettingsField>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT
          project_fields.id,
          project_fields.name,
          project_fields.field_type,
          project_fields.position,
          project_fields.settings,
          project_fields.cache_version,
          project_fields.updated_at,
          count(project_item_field_values.id)::bigint AS usage_count
        FROM project_fields
        LEFT JOIN project_item_field_values
          ON project_item_field_values.project_field_id = project_fields.id
        WHERE project_fields.project_id = $1
          AND project_fields.deleted_at IS NULL
        GROUP BY project_fields.id
        ORDER BY project_fields.position, project_fields.created_at
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let field_ids = rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let options = field_settings_options(pool, &field_ids).await?;
    let iterations = field_settings_iterations(pool, &field_ids).await?;
    let breaks = field_settings_breaks(pool, &field_ids).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let field_type: String = row.get("field_type");
            let built_in = is_builtin_project_field(&field_type);
            ProjectFieldSettingsField {
                id,
                name: row.get("name"),
                field_type,
                position: i64::from(row.get::<i32, _>("position")),
                settings: row.get("settings"),
                built_in,
                editable: !built_in,
                deletable: !built_in,
                usage_count: row.get("usage_count"),
                options: options
                    .iter()
                    .filter(|(field_id, _)| *field_id == id)
                    .map(|(_, option)| option.clone())
                    .collect(),
                iterations: iterations
                    .iter()
                    .filter(|(field_id, _)| *field_id == id)
                    .map(|(_, iteration)| iteration.clone())
                    .collect(),
                breaks: breaks
                    .iter()
                    .filter(|(field_id, _)| *field_id == id)
                    .map(|(_, iteration_break)| iteration_break.clone())
                    .collect(),
                cache_version: row.get("cache_version"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn field_settings_options(
    pool: &PgPool,
    field_ids: &[Uuid],
) -> Result<Vec<(Uuid, ProjectFieldOption)>, ProjectsError> {
    if field_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_field_id, id, name, color, position, description
        FROM project_field_options
        WHERE project_field_id = ANY($1)
        ORDER BY project_field_id, position, created_at
        "#,
    )
    .bind(field_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("project_field_id"),
                ProjectFieldOption {
                    id: row.get("id"),
                    name: row.get("name"),
                    color: row.get("color"),
                    position: i64::from(row.get::<i32, _>("position")),
                    description: row.get("description"),
                },
            )
        })
        .collect())
}

async fn field_settings_iterations(
    pool: &PgPool,
    field_ids: &[Uuid],
) -> Result<Vec<(Uuid, ProjectIteration)>, ProjectsError> {
    if field_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_field_id, id, name, start_date, duration_days, position
        FROM project_iterations
        WHERE project_field_id = ANY($1)
        ORDER BY project_field_id, position, start_date
        "#,
    )
    .bind(field_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("project_field_id"),
                ProjectIteration {
                    id: row.get("id"),
                    name: row.get("name"),
                    start_date: row.get("start_date"),
                    duration_days: i64::from(row.get::<i32, _>("duration_days")),
                    position: i64::from(row.get::<i32, _>("position")),
                },
            )
        })
        .collect())
}

async fn field_settings_breaks(
    pool: &PgPool,
    field_ids: &[Uuid],
) -> Result<Vec<(Uuid, ProjectIterationBreak)>, ProjectsError> {
    if field_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_field_id, id, name, start_date, duration_days
        FROM project_iteration_breaks
        WHERE project_field_id = ANY($1)
        ORDER BY project_field_id, start_date, created_at
        "#,
    )
    .bind(field_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("project_field_id"),
                ProjectIterationBreak {
                    id: row.get("id"),
                    name: row.get("name"),
                    start_date: row.get("start_date"),
                    duration_days: i64::from(row.get::<i32, _>("duration_days")),
                },
            )
        })
        .collect())
}

async fn ensure_default_project_workflows(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<(), ProjectsError> {
    let status_target = default_done_status_target(pool, project_id).await?;
    let defaults = [
        (
            "closed-item-to-done",
            "Closed issue or pull request",
            "Move closed issues and pull requests to Done.",
            "item_closed",
            json!({
                "condition": "state:closed",
                "target": status_target,
            }),
        ),
        (
            "merged-pr-to-done",
            "Merged pull request",
            "Move merged pull requests to Done.",
            "pull_request_merged",
            json!({
                "condition": "is:merged",
                "target": status_target,
            }),
        ),
        (
            "item-added-default-status",
            "Item added to project",
            "Set a default status when a matching issue or pull request is added.",
            "item_added",
            json!({
                "condition": "",
                "target": Value::Null,
            }),
        ),
        (
            "auto-archive-completed-items",
            "Auto-archive completed items",
            "Archive completed project items after a configured waiting period.",
            "archive_completed",
            json!({
                "condition": "status:done",
                "archiveAfterDays": 14,
            }),
        ),
    ];

    for (position, (key, name, description, event, configuration)) in
        defaults.into_iter().enumerate()
    {
        let enabled = matches!(key, "closed-item-to-done" | "merged-pr-to-done");
        let workflow_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO project_workflows
              (project_id, workflow_key, name, description, enabled, trigger_event, configuration)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (project_id, workflow_key) DO UPDATE
            SET description = EXCLUDED.description,
                trigger_event = EXCLUDED.trigger_event,
                configuration = CASE
                    WHEN project_workflows.configuration = '{}'::jsonb THEN EXCLUDED.configuration
                    ELSE project_workflows.configuration
                END
            RETURNING id
            "#,
        )
        .bind(project_id)
        .bind(key)
        .bind(name)
        .bind(description)
        .bind(enabled)
        .bind(event)
        .bind(&configuration)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO project_workflow_rules
              (project_workflow_id, rule_type, configuration, position)
            VALUES ($1, 'default_condition', $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(workflow_id)
        .bind(json!({
            "condition": configuration.get("condition").cloned().unwrap_or(Value::Null),
            "target": configuration.get("target").cloned().unwrap_or(Value::Null),
            "archiveAfterDays": configuration.get("archiveAfterDays").cloned().unwrap_or(Value::Null),
        }))
        .bind((position + 1) as i32)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ProjectAutomationItem {
    project_id: Uuid,
    item_id: Uuid,
    item_type: String,
    issue_id: Option<Uuid>,
    pull_request_id: Option<Uuid>,
    repository_id: Uuid,
}

#[derive(Debug, Clone)]
struct ProjectInvocationWorkflow {
    id: Uuid,
    workflow_key: String,
}

#[derive(Debug, Clone)]
struct ProjectInvocationItem {
    item_type: String,
    repository_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
struct ProjectInvocationField {
    id: Uuid,
    name: String,
    field_type: String,
}

fn normalize_project_automation_source(source: &str) -> Result<String, ProjectsError> {
    let normalized = source.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "actions" | "graphql" => Ok(normalized),
        _ => Err(ProjectsError::Validation(
            "Automation invocation source must be actions or graphql.".to_owned(),
        )),
    }
}

fn normalize_project_automation_idempotency_key(value: &str) -> Result<String, ProjectsError> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.len() > 160 {
        return Err(ProjectsError::Validation(
            "Automation invocation idempotency key is required and must be 160 characters or fewer."
                .to_owned(),
        ));
    }
    Ok(normalized.to_owned())
}

async fn resolve_project_invocation_workflow(
    pool: &PgPool,
    project_id: Uuid,
    workflow_id: Option<Uuid>,
    workflow_key: Option<&str>,
) -> Result<Option<ProjectInvocationWorkflow>, ProjectsError> {
    if workflow_id.is_none() && workflow_key.is_none() {
        return Ok(None);
    }
    let row = sqlx::query(
        r#"
        SELECT id, workflow_key
        FROM project_workflows
        WHERE project_id = $1
          AND ($2::uuid IS NULL OR id = $2)
          AND ($3::text IS NULL OR workflow_key = $3)
        "#,
    )
    .bind(project_id)
    .bind(workflow_id)
    .bind(
        workflow_key
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ProjectsError::Validation("Automation invocation workflow was not found.".to_owned())
    })?;
    Ok(Some(ProjectInvocationWorkflow {
        id: row.get("id"),
        workflow_key: row.get("workflow_key"),
    }))
}

async fn project_invocation_item(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
) -> Result<ProjectInvocationItem, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT project_items.item_type,
               COALESCE(issues.repository_id, pull_issues.repository_id, pull_requests.base_repository_id) AS repository_id
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN issues pull_issues ON pull_issues.id = pull_requests.issue_id
        WHERE project_items.project_id = $1
          AND project_items.id = $2
          AND project_items.archived_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;
    Ok(ProjectInvocationItem {
        item_type: row.get("item_type"),
        repository_id: row.get("repository_id"),
    })
}

async fn project_invocation_field(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
) -> Result<ProjectInvocationField, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT name, field_type
        FROM project_fields
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ProjectsError::Validation("Automation field update targets an unknown field.".to_owned())
    })?;
    Ok(ProjectInvocationField {
        id: field_id,
        name: row.get("name"),
        field_type: row.get("field_type"),
    })
}

async fn normalize_project_automation_field_value(
    pool: &PgPool,
    field: &ProjectInvocationField,
    value: &Value,
) -> Result<Value, ProjectsError> {
    match field.field_type.as_str() {
        "status" | "single_select" => {
            let Some(option_name) = value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                return Err(ProjectsError::Validation(
                    "Single-select automation values must be option names.".to_owned(),
                ));
            };
            let exists = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                  SELECT 1
                  FROM project_field_options
                  WHERE project_field_id = $1
                    AND lower(name) = lower($2)
                )
                "#,
            )
            .bind(field.id)
            .bind(option_name)
            .fetch_one(pool)
            .await?;
            if !exists {
                return Err(ProjectsError::Validation(
                    "Single-select automation value must match an existing option.".to_owned(),
                ));
            }
            Ok(json!(option_name))
        }
        "text" | "title" => {
            let Some(text) = value.as_str() else {
                return Err(ProjectsError::Validation(
                    "Text automation values must be strings.".to_owned(),
                ));
            };
            if text.len() > 1024 {
                return Err(ProjectsError::Validation(
                    "Text automation values must be 1024 characters or fewer.".to_owned(),
                ));
            }
            Ok(json!(text.trim()))
        }
        "number" => {
            if !value.is_number() {
                return Err(ProjectsError::Validation(
                    "Number automation values must be numeric.".to_owned(),
                ));
            }
            Ok(value.clone())
        }
        _ => Ok(value.clone()),
    }
}

async fn validate_actions_workflow_run_for_invocation(
    pool: &PgPool,
    run_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let repository_id =
        sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM workflow_runs WHERE id = $1")
            .bind(run_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                ProjectsError::Validation(
                    "Actions workflow run attribution was not found.".to_owned(),
                )
            })?;
    let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
    if !permission
        .as_ref()
        .is_some_and(|permission| permission.role.can_write())
    {
        return Err(ProjectsError::Forbidden);
    }
    Ok(())
}

async fn project_automation_input_for_item(
    pool: &PgPool,
    item_id: Uuid,
    actor_user_id: Uuid,
    event: ProjectAutomationEvent,
) -> Result<Option<ProjectAutomationInput>, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(issues.repository_id, pull_issues.repository_id, pull_requests.base_repository_id) AS repository_id,
               project_items.issue_id,
               project_items.pull_request_id
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN issues pull_issues ON pull_issues.id = pull_requests.issue_id
        WHERE project_items.id = $1
          AND project_items.archived_at IS NULL
          AND project_items.item_type IN ('issue', 'pull_request')
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|row| {
        row.get::<Option<Uuid>, _>("repository_id")
            .map(|repository_id| ProjectAutomationInput {
                actor_user_id,
                repository_id,
                issue_id: row.get("issue_id"),
                pull_request_id: row.get("pull_request_id"),
                event,
            })
    }))
}

async fn project_automation_items(
    pool: &PgPool,
    input: &ProjectAutomationInput,
) -> Result<Vec<ProjectAutomationItem>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_items.project_id,
               project_items.id AS item_id,
               project_items.item_type,
               project_items.issue_id,
               project_items.pull_request_id,
               COALESCE(issues.repository_id, pull_issues.repository_id, pull_requests.base_repository_id) AS repository_id
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN issues pull_issues ON pull_issues.id = pull_requests.issue_id
        WHERE project_items.archived_at IS NULL
          AND (
            ($1::uuid IS NOT NULL AND project_items.issue_id = $1)
            OR ($2::uuid IS NOT NULL AND project_items.pull_request_id = $2)
          )
        ORDER BY project_items.created_at
        "#,
    )
    .bind(input.issue_id)
    .bind(input.pull_request_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            row.get::<Option<Uuid>, _>("repository_id")
                .map(|repository_id| ProjectAutomationItem {
                    project_id: row.get("project_id"),
                    item_id: row.get("item_id"),
                    item_type: row.get("item_type"),
                    issue_id: row.get("issue_id"),
                    pull_request_id: row.get("pull_request_id"),
                    repository_id,
                })
        })
        .filter(|item| item.repository_id == input.repository_id)
        .collect())
}

async fn run_project_item_workflows(
    pool: &PgPool,
    item: &ProjectAutomationItem,
    input: &ProjectAutomationInput,
) -> Result<(), ProjectsError> {
    let workflows = sqlx::query(
        r#"
        SELECT project_workflows.id,
               project_workflows.workflow_key,
               project_workflows.trigger_event,
               project_workflows.configuration,
               EXISTS (
                 SELECT 1 FROM project_workflow_repository_targets targets
                 WHERE targets.project_workflow_id = project_workflows.id
               ) AS has_repository_targets,
               EXISTS (
                 SELECT 1 FROM project_workflow_repository_targets targets
                 WHERE targets.project_workflow_id = project_workflows.id
                   AND targets.repository_id = $3
               ) AS repository_target_matches
        FROM project_workflows
        WHERE project_workflows.project_id = $1
          AND project_workflows.enabled = true
          AND project_workflows.trigger_event = ANY($2)
        ORDER BY project_workflows.created_at
        "#,
    )
    .bind(item.project_id)
    .bind(input.event.workflow_events())
    .bind(input.repository_id)
    .fetch_all(pool)
    .await?;

    for workflow in workflows {
        let workflow_id: Uuid = workflow.get("id");
        let workflow_key: String = workflow.get("workflow_key");
        let trigger_event: String = workflow.get("trigger_event");
        let configuration: Value = workflow.get("configuration");
        let has_repository_targets: bool = workflow.get("has_repository_targets");
        let repository_target_matches: bool = workflow.get("repository_target_matches");
        let idempotency_key = format!("{}:{}:{}", workflow_id, item.item_id, input.event.as_str());

        if workflow_log_exists(pool, item.project_id, &idempotency_key).await? {
            continue;
        }
        if has_repository_targets && !repository_target_matches {
            record_workflow_execution(
                pool,
                &WorkflowExecutionRecord {
                    project_id: item.project_id,
                    workflow_id: Some(workflow_id),
                    item_id: Some(item.item_id),
                    actor_user_id: Some(input.actor_user_id),
                    source: "system",
                    event_type: input.event.as_str(),
                    status: "skipped",
                    message: "Linked repository is outside this workflow target list.",
                    metadata: json!({
                        "workflowKey": workflow_key,
                        "idempotencyKey": idempotency_key,
                        "repositoryId": input.repository_id,
                    }),
                },
            )
            .await?;
            continue;
        }
        if !workflow_condition_matches(&configuration, input.event, item) {
            record_workflow_execution(
                pool,
                &WorkflowExecutionRecord {
                    project_id: item.project_id,
                    workflow_id: Some(workflow_id),
                    item_id: Some(item.item_id),
                    actor_user_id: Some(input.actor_user_id),
                    source: "system",
                    event_type: input.event.as_str(),
                    status: "skipped",
                    message: "Workflow condition did not match this item event.",
                    metadata: json!({
                        "workflowKey": workflow_key,
                        "idempotencyKey": idempotency_key,
                        "condition": configuration.get("condition").cloned().unwrap_or(Value::Null),
                    }),
                },
            )
            .await?;
            continue;
        }

        let Some((field_id, value)) = workflow_target_value(pool, &configuration).await? else {
            record_workflow_execution(
                pool,
                &WorkflowExecutionRecord {
                    project_id: item.project_id,
                    workflow_id: Some(workflow_id),
                    item_id: Some(item.item_id),
                    actor_user_id: Some(input.actor_user_id),
                    source: "system",
                    event_type: input.event.as_str(),
                    status: "skipped",
                    message: "Workflow has no eligible target field and option configured.",
                    metadata: json!({
                        "workflowKey": workflow_key,
                        "idempotencyKey": idempotency_key,
                    }),
                },
            )
            .await?;
            continue;
        };

        sqlx::query(
            r#"
            INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_item_id, project_field_id)
            DO UPDATE SET value = EXCLUDED.value,
                          updated_by_user_id = EXCLUDED.updated_by_user_id,
                          updated_at = now()
            "#,
        )
        .bind(item.item_id)
        .bind(field_id)
        .bind(&value)
        .bind(input.actor_user_id)
        .execute(pool)
        .await?;
        record_project_item_event(
            pool,
            item.project_id,
            item.item_id,
            input.actor_user_id,
            "project.workflow.execute",
            json!({
                "workflowId": workflow_id,
                "workflowKey": workflow_key,
                "triggerEvent": trigger_event,
                "event": input.event.as_str(),
                "fieldId": field_id,
                "value": value,
                "actor": "@opengithub-project-automation",
            }),
        )
        .await?;
        record_workflow_execution(
            pool,
            &WorkflowExecutionRecord {
                project_id: item.project_id,
                workflow_id: Some(workflow_id),
                item_id: Some(item.item_id),
                actor_user_id: Some(input.actor_user_id),
                source: "system",
                event_type: input.event.as_str(),
                status: "success",
                message: "Project workflow updated the item.",
                metadata: json!({
                    "workflowKey": workflow_key,
                    "idempotencyKey": idempotency_key,
                    "fieldId": field_id,
                    "value": value,
                    "itemType": item.item_type,
                }),
            },
        )
        .await?;
        sqlx::query(
            r#"
            UPDATE project_workflows
            SET last_run_at = now(),
                last_run_status = 'success',
                last_run_message = 'Project workflow updated the item.',
                source = 'system',
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(workflow_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

fn workflow_condition_matches(
    configuration: &Value,
    event: ProjectAutomationEvent,
    item: &ProjectAutomationItem,
) -> bool {
    let condition = configuration
        .get("condition")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    if condition.contains("is:merged") && event != ProjectAutomationEvent::PullRequestMerged {
        return false;
    }
    if condition.contains("state:closed") && event.state() != "closed" {
        return false;
    }
    if condition.contains("state:open") && event.state() != "open" {
        return false;
    }
    if condition.contains("is:pr") && item.pull_request_id.is_none() {
        return false;
    }
    if condition.contains("is:issue") && item.issue_id.is_none() {
        return false;
    }
    true
}

async fn workflow_target_value(
    pool: &PgPool,
    configuration: &Value,
) -> Result<Option<(Uuid, Value)>, ProjectsError> {
    let Some(target) = configuration.get("target") else {
        return Ok(None);
    };
    let Some(field_id) = target
        .get("fieldId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        return Ok(None);
    };
    let Some(option_id) = target
        .get("optionId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        return Ok(None);
    };
    let option_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM project_field_options WHERE id = $1 AND project_field_id = $2",
    )
    .bind(option_id)
    .bind(field_id)
    .fetch_optional(pool)
    .await?;
    Ok(option_name.map(|name| (field_id, json!(name))))
}

async fn run_project_auto_archive(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT id, configuration
        FROM project_workflows
        WHERE project_id = $1
          AND enabled = true
          AND trigger_event = 'archive_completed'
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let workflow_id: Uuid = row.get("id");
        let configuration: Value = row.get("configuration");
        let days = configuration
            .get("archiveAfterDays")
            .and_then(Value::as_i64)
            .unwrap_or(14)
            .clamp(1, 365);
        let Some((field_id, value)) = default_done_status_target(pool, project_id)
            .await?
            .get("fieldId")
            .and_then(Value::as_str)
            .and_then(|field| Uuid::parse_str(field).ok())
            .map(|field_id| (field_id, json!("Done")))
        else {
            continue;
        };
        let candidates = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT project_items.id
            FROM project_items
            JOIN project_item_field_values values
              ON values.project_item_id = project_items.id
             AND values.project_field_id = $2
             AND lower(values.value #>> '{}') = lower($3)
            WHERE project_items.project_id = $1
              AND project_items.archived_at IS NULL
              AND values.updated_at <= now() - ($4::int * interval '1 day')
            ORDER BY values.updated_at
            LIMIT 25
            "#,
        )
        .bind(project_id)
        .bind(field_id)
        .bind(value.as_str().unwrap_or("Done"))
        .bind(days as i32)
        .fetch_all(pool)
        .await?;
        for item_id in candidates {
            let idempotency_key = format!("{}:{}:archive_completed", workflow_id, item_id);
            if workflow_log_exists(pool, project_id, &idempotency_key).await? {
                continue;
            }
            sqlx::query(
                r#"
                UPDATE project_items
                SET archived_at = now(),
                    archived_by_user_id = $2,
                    restored_at = NULL,
                    restored_by_user_id = NULL,
                    updated_at = now()
                WHERE id = $1 AND archived_at IS NULL
                "#,
            )
            .bind(item_id)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
            record_project_item_event(
                pool,
                project_id,
                item_id,
                actor_user_id,
                "project.workflow.archive",
                json!({
                    "workflowId": workflow_id,
                    "archiveAfterDays": days,
                    "actor": "@opengithub-project-automation",
                }),
            )
            .await?;
            record_workflow_execution(
                pool,
                &WorkflowExecutionRecord {
                    project_id,
                    workflow_id: Some(workflow_id),
                    item_id: Some(item_id),
                    actor_user_id: Some(actor_user_id),
                    source: "system",
                    event_type: "archive_completed",
                    status: "success",
                    message: "Project workflow archived a completed item.",
                    metadata: json!({
                        "idempotencyKey": idempotency_key,
                        "archiveAfterDays": days,
                    }),
                },
            )
            .await?;
        }
    }
    Ok(())
}

struct WorkflowExecutionRecord<'a> {
    project_id: Uuid,
    workflow_id: Option<Uuid>,
    item_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
    source: &'a str,
    event_type: &'a str,
    status: &'a str,
    message: &'a str,
    metadata: Value,
}

async fn record_workflow_execution(
    pool: &PgPool,
    record: &WorkflowExecutionRecord<'_>,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO workflow_execution_logs
          (project_id, project_workflow_id, project_item_id, actor_user_id, source, event_type, status, message, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(record.project_id)
    .bind(record.workflow_id)
    .bind(record.item_id)
    .bind(record.actor_user_id)
    .bind(record.source)
    .bind(record.event_type)
    .bind(record.status)
    .bind(record.message)
    .bind(&record.metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn workflow_log_exists(
    pool: &PgPool,
    project_id: Uuid,
    idempotency_key: &str,
) -> Result<bool, ProjectsError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM workflow_execution_logs
          WHERE project_id = $1
            AND metadata->>'idempotencyKey' = $2
        )
        "#,
    )
    .bind(project_id)
    .bind(idempotency_key)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn default_done_status_target(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Value, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT
          project_fields.id AS field_id,
          project_field_options.id AS option_id,
          project_field_options.name AS option_name
        FROM project_fields
        LEFT JOIN project_field_options
          ON project_field_options.project_field_id = project_fields.id
         AND lower(project_field_options.name) = 'done'
        WHERE project_fields.project_id = $1
          AND project_fields.deleted_at IS NULL
          AND project_fields.field_type IN ('status', 'single_select')
        ORDER BY
          CASE WHEN lower(project_fields.name) = 'status' THEN 0 ELSE 1 END,
          project_fields.position
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map_or(Value::Null, |row| {
        json!({
            "fieldId": row.get::<Uuid, _>("field_id"),
            "optionId": row.try_get::<Option<Uuid>, _>("option_id").ok().flatten(),
            "optionName": row.try_get::<Option<String>, _>("option_name").ok().flatten(),
            "missingOption": row
                .try_get::<Option<Uuid>, _>("option_id")
                .ok()
                .flatten()
                .is_none(),
        })
    }))
}

async fn project_workflow_definitions(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectWorkflowDefinition>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT id, workflow_key, name, description, enabled, trigger_event,
               configuration, actor_label, source, last_run_at,
               last_run_status, last_run_message, updated_at
        FROM project_workflows
        WHERE project_id = $1
        ORDER BY
          CASE workflow_key
            WHEN 'closed-item-to-done' THEN 1
            WHEN 'merged-pr-to-done' THEN 2
            WHEN 'item-added-default-status' THEN 3
            WHEN 'auto-archive-completed-items' THEN 4
            ELSE 100
          END,
          lower(name)
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let workflow_ids = rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let rules = project_workflow_rules(pool, &workflow_ids).await?;
    let targets = project_workflow_target_ids(pool, &workflow_ids).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            ProjectWorkflowDefinition {
                id,
                workflow_key: row.get("workflow_key"),
                name: row.get("name"),
                description: row
                    .try_get::<Option<String>, _>("description")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                enabled: row.get("enabled"),
                trigger_event: row.get("trigger_event"),
                configuration: row.get("configuration"),
                rules: rules
                    .iter()
                    .filter(|(workflow_id, _)| *workflow_id == id)
                    .map(|(_, rule)| rule.clone())
                    .collect(),
                repository_target_ids: targets
                    .iter()
                    .filter(|(workflow_id, _)| *workflow_id == id)
                    .map(|(_, repository_id)| *repository_id)
                    .collect(),
                actor_label: row.get("actor_label"),
                source: row.get("source"),
                last_run_at: row.get("last_run_at"),
                last_run_status: row.get("last_run_status"),
                last_run_message: row.get("last_run_message"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn project_workflow_rules(
    pool: &PgPool,
    workflow_ids: &[Uuid],
) -> Result<Vec<(Uuid, ProjectWorkflowRule)>, ProjectsError> {
    if workflow_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_workflow_id, id, rule_type, configuration, position
        FROM project_workflow_rules
        WHERE project_workflow_id = ANY($1)
        ORDER BY project_workflow_id, position, created_at
        "#,
    )
    .bind(workflow_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("project_workflow_id"),
                ProjectWorkflowRule {
                    id: row.get("id"),
                    rule_type: row.get("rule_type"),
                    configuration: row.get("configuration"),
                    position: i64::from(row.get::<i32, _>("position")),
                },
            )
        })
        .collect())
}

async fn project_workflow_target_ids(
    pool: &PgPool,
    workflow_ids: &[Uuid],
) -> Result<Vec<(Uuid, Uuid)>, ProjectsError> {
    if workflow_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_workflow_id, repository_id
        FROM project_workflow_repository_targets
        WHERE project_workflow_id = ANY($1)
        ORDER BY created_at
        "#,
    )
    .bind(workflow_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("project_workflow_id"), row.get("repository_id")))
        .collect())
}

async fn project_workflow_eligible_fields(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectWorkflowEligibleField>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, field_type
        FROM project_fields
        WHERE project_id = $1
          AND deleted_at IS NULL
          AND field_type IN ('status', 'single_select', 'date')
        ORDER BY position, created_at
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    let field_ids = rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let options = field_settings_options(pool, &field_ids).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let field_type: String = row.get("field_type");
            ProjectWorkflowEligibleField {
                id,
                name: row.get("name"),
                field_type: field_type.clone(),
                options: options
                    .iter()
                    .filter(|(field_id, _)| *field_id == id)
                    .map(|(_, option)| option.clone())
                    .collect(),
                supports_status_target: matches!(field_type.as_str(), "status" | "single_select"),
                supports_archive_criteria: true,
            }
        })
        .collect())
}

async fn project_workflow_repository_targets(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<ProjectWorkflowRepositoryTarget>, ProjectsError> {
    let Some(viewer_user_id) = viewer_user_id else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
          repositories.id,
          repositories.name,
          repositories.visibility,
          COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login
        FROM (
          SELECT default_repository_id AS repository_id
          FROM projects
          WHERE id = $1 AND default_repository_id IS NOT NULL
          UNION
          SELECT repository_id
          FROM project_repositories
          WHERE project_id = $1
        ) linked
        JOIN repositories ON repositories.id = linked.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        ORDER BY owner_login, repositories.name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let mut targets = Vec::new();
    for row in rows {
        let repository_id: Uuid = row.get("id");
        let permission =
            repository_permission_for_user(pool, repository_id, viewer_user_id).await?;
        if let Some(permission) = permission {
            let owner = row
                .try_get::<Option<String>, _>("owner_login")?
                .unwrap_or_else(|| "unknown".to_owned());
            let name: String = row.get("name");
            targets.push(ProjectWorkflowRepositoryTarget {
                id: repository_id,
                owner: owner.clone(),
                name: name.clone(),
                full_name: format!("{owner}/{name}"),
                href: format!("/{owner}/{name}"),
                visibility: row.get("visibility"),
                permission: permission.role.as_str().to_owned(),
            });
        }
    }
    Ok(targets)
}

async fn validate_project_workflow_status_target(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    option_id: Uuid,
) -> Result<(), ProjectsError> {
    let valid = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM project_fields
        JOIN project_field_options
          ON project_field_options.project_field_id = project_fields.id
         AND project_field_options.id = $3
        WHERE project_fields.project_id = $1
          AND project_fields.id = $2
          AND project_fields.deleted_at IS NULL
          AND project_fields.field_type IN ('status', 'single_select')
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .bind(option_id)
    .fetch_one(pool)
    .await?;
    if valid == 0 {
        return Err(ProjectsError::Validation(
            "Workflow target must use an eligible status field and option.".to_owned(),
        ));
    }
    Ok(())
}

async fn project_workflow_execution_logs(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectWorkflowExecutionLog>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT
          workflow_execution_logs.id,
          workflow_execution_logs.project_workflow_id,
          project_workflows.workflow_key,
          workflow_execution_logs.project_item_id,
          workflow_execution_logs.source,
          workflow_execution_logs.event_type,
          workflow_execution_logs.status,
          workflow_execution_logs.message,
          workflow_execution_logs.metadata,
          workflow_execution_logs.created_at,
          users.id AS actor_id,
          COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
          users.avatar_url AS actor_avatar_url
        FROM workflow_execution_logs
        LEFT JOIN project_workflows
          ON project_workflows.id = workflow_execution_logs.project_workflow_id
        LEFT JOIN users ON users.id = workflow_execution_logs.actor_user_id
        WHERE workflow_execution_logs.project_id = $1
        ORDER BY workflow_execution_logs.created_at DESC
        LIMIT 20
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let actor_id: Option<Uuid> = row.get("actor_id");
            ProjectWorkflowExecutionLog {
                id: row.get("id"),
                workflow_id: row.get("project_workflow_id"),
                workflow_key: row.get("workflow_key"),
                item_id: row.get("project_item_id"),
                actor: actor_id.map(|id| ProjectWorkspaceUser {
                    id,
                    login: row
                        .try_get::<Option<String>, _>("actor_login")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "unknown".to_owned()),
                    avatar_url: row.get("actor_avatar_url"),
                }),
                source: row.get("source"),
                event_type: row.get("event_type"),
                status: row.get("status"),
                message: row.get("message"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            }
        })
        .collect())
}

struct ProjectFieldAdminTarget {
    name: String,
    field_type: String,
    settings: Value,
    updated_at: DateTime<Utc>,
}

struct ProjectOptionAdminTarget {
    name: String,
}

async fn iteration_admin_target(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectFieldAdminTarget, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }
    let field = project_field_admin_target(pool, project_id, field_id).await?;
    if field.field_type != "iteration" {
        return Err(ProjectsError::Validation(
            "Iterations can only be managed on iteration fields.".to_owned(),
        ));
    }
    Ok(field)
}

async fn project_field_admin_target(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
) -> Result<ProjectFieldAdminTarget, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT name, field_type, settings, updated_at
        FROM project_fields
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ProjectsError::InvalidFilter("field must reference a project field".to_owned())
    })?;
    Ok(ProjectFieldAdminTarget {
        name: row.get("name"),
        field_type: row.get("field_type"),
        settings: row.get("settings"),
        updated_at: row.get("updated_at"),
    })
}

async fn option_admin_target(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectFieldAdminTarget, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    let can_manage = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);
    if !can_manage {
        return Err(ProjectsError::Forbidden);
    }
    let field = project_field_admin_target(pool, project_id, field_id).await?;
    if !matches!(field.field_type.as_str(), "single_select" | "status") {
        return Err(ProjectsError::Validation(
            "Options can only be managed on single-select fields.".to_owned(),
        ));
    }
    Ok(field)
}

async fn project_option_admin_target(
    pool: &PgPool,
    field_id: Uuid,
    option_id: Uuid,
) -> Result<ProjectOptionAdminTarget, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT name
        FROM project_field_options
        WHERE project_field_id = $1 AND id = $2
        "#,
    )
    .bind(field_id)
    .bind(option_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ProjectsError::InvalidFilter("option must reference a project field option".to_owned())
    })?;
    Ok(ProjectOptionAdminTarget {
        name: row.get("name"),
    })
}

fn normalize_project_field_name(input: &str) -> Result<String, ProjectsError> {
    let name = input.trim().chars().take(80).collect::<String>();
    if name.is_empty() {
        return Err(ProjectsError::Validation(
            "Project field name is required.".to_owned(),
        ));
    }
    Ok(name)
}

fn normalize_custom_project_field_type(input: &str) -> Result<String, ProjectsError> {
    let field_type = input.trim().to_ascii_lowercase();
    if matches!(
        field_type.as_str(),
        "single_select" | "iteration" | "date" | "text" | "number"
    ) {
        Ok(field_type)
    } else {
        Err(ProjectsError::Validation(
            "Project field type must be single_select, iteration, date, text, or number."
                .to_owned(),
        ))
    }
}

fn normalize_project_option_name(input: &str) -> Result<String, ProjectsError> {
    let name = input.trim().chars().take(80).collect::<String>();
    if name.is_empty() {
        return Err(ProjectsError::Validation(
            "Project option name is required.".to_owned(),
        ));
    }
    Ok(name)
}

fn normalize_project_option_color(input: Option<&str>) -> Result<String, ProjectsError> {
    let color = input.unwrap_or("gray").trim().to_ascii_lowercase();
    if matches!(
        color.as_str(),
        "gray" | "red" | "orange" | "yellow" | "green" | "blue" | "purple" | "pink"
    ) {
        Ok(color)
    } else {
        Err(ProjectsError::Validation(
            "Project option color must be gray, red, orange, yellow, green, blue, purple, or pink."
                .to_owned(),
        ))
    }
}

fn normalize_project_option_description(input: Option<&str>) -> Option<String> {
    input.and_then(|value| {
        let description = value.trim().chars().take(200).collect::<String>();
        if description.is_empty() {
            None
        } else {
            Some(description)
        }
    })
}

fn normalize_iteration_duration(duration: i64, unit: &str) -> Result<i64, ProjectsError> {
    let unit = unit.trim().to_ascii_lowercase();
    let duration_days = match unit.as_str() {
        "days" => duration,
        "weeks" => duration * 7,
        _ => {
            return Err(ProjectsError::Validation(
                "Iteration duration unit must be days or weeks.".to_owned(),
            ));
        }
    };
    normalize_iteration_duration_days(duration_days)?;
    Ok(duration_days)
}

fn normalize_iteration_duration_days(duration_days: i64) -> Result<(), ProjectsError> {
    if !(1..=365).contains(&duration_days) {
        return Err(ProjectsError::Validation(
            "Iteration duration must be between 1 and 365 days.".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_iteration_name(input: String) -> Result<String, ProjectsError> {
    let name = input.trim().chars().take(80).collect::<String>();
    if name.is_empty() {
        return Err(ProjectsError::Validation(
            "Iteration name is required.".to_owned(),
        ));
    }
    Ok(name)
}

fn default_project_field_settings(field_type: &str) -> Value {
    match field_type {
        "iteration" => json!({ "durationUnit": "weeks", "duration": 2 }),
        "number" => json!({ "format": "number" }),
        _ => json!({}),
    }
}

async fn seed_default_project_iterations(
    pool: &PgPool,
    field_id: Uuid,
    start_date: NaiveDate,
    duration_days: i64,
    count: i64,
) -> Result<(), ProjectsError> {
    normalize_iteration_duration_days(duration_days)?;
    for index in 0..count {
        let starts = start_date + Duration::days(duration_days * index);
        sqlx::query(
            r#"
            INSERT INTO project_iterations (project_field_id, name, start_date, duration_days, position)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(field_id)
        .bind(format!("Iteration {}", index + 1))
        .bind(starts)
        .bind(duration_days as i32)
        .bind((index + 1) as i32)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn next_iteration_position(pool: &PgPool, field_id: Uuid) -> Result<i64, ProjectsError> {
    let position: Option<i32> = sqlx::query_scalar(
        "SELECT max(position) FROM project_iterations WHERE project_field_id = $1",
    )
    .bind(field_id)
    .fetch_one(pool)
    .await?;
    Ok(i64::from(position.unwrap_or(0)) + 1)
}

async fn next_iteration_start_date(
    pool: &PgPool,
    field_id: Uuid,
    duration_days: i64,
) -> Result<NaiveDate, ProjectsError> {
    let start: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT max(start_date + duration_days) FROM project_iterations WHERE project_field_id = $1",
    )
    .bind(field_id)
    .fetch_one(pool)
    .await?;
    Ok(start.unwrap_or_else(|| Utc::now().date_naive()) + Duration::days(duration_days))
}

async fn ensure_iteration_exists(
    pool: &PgPool,
    field_id: Uuid,
    iteration_id: Uuid,
) -> Result<(), ProjectsError> {
    let exists: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM project_iterations WHERE project_field_id = $1 AND id = $2",
    )
    .bind(field_id)
    .bind(iteration_id)
    .fetch_optional(pool)
    .await?;
    if exists.is_none() {
        return Err(ProjectsError::NotFound);
    }
    Ok(())
}

async fn ensure_iteration_range_available(
    pool: &PgPool,
    field_id: Uuid,
    current_iteration_id: Option<Uuid>,
    start_date: NaiveDate,
    duration_days: i64,
) -> Result<(), ProjectsError> {
    normalize_iteration_duration_days(duration_days)?;
    let end_date = start_date + Duration::days(duration_days);
    let overlaps: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM (
          SELECT id, start_date, duration_days FROM project_iterations WHERE project_field_id = $1
          UNION ALL
          SELECT id, start_date, duration_days FROM project_iteration_breaks WHERE project_field_id = $1
        ) ranges
        WHERE ($4::uuid IS NULL OR ranges.id <> $4)
          AND ranges.start_date < $3
          AND (ranges.start_date + ranges.duration_days) > $2
        "#,
    )
    .bind(field_id)
    .bind(start_date)
    .bind(end_date)
    .bind(current_iteration_id)
    .fetch_one(pool)
    .await?;
    if overlaps > 0 {
        return Err(ProjectsError::Validation(
            "Iteration ranges and breaks cannot overlap.".to_owned(),
        ));
    }
    Ok(())
}

async fn audit_project_iteration_change(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    target_id: Uuid,
    metadata: Value,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'project_iteration', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_id.to_string())
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_unique_project_field_name(
    pool: &PgPool,
    project_id: Uuid,
    current_field_id: Option<Uuid>,
    name: &str,
) -> Result<(), ProjectsError> {
    let duplicate: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM project_fields
        WHERE project_id = $1
          AND deleted_at IS NULL
          AND lower(name) = lower($2)
          AND ($3::uuid IS NULL OR id <> $3)
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .bind(name)
    .bind(current_field_id)
    .fetch_optional(pool)
    .await?;
    if duplicate.is_some() {
        return Err(ProjectsError::Validation(
            "A project field with that name already exists.".to_owned(),
        ));
    }
    Ok(())
}

async fn ensure_unique_project_option_name(
    pool: &PgPool,
    field_id: Uuid,
    current_option_id: Option<Uuid>,
    name: &str,
) -> Result<(), ProjectsError> {
    let duplicate: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM project_field_options
        WHERE project_field_id = $1
          AND lower(name) = lower($2)
          AND ($3::uuid IS NULL OR id <> $3)
        LIMIT 1
        "#,
    )
    .bind(field_id)
    .bind(name)
    .bind(current_option_id)
    .fetch_optional(pool)
    .await?;
    if duplicate.is_some() {
        return Err(ProjectsError::Validation(
            "A project option with that name already exists.".to_owned(),
        ));
    }
    Ok(())
}

async fn next_project_option_position(pool: &PgPool, field_id: Uuid) -> Result<i64, ProjectsError> {
    let max_position: Option<i32> = sqlx::query_scalar(
        "SELECT max(position) FROM project_field_options WHERE project_field_id = $1",
    )
    .bind(field_id)
    .fetch_one(pool)
    .await?;
    Ok(i64::from(max_position.unwrap_or(0)) + 1)
}

async fn touch_project_field(
    pool: &PgPool,
    project_id: Uuid,
    field_id: Uuid,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        UPDATE project_fields
        SET cache_version = cache_version + 1, updated_at = now()
        WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(field_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn audit_project_settings_change(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    project_id: Uuid,
    metadata: Value,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'project', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(project_id.to_string())
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn audit_project_option_change(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    target_id: Uuid,
    metadata: Value,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'project_field_option', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_id.to_string())
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn invalidate_project_view_caches(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<(), ProjectsError> {
    sqlx::query("UPDATE project_views SET updated_at = now() WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn is_builtin_project_field(field_type: &str) -> bool {
    matches!(
        field_type,
        "title" | "assignees" | "labels" | "milestone" | "repository"
    )
}

fn workspace_layout_choices(
    selected_view: &ProjectWorkspaceView,
    can_edit: bool,
    fields: &[ProjectWorkspaceField],
) -> Vec<ProjectWorkspaceLayoutChoice> {
    let board_ready = fields
        .iter()
        .any(|field| is_board_column_field(&field.field_type));
    let roadmap_ready = fields
        .iter()
        .any(|field| is_roadmap_date_field(&field.field_type));
    [
        ("table", "Table", "t", true, None),
        (
            "board",
            "Board",
            "b",
            board_ready,
            (!board_ready).then(|| {
                "Add a status, single-select, or iteration field before using Board layout."
                    .to_owned()
            }),
        ),
        (
            "roadmap",
            "Roadmap",
            "r",
            roadmap_ready,
            (!roadmap_ready)
                .then(|| "Add a date or iteration field before using Roadmap layout.".to_owned()),
        ),
    ]
    .into_iter()
    .map(
        |(layout, label, keyboard_hint, has_required_fields, unavailable_reason)| {
            ProjectWorkspaceLayoutChoice {
                layout: layout.to_owned(),
                label: label.to_owned(),
                keyboard_hint: keyboard_hint.to_owned(),
                active: selected_view.layout == layout,
                enabled: can_edit && has_required_fields,
                unavailable_reason: if can_edit {
                    unavailable_reason
                } else {
                    Some("Write access is required to change this view layout.".to_owned())
                },
            }
        },
    )
    .collect()
}

async fn workspace_board_config(
    pool: &PgPool,
    selected_view: &ProjectWorkspaceView,
    fields: &[ProjectWorkspaceField],
    items: &[ProjectWorkspaceItem],
) -> Result<ProjectWorkspaceBoardConfig, ProjectsError> {
    let eligible_column_fields = fields
        .iter()
        .filter(|field| is_board_column_field(&field.field_type))
        .map(layout_field_from_workspace_field)
        .collect::<Vec<_>>();
    let eligible_swimlane_fields = fields
        .iter()
        .filter(|field| is_board_swimlane_field(&field.field_type))
        .map(layout_field_from_workspace_field)
        .collect::<Vec<_>>();
    let configured_column_id = configuration_uuid(&selected_view.configuration, "columnFieldId");
    let configured_column_name = configuration_string(&selected_view.configuration, "columnField");
    let column_field = configured_column_id
        .and_then(|id| fields.iter().find(|field| field.id == id))
        .or_else(|| {
            configured_column_name.as_deref().and_then(|name| {
                fields
                    .iter()
                    .find(|field| field.name.eq_ignore_ascii_case(name))
            })
        })
        .or_else(|| {
            fields
                .iter()
                .find(|field| is_board_column_field(&field.field_type))
        });
    let configured_swimlane_id =
        configuration_uuid(&selected_view.configuration, "swimlaneFieldId");
    let configured_swimlane_name =
        configuration_string(&selected_view.configuration, "swimlaneField");
    let swimlane_field = configured_swimlane_id
        .and_then(|id| fields.iter().find(|field| field.id == id))
        .or_else(|| {
            configured_swimlane_name.as_deref().and_then(|name| {
                fields
                    .iter()
                    .find(|field| field.name.eq_ignore_ascii_case(name))
            })
        });

    let mut columns = if let Some(field) = column_field {
        workspace_board_columns_from_settings(pool, selected_view.id, field, items).await?
    } else {
        Vec::new()
    };
    if let Some(field) = column_field {
        let mut dynamic = workspace_board_columns_from_items(field, items);
        for column in dynamic.drain(..) {
            if !columns.iter().any(|existing| existing.key == column.key) {
                columns.push(column);
            }
        }
    }
    if columns.is_empty() {
        if let Some(field) = column_field {
            columns.push(ProjectWorkspaceBoardColumn {
                key: "no-value".to_owned(),
                label: "No value".to_owned(),
                field_id: field.id,
                count: items.len() as i64,
                item_limit: None,
                over_limit: false,
                visible: true,
            });
        }
    }

    Ok(ProjectWorkspaceBoardConfig {
        column_field: column_field.map(layout_field_from_workspace_field),
        swimlane_field: swimlane_field.map(layout_field_from_workspace_field),
        eligible_column_fields,
        eligible_swimlane_fields,
        columns,
        empty_columns_visible: selected_view
            .configuration
            .get("emptyColumnsVisible")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        unavailable_reason: column_field
            .is_none()
            .then(|| "Board layout needs a status, single-select, or iteration field.".to_owned()),
    })
}

async fn workspace_board_columns_from_settings(
    pool: &PgPool,
    view_id: Uuid,
    field: &ProjectWorkspaceField,
    items: &[ProjectWorkspaceItem],
) -> Result<Vec<ProjectWorkspaceBoardColumn>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT option_key, label, item_limit, visible
        FROM project_board_column_settings
        WHERE project_view_id = $1 AND project_field_id = $2
        ORDER BY position, created_at
        "#,
    )
    .bind(view_id)
    .bind(field.id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let key: String = row.get("option_key");
            let item_limit: Option<i32> = row.get("item_limit");
            let count = count_items_for_field_value(items, field, &key);
            let limit = item_limit.map(i64::from);
            ProjectWorkspaceBoardColumn {
                key,
                label: row.get("label"),
                field_id: field.id,
                count,
                item_limit: limit,
                over_limit: limit.is_some_and(|limit| count > limit),
                visible: row.get("visible"),
            }
        })
        .collect())
}

fn workspace_board_columns_from_items(
    field: &ProjectWorkspaceField,
    items: &[ProjectWorkspaceItem],
) -> Vec<ProjectWorkspaceBoardColumn> {
    let mut counts = std::collections::BTreeMap::<String, i64>::new();
    for item in items {
        let key = display_field_for_item(item, field);
        *counts.entry(key).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(key, count)| ProjectWorkspaceBoardColumn {
            label: key.clone(),
            key,
            field_id: field.id,
            count,
            item_limit: None,
            over_limit: false,
            visible: true,
        })
        .collect()
}

async fn workspace_roadmap_config(
    pool: &PgPool,
    selected_view: &ProjectWorkspaceView,
    fields: &[ProjectWorkspaceField],
) -> Result<ProjectWorkspaceRoadmapConfig, ProjectsError> {
    let eligible_date_fields = fields
        .iter()
        .filter(|field| is_roadmap_date_field(&field.field_type))
        .map(layout_field_from_workspace_field)
        .collect::<Vec<_>>();
    let eligible_marker_fields = fields
        .iter()
        .filter(|field| is_roadmap_marker_field(&field.field_type))
        .map(layout_field_from_workspace_field)
        .collect::<Vec<_>>();

    let row = sqlx::query(
        r#"
        SELECT start_field_id, target_field_id, marker_field_ids, zoom
        FROM project_roadmap_settings
        WHERE project_view_id = $1
        "#,
    )
    .bind(selected_view.id)
    .fetch_optional(pool)
    .await?;
    let start_id = row
        .as_ref()
        .and_then(|row| row.get::<Option<Uuid>, _>("start_field_id"))
        .or_else(|| configuration_uuid(&selected_view.configuration, "startFieldId"));
    let target_id = row
        .as_ref()
        .and_then(|row| row.get::<Option<Uuid>, _>("target_field_id"))
        .or_else(|| configuration_uuid(&selected_view.configuration, "targetFieldId"));
    let marker_ids = row
        .as_ref()
        .map(|row| row.get::<Vec<Uuid>, _>("marker_field_ids"))
        .filter(|ids| !ids.is_empty())
        .unwrap_or_else(|| {
            configuration_uuid_array(&selected_view.configuration, "markerFieldIds")
        });
    let zoom = row
        .as_ref()
        .map(|row| row.get::<String, _>("zoom"))
        .or_else(|| configuration_string(&selected_view.configuration, "zoom"))
        .filter(|value| matches!(value.as_str(), "month" | "quarter" | "year"))
        .unwrap_or_else(|| "month".to_owned());
    let first_date_field = fields
        .iter()
        .find(|field| is_roadmap_date_field(&field.field_type));
    let start_date_field = start_id
        .and_then(|id| fields.iter().find(|field| field.id == id))
        .or(first_date_field)
        .map(layout_field_from_workspace_field);
    let target_date_field = target_id
        .and_then(|id| fields.iter().find(|field| field.id == id))
        .or(first_date_field)
        .map(layout_field_from_workspace_field);
    let marker_fields = marker_ids
        .into_iter()
        .filter_map(|id| fields.iter().find(|field| field.id == id))
        .filter(|field| is_roadmap_marker_field(&field.field_type))
        .map(layout_field_from_workspace_field)
        .collect::<Vec<_>>();

    Ok(ProjectWorkspaceRoadmapConfig {
        start_date_field,
        target_date_field,
        marker_fields,
        eligible_date_fields,
        eligible_marker_fields,
        zoom,
        zoom_options: vec!["month".to_owned(), "quarter".to_owned(), "year".to_owned()],
        unavailable_reason: first_date_field
            .is_none()
            .then(|| "Roadmap layout needs at least one date or iteration field.".to_owned()),
    })
}

fn layout_field_from_workspace_field(field: &ProjectWorkspaceField) -> ProjectWorkspaceLayoutField {
    ProjectWorkspaceLayoutField {
        id: field.id,
        name: field.name.clone(),
        field_type: field.field_type.clone(),
    }
}

fn display_field_for_item(item: &ProjectWorkspaceItem, field: &ProjectWorkspaceField) -> String {
    item.field_values
        .iter()
        .find(|value| value.field_id == field.id)
        .map(|value| value.display_value.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "No value".to_owned())
}

fn count_items_for_field_value(
    items: &[ProjectWorkspaceItem],
    field: &ProjectWorkspaceField,
    key: &str,
) -> i64 {
    items
        .iter()
        .filter(|item| display_field_for_item(item, field) == key)
        .count() as i64
}

fn is_board_column_field(field_type: &str) -> bool {
    matches!(field_type, "status" | "single_select" | "iteration")
}

fn is_board_swimlane_field(field_type: &str) -> bool {
    matches!(
        field_type,
        "status" | "single_select" | "iteration" | "repository" | "assignees" | "milestone"
    )
}

fn is_roadmap_date_field(field_type: &str) -> bool {
    matches!(field_type, "date" | "iteration")
}

fn is_roadmap_marker_field(field_type: &str) -> bool {
    matches!(field_type, "date" | "iteration" | "milestone")
}

fn configuration_uuid(configuration: &Value, key: &str) -> Option<Uuid> {
    configuration
        .get(key)
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn configuration_uuid_array(configuration: &Value, key: &str) -> Vec<Uuid> {
    configuration
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(|value| Uuid::parse_str(value).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_workspace_filters(
    query: ProjectWorkspaceQuery<'_>,
    selected_view: &ProjectWorkspaceView,
    fields: &[ProjectWorkspaceField],
) -> Result<ProjectWorkspaceFilters, ProjectsError> {
    let pagination = normalize_pagination(query.page, query.page_size);
    let configured_sort = selected_view
        .configuration
        .get("sort")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    let sort = query
        .sort
        .unwrap_or(configured_sort)
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        sort.as_str(),
        "manual" | "updated_desc" | "updated_asc" | "title_asc" | "title_desc"
    ) {
        return Err(ProjectsError::InvalidFilter(
            "sort must be manual, updated_desc, updated_asc, title_asc, or title_desc".to_owned(),
        ));
    }
    let query_text = query
        .query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(300).collect::<String>())
        .or_else(|| configuration_string(&selected_view.configuration, "query"));
    let tokens = query_text
        .as_deref()
        .map(|value| {
            value
                .split_whitespace()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let group = query
        .group
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| configuration_string(&selected_view.configuration, "group"));
    let slice = query
        .slice
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| configuration_string(&selected_view.configuration, "slice"));
    let group = normalize_field_selector(group.as_deref(), fields, "group")?;
    let slice = normalize_field_selector(slice.as_deref(), fields, "slice")?;
    Ok(ProjectWorkspaceFilters {
        query: query_text,
        sort,
        group,
        slice,
        tokens,
        page: pagination.page,
        page_size: pagination.page_size,
    })
}

#[derive(Debug)]
struct ValidProjectViewState {
    query: Option<String>,
    sort: String,
    group: Option<String>,
    slice: Option<String>,
    hidden_field_ids: Vec<String>,
}

#[derive(Debug)]
struct ValidProjectViewLayout {
    layout: String,
    column_field_id: Option<Uuid>,
    swimlane_field_id: Option<Uuid>,
    start_field_id: Option<Uuid>,
    target_field_id: Option<Uuid>,
}

#[derive(Debug)]
struct ValidProjectRoadmapSettings {
    start_field_id: Uuid,
    target_field_id: Uuid,
    marker_field_ids: Vec<Uuid>,
    zoom: String,
}

fn validate_project_view_layout_request(
    request: &ProjectViewLayoutRequest,
    fields: &[ProjectWorkspaceField],
) -> Result<ValidProjectViewLayout, ProjectsError> {
    let layout = request.layout.trim().to_ascii_lowercase();
    if !matches!(layout.as_str(), "table" | "board" | "roadmap") {
        return Err(ProjectsError::InvalidFilter(
            "layout must be table, board, or roadmap".to_owned(),
        ));
    }

    let column_field_id = if layout == "board" {
        Some(validate_layout_field_id(
            request.column_field_id,
            fields,
            "columnFieldId",
            is_board_column_field,
        )?)
    } else {
        None
    };
    let swimlane_field_id = if layout == "board" {
        request
            .swimlane_field_id
            .map(|id| {
                validate_layout_field_id(
                    Some(id),
                    fields,
                    "swimlaneFieldId",
                    is_board_swimlane_field,
                )
            })
            .transpose()?
    } else {
        None
    };
    let start_field_id = if layout == "roadmap" {
        Some(validate_layout_field_id(
            request.start_field_id,
            fields,
            "startFieldId",
            is_roadmap_date_field,
        )?)
    } else {
        None
    };
    let target_field_id = if layout == "roadmap" {
        Some(validate_layout_field_id(
            request.target_field_id.or(start_field_id),
            fields,
            "targetFieldId",
            is_roadmap_date_field,
        )?)
    } else {
        None
    };

    Ok(ValidProjectViewLayout {
        layout,
        column_field_id,
        swimlane_field_id,
        start_field_id,
        target_field_id,
    })
}

fn validate_project_roadmap_settings_request(
    request: &ProjectRoadmapSettingsRequest,
    fields: &[ProjectWorkspaceField],
) -> Result<ValidProjectRoadmapSettings, ProjectsError> {
    let start_field_id = validate_layout_field_id(
        Some(request.start_field_id),
        fields,
        "startFieldId",
        is_roadmap_date_field,
    )?;
    let target_field_id = validate_layout_field_id(
        Some(request.target_field_id),
        fields,
        "targetFieldId",
        is_roadmap_date_field,
    )?;
    let mut marker_field_ids = Vec::new();
    for id in &request.marker_field_ids {
        let marker_id =
            validate_layout_field_id(Some(*id), fields, "markerFieldIds", is_roadmap_marker_field)?;
        if !marker_field_ids.contains(&marker_id) {
            marker_field_ids.push(marker_id);
        }
    }
    let zoom = request.zoom.trim().to_ascii_lowercase();
    if !matches!(zoom.as_str(), "month" | "quarter" | "year") {
        return Err(ProjectsError::InvalidFilter(
            "zoom must be month, quarter, or year".to_owned(),
        ));
    }

    Ok(ValidProjectRoadmapSettings {
        start_field_id,
        target_field_id,
        marker_field_ids,
        zoom,
    })
}

fn validate_layout_field_id(
    requested: Option<Uuid>,
    fields: &[ProjectWorkspaceField],
    name: &str,
    compatible: fn(&str) -> bool,
) -> Result<Uuid, ProjectsError> {
    let field = if let Some(requested) = requested {
        fields.iter().find(|field| field.id == requested)
    } else {
        fields.iter().find(|field| compatible(&field.field_type))
    }
    .ok_or_else(|| {
        ProjectsError::InvalidFilter(format!("{name} must reference a compatible project field"))
    })?;
    if !compatible(&field.field_type) {
        return Err(ProjectsError::InvalidFilter(format!(
            "{name} must reference a compatible project field"
        )));
    }
    Ok(field.id)
}

fn validate_project_view_state_request(
    request: &ProjectViewStateRequest,
    fields: &[ProjectWorkspaceField],
) -> Result<ValidProjectViewState, ProjectsError> {
    let query = request
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(validate_workspace_query)
        .transpose()?;
    let sort = request
        .sort
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual")
        .to_ascii_lowercase();
    if !matches!(
        sort.as_str(),
        "manual" | "updated_desc" | "updated_asc" | "title_asc" | "title_desc"
    ) {
        return Err(ProjectsError::InvalidFilter(
            "sort must be manual, updated_desc, updated_asc, title_asc, or title_desc".to_owned(),
        ));
    }

    let group = normalize_state_field_selector(request.group.as_deref(), fields, "group")?;
    let slice = normalize_state_field_selector(request.slice.as_deref(), fields, "slice")?;
    let mut hidden_field_ids = Vec::new();
    for id in &request.hidden_field_ids {
        if !fields.iter().any(|field| field.id == *id) {
            return Err(ProjectsError::InvalidFilter(
                "hiddenFieldIds must reference project fields".to_owned(),
            ));
        }
        if !hidden_field_ids.contains(&id.to_string()) {
            hidden_field_ids.push(id.to_string());
        }
    }

    Ok(ValidProjectViewState {
        query,
        sort,
        group,
        slice,
        hidden_field_ids,
    })
}

fn validate_workspace_query(value: &str) -> Result<String, ProjectsError> {
    let query = value.chars().take(300).collect::<String>();
    for token in query.split_whitespace() {
        let valid = matches!(
            token,
            "is:open"
                | "is:closed"
                | "is:issue"
                | "is:pr"
                | "is:draft"
                | "assignee:@me"
                | "no:assignee"
                | "no:label"
        ) || token.starts_with("repo:")
            || token.starts_with("assignee:")
            || token.starts_with("label:")
            || token.contains(':')
                && token.split_once(':').is_some_and(|(field, value)| {
                    !field.trim().is_empty()
                        && !value.trim().is_empty()
                        && field
                            .chars()
                            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
                })
            || !token.contains(':');
        if !valid {
            return Err(ProjectsError::InvalidFilter(format!(
                "unsupported project filter token: {token}"
            )));
        }
    }
    Ok(query)
}

fn normalize_state_field_selector(
    value: Option<&str>,
    fields: &[ProjectWorkspaceField],
    name: &str,
) -> Result<Option<String>, ProjectsError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let normalized = value.to_ascii_lowercase();
    let field = fields
        .iter()
        .find(|field| {
            field.id.to_string() == value || field.name.to_ascii_lowercase() == normalized
        })
        .ok_or_else(|| {
            ProjectsError::InvalidFilter(format!("{name} must reference a project field"))
        })?;
    if matches!(field.field_type.as_str(), "text" | "number") && name != "slice" {
        return Err(ProjectsError::InvalidFilter(format!(
            "{name} field must be status, single_select, iteration, date, repository, or assignee"
        )));
    }
    Ok(Some(field.name.clone()))
}

fn normalize_field_selector(
    value: Option<&str>,
    fields: &[ProjectWorkspaceField],
    name: &str,
) -> Result<Option<String>, ProjectsError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let normalized = value.to_ascii_lowercase();
    let found = fields.iter().any(|field| {
        field.id.to_string() == value || field.name.to_ascii_lowercase() == normalized
    });
    if !found {
        return Err(ProjectsError::InvalidFilter(format!(
            "{name} must reference a visible project field"
        )));
    }
    Ok(Some(value.to_owned()))
}

fn workspace_unsaved_state(
    filters: &ProjectWorkspaceFilters,
    selected_view: &ProjectWorkspaceView,
) -> ProjectWorkspaceUnsavedState {
    let configured_query = configuration_string(&selected_view.configuration, "query");
    let configured_sort = selected_view
        .configuration
        .get("sort")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    let configured_group = configuration_string(&selected_view.configuration, "group");
    let configured_slice = configuration_string(&selected_view.configuration, "slice");
    let mut reasons = Vec::new();
    if filters.query != configured_query {
        reasons.push("filter".to_owned());
    }
    if filters.sort != configured_sort {
        reasons.push("sort".to_owned());
    }
    if filters.group != configured_group {
        reasons.push("group".to_owned());
    }
    if filters.slice != configured_slice {
        reasons.push("slice".to_owned());
    }
    ProjectWorkspaceUnsavedState {
        active: !reasons.is_empty(),
        reasons,
    }
}

fn configuration_string(configuration: &Value, key: &str) -> Option<String> {
    configuration
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Debug)]
struct ProjectItemDetailRow {
    id: Uuid,
    item_type: String,
    search_title: String,
    archived_at: Option<DateTime<Utc>>,
    archived_by: Option<ProjectWorkspaceUser>,
    row: sqlx::postgres::PgRow,
}

#[derive(Debug)]
struct ProjectArchivedItemFilters {
    item_type: Option<String>,
    query: Option<String>,
    page: i64,
    page_size: i64,
}

async fn visible_workspace_project(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProjectWorkspaceProject, ProjectsError> {
    let project = workspace_project_row(pool, project_id, viewer_user_id).await?;
    if project.visibility != "public" && project.viewer_role.is_none() {
        return if viewer_user_id.is_some() {
            Err(ProjectsError::Forbidden)
        } else {
            Err(ProjectsError::NotFound)
        };
    }
    Ok(project)
}

async fn workspace_fields_for_detail(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectWorkspaceField>, ProjectsError> {
    let views = workspace_views(pool, project_id, "project", 1).await?;
    let selected_view = views.first().ok_or(ProjectsError::NotFound)?;
    workspace_fields(pool, project_id, selected_view).await
}

fn normalize_archived_item_filters(
    query: ProjectItemsArchivedQuery<'_>,
) -> Result<ProjectArchivedItemFilters, ProjectsError> {
    let item_type = query
        .item_type
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
        .map(ToOwned::to_owned);
    if let Some(item_type) = item_type.as_deref() {
        if !matches!(item_type, "draft_issue" | "issue" | "pull_request") {
            return Err(ProjectsError::InvalidFilter(
                "Archived item type must be draft_issue, issue, or pull_request.".to_owned(),
            ));
        }
    }
    let pagination = normalize_pagination(query.page, query.page_size);
    Ok(ProjectArchivedItemFilters {
        item_type,
        query: query
            .query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        page: pagination.page,
        page_size: pagination.page_size,
    })
}

async fn project_items_for_detail(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
    item_id: Option<Uuid>,
    archived_only: bool,
) -> Result<Vec<ProjectItemDetailRow>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT
          project_items.id, project_items.item_type, project_items.title AS draft_title,
          project_items.body AS draft_body, project_items.position::text AS position_text,
          project_items.updated_at, project_items.issue_id, project_items.pull_request_id,
          project_items.archived_at, project_items.restored_at, project_items.source_synced_at,
          project_items.source_sync_version,
          archived_by.id AS archived_by_id,
          COALESCE(NULLIF(archived_by.username, ''), archived_by.email) AS archived_by_login,
          archived_by.avatar_url AS archived_by_avatar_url,
          issues.title AS issue_title, issues.body AS issue_body, issues.state AS issue_state,
          issues.number AS issue_number, COALESCE(issue_repositories.id, pull_repositories.id) AS issue_repository_id,
          COALESCE(
            NULLIF(issue_owner_user.username, ''),
            issue_owner_user.email,
            issue_owner_org.slug,
            NULLIF(pull_owner_user.username, ''),
            pull_owner_user.email,
            pull_owner_org.slug
          ) AS issue_owner,
          COALESCE(issue_repositories.name, pull_repositories.name) AS issue_repository_name,
          pull_requests.title AS pull_title, pull_requests.body AS pull_body, pull_requests.state AS pull_state,
          pull_requests.number AS pull_number
        FROM project_items
        LEFT JOIN users archived_by ON archived_by.id = project_items.archived_by_user_id
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN repositories issue_repositories ON issue_repositories.id = issues.repository_id
        LEFT JOIN users issue_owner_user ON issue_owner_user.id = issue_repositories.owner_user_id
        LEFT JOIN organizations issue_owner_org ON issue_owner_org.id = issue_repositories.owner_organization_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN repositories pull_repositories ON pull_repositories.id = pull_requests.repository_id
        LEFT JOIN users pull_owner_user ON pull_owner_user.id = pull_repositories.owner_user_id
        LEFT JOIN organizations pull_owner_org ON pull_owner_org.id = pull_repositories.owner_organization_id
        WHERE project_items.project_id = $1
          AND ($2::uuid IS NULL OR project_items.id = $2)
          AND (($3::bool = true AND project_items.archived_at IS NOT NULL)
            OR ($3::bool = false AND project_items.archived_at IS NULL))
          AND (
            COALESCE(issue_repositories.id, pull_repositories.id) IS NULL
            OR COALESCE(issue_repositories.visibility, pull_repositories.visibility) = 'public'
            OR COALESCE(issue_repositories.owner_user_id, pull_repositories.owner_user_id) = $4
            OR EXISTS (
              SELECT 1 FROM repository_permissions
              WHERE repository_permissions.repository_id = COALESCE(issue_repositories.id, pull_repositories.id)
                AND repository_permissions.user_id = $4
            )
            OR EXISTS (
              SELECT 1 FROM organization_memberships
              WHERE organization_memberships.organization_id = COALESCE(issue_repositories.owner_organization_id, pull_repositories.owner_organization_id)
                AND organization_memberships.user_id = $4
            )
          )
        ORDER BY project_items.archived_at DESC NULLS LAST, project_items.position, project_items.created_at
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(archived_only)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let id = row.get("id");
            let item_type: String = row.get("item_type");
            let search_title = row
                .get::<Option<String>, _>("issue_title")
                .or_else(|| row.get::<Option<String>, _>("pull_title"))
                .or_else(|| row.get::<Option<String>, _>("draft_title"))
                .unwrap_or_else(|| "Untitled item".to_owned());
            let archived_by = row
                .get::<Option<Uuid>, _>("archived_by_id")
                .zip(row.get::<Option<String>, _>("archived_by_login"))
                .map(|(id, login)| ProjectWorkspaceUser {
                    id,
                    login,
                    avatar_url: row.get("archived_by_avatar_url"),
                });
            Ok(ProjectItemDetailRow {
                id,
                item_type,
                search_title,
                archived_at: row.get("archived_at"),
                archived_by,
                row,
            })
        })
        .collect()
}

async fn project_item_source(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Option<ProjectItemSourceSummary>, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT
          project_items.item_type,
          project_items.source_synced_at,
          project_items.source_sync_version,
          COALESCE(issues.id, pull_requests.id) AS source_id,
          COALESCE(issues.number, pull_requests.number) AS source_number,
          COALESCE(issues.title, pull_requests.title) AS source_title,
          COALESCE(issues.state, pull_requests.state) AS source_state,
          COALESCE(issues.updated_at, pull_requests.updated_at) AS source_updated_at,
          repositories.id AS repository_id,
          COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS repository_owner,
          repositories.name AS repository_name
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN repositories ON repositories.id = COALESCE(issues.repository_id, pull_requests.repository_id)
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        WHERE project_items.id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;
    let Some(source_id) = row.get::<Option<Uuid>, _>("source_id") else {
        return Ok(None);
    };
    let Some(repository_id) = row.get::<Option<Uuid>, _>("repository_id") else {
        return Ok(None);
    };
    let owner = row
        .get::<Option<String>, _>("repository_owner")
        .unwrap_or_else(|| "unknown".to_owned());
    let name = row
        .get::<Option<String>, _>("repository_name")
        .unwrap_or_else(|| "unknown".to_owned());
    let item_type: String = row.get("item_type");
    let number: i64 = row.get("source_number");
    let segment = if item_type == "pull_request" {
        "pull"
    } else {
        "issues"
    };
    Ok(Some(ProjectItemSourceSummary {
        source_type: item_type,
        id: source_id,
        number,
        title: row.get("source_title"),
        state: row.get("source_state"),
        href: format!("/{owner}/{name}/{segment}/{number}"),
        repository: ProjectRepositoryScopeSummary {
            id: repository_id,
            owner: owner.clone(),
            name: name.clone(),
            full_name: format!("{owner}/{name}"),
            href: format!("/{owner}/{name}"),
        },
        updated_at: row.get("source_updated_at"),
        synced_at: row.get("source_synced_at"),
        sync_version: row.get("source_sync_version"),
    }))
}

async fn project_item_archive_state(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<ProjectItemArchiveState, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT project_items.archived_at, project_items.restored_at,
          archived_by.id AS archived_by_id,
          COALESCE(NULLIF(archived_by.username, ''), archived_by.email) AS archived_by_login,
          archived_by.avatar_url AS archived_by_avatar_url,
          restored_by.id AS restored_by_id,
          COALESCE(NULLIF(restored_by.username, ''), restored_by.email) AS restored_by_login,
          restored_by.avatar_url AS restored_by_avatar_url
        FROM project_items
        LEFT JOIN users archived_by ON archived_by.id = project_items.archived_by_user_id
        LEFT JOIN users restored_by ON restored_by.id = project_items.restored_by_user_id
        WHERE project_items.id = $1
        "#,
    )
    .bind(item_id)
    .fetch_one(pool)
    .await?;
    let archived_at: Option<DateTime<Utc>> = row.get("archived_at");
    Ok(ProjectItemArchiveState {
        archived: archived_at.is_some(),
        archived_at,
        archived_by: row
            .get::<Option<Uuid>, _>("archived_by_id")
            .zip(row.get::<Option<String>, _>("archived_by_login"))
            .map(|(id, login)| ProjectWorkspaceUser {
                id,
                login,
                avatar_url: row.get("archived_by_avatar_url"),
            }),
        restored_at: row.get("restored_at"),
        restored_by: row
            .get::<Option<Uuid>, _>("restored_by_id")
            .zip(row.get::<Option<String>, _>("restored_by_login"))
            .map(|(id, login)| ProjectWorkspaceUser {
                id,
                login,
                avatar_url: row.get("restored_by_avatar_url"),
            }),
    })
}

async fn project_item_activity(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ProjectItemActivity>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_item_events.id, project_item_events.event_type,
          project_item_events.metadata, project_item_events.created_at,
          users.id AS actor_id,
          COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
          users.avatar_url AS actor_avatar_url
        FROM project_item_events
        LEFT JOIN users ON users.id = project_item_events.actor_user_id
        WHERE project_item_events.project_item_id = $1
        ORDER BY project_item_events.created_at, project_item_events.id
        "#,
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(ProjectItemActivity {
                id: row.get("id"),
                event_type: row.get("event_type"),
                actor: row
                    .get::<Option<Uuid>, _>("actor_id")
                    .zip(row.get::<Option<String>, _>("actor_login"))
                    .map(|(id, login)| ProjectWorkspaceUser {
                        id,
                        login,
                        avatar_url: row.get("actor_avatar_url"),
                    }),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            })
        })
        .collect()
}

async fn project_item_comments(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ProjectItemComment>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT project_item_comments.id, project_item_comments.body,
          project_item_comments.is_deleted, project_item_comments.created_at,
          project_item_comments.updated_at, users.id AS author_id,
          COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
          users.avatar_url AS author_avatar_url
        FROM project_item_comments
        JOIN users ON users.id = project_item_comments.author_user_id
        WHERE project_item_comments.project_item_id = $1
        ORDER BY project_item_comments.created_at, project_item_comments.id
        "#,
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(ProjectItemComment {
                id: row.get("id"),
                author: ProjectWorkspaceUser {
                    id: row.get("author_id"),
                    login: row.get("author_login"),
                    avatar_url: row.get("author_avatar_url"),
                },
                body: row.get("body"),
                is_deleted: row.get("is_deleted"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .collect()
}

fn project_item_permissions(
    viewer_user_id: Option<Uuid>,
    viewer_role: Option<String>,
    can_edit: bool,
    is_draft: bool,
    archived: bool,
) -> ProjectItemDetailPermissions {
    ProjectItemDetailPermissions {
        authenticated: viewer_user_id.is_some(),
        viewer_role,
        can_edit: can_edit && !archived,
        can_comment: can_edit && !archived,
        can_convert: can_edit && is_draft && !archived,
        can_archive: can_edit && !archived,
        can_restore: can_edit && archived,
        can_remove: can_edit,
    }
}

async fn workspace_items(
    pool: &PgPool,
    project_id: Uuid,
    viewer_user_id: Option<Uuid>,
    fields: &[ProjectWorkspaceField],
) -> Result<Vec<ProjectWorkspaceItem>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT
          project_items.id, project_items.item_type, project_items.title AS draft_title,
          project_items.body AS draft_body, project_items.position::text AS position_text,
          project_items.updated_at, project_items.issue_id, project_items.pull_request_id,
          issues.title AS issue_title, issues.body AS issue_body, issues.state AS issue_state,
          issues.number AS issue_number, issue_repositories.id AS issue_repository_id,
          COALESCE(NULLIF(issue_owner_user.username, ''), issue_owner_user.email, issue_owner_org.slug) AS issue_owner,
          issue_repositories.name AS issue_repository_name,
          pull_requests.title AS pull_title, pull_requests.state AS pull_state,
          pull_requests.number AS pull_number
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN repositories issue_repositories ON issue_repositories.id = issues.repository_id
        LEFT JOIN users issue_owner_user ON issue_owner_user.id = issue_repositories.owner_user_id
        LEFT JOIN organizations issue_owner_org ON issue_owner_org.id = issue_repositories.owner_organization_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        WHERE project_items.project_id = $1
          AND project_items.archived_at IS NULL
          AND (
            issue_repositories.id IS NULL
            OR issue_repositories.visibility = 'public'
            OR issue_repositories.owner_user_id = $2
            OR EXISTS (
              SELECT 1 FROM repository_permissions
              WHERE repository_permissions.repository_id = issue_repositories.id
                AND repository_permissions.user_id = $2
            )
            OR EXISTS (
              SELECT 1 FROM organization_memberships
              WHERE organization_memberships.organization_id = issue_repositories.owner_organization_id
                AND organization_memberships.user_id = $2
            )
          )
        ORDER BY project_items.position, project_items.created_at
        "#,
    )
    .bind(project_id)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;
    let item_ids = rows.iter().map(|row| row.get("id")).collect::<Vec<Uuid>>();
    let values = workspace_field_values(pool, &item_ids).await?;
    let labels = workspace_labels(pool, &item_ids).await?;
    let assignees = workspace_assignees(pool, &item_ids).await?;

    rows.into_iter()
        .map(|row| workspace_item_from_row(row, fields, &values, &labels, &assignees))
        .collect::<Result<Vec<_>, _>>()
}

async fn workspace_field_values(
    pool: &PgPool,
    item_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<(Uuid, Value)>>, ProjectsError> {
    if item_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query(
        "SELECT project_item_id, project_field_id, value FROM project_item_field_values WHERE project_item_id = ANY($1)",
    )
    .bind(item_ids)
    .fetch_all(pool)
    .await?;
    let mut values = std::collections::HashMap::<Uuid, Vec<(Uuid, Value)>>::new();
    for row in rows {
        values
            .entry(row.get("project_item_id"))
            .or_default()
            .push((row.get("project_field_id"), row.get("value")));
    }
    Ok(values)
}

async fn workspace_labels(
    pool: &PgPool,
    item_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<ProjectWorkspaceLabel>>, ProjectsError> {
    if item_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_items.id AS item_id, labels.id, labels.name, labels.color
        FROM project_items
        JOIN issues ON issues.id = project_items.issue_id
        JOIN issue_labels ON issue_labels.issue_id = issues.id
        JOIN labels ON labels.id = issue_labels.label_id
        WHERE project_items.id = ANY($1)
        ORDER BY labels.name
        "#,
    )
    .bind(item_ids)
    .fetch_all(pool)
    .await?;
    let mut labels = std::collections::HashMap::<Uuid, Vec<ProjectWorkspaceLabel>>::new();
    for row in rows {
        labels
            .entry(row.get("item_id"))
            .or_default()
            .push(ProjectWorkspaceLabel {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
            });
    }
    Ok(labels)
}

async fn workspace_assignees(
    pool: &PgPool,
    item_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<ProjectWorkspaceUser>>, ProjectsError> {
    if item_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT project_items.id AS item_id, users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM project_items
        JOIN issues ON issues.id = project_items.issue_id
        JOIN issue_assignees ON issue_assignees.issue_id = issues.id
        JOIN users ON users.id = issue_assignees.user_id
        WHERE project_items.id = ANY($1)
        ORDER BY login
        "#,
    )
    .bind(item_ids)
    .fetch_all(pool)
    .await?;
    let mut assignees = std::collections::HashMap::<Uuid, Vec<ProjectWorkspaceUser>>::new();
    for row in rows {
        assignees
            .entry(row.get("item_id"))
            .or_default()
            .push(ProjectWorkspaceUser {
                id: row.get("id"),
                login: row.get("login"),
                avatar_url: row.get("avatar_url"),
            });
    }
    Ok(assignees)
}

fn workspace_item_from_row(
    row: sqlx::postgres::PgRow,
    fields: &[ProjectWorkspaceField],
    values: &std::collections::HashMap<Uuid, Vec<(Uuid, Value)>>,
    labels: &std::collections::HashMap<Uuid, Vec<ProjectWorkspaceLabel>>,
    assignees: &std::collections::HashMap<Uuid, Vec<ProjectWorkspaceUser>>,
) -> Result<ProjectWorkspaceItem, ProjectsError> {
    let id: Uuid = row.get("id");
    let item_type: String = row.get("item_type");
    let issue_number: Option<i64> = row.get("issue_number");
    let pull_number: Option<i64> = row.get("pull_number");
    let repo_owner: Option<String> = row.get("issue_owner");
    let repo_name: Option<String> = row.get("issue_repository_name");
    let repository = row
        .get::<Option<Uuid>, _>("issue_repository_id")
        .zip(repo_owner.clone())
        .zip(repo_name.clone())
        .map(|((repo_id, owner), name)| ProjectRepositoryScopeSummary {
            id: repo_id,
            owner: owner.clone(),
            name: name.clone(),
            full_name: format!("{owner}/{name}"),
            href: format!("/{owner}/{name}"),
        });
    let title = match item_type.as_str() {
        "issue" => row.get::<Option<String>, _>("issue_title"),
        "pull_request" => row
            .get::<Option<String>, _>("pull_title")
            .or_else(|| row.get::<Option<String>, _>("issue_title")),
        _ => row.get::<Option<String>, _>("draft_title"),
    }
    .unwrap_or_else(|| "Untitled item".to_owned());
    let state = match item_type.as_str() {
        "issue" => row.get("issue_state"),
        "pull_request" => row
            .get::<Option<String>, _>("pull_state")
            .or_else(|| row.get("issue_state")),
        _ => Some("draft".to_owned()),
    };
    let number = pull_number.or(issue_number);
    let href = repository.as_ref().and_then(|repository| {
        number.map(|number| {
            let segment = if item_type == "pull_request" {
                "pull"
            } else {
                "issues"
            };
            format!("{}/{segment}/{number}", repository.href)
        })
    });
    let explicit_values = values.get(&id).cloned().unwrap_or_default();
    let mut field_values = Vec::new();
    for field in fields {
        if let Some((_, value)) = explicit_values
            .iter()
            .find(|(field_id, _)| *field_id == field.id)
        {
            field_values.push(ProjectWorkspaceFieldValue {
                field_id: field.id,
                value: value.clone(),
                display_value: display_field_value(value),
            });
        } else if let Some(value) =
            intrinsic_field_value(field, &title, &state, repository.as_ref())
        {
            field_values.push(ProjectWorkspaceFieldValue {
                field_id: field.id,
                display_value: display_field_value(&value),
                value,
            });
        }
    }
    Ok(ProjectWorkspaceItem {
        id,
        item_type,
        position: row.get("position_text"),
        title,
        body: row
            .get::<Option<String>, _>("draft_body")
            .or_else(|| row.get("issue_body")),
        state,
        number,
        href,
        repository,
        field_values,
        labels: labels.get(&id).cloned().unwrap_or_default(),
        assignees: assignees.get(&id).cloned().unwrap_or_default(),
        updated_at: row.get("updated_at"),
    })
}

fn intrinsic_field_value(
    field: &ProjectWorkspaceField,
    title: &str,
    state: &Option<String>,
    repository: Option<&ProjectRepositoryScopeSummary>,
) -> Option<Value> {
    match field.field_type.as_str() {
        "title" => Some(json!(title)),
        "status" => Some(json!(state.as_deref().unwrap_or("draft"))),
        "repository" => repository.map(|repository| json!(repository.full_name)),
        _ => None,
    }
}

fn display_field_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Array(values) => values
            .iter()
            .map(display_field_value)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => value.to_string(),
    }
}

#[derive(Debug, Clone)]
struct ProjectWorkspaceEditItem {
    id: Uuid,
    item_type: String,
    issue_id: Option<Uuid>,
    pull_request_id: Option<Uuid>,
    pull_request_issue_id: Option<Uuid>,
    repository_id: Option<Uuid>,
    archived_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
}

async fn workspace_item_edit_target(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
) -> Result<ProjectWorkspaceEditItem, ProjectsError> {
    let row = sqlx::query(
        r#"
        SELECT
          project_items.id,
          project_items.item_type,
          project_items.issue_id,
          project_items.pull_request_id,
          pull_requests.issue_id AS pull_request_issue_id,
          COALESCE(issues.repository_id, pull_issues.repository_id, pull_requests.base_repository_id) AS repository_id,
          project_items.archived_at,
          project_items.updated_at
        FROM project_items
        LEFT JOIN issues ON issues.id = project_items.issue_id
        LEFT JOIN pull_requests ON pull_requests.id = project_items.pull_request_id
        LEFT JOIN issues pull_issues ON pull_issues.id = pull_requests.issue_id
        WHERE project_items.project_id = $1 AND project_items.id = $2
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;

    Ok(ProjectWorkspaceEditItem {
        id: row.get("id"),
        item_type: row.get("item_type"),
        issue_id: row.get("issue_id"),
        pull_request_id: row.get("pull_request_id"),
        pull_request_issue_id: row.get("pull_request_issue_id"),
        repository_id: row.get("repository_id"),
        archived_at: row.get("archived_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn draft_item_edit_target(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    expected_updated_at: Option<DateTime<Utc>>,
) -> Result<ProjectWorkspaceEditItem, ProjectsError> {
    let item = workspace_item_edit_target(pool, project_id, item_id).await?;
    if item.item_type != "draft_issue" {
        return Err(ProjectsError::Validation(
            "Only draft project items can be edited from this panel".to_owned(),
        ));
    }
    if item.archived_at.is_some() {
        return Err(ProjectsError::Validation(
            "Archived project items cannot be edited".to_owned(),
        ));
    }
    if let Some(expected) = expected_updated_at {
        if item.updated_at != expected {
            return Err(ProjectsError::Validation(
                "Project item changed since it was loaded. Refresh before editing.".to_owned(),
            ));
        }
    }
    Ok(item)
}

fn normalize_draft_title(value: &str) -> Result<String, ProjectsError> {
    let title = value.trim();
    if title.is_empty() {
        return Err(ProjectsError::Validation(
            "Draft project items require a title".to_owned(),
        ));
    }
    if title.chars().count() > 256 {
        return Err(ProjectsError::Validation(
            "Draft project item title must be 256 characters or fewer".to_owned(),
        ));
    }
    Ok(title.to_owned())
}

fn normalize_draft_body(value: Option<&str>) -> Result<Option<String>, ProjectsError> {
    let body = value.map(str::trim).filter(|value| !value.is_empty());
    if let Some(body) = body {
        if body.chars().count() > 65_536 {
            return Err(ProjectsError::Validation(
                "Draft project item body must be 65536 characters or fewer".to_owned(),
            ));
        }
        return Ok(Some(body.to_owned()));
    }
    Ok(None)
}

fn normalize_project_item_comment_body(value: &str) -> Result<String, ProjectsError> {
    let body = value.trim();
    if body.is_empty() {
        return Err(ProjectsError::Validation(
            "Project item comments require a body".to_owned(),
        ));
    }
    if body.chars().count() > 65_536 {
        return Err(ProjectsError::Validation(
            "Project item comments must be 65536 characters or fewer".to_owned(),
        ));
    }
    Ok(body.to_owned())
}

async fn ensure_project_item_comment(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    comment_id: Uuid,
) -> Result<(), ProjectsError> {
    let exists: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM project_item_comments
        WHERE project_id = $1 AND project_item_id = $2 AND id = $3
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(comment_id)
    .fetch_optional(pool)
    .await?;
    exists.map(|_| ()).ok_or(ProjectsError::NotFound)
}

async fn writable_project_repositories(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Vec<ProjectConversionRepository>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT repositories.id,
          COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
          repositories.name
        FROM repositories
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        LEFT JOIN project_repositories ON project_repositories.repository_id = repositories.id
        LEFT JOIN projects ON projects.default_repository_id = repositories.id
        WHERE project_repositories.project_id = $1 OR projects.id = $1
        ORDER BY owner_login, repositories.name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    let mut repositories = Vec::new();
    for row in rows {
        let repository_id: Uuid = row.get("id");
        if !repository_permission_for_user(pool, repository_id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_write())
        {
            continue;
        }
        let owner = row.get::<String, _>("owner_login");
        let name = row.get::<String, _>("name");
        repositories.push(ProjectConversionRepository {
            id: repository_id,
            owner: owner.clone(),
            name: name.clone(),
            full_name: format!("{owner}/{name}"),
            href: format!("/{owner}/{name}"),
            labels: conversion_labels(pool, repository_id).await?,
            assignees: conversion_assignees(pool, repository_id).await?,
            milestones: conversion_milestones(pool, repository_id).await?,
        });
    }
    Ok(repositories)
}

async fn ensure_project_repository_write(
    pool: &PgPool,
    project_id: Uuid,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let linked: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM project_repositories
          WHERE project_id = $1 AND repository_id = $2
          UNION
          SELECT 1 FROM projects
          WHERE id = $1 AND default_repository_id = $2
        )
        "#,
    )
    .bind(project_id)
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    if !linked {
        return Err(ProjectsError::Validation(
            "Choose a repository linked to this project.".to_owned(),
        ));
    }
    let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
    if permission.is_some_and(|permission| permission.role.can_write()) {
        Ok(())
    } else {
        Err(ProjectsError::Forbidden)
    }
}

async fn ensure_repository_write_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
    if permission.is_some_and(|permission| permission.role.can_write()) {
        Ok(())
    } else {
        Err(ProjectsError::Forbidden)
    }
}

async fn ensure_repository_link_allowed_for_project(
    pool: &PgPool,
    project_id: Uuid,
    repository_id: Uuid,
) -> Result<(), ProjectsError> {
    let allowed: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1
          FROM projects
          JOIN repositories ON repositories.id = $2
          WHERE projects.id = $1
            AND (
              projects.owner_user_id = repositories.owner_user_id
              OR projects.owner_organization_id = repositories.owner_organization_id
            )
        )
        "#,
    )
    .bind(project_id)
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    if allowed {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "Repository must belong to the same owner as the project.".to_owned(),
        ))
    }
}

async fn conversion_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ProjectWorkspaceLabel>, ProjectsError> {
    let rows =
        sqlx::query("SELECT id, name, color FROM labels WHERE repository_id = $1 ORDER BY name")
            .bind(repository_id)
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectWorkspaceLabel {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
        })
        .collect())
}

async fn conversion_assignees(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ProjectWorkspaceUser>, ProjectsError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT users.id, COALESCE(NULLIF(users.username, ''), users.email) AS login,
          users.avatar_url
        FROM users
        JOIN repository_permissions ON repository_permissions.user_id = users.id
        WHERE repository_permissions.repository_id = $1
          AND repository_permissions.role IN ('write', 'maintain', 'admin')
        ORDER BY login
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectWorkspaceUser {
            id: row.get("id"),
            login: row.get("login"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

async fn conversion_milestones(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ProjectConversionMilestone>, ProjectsError> {
    let rows = sqlx::query(
        "SELECT id, title, state FROM milestones WHERE repository_id = $1 ORDER BY title",
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectConversionMilestone {
            id: row.get("id"),
            title: row.get("title"),
            state: row.get("state"),
        })
        .collect())
}

async fn validate_conversion_labels(
    pool: &PgPool,
    repository_id: Uuid,
    label_ids: &[Uuid],
) -> Result<(), ProjectsError> {
    if label_ids.is_empty() {
        return Ok(());
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM labels WHERE repository_id = $1 AND id = ANY($2)",
    )
    .bind(repository_id)
    .bind(label_ids)
    .fetch_one(pool)
    .await?;
    if count == label_ids.len() as i64 {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "One or more labels are unavailable for the selected repository.".to_owned(),
        ))
    }
}

async fn validate_conversion_assignees(
    pool: &PgPool,
    repository_id: Uuid,
    assignee_user_ids: &[Uuid],
) -> Result<(), ProjectsError> {
    for user_id in assignee_user_ids {
        let permission = repository_permission_for_user(pool, repository_id, *user_id).await?;
        if !permission.is_some_and(|permission| permission.role.can_read()) {
            return Err(ProjectsError::Validation(
                "One or more assignees cannot access the selected repository.".to_owned(),
            ));
        }
    }
    Ok(())
}

async fn validate_conversion_milestone(
    pool: &PgPool,
    repository_id: Uuid,
    milestone_id: Option<Uuid>,
) -> Result<(), ProjectsError> {
    let Some(milestone_id) = milestone_id else {
        return Ok(());
    };
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM milestones WHERE repository_id = $1 AND id = $2)",
    )
    .bind(repository_id)
    .bind(milestone_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ProjectsError::Validation(
            "Milestone is unavailable for the selected repository.".to_owned(),
        ))
    }
}

async fn next_issue_number(pool: &PgPool, repository_id: Uuid) -> Result<i64, ProjectsError> {
    sqlx::query_scalar("SELECT COALESCE(max(number), 0) + 1 FROM issues WHERE repository_id = $1")
        .bind(repository_id)
        .fetch_one(pool)
        .await
        .map_err(ProjectsError::from)
}

async fn writable_workspace_project(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectWorkspaceProject, ProjectsError> {
    let project = workspace_project_row(pool, project_id, Some(actor_user_id)).await?;
    if !project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role)
    {
        return Err(ProjectsError::Forbidden);
    }
    Ok(project)
}

async fn project_workspace_after_item_mutation(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
) -> Result<ProjectWorkspace, ProjectsError> {
    project_workspace(
        pool,
        project_id,
        Some(actor_user_id),
        ProjectWorkspaceQuery {
            view: None,
            query: None,
            sort: Some("manual"),
            group: None,
            slice: None,
            page: Some(1),
            page_size: None,
        },
    )
    .await
}

async fn create_project_item(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemAddRequest,
) -> Result<Uuid, ProjectsError> {
    let item_type = request
        .item_type
        .as_deref()
        .unwrap_or(if request.pull_request_id.is_some() {
            "pull_request"
        } else if request.issue_id.is_some() || request.url.is_some() {
            "issue"
        } else {
            "draft_issue"
        })
        .trim();
    match item_type {
        "draft_issue" => create_draft_project_item(pool, project_id, actor_user_id, request).await,
        "issue" | "pull_request" => {
            let linked =
                resolve_linked_project_item(pool, actor_user_id, item_type, &request).await?;
            create_linked_project_item(pool, project_id, actor_user_id, linked, request).await
        }
        _ => Err(ProjectsError::Validation(
            "Project item type must be draft_issue, issue, or pull_request".to_owned(),
        )),
    }
}

async fn create_draft_project_item(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    request: ProjectItemAddRequest,
) -> Result<Uuid, ProjectsError> {
    let title = request
        .title
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if title.is_empty() {
        return Err(ProjectsError::Validation(
            "Draft project items require a title".to_owned(),
        ));
    }
    if title.len() > 256 {
        return Err(ProjectsError::Validation(
            "Draft project item title must be 256 characters or fewer".to_owned(),
        ));
    }
    let position =
        next_project_item_position(pool, project_id, request.position_after_item_id, None).await?;
    let item_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_items (project_id, item_type, title, body, position)
        VALUES ($1, 'draft_issue', $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(title)
    .bind(
        request
            .body
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(position)
    .fetch_one(pool)
    .await?;
    record_project_item_event(
        pool,
        project_id,
        item_id,
        actor_user_id,
        "project.item.draft_create",
        json!({ "title": request.title }),
    )
    .await?;
    Ok(item_id)
}

#[derive(Debug, Clone, Copy)]
struct LinkedProjectItemTarget {
    item_type: &'static str,
    issue_id: Option<Uuid>,
    pull_request_id: Option<Uuid>,
    repository_id: Uuid,
}

async fn resolve_linked_project_item(
    pool: &PgPool,
    actor_user_id: Uuid,
    requested_type: &str,
    request: &ProjectItemAddRequest,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    if let Some(pull_request_id) = request.pull_request_id {
        return linked_pull_request_target(pool, actor_user_id, pull_request_id).await;
    }
    if let Some(issue_id) = request.issue_id {
        return linked_issue_target(pool, actor_user_id, issue_id).await;
    }
    if let Some(url) = request.url.as_deref() {
        return linked_target_from_url(pool, actor_user_id, requested_type, url).await;
    }
    Err(ProjectsError::Validation(
        "Linked project items require an issue, pull request, or URL".to_owned(),
    ))
}

async fn linked_target_from_url(
    pool: &PgPool,
    actor_user_id: Uuid,
    requested_type: &str,
    url: &str,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    let path = url
        .split('?')
        .next()
        .unwrap_or(url)
        .trim_end_matches('/')
        .trim();
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let window = parts
        .windows(4)
        .find(|parts| matches!(parts[2], "issues" | "pull"))
        .ok_or_else(|| {
            ProjectsError::Validation(
                "Paste a URL like /owner/repo/issues/1 or /owner/repo/pull/1".to_owned(),
            )
        })?;
    let owner = window[0];
    let repo = window[1];
    let kind = window[2];
    let number = window[3]
        .parse::<i64>()
        .map_err(|_| ProjectsError::Validation("Linked item number must be numeric".to_owned()))?;
    if requested_type == "pull_request" || kind == "pull" {
        linked_pull_request_target_by_number(pool, actor_user_id, owner, repo, number).await
    } else {
        linked_issue_target_by_number(pool, actor_user_id, owner, repo, number).await
    }
}

async fn linked_issue_target_by_number(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    number: i64,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await?
        .ok_or(ProjectsError::NotFound)?;
    ensure_repository_readable(pool, repository.id, actor_user_id).await?;
    let issue_id: Uuid =
        sqlx::query_scalar("SELECT id FROM issues WHERE repository_id = $1 AND number = $2")
            .bind(repository.id)
            .bind(number)
            .fetch_optional(pool)
            .await?
            .ok_or(ProjectsError::NotFound)?;
    linked_issue_target(pool, actor_user_id, issue_id).await
}

async fn linked_pull_request_target_by_number(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    number: i64,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await?
        .ok_or(ProjectsError::NotFound)?;
    ensure_repository_readable(pool, repository.id, actor_user_id).await?;
    let pull_request_id: Uuid =
        sqlx::query_scalar("SELECT id FROM pull_requests WHERE repository_id = $1 AND number = $2")
            .bind(repository.id)
            .bind(number)
            .fetch_optional(pool)
            .await?
            .ok_or(ProjectsError::NotFound)?;
    linked_pull_request_target(pool, actor_user_id, pull_request_id).await
}

async fn linked_issue_target(
    pool: &PgPool,
    actor_user_id: Uuid,
    issue_id: Uuid,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    let repository_id: Uuid = sqlx::query_scalar("SELECT repository_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ProjectsError::NotFound)?;
    ensure_repository_readable(pool, repository_id, actor_user_id).await?;
    Ok(LinkedProjectItemTarget {
        item_type: "issue",
        issue_id: Some(issue_id),
        pull_request_id: None,
        repository_id,
    })
}

async fn linked_pull_request_target(
    pool: &PgPool,
    actor_user_id: Uuid,
    pull_request_id: Uuid,
) -> Result<LinkedProjectItemTarget, ProjectsError> {
    let row = sqlx::query(
        "SELECT issue_id, COALESCE(base_repository_id, repository_id) AS repository_id FROM pull_requests WHERE id = $1",
    )
    .bind(pull_request_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)?;
    let repository_id: Uuid = row.get("repository_id");
    ensure_repository_readable(pool, repository_id, actor_user_id).await?;
    Ok(LinkedProjectItemTarget {
        item_type: "pull_request",
        issue_id: Some(row.get("issue_id")),
        pull_request_id: Some(pull_request_id),
        repository_id,
    })
}

async fn ensure_repository_readable(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let repository =
        sqlx::query("SELECT visibility, owner_user_id FROM repositories WHERE id = $1")
            .bind(repository_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ProjectsError::NotFound)?;
    let visibility: String = repository.get("visibility");
    let owner_user_id: Option<Uuid> = repository.get("owner_user_id");
    let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
    if visibility == "public"
        || owner_user_id == Some(actor_user_id)
        || permission.is_some_and(|permission| permission.role.can_read())
    {
        Ok(())
    } else {
        Err(ProjectsError::Forbidden)
    }
}

async fn create_linked_project_item(
    pool: &PgPool,
    project_id: Uuid,
    actor_user_id: Uuid,
    target: LinkedProjectItemTarget,
    request: ProjectItemAddRequest,
) -> Result<Uuid, ProjectsError> {
    let duplicate: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id FROM project_items
        WHERE project_id = $1
          AND archived_at IS NULL
          AND (
            ($2::uuid IS NOT NULL AND issue_id = $2)
            OR ($3::uuid IS NOT NULL AND pull_request_id = $3)
          )
        "#,
    )
    .bind(project_id)
    .bind(target.issue_id)
    .bind(target.pull_request_id)
    .fetch_optional(pool)
    .await?;
    if duplicate.is_some() {
        return Err(ProjectsError::Validation(
            "This issue or pull request is already in the project".to_owned(),
        ));
    }
    let position =
        next_project_item_position(pool, project_id, request.position_after_item_id, None).await?;
    let item_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO project_items (project_id, item_type, issue_id, pull_request_id, position)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(project_id)
    .bind(target.item_type)
    .bind(target.issue_id)
    .bind(target.pull_request_id)
    .bind(position)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO timeline_events (repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata)
        VALUES ($1, $2, $3, $4, 'project_item_added', $5)
        "#,
    )
    .bind(target.repository_id)
    .bind(target.issue_id)
    .bind(target.pull_request_id)
    .bind(actor_user_id)
    .bind(json!({ "projectId": project_id, "projectItemId": item_id }))
    .execute(pool)
    .await?;
    Ok(item_id)
}

async fn next_project_item_position(
    pool: &PgPool,
    project_id: Uuid,
    after_item_id: Option<Uuid>,
    before_item_id: Option<Uuid>,
) -> Result<f64, ProjectsError> {
    let before = if let Some(before_item_id) = before_item_id {
        Some(project_item_position(pool, project_id, before_item_id).await?)
    } else {
        None
    };
    let after = if let Some(after_item_id) = after_item_id {
        Some(project_item_position(pool, project_id, after_item_id).await?)
    } else {
        None
    };
    match (after, before) {
        (Some(after), Some(before)) if before > after => Ok((after + before) / 2.0),
        (Some(after), _) => Ok(after + 1.0),
        (_, Some(before)) if before > 1.0 => Ok(before / 2.0),
        (_, Some(before)) => Ok(before - 1.0),
        (None, None) => {
            let max: Option<f64> = sqlx::query_scalar(
                "SELECT max(position)::float8 FROM project_items WHERE project_id = $1 AND archived_at IS NULL",
            )
            .bind(project_id)
            .fetch_one(pool)
            .await?;
            Ok(max.unwrap_or(0.0) + 1.0)
        }
    }
}

async fn project_item_position(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
) -> Result<f64, ProjectsError> {
    sqlx::query_scalar(
        "SELECT position::float8 FROM project_items WHERE project_id = $1 AND id = $2 AND archived_at IS NULL",
    )
    .bind(project_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectsError::NotFound)
}

async fn record_project_item_event(
    pool: &PgPool,
    project_id: Uuid,
    item_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    metadata: Value,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO project_item_events (project_id, project_item_id, actor_user_id, event_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(project_id)
    .bind(item_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_project_audit(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    target_id: Uuid,
    metadata: Value,
) -> Result<(), ProjectsError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'project_item', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_id.to_string())
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

fn is_linked_native_field(field: &ProjectWorkspaceField) -> bool {
    matches!(
        field.field_type.as_str(),
        "title" | "status" | "assignees" | "labels" | "milestone"
    )
}

fn normalize_project_field_value(
    field: &ProjectWorkspaceField,
    value: &Value,
) -> Result<Value, ProjectsError> {
    match field.field_type.as_str() {
        "title" | "text" => {
            let text = value
                .as_str()
                .ok_or_else(|| ProjectsError::Validation(format!("{} must be text", field.name)))?
                .trim()
                .to_owned();
            if field.field_type == "title" && text.is_empty() {
                return Err(ProjectsError::Validation(
                    "Title cannot be blank".to_owned(),
                ));
            }
            if text.len() > 1024 {
                return Err(ProjectsError::Validation(format!(
                    "{} must be 1024 characters or fewer",
                    field.name
                )));
            }
            Ok(json!(text))
        }
        "number" => {
            let number = value.as_f64().ok_or_else(|| {
                ProjectsError::Validation(format!("{} must be a number", field.name))
            })?;
            if !number.is_finite() {
                return Err(ProjectsError::Validation(format!(
                    "{} must be a finite number",
                    field.name
                )));
            }
            Ok(json!(number))
        }
        "date" => {
            let date = value
                .as_str()
                .ok_or_else(|| ProjectsError::Validation(format!("{} must be a date", field.name)))?
                .trim();
            NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
                ProjectsError::Validation(format!("{} must use YYYY-MM-DD", field.name))
            })?;
            Ok(json!(date))
        }
        "status" | "single_select" | "iteration" | "milestone" => {
            let text = value
                .as_str()
                .ok_or_else(|| ProjectsError::Validation(format!("{} must be text", field.name)))?
                .trim()
                .to_owned();
            if text.is_empty() {
                Ok(Value::Null)
            } else {
                validate_option_value(field, &text)?;
                Ok(json!(text))
            }
        }
        "assignees" | "labels" => {
            let values = value.as_array().ok_or_else(|| {
                ProjectsError::Validation(format!("{} must be a list", field.name))
            })?;
            let mut normalized = Vec::new();
            for entry in values {
                let text = entry
                    .as_str()
                    .ok_or_else(|| {
                        ProjectsError::Validation(format!("{} values must be text", field.name))
                    })?
                    .trim()
                    .trim_start_matches('@')
                    .to_owned();
                if !text.is_empty() && !normalized.contains(&text) {
                    normalized.push(text);
                }
            }
            Ok(json!(normalized))
        }
        "repository" => Err(ProjectsError::Validation(
            "Repository fields cannot be edited inline".to_owned(),
        )),
        other => Err(ProjectsError::Validation(format!(
            "{other} fields are not editable from the table workspace"
        ))),
    }
}

fn validate_option_value(field: &ProjectWorkspaceField, value: &str) -> Result<(), ProjectsError> {
    let Some(options) = field.settings.get("options").and_then(Value::as_array) else {
        return Ok(());
    };
    if options.iter().any(|option| {
        option.as_str() == Some(value)
            || option.get("name").and_then(Value::as_str) == Some(value)
            || option.get("title").and_then(Value::as_str) == Some(value)
    }) {
        Ok(())
    } else {
        Err(ProjectsError::Validation(format!(
            "{} must match a configured option",
            field.name
        )))
    }
}

async fn apply_project_field_value(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    field: &ProjectWorkspaceField,
    value: &Value,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    match field.field_type.as_str() {
        "title" if item.item_type == "draft_issue" => {
            sqlx::query("UPDATE project_items SET title = $2 WHERE id = $1")
                .bind(item.id)
                .bind(value.as_str().unwrap_or_default())
                .execute(pool)
                .await?;
        }
        "title" => {
            update_linked_issue_title(pool, item, value.as_str().unwrap_or_default()).await?
        }
        "status" if item.issue_id.is_some() || item.pull_request_issue_id.is_some() => {
            let state = value.as_str().unwrap_or("open");
            if !matches!(state, "open" | "closed") {
                return Err(ProjectsError::Validation(
                    "Status must be open or closed for linked issues and pull requests".to_owned(),
                ));
            }
            update_linked_issue_state(pool, item, state, actor_user_id).await?;
        }
        "labels" if item.issue_id.is_some() || item.pull_request_issue_id.is_some() => {
            sync_linked_issue_labels(pool, item, value).await?;
        }
        "assignees" if item.issue_id.is_some() || item.pull_request_issue_id.is_some() => {
            sync_linked_issue_assignees(pool, item, value, actor_user_id).await?;
        }
        "milestone" if item.issue_id.is_some() || item.pull_request_issue_id.is_some() => {
            sync_linked_issue_milestone(pool, item, value).await?;
        }
        _ => {}
    }

    sqlx::query(
        r#"
        INSERT INTO project_item_field_values (project_item_id, project_field_id, value, updated_by_user_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (project_item_id, project_field_id)
        DO UPDATE SET value = EXCLUDED.value, updated_by_user_id = EXCLUDED.updated_by_user_id, updated_at = now()
        "#,
    )
    .bind(item.id)
    .bind(field.id)
    .bind(value)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_linked_issue_title(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    title: &str,
) -> Result<(), ProjectsError> {
    let issue_id = item
        .issue_id
        .or(item.pull_request_issue_id)
        .ok_or_else(|| {
            ProjectsError::Validation("Linked issue metadata was not found".to_owned())
        })?;
    sqlx::query("UPDATE issues SET title = $2 WHERE id = $1")
        .bind(issue_id)
        .bind(title)
        .execute(pool)
        .await?;
    if let Some(pull_request_id) = item.pull_request_id {
        sqlx::query("UPDATE pull_requests SET title = $2 WHERE id = $1")
            .bind(pull_request_id)
            .bind(title)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn update_linked_issue_state(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    state: &str,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let issue_id = item
        .issue_id
        .or(item.pull_request_issue_id)
        .ok_or_else(|| {
            ProjectsError::Validation("Linked issue metadata was not found".to_owned())
        })?;
    sqlx::query(
        "UPDATE issues SET state = $2, closed_by_user_id = CASE WHEN $2 = 'closed' THEN $3 ELSE NULL END, closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE NULL END WHERE id = $1",
    )
    .bind(issue_id)
    .bind(state)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    if let Some(pull_request_id) = item.pull_request_id {
        sqlx::query(
            "UPDATE pull_requests SET state = $2, closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE NULL END WHERE id = $1 AND state <> 'merged'",
        )
        .bind(pull_request_id)
        .bind(state)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn sync_linked_issue_labels(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    value: &Value,
) -> Result<(), ProjectsError> {
    let issue_id = item
        .issue_id
        .or(item.pull_request_issue_id)
        .ok_or_else(|| {
            ProjectsError::Validation("Linked issue metadata was not found".to_owned())
        })?;
    let repository_id = item.repository_id.ok_or_else(|| {
        ProjectsError::Validation("Linked repository metadata was not found".to_owned())
    })?;
    sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
        .bind(issue_id)
        .execute(pool)
        .await?;
    for name in value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let label_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM labels WHERE repository_id = $1 AND lower(name) = lower($2)",
        )
        .bind(repository_id)
        .bind(name)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ProjectsError::Validation(format!("Label `{name}` was not found")))?;
        sqlx::query(
            "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(issue_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn sync_linked_issue_assignees(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    value: &Value,
    actor_user_id: Uuid,
) -> Result<(), ProjectsError> {
    let issue_id = item
        .issue_id
        .or(item.pull_request_issue_id)
        .ok_or_else(|| {
            ProjectsError::Validation("Linked issue metadata was not found".to_owned())
        })?;
    sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1")
        .bind(issue_id)
        .execute(pool)
        .await?;
    for login in value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let user_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM users WHERE lower(username) = lower($1) OR lower(email) = lower($1)",
        )
        .bind(login)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ProjectsError::Validation(format!("User `{login}` was not found")))?;
        sqlx::query("INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(issue_id)
            .bind(user_id)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn sync_linked_issue_milestone(
    pool: &PgPool,
    item: &ProjectWorkspaceEditItem,
    value: &Value,
) -> Result<(), ProjectsError> {
    let issue_id = item
        .issue_id
        .or(item.pull_request_issue_id)
        .ok_or_else(|| {
            ProjectsError::Validation("Linked issue metadata was not found".to_owned())
        })?;
    let repository_id = item.repository_id.ok_or_else(|| {
        ProjectsError::Validation("Linked repository metadata was not found".to_owned())
    })?;
    let title = value.as_str().unwrap_or_default();
    let milestone_id: Option<Uuid> = if title.is_empty() {
        None
    } else {
        Some(
            sqlx::query_scalar(
                "SELECT id FROM milestones WHERE repository_id = $1 AND lower(title) = lower($2)",
            )
            .bind(repository_id)
            .bind(title)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                ProjectsError::Validation(format!("Milestone `{title}` was not found"))
            })?,
        )
    };
    sqlx::query("UPDATE issues SET milestone_id = $2 WHERE id = $1")
        .bind(issue_id)
        .bind(milestone_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn apply_workspace_filters(
    items: &mut Vec<ProjectWorkspaceItem>,
    filters: &ProjectWorkspaceFilters,
) {
    if let Some(query) = &filters.query {
        let terms = query
            .to_ascii_lowercase()
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        items.retain(|item| {
            terms.iter().all(|term| match term.as_str() {
                "is:open" => item.state.as_deref() == Some("open"),
                "is:closed" => item.state.as_deref() == Some("closed"),
                "is:draft" => item.item_type == "draft_issue",
                "is:issue" => item.item_type == "issue",
                "is:pr" => item.item_type == "pull_request",
                other => {
                    matches_workspace_field_filter(item, other)
                        || item.title.to_ascii_lowercase().contains(other)
                        || item
                            .repository
                            .as_ref()
                            .is_some_and(|repo| repo.full_name.to_ascii_lowercase().contains(other))
                        || item.labels.iter().any(|label| {
                            label.name.to_ascii_lowercase().contains(other)
                                || format!("label:{}", label.name).to_ascii_lowercase() == other
                        })
                }
            })
        });
    }
}

fn matches_workspace_field_filter(item: &ProjectWorkspaceItem, token: &str) -> bool {
    let Some((field_name, expected)) = token.split_once(':') else {
        return false;
    };
    let field_name = field_name.to_ascii_lowercase();
    let expected = expected.trim();
    if expected.is_empty() {
        return false;
    }
    item.field_values.iter().any(|value| {
        value.display_value.to_ascii_lowercase().contains(expected)
            || field_value_date_matches(&value.value, expected)
            || (field_name == "iteration" && field_value_date_matches(&value.value, expected))
    })
}

fn field_value_date_matches(value: &Value, expected: &str) -> bool {
    let Some(date_text) = value.as_str() else {
        return false;
    };
    let Ok(date) = NaiveDate::parse_from_str(date_text, "%Y-%m-%d") else {
        return false;
    };
    let today = Utc::now().date_naive();
    match expected {
        "@current" => date <= today && today < date + Duration::days(14),
        "@previous" => date < today,
        "@next" => date > today,
        _ if expected.contains("..") => {
            let Some((start, end)) = expected.split_once("..") else {
                return false;
            };
            let start = NaiveDate::parse_from_str(start, "%Y-%m-%d").ok();
            let end = NaiveDate::parse_from_str(end, "%Y-%m-%d").ok();
            start.is_some_and(|start| date >= start) && end.is_some_and(|end| date <= end)
        }
        _ if expected.starts_with(">=") => {
            NaiveDate::parse_from_str(&expected[2..], "%Y-%m-%d").is_ok_and(|target| date >= target)
        }
        _ if expected.starts_with("<=") => {
            NaiveDate::parse_from_str(&expected[2..], "%Y-%m-%d").is_ok_and(|target| date <= target)
        }
        _ if expected.starts_with('>') => {
            NaiveDate::parse_from_str(&expected[1..], "%Y-%m-%d").is_ok_and(|target| date > target)
        }
        _ if expected.starts_with('<') => {
            NaiveDate::parse_from_str(&expected[1..], "%Y-%m-%d").is_ok_and(|target| date < target)
        }
        _ => false,
    }
}

fn sort_workspace_items(items: &mut [ProjectWorkspaceItem], sort: &str) {
    match sort {
        "updated_desc" => items.sort_by_key(|item| std::cmp::Reverse(item.updated_at)),
        "updated_asc" => items.sort_by_key(|item| item.updated_at),
        "title_asc" => items.sort_by_key(|item| item.title.to_ascii_lowercase()),
        "title_desc" => {
            items.sort_by_key(|item| std::cmp::Reverse(item.title.to_ascii_lowercase()))
        }
        _ => {}
    }
}

fn workspace_groups(
    items: &[ProjectWorkspaceItem],
    group: Option<&str>,
    fields: &[ProjectWorkspaceField],
) -> Vec<ProjectWorkspaceGroup> {
    let Some(group) = group else {
        return vec![ProjectWorkspaceGroup {
            key: "all".to_owned(),
            label: "All items".to_owned(),
            count: items.len() as i64,
        }];
    };
    let Some(field) = find_workspace_field(fields, group) else {
        return Vec::new();
    };
    counted_field_values(items, field)
        .into_iter()
        .map(|(key, count)| ProjectWorkspaceGroup {
            label: if key.is_empty() {
                "No value".to_owned()
            } else {
                key.clone()
            },
            key,
            count,
        })
        .collect()
}

fn workspace_slices(
    items: &[ProjectWorkspaceItem],
    slice: Option<&str>,
    fields: &[ProjectWorkspaceField],
) -> Vec<ProjectWorkspaceSlice> {
    let Some(slice) = slice else {
        return Vec::new();
    };
    let Some(field) = find_workspace_field(fields, slice) else {
        return Vec::new();
    };
    counted_field_values(items, field)
        .into_iter()
        .map(|(key, count)| ProjectWorkspaceSlice {
            label: if key.is_empty() {
                "No value".to_owned()
            } else {
                key.clone()
            },
            key,
            count,
        })
        .collect()
}

fn find_workspace_field<'a>(
    fields: &'a [ProjectWorkspaceField],
    selector: &str,
) -> Option<&'a ProjectWorkspaceField> {
    let normalized = selector.to_ascii_lowercase();
    fields.iter().find(|field| {
        field.id.to_string() == selector || field.name.to_ascii_lowercase() == normalized
    })
}

fn counted_field_values(
    items: &[ProjectWorkspaceItem],
    field: &ProjectWorkspaceField,
) -> Vec<(String, i64)> {
    let mut counts = std::collections::BTreeMap::<String, i64>::new();
    for item in items {
        let value = item
            .field_values
            .iter()
            .find(|value| value.field_id == field.id)
            .map(|value| value.display_value.clone())
            .unwrap_or_default();
        *counts.entry(value).or_default() += 1;
    }
    counts.into_iter().collect()
}

fn apply_project_filters(projects: &mut Vec<ProjectRow>, filters: &ProjectListFilters) {
    if let Some(query) = &filters.query {
        let normalized = query.to_ascii_lowercase();
        let terms = normalized
            .split_whitespace()
            .filter(|term| {
                !matches!(
                    *term,
                    "is:open" | "state:open" | "is:closed" | "state:closed"
                )
            })
            .collect::<Vec<_>>();
        projects.retain(|project| {
            if (normalized.contains("is:open") || normalized.contains("state:open"))
                && project.state != "open"
            {
                return false;
            }
            if (normalized.contains("is:closed") || normalized.contains("state:closed"))
                && project.state != "closed"
            {
                return false;
            }
            terms.is_empty()
                || terms.iter().all(|term| {
                    project.title.to_ascii_lowercase().contains(term)
                        || project
                            .description
                            .as_deref()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains(term)
                        || project
                            .status
                            .as_ref()
                            .is_some_and(|status| status.label.to_ascii_lowercase().contains(term))
                })
        });
    }
    if filters.tab != "templates" {
        projects.retain(|project| project.state == filters.state);
    }
}

fn sort_projects(projects: &mut [ProjectRow], sort: &str) {
    match sort {
        "name_asc" => projects.sort_by_key(|project| project.title.to_ascii_lowercase()),
        "name_desc" => {
            projects.sort_by_key(|project| std::cmp::Reverse(project.title.to_ascii_lowercase()))
        }
        "created_asc" => projects.sort_by_key(|project| project.created_at),
        "created_desc" => projects.sort_by_key(|project| std::cmp::Reverse(project.created_at)),
        _ => projects.sort_by_key(|project| std::cmp::Reverse(project.updated_at)),
    }
}

fn project_counts(projects: &[ProjectRow]) -> ProjectCounts {
    ProjectCounts {
        open: projects
            .iter()
            .filter(|project| project.state == "open")
            .count() as i64,
        closed: projects
            .iter()
            .filter(|project| project.state == "closed")
            .count() as i64,
        templates: projects
            .iter()
            .filter(|project| project.is_template)
            .count() as i64,
        total: projects.len() as i64,
    }
}

fn template_rows(projects: &[ProjectRow]) -> Vec<ProjectTemplateRow> {
    projects
        .iter()
        .filter(|project| project.is_template)
        .map(|project| ProjectTemplateRow {
            id: project.id,
            project_id: project.id,
            title: project.title.clone(),
            description: project.description.clone(),
            project_title: project.title.clone(),
            project_href: project.href.clone(),
            is_public: project.visibility == "public",
            viewer_can_copy: project.viewer_can_copy,
            created_at: project.created_at,
        })
        .collect()
}

fn permissions_for_scope(
    scope: &ProjectScope,
    viewer_user_id: Option<Uuid>,
) -> ProjectListPermissions {
    let viewer_role = match scope {
        ProjectScope::User { id, .. } if Some(*id) == viewer_user_id => Some("admin".to_owned()),
        ProjectScope::Organization { viewer_role, .. } => viewer_role.clone(),
        ProjectScope::Repository { viewer_role, .. } => viewer_role.clone(),
        _ => None,
    };
    let can_create = viewer_role
        .as_deref()
        .is_some_and(|role| matches!(role, "owner" | "admin" | "write"));
    let can_copy = viewer_role.as_deref().is_some_and(can_write_project_role);
    ProjectListPermissions {
        authenticated: viewer_user_id.is_some(),
        viewer_role,
        can_create,
        can_copy,
    }
}

fn scope_summary(scope: &ProjectScope) -> ProjectListScopeSummary {
    match scope {
        ProjectScope::User { login, .. } => ProjectListScopeSummary {
            kind: "user".to_owned(),
            login: login.clone(),
            repository: None,
            href: format!("/{login}?tab=projects"),
        },
        ProjectScope::Organization { login, .. } => ProjectListScopeSummary {
            kind: "organization".to_owned(),
            login: login.clone(),
            repository: None,
            href: format!("/orgs/{login}/projects"),
        },
        ProjectScope::Repository {
            id,
            owner_login,
            name,
            full_name,
            ..
        } => ProjectListScopeSummary {
            kind: "repository".to_owned(),
            login: owner_login.clone(),
            repository: Some(ProjectRepositoryScopeSummary {
                id: *id,
                owner: owner_login.clone(),
                name: name.clone(),
                full_name: full_name.clone(),
                href: format!("/{owner_login}/{name}"),
            }),
            href: format!("/{owner_login}/{name}/projects"),
        },
    }
}

fn unavailable_reason_for_scope(scope: &ProjectScope) -> Option<String> {
    match scope {
        ProjectScope::Organization {
            projects_enabled: false,
            ..
        } => Some("Organization Projects are disabled by policy.".to_owned()),
        _ => None,
    }
}

fn status_label(status: &str) -> String {
    match status {
        "on_track" => "On track",
        "at_risk" => "At risk",
        "off_track" => "Off track",
        "complete" => "Complete",
        other => other,
    }
    .to_owned()
}

fn can_write_project_role(role: &str) -> bool {
    matches!(role, "owner" | "admin" | "write")
}
