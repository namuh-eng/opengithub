use chrono::{DateTime, NaiveDate, Utc};
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
    sqlx::query("UPDATE project_items SET archived_at = now(), updated_at = now() WHERE id = $1")
        .bind(item_id)
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
    let project_role: Option<String> = row.try_get("project_role")?;
    let org_role: Option<String> = row.try_get("organization_role")?;
    let org_base_role: Option<String> = row.try_get("organization_base_role")?;
    Ok(project_role
        .or(org_role)
        .or(org_base_role.filter(|role| role != "none")))
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
        WHERE project_id = $1 AND id = $2
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
                    item.title.to_ascii_lowercase().contains(other)
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
