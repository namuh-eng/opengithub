use opengithub_api::jobs::{
    enqueue_job,
    worker::{run_once, WorkerConfig, EMAIL_FANOUT_QUEUE, RETENTION_CLEANUP_QUEUE},
};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

async fn database_pool() -> Option<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|value| !value.trim().is_empty())?;

    let pool = opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
        .ok()?;
    MIGRATOR.run(&pool).await.ok()?;
    Some(pool)
}

fn test_config() -> WorkerConfig {
    WorkerConfig {
        worker_id: format!("worker-test-{}", Uuid::new_v4()),
        poll_interval: Duration::from_millis(1),
        lease_seconds: 30,
        once: true,
    }
}

#[tokio::test]
async fn worker_processes_generic_email_fanout_lease() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping worker lease smoke; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let key = format!("missing-user-{}", Uuid::new_v4());
    let lease = enqueue_job(
        &pool,
        EMAIL_FANOUT_QUEUE,
        &key,
        json!({"kind": "smoke", "subject": "worker smoke"}),
    )
    .await
    .expect("job should enqueue");

    let stats = run_once(&pool, &test_config())
        .await
        .expect("worker run_once should succeed");
    assert_eq!(stats.processed, 1);

    let row =
        sqlx::query("SELECT completed_at, locked_by, locked_until FROM job_leases WHERE id = $1")
            .bind(lease.id)
            .fetch_one(&pool)
            .await
            .expect("lease should load");
    let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
    let locked_by: Option<String> = row.get("locked_by");
    let locked_until: Option<chrono::DateTime<chrono::Utc>> = row.get("locked_until");
    assert!(completed_at.is_some());
    assert!(locked_by.is_none());
    assert!(locked_until.is_none());
}

#[tokio::test]
async fn worker_processes_retention_cleanup_lease_without_aws() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping worker retention smoke; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };
    let key = format!("retention-smoke-{}", Uuid::new_v4());
    let lease = enqueue_job(&pool, RETENTION_CLEANUP_QUEUE, &key, json!({"smoke": true}))
        .await
        .expect("retention job should enqueue");

    let mut completed = None;
    for _ in 0..5 {
        let _stats = run_once(&pool, &test_config())
            .await
            .expect("worker retention should succeed");
        completed = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT completed_at FROM job_leases WHERE id = $1",
        )
        .bind(lease.id)
        .fetch_one(&pool)
        .await
        .expect("lease should load");
        if completed.is_some() {
            break;
        }
    }
    assert!(completed.is_some());
}
