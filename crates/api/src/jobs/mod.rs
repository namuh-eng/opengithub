use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub mod pages;
pub mod repository_imports;
pub mod webhooks;
pub mod worker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobLease {
    pub id: Uuid,
    pub queue: String,
    pub lease_key: String,
    pub payload: Value,
    pub locked_by: Option<String>,
    pub locked_until: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum JobLeaseError {
    #[error("job lease was not found")]
    NotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn enqueue_job(
    pool: &PgPool,
    queue: &str,
    lease_key: &str,
    payload: Value,
) -> Result<JobLease, JobLeaseError> {
    let row = sqlx::query(
        r#"
        INSERT INTO job_leases (queue, lease_key, payload)
        VALUES ($1, $2, $3)
        ON CONFLICT (queue, lease_key) DO UPDATE SET payload = EXCLUDED.payload
        RETURNING id, queue, lease_key, payload, locked_by, locked_until, attempts,
                  last_error, completed_at, created_at, updated_at
        "#,
    )
    .bind(queue)
    .bind(lease_key)
    .bind(payload)
    .fetch_one(pool)
    .await?;

    Ok(job_lease_from_row(row))
}

pub async fn acquire_job_lease(
    pool: &PgPool,
    queue: &str,
    lease_key: &str,
    worker_id: &str,
    lease_seconds: i64,
) -> Result<Option<JobLease>, JobLeaseError> {
    let row = sqlx::query(
        r#"
        UPDATE job_leases
        SET locked_by = $3,
            locked_until = now() + ($4::bigint * interval '1 second'),
            attempts = attempts + 1,
            last_error = NULL
        WHERE queue = $1
          AND lease_key = $2
          AND completed_at IS NULL
          AND (locked_until IS NULL OR locked_until <= now() OR locked_by = $3)
        RETURNING id, queue, lease_key, payload, locked_by, locked_until, attempts,
                  last_error, completed_at, created_at, updated_at
        "#,
    )
    .bind(queue)
    .bind(lease_key)
    .bind(worker_id)
    .bind(lease_seconds)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(job_lease_from_row))
}

pub async fn acquire_next_job_lease(
    pool: &PgPool,
    queue: &str,
    worker_id: &str,
    lease_seconds: i64,
) -> Result<Option<JobLease>, JobLeaseError> {
    let row = sqlx::query(
        r#"
        WITH candidate AS (
            SELECT id
            FROM job_leases
            WHERE queue = $1
              AND completed_at IS NULL
              AND (locked_until IS NULL OR locked_until <= now() OR locked_by = $2)
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE job_leases
        SET locked_by = $2,
            locked_until = now() + ($3::bigint * interval '1 second'),
            attempts = attempts + 1,
            last_error = NULL
        WHERE id = (SELECT id FROM candidate)
        RETURNING id, queue, lease_key, payload, locked_by, locked_until, attempts,
                  last_error, completed_at, created_at, updated_at
        "#,
    )
    .bind(queue)
    .bind(worker_id)
    .bind(lease_seconds)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(job_lease_from_row))
}

pub async fn complete_job_lease(
    pool: &PgPool,
    lease_id: Uuid,
    worker_id: &str,
) -> Result<JobLease, JobLeaseError> {
    let row = sqlx::query(
        r#"
        UPDATE job_leases
        SET completed_at = now(), locked_by = NULL, locked_until = NULL
        WHERE id = $1 AND locked_by = $2
        RETURNING id, queue, lease_key, payload, locked_by, locked_until, attempts,
                  last_error, completed_at, created_at, updated_at
        "#,
    )
    .bind(lease_id)
    .bind(worker_id)
    .fetch_optional(pool)
    .await?
    .ok_or(JobLeaseError::NotFound)?;

    Ok(job_lease_from_row(row))
}

pub async fn fail_job_lease(
    pool: &PgPool,
    lease_id: Uuid,
    worker_id: &str,
    error: &str,
    retry_after_seconds: i64,
) -> Result<JobLease, JobLeaseError> {
    let row = sqlx::query(
        r#"
        UPDATE job_leases
        SET locked_by = NULL,
            locked_until = now() + ($4::bigint * interval '1 second'),
            last_error = $3
        WHERE id = $1 AND locked_by = $2
        RETURNING id, queue, lease_key, payload, locked_by, locked_until, attempts,
                  last_error, completed_at, created_at, updated_at
        "#,
    )
    .bind(lease_id)
    .bind(worker_id)
    .bind(error)
    .bind(retry_after_seconds)
    .fetch_optional(pool)
    .await?
    .ok_or(JobLeaseError::NotFound)?;

    Ok(job_lease_from_row(row))
}

fn job_lease_from_row(row: sqlx::postgres::PgRow) -> JobLease {
    JobLease {
        id: row.get("id"),
        queue: row.get("queue"),
        lease_key: row.get("lease_key"),
        payload: row.get("payload"),
        locked_by: row.get("locked_by"),
        locked_until: row.get("locked_until"),
        attempts: row.get("attempts"),
        last_error: row.get("last_error"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
