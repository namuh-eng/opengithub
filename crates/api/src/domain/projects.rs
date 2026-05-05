use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
    #[error("repository error")]
    Repository(#[from] super::repositories::RepositoryError),
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
