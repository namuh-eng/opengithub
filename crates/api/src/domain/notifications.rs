use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub repository_id: Option<Uuid>,
    pub subject_type: String,
    pub subject_id: Option<Uuid>,
    pub title: String,
    pub reason: String,
    pub unread: bool,
    pub saved: bool,
    pub last_read_at: Option<DateTime<Utc>>,
    pub saved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub repository_id: Option<Uuid>,
    pub subject_type: String,
    pub subject_id: Option<Uuid>,
    pub title: String,
    pub reason: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("notification was not found")]
    NotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTriageAction {
    Read,
    Unread,
    Save,
    Unsave,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTriageResponse {
    pub id: Uuid,
    pub unread: bool,
    pub saved: bool,
    pub done: bool,
    pub last_read_at: Option<DateTime<Utc>>,
    pub saved_at: Option<DateTime<Utc>>,
    pub unread_count: i64,
    pub folder_counts: NotificationFolderCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFolderCounts {
    pub inbox: i64,
    pub saved: i64,
    pub done: i64,
}

pub async fn create_notification(
    pool: &PgPool,
    input: CreateNotification,
) -> Result<Notification, NotificationError> {
    let row = sqlx::query(
        r#"
        INSERT INTO notifications (
            user_id, repository_id, subject_type, subject_id, title, reason
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, repository_id, subject_type, subject_id, title, reason,
                  unread, saved, last_read_at, saved_at, created_at, updated_at
        "#,
    )
    .bind(input.user_id)
    .bind(input.repository_id)
    .bind(&input.subject_type)
    .bind(input.subject_id)
    .bind(&input.title)
    .bind(&input.reason)
    .fetch_one(pool)
    .await?;

    Ok(notification_from_row(row))
}

pub async fn list_notifications(
    pool: &PgPool,
    user_id: Uuid,
    unread_only: bool,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Notification>, NotificationError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let unread_filter = if unread_only { Some(true) } else { None };

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM notifications
        WHERE user_id = $1 AND ($2::boolean IS NULL OR unread = $2)
        "#,
    )
    .bind(user_id)
    .bind(unread_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, user_id, repository_id, subject_type, subject_id, title, reason,
               unread, saved, last_read_at, saved_at, created_at, updated_at
        FROM notifications
        WHERE user_id = $1 AND ($2::boolean IS NULL OR unread = $2)
        ORDER BY updated_at DESC, created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(unread_filter)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(ListEnvelope {
        items: rows.into_iter().map(notification_from_row).collect(),
        total,
        page,
        page_size,
    })
}

pub async fn unread_notification_count(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i64, NotificationError> {
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notifications WHERE user_id = $1 AND unread = true",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(total)
}

pub async fn notification_folder_counts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<NotificationFolderCounts, NotificationError> {
    let row = sqlx::query(
        r#"
        SELECT count(*) AS inbox,
               count(*) FILTER (WHERE saved = true) AS saved
        FROM notifications
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(NotificationFolderCounts {
        inbox: row.get("inbox"),
        saved: row.get("saved"),
        done: 0,
    })
}

pub async fn mark_notification_read(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
) -> Result<Notification, NotificationError> {
    let row = sqlx::query(
        r#"
        UPDATE notifications
        SET unread = false, last_read_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, repository_id, subject_type, subject_id, title, reason,
                  unread, saved, last_read_at, saved_at, created_at, updated_at
        "#,
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificationError::NotFound)?;

    Ok(notification_from_row(row))
}

pub async fn triage_notification(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
    action: NotificationTriageAction,
) -> Result<NotificationTriageResponse, NotificationError> {
    let row = match action {
        NotificationTriageAction::Read => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET unread = false, last_read_at = now()
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Unread => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET unread = true, last_read_at = NULL
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Save => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET saved = true, saved_at = COALESCE(saved_at, now())
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Unsave => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET saved = false, saved_at = NULL
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
    }
    .ok_or(NotificationError::NotFound)?;

    Ok(NotificationTriageResponse {
        id: row.get("id"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        done: false,
        last_read_at: row.get("last_read_at"),
        saved_at: row.get("saved_at"),
        unread_count: unread_notification_count(pool, user_id).await?,
        folder_counts: notification_folder_counts(pool, user_id).await?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxView {
    pub query: NotificationInboxQueryView,
    pub folders: Vec<NotificationFacet>,
    pub filters: Vec<NotificationFacet>,
    pub repositories: Vec<NotificationFacet>,
    pub sort_options: Vec<NotificationChoice>,
    pub group_options: Vec<NotificationChoice>,
    pub groups: Vec<NotificationGroup>,
    pub total: i64,
    pub unread_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub empty_title: String,
    pub empty_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxQueryView {
    pub q: String,
    pub folder: String,
    pub tab: String,
    pub sort: String,
    pub group: String,
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFacet {
    pub id: String,
    pub label: String,
    pub query: String,
    pub href: String,
    pub count: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationChoice {
    pub id: String,
    pub label: String,
    pub href: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationGroup {
    pub id: String,
    pub label: String,
    pub count: i64,
    pub rows: Vec<NotificationInboxRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxRow {
    pub id: Uuid,
    pub repository_id: Option<Uuid>,
    pub repository_name: String,
    pub repository_href: Option<String>,
    pub subject_type: String,
    pub subject_number: Option<i64>,
    pub title: String,
    pub reason: String,
    pub reason_label: String,
    pub href: String,
    pub open_href: String,
    pub unread: bool,
    pub saved: bool,
    pub done: bool,
    pub subscribed: bool,
    pub updated_at: DateTime<Utc>,
    pub relative_time: String,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationInboxQuery {
    pub q: Option<String>,
    pub folder: Option<String>,
    pub tab: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub repo: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
struct InboxRowData {
    id: Uuid,
    repository_id: Option<Uuid>,
    owner_login: Option<String>,
    repo_name: Option<String>,
    subject_type: String,
    title: String,
    reason: String,
    unread: bool,
    saved: bool,
    updated_at: DateTime<Utc>,
    issue_number: Option<i64>,
    pull_number: Option<i64>,
    run_number: Option<i64>,
    release_tag: Option<String>,
    subscribed: bool,
}

#[derive(Debug, Clone)]
struct ParsedNotificationQuery {
    text_terms: Vec<String>,
    unread: Option<bool>,
    saved: Option<bool>,
    done: Option<bool>,
    reason: Option<String>,
    repo: Option<String>,
    subject_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct NotificationLinkContext<'a> {
    folder: &'a str,
    tab: &'a str,
    sort: &'a str,
    group: &'a str,
    q: &'a str,
    repo: Option<&'a str>,
}

pub async fn notification_inbox_view(
    pool: &PgPool,
    user_id: Uuid,
    query: NotificationInboxQuery,
) -> Result<NotificationInboxView, NotificationError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let folder = normalize_folder(query.folder.as_deref());
    let tab = normalize_tab(query.tab.as_deref());
    let sort = normalize_sort(query.sort.as_deref());
    let group = normalize_group(query.group.as_deref());
    let raw_q = query.q.unwrap_or_default().trim().to_owned();
    let mut parsed = parse_notification_query(&raw_q);
    if tab == "unread" {
        parsed.unread = Some(true);
    }
    match folder.as_str() {
        "saved" => parsed.saved = Some(true),
        "done" => parsed.done = Some(true),
        _ => {}
    }
    if let Some(repo) = query
        .repo
        .as_deref()
        .map(str::trim)
        .filter(|repo| !repo.is_empty())
    {
        parsed.repo = Some(repo.to_owned());
    }

    let total = count_matching_notifications(pool, user_id, &parsed).await?;
    let unread_count = unread_notification_count(pool, user_id).await?;
    let rows =
        fetch_matching_notifications(pool, user_id, &parsed, &sort, page_size, offset).await?;
    let inbox_rows: Vec<NotificationInboxRow> = rows.into_iter().map(inbox_row_view).collect();
    let groups = group_notifications(&inbox_rows, &group);
    let repositories = repository_facets(
        pool,
        user_id,
        NotificationLinkContext {
            folder: &folder,
            tab: &tab,
            sort: &sort,
            group: &group,
            q: &raw_q,
            repo: parsed.repo.as_deref(),
        },
    )
    .await?;
    let folder_facets = folder_facets(pool, user_id, &folder, &tab, &sort, &group, &raw_q).await?;
    let filters =
        default_filter_facets(pool, user_id, &folder, &tab, &sort, &group, &raw_q).await?;

    let empty_title = if folder == "saved" {
        "No saved notifications".to_owned()
    } else if folder == "done" {
        "No done notifications".to_owned()
    } else if tab == "unread" {
        "No unread notifications".to_owned()
    } else {
        "No matching notifications".to_owned()
    };

    Ok(NotificationInboxView {
        query: NotificationInboxQueryView {
            q: raw_q.clone(),
            folder: folder.clone(),
            tab: tab.clone(),
            sort: sort.clone(),
            group: group.clone(),
            repo: parsed.repo,
        },
        folders: folder_facets,
        filters,
        repositories,
        sort_options: choice_options(
            "sort",
            &[("newest", "Newest"), ("oldest", "Oldest")],
            &sort,
            NotificationLinkContext {
                folder: &folder,
                tab: &tab,
                sort: &sort,
                group: &group,
                q: &raw_q,
                repo: None,
            },
        ),
        group_options: choice_options(
            "group",
            &[("date", "Date"), ("repository", "Repository")],
            &group,
            NotificationLinkContext {
                folder: &folder,
                tab: &tab,
                sort: &sort,
                group: &group,
                q: &raw_q,
                repo: None,
            },
        ),
        groups,
        total,
        unread_count,
        page,
        page_size,
        empty_title,
        empty_message: "Adjust the query, folder, repository, or unread tab to broaden the inbox."
            .to_owned(),
    })
}

async fn count_matching_notifications(
    pool: &PgPool,
    user_id: Uuid,
    parsed: &ParsedNotificationQuery,
) -> Result<i64, NotificationError> {
    let mut builder = QueryBuilder::new(base_count_sql());
    push_filters(&mut builder, user_id, parsed);
    Ok(builder.build_query_scalar().fetch_one(pool).await?)
}

async fn fetch_matching_notifications(
    pool: &PgPool,
    user_id: Uuid,
    parsed: &ParsedNotificationQuery,
    sort: &str,
    page_size: i64,
    offset: i64,
) -> Result<Vec<InboxRowData>, NotificationError> {
    let mut builder = QueryBuilder::new(base_select_sql());
    push_filters(&mut builder, user_id, parsed);
    if sort == "oldest" {
        builder.push(" ORDER BY notifications.updated_at ASC, notifications.created_at ASC ");
    } else {
        builder.push(" ORDER BY notifications.updated_at DESC, notifications.created_at DESC ");
    }
    builder
        .push(" LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = builder.build().fetch_all(pool).await?;
    Ok(rows.into_iter().map(inbox_row_data_from_row).collect())
}

fn base_count_sql() -> &'static str {
    r#"
    SELECT count(*)
    FROM notifications
    LEFT JOIN repositories ON repositories.id = notifications.repository_id
    LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
    LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
    LEFT JOIN issues ON notifications.subject_type = 'issue' AND issues.id = notifications.subject_id
    LEFT JOIN pull_requests ON notifications.subject_type = 'pull_request' AND pull_requests.id = notifications.subject_id
    LEFT JOIN workflow_runs ON notifications.subject_type = 'workflow_run' AND workflow_runs.id = notifications.subject_id
    LEFT JOIN releases ON notifications.subject_type = 'release' AND releases.id = notifications.subject_id
    "#
}

fn base_select_sql() -> &'static str {
    r#"
    SELECT notifications.id, notifications.repository_id,
           COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
           repositories.name AS repo_name,
           notifications.subject_type, notifications.title,
           notifications.reason, notifications.unread, notifications.saved, notifications.updated_at,
           issues.number AS issue_number, pull_requests.number AS pull_number,
           workflow_runs.run_number AS run_number, releases.tag_name AS release_tag,
           EXISTS (
               SELECT 1 FROM repository_watches
               WHERE repository_watches.user_id = notifications.user_id
                 AND repository_watches.repository_id = notifications.repository_id
           ) AS subscribed
    FROM notifications
    LEFT JOIN repositories ON repositories.id = notifications.repository_id
    LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
    LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
    LEFT JOIN issues ON notifications.subject_type = 'issue' AND issues.id = notifications.subject_id
    LEFT JOIN pull_requests ON notifications.subject_type = 'pull_request' AND pull_requests.id = notifications.subject_id
    LEFT JOIN workflow_runs ON notifications.subject_type = 'workflow_run' AND workflow_runs.id = notifications.subject_id
    LEFT JOIN releases ON notifications.subject_type = 'release' AND releases.id = notifications.subject_id
    "#
}

fn push_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    user_id: Uuid,
    parsed: &'a ParsedNotificationQuery,
) {
    builder
        .push(" WHERE notifications.user_id = ")
        .push_bind(user_id);
    if let Some(unread) = parsed.unread {
        builder
            .push(" AND notifications.unread = ")
            .push_bind(unread);
    }
    if let Some(saved) = parsed.saved {
        builder.push(" AND notifications.saved = ").push_bind(saved);
    }
    if let Some(done) = parsed.done {
        builder.push(if done { " AND false" } else { " AND true" });
    }
    if let Some(reason) = &parsed.reason {
        builder
            .push(" AND lower(notifications.reason) = lower(")
            .push_bind(reason)
            .push(")");
    }
    if let Some(subject_type) = &parsed.subject_type {
        builder
            .push(" AND notifications.subject_type = ")
            .push_bind(subject_type);
    }
    if let Some(repo) = &parsed.repo {
        let pattern = format!("%{}%", repo.to_lowercase());
        builder.push(" AND lower(COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug, '') || '/' || COALESCE(repositories.name, '')) LIKE ").push_bind(pattern);
    }
    for term in &parsed.text_terms {
        let pattern = format!("%{}%", term.to_lowercase());
        builder
            .push(" AND (lower(notifications.title) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR lower(notifications.reason) LIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

async fn repository_facets(
    pool: &PgPool,
    user_id: Uuid,
    context: NotificationLinkContext<'_>,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let rows = sqlx::query(
        r#"
        SELECT COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
               repositories.name AS repo_name,
               count(*) AS count
        FROM notifications
        JOIN repositories ON repositories.id = notifications.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        WHERE notifications.user_id = $1
        GROUP BY owner_login, repositories.name
        ORDER BY count DESC, owner_login ASC, repositories.name ASC
        LIMIT 12
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let owner: String = row.get("owner_login");
            let repo_name: String = row.get("repo_name");
            let full = format!("{owner}/{repo_name}");
            let active = context
                .repo
                .is_some_and(|repo| repo.eq_ignore_ascii_case(&full));
            NotificationFacet {
                id: full.clone(),
                label: full.clone(),
                query: context.q.to_owned(),
                href: notification_href(
                    context.folder,
                    context.tab,
                    context.sort,
                    context.group,
                    context.q,
                    Some(&full),
                ),
                count: row.get("count"),
                active,
            }
        })
        .collect())
}

async fn folder_facets(
    pool: &PgPool,
    user_id: Uuid,
    active_folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let counts = notification_folder_counts(pool, user_id).await?;
    let specs = [
        ("inbox", "Inbox", counts.inbox),
        ("saved", "Saved", counts.saved),
        ("done", "Done", counts.done),
    ];
    Ok(specs
        .into_iter()
        .map(|(id, label, count)| NotificationFacet {
            id: id.to_owned(),
            label: label.to_owned(),
            query: q.to_owned(),
            href: notification_href(id, tab, sort, group, q, None),
            count,
            active: id == active_folder,
        })
        .collect())
}

async fn default_filter_facets(
    pool: &PgPool,
    user_id: Uuid,
    folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let specs = [
        ("assigned", "Assigned", "reason:assigned"),
        ("participating", "Participating", "reason:participating"),
        ("mentioned", "Mentioned", "reason:mention"),
        ("team-mentioned", "Team mentioned", "reason:team_mention"),
        (
            "review-requested",
            "Review requested",
            "reason:review_requested",
        ),
    ];
    let mut facets = Vec::with_capacity(specs.len());
    for (id, label, query) in specs {
        let parsed = parse_notification_query(query);
        let count = count_matching_notifications(pool, user_id, &parsed).await?;
        facets.push(NotificationFacet {
            id: id.to_owned(),
            label: label.to_owned(),
            query: query.to_owned(),
            href: notification_href(folder, tab, sort, group, query, None),
            count,
            active: q == query,
        });
    }
    Ok(facets)
}

fn choice_options(
    param: &str,
    specs: &[(&str, &str)],
    active: &str,
    context: NotificationLinkContext<'_>,
) -> Vec<NotificationChoice> {
    specs
        .iter()
        .map(|(id, label)| {
            let (sort, group) = if param == "sort" {
                (*id, context.group)
            } else {
                (context.sort, *id)
            };
            NotificationChoice {
                id: (*id).to_owned(),
                label: (*label).to_owned(),
                href: notification_href(
                    context.folder,
                    context.tab,
                    sort,
                    group,
                    context.q,
                    context.repo,
                ),
                active: *id == active,
            }
        })
        .collect()
}

fn notification_href(
    folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
    repo: Option<&str>,
) -> String {
    let mut params = Vec::new();
    if folder != "inbox" {
        params.push(("folder", folder.to_owned()));
    }
    if tab != "all" {
        params.push(("tab", tab.to_owned()));
    }
    if !q.trim().is_empty() {
        params.push(("q", q.trim().to_owned()));
    }
    if sort != "newest" {
        params.push(("sort", sort.to_owned()));
    }
    if group != "date" {
        params.push(("group", group.to_owned()));
    }
    if let Some(repo) = repo {
        params.push(("repo", repo.to_owned()));
    }
    if params.is_empty() {
        return "/notifications".to_owned();
    }
    let query = params
        .into_iter()
        .map(|(key, value)| format!("{}={}", key, url_encode(&value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("/notifications?{query}")
}

fn inbox_row_view(row: InboxRowData) -> NotificationInboxRow {
    let repository_name = match (&row.owner_login, &row.repo_name) {
        (Some(owner), Some(repo)) => format!("{owner}/{repo}"),
        _ => "OpenGitHub".to_owned(),
    };
    let repository_href = match (&row.owner_login, &row.repo_name) {
        (Some(owner), Some(repo)) => Some(format!("/{owner}/{repo}")),
        _ => None,
    };
    let subject_number = row.pull_number.or(row.issue_number).or(row.run_number);
    let href = target_href(&row, repository_href.as_deref());
    let open_href = format!("/notifications/{}/open?next={}", row.id, url_encode(&href));
    NotificationInboxRow {
        id: row.id,
        repository_id: row.repository_id,
        repository_name,
        repository_href,
        subject_type: row.subject_type,
        subject_number,
        title: row.title,
        reason_label: reason_label(&row.reason),
        reason: row.reason,
        href,
        open_href,
        unread: row.unread,
        saved: row.saved,
        done: false,
        subscribed: row.subscribed,
        updated_at: row.updated_at,
        relative_time: relative_time(row.updated_at),
    }
}

fn target_href(row: &InboxRowData, repository_href: Option<&str>) -> String {
    let Some(repo_href) = repository_href else {
        return "/notifications".to_owned();
    };
    match row.subject_type.as_str() {
        "pull_request" => row
            .pull_number
            .map(|n| format!("{repo_href}/pull/{n}"))
            .unwrap_or_else(|| repo_href.to_owned()),
        "issue" => row
            .issue_number
            .map(|n| format!("{repo_href}/issues/{n}"))
            .unwrap_or_else(|| repo_href.to_owned()),
        "workflow_run" => row
            .run_number
            .map(|n| format!("{repo_href}/actions/runs/{n}"))
            .unwrap_or_else(|| format!("{repo_href}/actions")),
        "release" => row
            .release_tag
            .as_ref()
            .map(|tag| format!("{repo_href}/releases/tag/{tag}"))
            .unwrap_or_else(|| format!("{repo_href}/releases")),
        _ => repo_href.to_owned(),
    }
}

fn group_notifications(rows: &[NotificationInboxRow], group: &str) -> Vec<NotificationGroup> {
    let mut groups: Vec<NotificationGroup> = Vec::new();
    for row in rows {
        let (id, label) = if group == "repository" {
            (row.repository_name.clone(), row.repository_name.clone())
        } else {
            let age = Utc::now().signed_duration_since(row.updated_at);
            let label = if age.num_days() == 0 {
                "Today"
            } else if age.num_days() == 1 {
                "Yesterday"
            } else {
                "Earlier"
            };
            (label.to_lowercase(), label.to_owned())
        };
        if let Some(existing) = groups.iter_mut().find(|g| g.id == id) {
            existing.count += 1;
            existing.rows.push(row.clone());
        } else {
            groups.push(NotificationGroup {
                id,
                label,
                count: 1,
                rows: vec![row.clone()],
            });
        }
    }
    groups
}

fn inbox_row_data_from_row(row: sqlx::postgres::PgRow) -> InboxRowData {
    InboxRowData {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        owner_login: row.get("owner_login"),
        repo_name: row.get("repo_name"),
        subject_type: row.get("subject_type"),
        title: row.get("title"),
        reason: row.get("reason"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        updated_at: row.get("updated_at"),
        issue_number: row.get("issue_number"),
        pull_number: row.get("pull_number"),
        run_number: row.try_get("run_number").ok(),
        release_tag: row.try_get("release_tag").ok(),
        subscribed: row.get("subscribed"),
    }
}

fn parse_notification_query(query: &str) -> ParsedNotificationQuery {
    let mut parsed = ParsedNotificationQuery {
        text_terms: Vec::new(),
        unread: None,
        saved: None,
        done: None,
        reason: None,
        repo: None,
        subject_type: None,
    };
    for token in query.split_whitespace() {
        if let Some(value) = token.strip_prefix("is:") {
            match value {
                "unread" => parsed.unread = Some(true),
                "read" => parsed.unread = Some(false),
                "saved" => parsed.saved = Some(true),
                "done" => parsed.done = Some(true),
                _ => parsed.text_terms.push(token.to_owned()),
            }
        } else if let Some(value) = token.strip_prefix("reason:") {
            parsed.reason = Some(value.trim_matches('"').to_owned());
        } else if let Some(value) = token.strip_prefix("repo:") {
            parsed.repo = Some(value.trim_matches('"').to_owned());
        } else if let Some(value) = token.strip_prefix("type:") {
            parsed.subject_type = Some(match value {
                "pr" | "pull" | "pull_request" => "pull_request".to_owned(),
                "run" | "workflow" | "workflow_run" => "workflow_run".to_owned(),
                other => other.to_owned(),
            });
        } else {
            parsed.text_terms.push(token.to_owned());
        }
    }
    parsed
}

fn normalize_folder(value: Option<&str>) -> String {
    match value {
        Some("saved") => "saved",
        Some("done") => "done",
        _ => "inbox",
    }
    .to_owned()
}
fn normalize_tab(value: Option<&str>) -> String {
    match value {
        Some("unread") => "unread",
        _ => "all",
    }
    .to_owned()
}
fn normalize_sort(value: Option<&str>) -> String {
    match value {
        Some("oldest") => "oldest",
        _ => "newest",
    }
    .to_owned()
}
fn normalize_group(value: Option<&str>) -> String {
    match value {
        Some("repository") => "repository",
        _ => "date",
    }
    .to_owned()
}
fn reason_label(reason: &str) -> String {
    reason
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn relative_time(updated_at: DateTime<Utc>) -> String {
    let age = Utc::now().signed_duration_since(updated_at);
    if age.num_minutes() < 1 {
        "just now".to_owned()
    } else if age.num_hours() < 1 {
        format!("{}m ago", age.num_minutes())
    } else if age.num_days() < 1 {
        format!("{}h ago", age.num_hours())
    } else {
        format!("{}d ago", age.num_days())
    }
}

fn notification_from_row(row: sqlx::postgres::PgRow) -> Notification {
    Notification {
        id: row.get("id"),
        user_id: row.get("user_id"),
        repository_id: row.get("repository_id"),
        subject_type: row.get("subject_type"),
        subject_id: row.get("subject_id"),
        title: row.get("title"),
        reason: row.get("reason"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        last_read_at: row.get("last_read_at"),
        saved_at: row.get("saved_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
