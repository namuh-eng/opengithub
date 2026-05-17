use opengithub_api::jobs::{
    email_delivery::{run_email_delivery_once, EmailDeliveryConfig, EmailProviderKind},
    enqueue_job,
};
use sqlx::PgPool;
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

#[tokio::test]
async fn email_worker_sends_local_noop_and_persists_delivery_status() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping email worker scenario; set TEST_DATABASE_URL");
        return;
    };

    let lease_key = format!("email-smoke:{}", Uuid::new_v4());
    let job = enqueue_job(
        &pool,
        "email_delivery",
        &lease_key,
        serde_json::json!({
            "kind": "test",
            "to": "Smoke@Test.OpenGitHub.Local",
            "subject": "Worker email smoke",
            "body": "The local noop provider accepted this message."
        }),
    )
    .await
    .expect("email job should enqueue");

    let record = run_email_delivery_once(
        &pool,
        &lease_key,
        "email-worker-test",
        &EmailDeliveryConfig::local_noop(),
    )
    .await
    .expect("email worker should not crash")
    .expect("job should be processed");

    assert_eq!(record.job_lease_id, job.id);
    assert_eq!(record.recipient, "smoke@test.opengithub.local");
    assert_eq!(record.status, "sent");
    assert_eq!(record.provider, "noop");

    let completed_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT completed_at FROM job_leases WHERE id = $1")
            .bind(job.id)
            .fetch_one(&pool)
            .await
            .expect("lease should exist");
    assert!(completed_at.is_some());
}

#[tokio::test]
async fn email_worker_persists_failures_without_crashing_or_completing_job() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping email worker failure scenario; set TEST_DATABASE_URL");
        return;
    };

    let lease_key = format!("email-failure:{}", Uuid::new_v4());
    let job = enqueue_job(
        &pool,
        "email_delivery",
        &lease_key,
        serde_json::json!({
            "kind": "test",
            "to": "not-an-email",
            "subject": "Invalid email smoke",
            "body": "This payload should fail safely."
        }),
    )
    .await
    .expect("email job should enqueue");

    let record = run_email_delivery_once(
        &pool,
        &lease_key,
        "email-worker-failure-test",
        &EmailDeliveryConfig {
            provider: EmailProviderKind::Noop,
            from_address: None,
            aws_region: None,
            configuration_set: None,
        },
    )
    .await
    .expect("email worker should persist payload failure")
    .expect("job should be processed");

    assert_eq!(record.job_lease_id, job.id);
    assert_eq!(record.status, "failed");
    assert_eq!(record.error_code.as_deref(), Some("invalid_payload"));

    let row: (Option<chrono::DateTime<chrono::Utc>>, Option<String>) =
        sqlx::query_as("SELECT completed_at, last_error FROM job_leases WHERE id = $1")
            .bind(job.id)
            .fetch_one(&pool)
            .await
            .expect("lease should exist");
    assert!(row.0.is_none());
    assert_eq!(row.1.as_deref(), Some("invalid_payload"));
}
