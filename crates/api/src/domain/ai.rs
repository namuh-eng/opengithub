use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

use super::repositories::{
    can_read_repository, get_repository_by_owner_name, Repository, RepositoryError,
    RepositoryVisibility,
};

const PROMPT_VERSION: &str = "ai-001-v1";
const REPO_MODEL: &str = "gpt-4o-mini";
const PR_MODEL: &str = "gpt-4o";
const CHANGELOG_MODEL: &str = "gpt-4o";

#[derive(Debug, Error)]
pub enum AiError {
    #[error("repository not found")]
    RepositoryNotFound,
    #[error("pull request not found")]
    PullRequestNotFound,
    #[error("release not found")]
    ReleaseNotFound,
    #[error("permission denied")]
    PermissionDenied,
    #[error("AI features are disabled for this repository")]
    Disabled,
    #[error("OPENAI_API_KEY is not configured")]
    ProviderNotConfigured,
    #[error("AI provider request failed")]
    ProviderFailed,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiOutput {
    pub id: Uuid,
    pub kind: String,
    pub scope_type: String,
    pub scope_id: Uuid,
    pub content_hash: String,
    pub prompt_version: String,
    pub model: String,
    pub output: String,
    pub generated_at: DateTime<Utc>,
    pub regenerated_count: i32,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAiSummary {
    pub enabled: bool,
    pub reason: Option<String>,
    pub output: Option<AiOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestAiSummary {
    pub enabled: bool,
    pub reason: Option<String>,
    pub output: Option<AiOutput>,
    pub files_of_interest: Vec<AiFileRisk>,
    pub suggested_reviewers: Vec<AiSuggestedReviewer>,
    pub inline_comment_seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiFileRisk {
    pub path: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestedReviewer {
    pub login: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiChangelogRequest {
    pub previous_tag: Option<String>,
    pub target_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiChangelog {
    pub enabled: bool,
    pub reason: Option<String>,
    pub output: Option<AiOutput>,
    pub previous_tag: Option<String>,
    pub target_tag: String,
}

pub async fn repository_ai_summary(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
    regenerate: bool,
) -> Result<RepositoryAiSummary, AiError> {
    let repository = readable_repository(pool, owner, repo, actor_user_id).await?;
    let gate = ai_gate(pool, &repository, actor_user_id).await?;
    if let Some(reason) = gate {
        return Ok(RepositoryAiSummary {
            enabled: false,
            reason: Some(reason),
            output: latest_output(pool, "repo_summary", "repository", repository.id).await?,
        });
    }

    let context = repository_context(pool, &repository).await?;
    let content_hash = content_hash(&context);
    let output = cached_or_generate(
        pool,
        "repo_summary",
        "repository",
        repository.id,
        &content_hash,
        PROMPT_VERSION,
        REPO_MODEL,
        actor_user_id,
        regenerate,
        "Summarize this repository in 3 concise bullets for a code hosting UI. Mention purpose, notable files, and recent activity.",
        &context,
    )
    .await?;

    Ok(RepositoryAiSummary {
        enabled: true,
        reason: None,
        output: Some(output),
    })
}

pub async fn pull_request_ai_summary(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    number: i64,
    actor_user_id: Option<Uuid>,
    regenerate: bool,
) -> Result<PullRequestAiSummary, AiError> {
    let repository = readable_repository(pool, owner, repo, actor_user_id).await?;
    let gate = ai_gate(pool, &repository, actor_user_id).await?;
    let pull_request = pull_request_row(pool, repository.id, number).await?;
    let files = pull_request_files(pool, pull_request.id).await?;
    let reviewers = suggested_reviewers(pool, pull_request.id, repository.id).await?;
    let risks = files
        .iter()
        .take(5)
        .map(|file| AiFileRisk {
            path: file.path.clone(),
            note: format!(
                "{} with {} additions and {} deletions",
                file.status, file.additions, file.deletions
            ),
        })
        .collect::<Vec<_>>();

    if let Some(reason) = gate {
        return Ok(PullRequestAiSummary {
            enabled: false,
            reason: Some(reason),
            output: latest_output(pool, "pr_summary", "pull_request", pull_request.id).await?,
            files_of_interest: risks,
            suggested_reviewers: reviewers,
            inline_comment_seed: None,
        });
    }

    let context = format!(
        "Title: {}\nBody: {}\nBase: {}\nHead: {}\nFiles:\n{}\nCommits:\n{}",
        pull_request.title,
        pull_request.body.as_deref().unwrap_or(""),
        pull_request.base_ref,
        pull_request.head_ref,
        files
            .iter()
            .map(|file| format!(
                "- {} {} +{} -{}",
                file.status, file.path, file.additions, file.deletions
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        pull_request_commits(pool, pull_request.id)
            .await?
            .join("\n")
    );
    let content_hash = content_hash(&context);
    let output = cached_or_generate(
        pool,
        "pr_summary",
        "pull_request",
        pull_request.id,
        &content_hash,
        PROMPT_VERSION,
        PR_MODEL,
        actor_user_id,
        regenerate,
        "Write a pull request summary with sections: TL;DR, Files of interest, Suggested review focus, Inline comment seed. Keep it factual.",
        &context,
    )
    .await?;

    Ok(PullRequestAiSummary {
        enabled: true,
        reason: None,
        output: Some(output),
        files_of_interest: risks,
        suggested_reviewers: reviewers,
        inline_comment_seed: pr_author_inline_seed(actor_user_id, pull_request.author_user_id),
    })
}

fn pr_author_inline_seed(actor_user_id: Option<Uuid>, author_user_id: Uuid) -> Option<String> {
    if actor_user_id == Some(author_user_id) {
        Some("Check whether this change needs an integration test before merge.".to_owned())
    } else {
        None
    }
}

pub async fn ai_changelog(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
    request: AiChangelogRequest,
    regenerate: bool,
) -> Result<AiChangelog, AiError> {
    let repository = readable_repository(pool, owner, repo, actor_user_id).await?;
    let gate = ai_gate(pool, &repository, actor_user_id).await?;
    let release_id = release_id_for_tag(pool, repository.id, &request.target_tag).await?;
    if let Some(reason) = gate {
        return Ok(AiChangelog {
            enabled: false,
            reason: Some(reason),
            output: latest_output(pool, "changelog", "release", release_id).await?,
            previous_tag: request.previous_tag,
            target_tag: request.target_tag,
        });
    }

    let context = release_commit_context(
        pool,
        repository.id,
        request.previous_tag.as_deref(),
        &request.target_tag,
    )
    .await?;
    let content_hash = content_hash(&format!(
        "{}:{}:{context}",
        request.previous_tag.as_deref().unwrap_or(""),
        request.target_tag
    ));
    let output = cached_or_generate(
        pool,
        "changelog",
        "release",
        release_id,
        &content_hash,
        PROMPT_VERSION,
        CHANGELOG_MODEL,
        actor_user_id,
        regenerate,
        "Generate Markdown release changelog bullets grouped as Added, Changed, Fixed, Deprecated. Use only supplied commits.",
        &context,
    )
    .await?;

    Ok(AiChangelog {
        enabled: true,
        reason: None,
        output: Some(output),
        previous_tag: request.previous_tag,
        target_tag: request.target_tag,
    })
}

async fn readable_repository(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, AiError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await?
        .ok_or(AiError::RepositoryNotFound)?;
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(repository);
    }
    let Some(actor_user_id) = actor_user_id else {
        return Err(AiError::PermissionDenied);
    };
    if can_read_repository(pool, &repository, actor_user_id).await? {
        Ok(repository)
    } else {
        Err(AiError::PermissionDenied)
    }
}

async fn ai_gate(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<Option<String>, AiError> {
    let repo_enabled =
        sqlx::query_scalar::<_, bool>("SELECT ai_features_enabled FROM repositories WHERE id = $1")
            .bind(repository.id)
            .fetch_one(pool)
            .await?;
    if !repo_enabled {
        let reason = if repository.visibility == RepositoryVisibility::Public {
            "AI features are disabled for this repository."
        } else {
            "AI features are disabled for private repository content."
        };
        return Ok(Some(reason.to_owned()));
    }
    if let Some(user_id) = actor_user_id {
        let user_enabled = sqlx::query_scalar::<_, bool>(
            "SELECT COALESCE((SELECT ai_features_enabled FROM user_settings WHERE user_id = $1), true)",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        if !user_enabled {
            return Ok(Some(
                "AI features are disabled in your account settings.".to_owned(),
            ));
        }
    }
    Ok(None)
}

async fn repository_context(pool: &PgPool, repository: &Repository) -> Result<String, AiError> {
    let files = sqlx::query(
        r#"
        SELECT path, left(content, 1200) AS content
        FROM repository_files
        WHERE repository_id = $1
        ORDER BY CASE WHEN lower(path) = 'readme.md' THEN 0 ELSE 1 END, path
        LIMIT 8
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    let commits = recent_commits(pool, repository.id, 8).await?;
    Ok(format!(
        "Repository: {}/{}\nDescription: {}\nFiles:\n{}\nRecent commits:\n{}",
        repository.owner_login,
        repository.name,
        repository.description.as_deref().unwrap_or(""),
        files
            .iter()
            .map(|row| format!(
                "## {}\n{}",
                row.get::<String, _>("path"),
                row.get::<Option<String>, _>("content").unwrap_or_default()
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        commits.join("\n")
    ))
}

#[allow(clippy::too_many_arguments)]
async fn cached_or_generate(
    pool: &PgPool,
    kind: &str,
    scope_type: &str,
    scope_id: Uuid,
    content_hash: &str,
    prompt_version: &str,
    model: &str,
    actor_user_id: Option<Uuid>,
    regenerate: bool,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<AiOutput, AiError> {
    if !regenerate {
        if let Some(output) = cached_output(
            pool,
            kind,
            scope_type,
            scope_id,
            content_hash,
            prompt_version,
            model,
        )
        .await?
        {
            return Ok(output);
        }
    }
    let output = call_openai(model, system_prompt, user_prompt).await?;
    upsert_output(
        pool,
        kind,
        scope_type,
        scope_id,
        content_hash,
        prompt_version,
        model,
        &output,
        actor_user_id,
    )
    .await
}

async fn call_openai(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AiError> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or(AiError::ProviderNotConfigured)?;
    let response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt.chars().take(120_000).collect::<String>() }
            ],
            "temperature": 0.2
        }))
        .send()
        .await
        .map_err(|_| AiError::ProviderFailed)?;
    if !response.status().is_success() {
        return Err(AiError::ProviderFailed);
    }
    let body: serde_json::Value = response.json().await.map_err(|_| AiError::ProviderFailed)?;
    body.pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(AiError::ProviderFailed)
}

async fn cached_output(
    pool: &PgPool,
    kind: &str,
    scope_type: &str,
    scope_id: Uuid,
    content_hash: &str,
    prompt_version: &str,
    model: &str,
) -> Result<Option<AiOutput>, AiError> {
    let row = sqlx::query(
        r#"
        SELECT id, kind, scope_type, scope_id, content_hash, prompt_version, model,
               output, generated_at, regenerated_count
        FROM ai_outputs
        WHERE kind = $1 AND scope_type = $2 AND scope_id = $3
          AND content_hash = $4 AND prompt_version = $5 AND model = $6
        "#,
    )
    .bind(kind)
    .bind(scope_type)
    .bind(scope_id)
    .bind(content_hash)
    .bind(prompt_version)
    .bind(model)
    .fetch_optional(pool)
    .await?;
    row.map(|row| ai_output_from_row(row, true)).transpose()
}

async fn latest_output(
    pool: &PgPool,
    kind: &str,
    scope_type: &str,
    scope_id: Uuid,
) -> Result<Option<AiOutput>, AiError> {
    let row = sqlx::query(
        r#"
        SELECT id, kind, scope_type, scope_id, content_hash, prompt_version, model,
               output, generated_at, regenerated_count
        FROM ai_outputs
        WHERE kind = $1 AND scope_type = $2 AND scope_id = $3
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
    )
    .bind(kind)
    .bind(scope_type)
    .bind(scope_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| ai_output_from_row(row, true)).transpose()
}

#[allow(clippy::too_many_arguments)]
async fn upsert_output(
    pool: &PgPool,
    kind: &str,
    scope_type: &str,
    scope_id: Uuid,
    content_hash: &str,
    prompt_version: &str,
    model: &str,
    output: &str,
    actor_user_id: Option<Uuid>,
) -> Result<AiOutput, AiError> {
    let row = sqlx::query(
        r#"
        INSERT INTO ai_outputs (
            kind, scope_type, scope_id, content_hash, prompt_version, model,
            output, created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (kind, scope_type, scope_id, content_hash, prompt_version, model)
        DO UPDATE SET
            output = EXCLUDED.output,
            generated_at = now(),
            regenerated_count = ai_outputs.regenerated_count + 1,
            created_by_user_id = EXCLUDED.created_by_user_id
        RETURNING id, kind, scope_type, scope_id, content_hash, prompt_version, model,
                  output, generated_at, regenerated_count
        "#,
    )
    .bind(kind)
    .bind(scope_type)
    .bind(scope_id)
    .bind(content_hash)
    .bind(prompt_version)
    .bind(model)
    .bind(output)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    ai_output_from_row(row, false)
}

fn ai_output_from_row(row: sqlx::postgres::PgRow, cached: bool) -> Result<AiOutput, AiError> {
    Ok(AiOutput {
        id: row.try_get("id")?,
        kind: row.try_get("kind")?,
        scope_type: row.try_get("scope_type")?,
        scope_id: row.try_get("scope_id")?,
        content_hash: row.try_get("content_hash")?,
        prompt_version: row.try_get("prompt_version")?,
        model: row.try_get("model")?,
        output: row.try_get("output")?,
        generated_at: row.try_get("generated_at")?,
        regenerated_count: row.try_get("regenerated_count")?,
        cached,
    })
}

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

struct PullRequestLite {
    id: Uuid,
    title: String,
    body: Option<String>,
    head_ref: String,
    base_ref: String,
    author_user_id: Uuid,
}

async fn pull_request_row(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
) -> Result<PullRequestLite, AiError> {
    let row = sqlx::query(
        "SELECT id, title, body, head_ref, base_ref, author_user_id FROM pull_requests WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(AiError::PullRequestNotFound)?;
    Ok(PullRequestLite {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        body: row.try_get("body")?,
        head_ref: row.try_get("head_ref")?,
        base_ref: row.try_get("base_ref")?,
        author_user_id: row.try_get("author_user_id")?,
    })
}

struct PullFile {
    path: String,
    status: String,
    additions: i64,
    deletions: i64,
}

async fn pull_request_files(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<Vec<PullFile>, AiError> {
    sqlx::query(
        "SELECT path, status, additions, deletions FROM pull_request_files WHERE pull_request_id = $1 ORDER BY additions + deletions DESC, path LIMIT 30",
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(PullFile {
            path: row.try_get("path")?,
            status: row.try_get("status")?,
            additions: row.try_get("additions")?,
            deletions: row.try_get("deletions")?,
        })
    })
    .collect()
}

async fn pull_request_commits(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<Vec<String>, AiError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT commits.message
        FROM pull_request_commits
        JOIN commits ON commits.id = pull_request_commits.commit_id
        WHERE pull_request_commits.pull_request_id = $1
        ORDER BY pull_request_commits.position
        LIMIT 30
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?)
}

async fn suggested_reviewers(
    pool: &PgPool,
    pull_request_id: Uuid,
    repository_id: Uuid,
) -> Result<Vec<AiSuggestedReviewer>, AiError> {
    let mut reviewers = sqlx::query(
        r#"
        SELECT DISTINCT COALESCE(NULLIF(users.username, ''), users.email) AS login,
               'requested reviewer' AS reason
        FROM pull_request_review_requests
        JOIN users ON users.id = pull_request_review_requests.requested_user_id
        WHERE pull_request_review_requests.pull_request_id = $1
        ORDER BY login
        LIMIT 5
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| AiSuggestedReviewer {
        login: row.get("login"),
        reason: row.get("reason"),
    })
    .collect::<Vec<_>>();
    if reviewers.is_empty() {
        reviewers = sqlx::query(
            r#"
            SELECT DISTINCT COALESCE(NULLIF(users.username, ''), users.email) AS login,
                   'recent committer' AS reason
            FROM commits
            JOIN users ON users.id = commits.author_user_id
            WHERE commits.repository_id = $1
            ORDER BY login
            LIMIT 3
            "#,
        )
        .bind(repository_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| AiSuggestedReviewer {
            login: row.get("login"),
            reason: row.get("reason"),
        })
        .collect();
    }
    Ok(reviewers)
}

async fn recent_commits(
    pool: &PgPool,
    repository_id: Uuid,
    limit: i64,
) -> Result<Vec<String>, AiError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT message FROM commits WHERE repository_id = $1 ORDER BY committed_at DESC LIMIT $2",
    )
    .bind(repository_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

async fn release_id_for_tag(
    pool: &PgPool,
    repository_id: Uuid,
    tag: &str,
) -> Result<Uuid, AiError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM releases WHERE repository_id = $1 AND tag_name = $2 AND deleted_at IS NULL",
    )
    .bind(repository_id)
    .bind(tag)
    .fetch_optional(pool)
    .await?
    .ok_or(AiError::ReleaseNotFound)
}

async fn release_commit_context(
    pool: &PgPool,
    repository_id: Uuid,
    previous_tag: Option<&str>,
    target_tag: &str,
) -> Result<String, AiError> {
    let previous_committed_at = match previous_tag {
        Some(tag) => Some(tag_committed_at(pool, repository_id, tag).await?),
        None => None,
    };
    let target_committed_at = tag_committed_at(pool, repository_id, target_tag).await?;
    let commits = sqlx::query(
        r#"
        SELECT oid, message
        FROM commits
        WHERE repository_id = $1
          AND ($2::timestamptz IS NULL OR committed_at > $2)
          AND committed_at <= $3
        ORDER BY committed_at DESC, created_at DESC
        LIMIT 60
        "#,
    )
    .bind(repository_id)
    .bind(previous_committed_at)
    .bind(target_committed_at)
    .fetch_all(pool)
    .await?;
    Ok(commits
        .into_iter()
        .map(|row| {
            format!(
                "{} {}",
                row.get::<String, _>("oid"),
                row.get::<String, _>("message")
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

async fn tag_committed_at(
    pool: &PgPool,
    repository_id: Uuid,
    tag: &str,
) -> Result<DateTime<Utc>, AiError> {
    sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        SELECT commits.committed_at
        FROM commits
        WHERE commits.repository_id = $1
          AND commits.id = COALESCE(
              (
                  SELECT releases.target_commit_id
                  FROM releases
                  WHERE releases.repository_id = $1
                    AND lower(releases.tag_name) = lower($2)
                    AND releases.deleted_at IS NULL
                    AND releases.target_commit_id IS NOT NULL
                  LIMIT 1
              ),
              (
                  SELECT refs.target_commit_id
                  FROM repository_git_refs refs
                  WHERE refs.repository_id = $1
                    AND lower(regexp_replace(refs.name, '^refs/tags/', '')) = lower($2)
                    AND refs.kind = 'tag'
                    AND refs.target_commit_id IS NOT NULL
                  LIMIT 1
              )
          )
        "#,
    )
    .bind(repository_id)
    .bind(clean_tag_name(tag))
    .fetch_optional(pool)
    .await?
    .ok_or(AiError::ReleaseNotFound)
}

fn clean_tag_name(tag: &str) -> String {
    tag.trim().trim_start_matches("refs/tags/").to_owned()
}

#[cfg(test)]
mod tests {
    use super::pr_author_inline_seed;
    use uuid::Uuid;

    #[test]
    fn pr_inline_comment_seed_is_author_only() {
        let author = Uuid::new_v4();
        let reader = Uuid::new_v4();

        assert!(pr_author_inline_seed(Some(author), author).is_some());
        assert_eq!(pr_author_inline_seed(Some(reader), author), None);
        assert_eq!(pr_author_inline_seed(None, author), None);
    }
}
