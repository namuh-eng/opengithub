use std::{env, time::Duration};

use serde_json::json;
use sqlx::PgPool;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use crate::jobs::{
    acquire_next_job_lease, complete_job_lease, fail_job_lease, repository_imports, webhooks,
    JobLease, JobLeaseError,
};

pub const REPOSITORY_IMPORT_QUEUE: &str = "repository_import";
pub const WEBHOOK_RETRY_QUEUE: &str = "webhook-delivery";
pub const EMAIL_FANOUT_QUEUE: &str = "email_delivery";
pub const RETENTION_CLEANUP_QUEUE: &str = "retention-cleanup";

const DEFAULT_POLL_INTERVAL_MS: u64 = 1_000;
const DEFAULT_LEASE_SECONDS: i64 = 120;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub poll_interval: Duration,
    pub lease_seconds: i64,
    pub once: bool,
}

impl WorkerConfig {
    pub fn from_env() -> Self {
        Self {
            worker_id: env::var("WORKER_ID")
                .unwrap_or_else(|_| format!("worker-{}", Uuid::new_v4())),
            poll_interval: Duration::from_millis(
                env::var("WORKER_POLL_INTERVAL_MS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(DEFAULT_POLL_INTERVAL_MS),
            ),
            lease_seconds: env::var("WORKER_LEASE_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(DEFAULT_LEASE_SECONDS),
            once: env::var("WORKER_RUN_ONCE")
                .ok()
                .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WorkerRunStats {
    pub processed: u64,
    pub idle: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    RepositoryImport(#[from] repository_imports::RepositoryImportWorkerError),
    #[error(transparent)]
    Webhook(#[from] webhooks::WebhookDeliveryWorkerError),
}

pub async fn run_until_shutdown(
    pool: PgPool,
    config: WorkerConfig,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), WorkerError> {
    tracing::info!(worker_id = %config.worker_id, once = config.once, "opengithub worker started");
    loop {
        schedule_periodic_retention(&pool).await?;
        let stats = run_once(&pool, &config).await?;
        if config.once {
            tracing::info!(
                processed = stats.processed,
                idle = stats.idle,
                "worker one-shot completed"
            );
            return Ok(());
        }
        if stats.processed == 0 {
            tokio::select! {
                _ = shutdown.changed() => {
                    tracing::info!(worker_id = %config.worker_id, "worker shutdown requested");
                    return Ok(());
                }
                _ = sleep(config.poll_interval) => {}
            }
        }
    }
}

pub async fn run_once(pool: &PgPool, config: &WorkerConfig) -> Result<WorkerRunStats, WorkerError> {
    if repository_imports::run_next_repository_import(pool, &config.worker_id)
        .await?
        .is_some()
    {
        return Ok(WorkerRunStats {
            processed: 1,
            idle: false,
        });
    }
    if webhooks::run_next_webhook_delivery(pool, &config.worker_id)
        .await?
        .is_some()
    {
        return Ok(WorkerRunStats {
            processed: 1,
            idle: false,
        });
    }
    if process_next_generic_queue(
        pool,
        EMAIL_FANOUT_QUEUE,
        &config.worker_id,
        config.lease_seconds,
    )
    .await?
    {
        return Ok(WorkerRunStats {
            processed: 1,
            idle: false,
        });
    }
    if process_next_generic_queue(
        pool,
        RETENTION_CLEANUP_QUEUE,
        &config.worker_id,
        config.lease_seconds,
    )
    .await?
    {
        return Ok(WorkerRunStats {
            processed: 1,
            idle: false,
        });
    }
    Ok(WorkerRunStats {
        processed: 0,
        idle: true,
    })
}

async fn process_next_generic_queue(
    pool: &PgPool,
    queue: &str,
    worker_id: &str,
    lease_seconds: i64,
) -> Result<bool, WorkerError> {
    let Some(lease) = acquire_next_job_lease(pool, queue, worker_id, lease_seconds).await? else {
        return Ok(false);
    };
    let result = match queue {
        EMAIL_FANOUT_QUEUE => process_email_fanout(pool, &lease).await,
        RETENTION_CLEANUP_QUEUE => process_retention_cleanup(pool).await,
        _ => Ok(()),
    };
    match result {
        Ok(()) => {
            complete_job_lease(pool, lease.id, worker_id).await?;
            tracing::info!(queue = %queue, lease_key = %lease.lease_key, job_id = %lease.id, "worker completed job");
        }
        Err(error) => {
            let message = error.to_string();
            fail_job_lease(
                pool,
                lease.id,
                worker_id,
                &message,
                retry_after_seconds(lease.attempts),
            )
            .await?;
            tracing::warn!(queue = %queue, lease_key = %lease.lease_key, job_id = %lease.id, %message, "worker failed job");
        }
    }
    Ok(true)
}

async fn process_email_fanout(pool: &PgPool, lease: &JobLease) -> Result<(), WorkerError> {
    let Some(user_id) = lease
        .payload
        .get("userId")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        tracing::info!(job_id = %lease.id, "email fanout job has no userId; marking as degraded no-op");
        return Ok(());
    };
    let preference_key = lease
        .payload
        .get("preferenceKey")
        .and_then(|v| v.as_str())
        .unwrap_or("background_job");
    let subject_type = lease
        .payload
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("background_job");
    sqlx::query(
        r#"
        INSERT INTO notification_email_deliveries (user_id, preference_key, subject_type, delivery_status, provider_message_id)
        VALUES ($1, $2, $3, 'degraded', $4)
        "#,
    )
    .bind(user_id)
    .bind(preference_key)
    .bind(subject_type)
    .bind(format!("local-worker:{}", lease.id))
    .execute(pool)
    .await?;
    tracing::info!(job_id = %lease.id, user_id = %user_id, "email fanout recorded degraded delivery placeholder");
    Ok(())
}

async fn process_retention_cleanup(pool: &PgPool) -> Result<(), WorkerError> {
    let expired_request_logs =
        sqlx::query("DELETE FROM api_request_logs WHERE retention_expires_at <= now()")
            .execute(pool)
            .await?
            .rows_affected();
    let old_completed_jobs = sqlx::query(
        "DELETE FROM job_leases WHERE completed_at IS NOT NULL AND completed_at < now() - interval '30 days'",
    )
    .execute(pool)
    .await?
    .rows_affected();
    let expired_artifacts = sqlx::query(
        "UPDATE workflow_artifacts SET deleted_at = COALESCE(deleted_at, now()) WHERE expired_at <= now() AND deleted_at IS NULL",
    )
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .unwrap_or(0);
    tracing::info!(
        expired_request_logs,
        old_completed_jobs,
        expired_artifacts,
        "retention cleanup completed"
    );
    Ok(())
}

async fn schedule_periodic_retention(pool: &PgPool) -> Result<(), WorkerError> {
    let key: String =
        sqlx::query_scalar("SELECT to_char(date_trunc('hour', now()), 'YYYYMMDDHH24')")
            .fetch_one(pool)
            .await?;
    crate::jobs::enqueue_job(
        pool,
        RETENTION_CLEANUP_QUEUE,
        &key,
        json!({"scheduled": true}),
    )
    .await?;
    Ok(())
}

fn retry_after_seconds(attempts: i32) -> i64 {
    match attempts {
        count if count <= 1 => 30,
        2 => 120,
        _ => 300,
    }
}

pub async fn shutdown_signal(sender: tokio::sync::watch::Sender<bool>) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        else {
            return;
        };
        signal.recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
    let _ = sender.send(true);
    let _ = timeout(Duration::from_secs(1), sleep(Duration::from_millis(10))).await;
}
