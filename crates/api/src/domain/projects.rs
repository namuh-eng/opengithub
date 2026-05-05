use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspace {
    pub project: ProjectWorkspaceProject,
    pub selected_view: ProjectWorkspaceView,
    pub views: Vec<ProjectWorkspaceView>,
    pub fields: Vec<ProjectWorkspaceField>,
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
    if selected_view.layout != "table" {
        return Err(ProjectsError::InvalidFilter(
            "selected view must use the table layout".to_owned(),
        ));
    }
    let fields = workspace_fields(pool, project_id, &selected_view).await?;
    let filters = normalize_workspace_filters(query, &selected_view, &fields)?;
    let unsaved_view = workspace_unsaved_state(&filters, &selected_view);
    let mut items = workspace_items(pool, project_id, viewer_user_id, &fields).await?;
    apply_workspace_filters(&mut items, &filters);
    sort_workspace_items(&mut items, &filters.sort);
    let groups = workspace_groups(&items, filters.group.as_deref(), &fields);
    let slices = workspace_slices(&items, filters.slice.as_deref(), &fields);
    let total = items.len() as i64;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let page_items = items
        .into_iter()
        .skip(offset)
        .take(filters.page_size as usize)
        .collect();
    let can_edit = project
        .viewer_role
        .as_deref()
        .is_some_and(can_write_project_role);

    Ok(ProjectWorkspace {
        project,
        selected_view,
        views,
        fields,
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
