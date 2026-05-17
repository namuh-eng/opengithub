use opengithub_api::jobs::email_delivery::{
    enqueue_test_email, run_next_email_delivery, EmailDeliveryConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = EmailDeliveryConfig::from_env();
    config.validate()?;
    let pool = opengithub_api::db::pool_from_env()
        .await?
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required for the worker"))?;
    let worker_id = std::env::var("WORKER_ID")
        .unwrap_or_else(|_| format!("email-worker-{}", std::process::id()));

    let args: Vec<String> = std::env::args().collect();
    if let Some(recipient) = args
        .iter()
        .position(|arg| arg == "--send-test-email")
        .and_then(|idx| args.get(idx + 1))
        .cloned()
        .or_else(|| std::env::var("SES_TEST_RECIPIENT").ok())
    {
        let job = enqueue_test_email(&pool, &recipient).await?;
        tracing::info!(job_lease_id = %job.id, recipient = %recipient, "queued SES test email");
    }

    let once = args.iter().any(|arg| arg == "--once");
    loop {
        match run_next_email_delivery(&pool, &worker_id, &config).await? {
            Some(record) => {
                tracing::info!(delivery_id = %record.id, status = %record.status, provider = %record.provider, "email worker processed job")
            }
            None if once => break,
            None => tokio::time::sleep(std::time::Duration::from_secs(5)).await,
        }
        if once {
            break;
        }
    }

    Ok(())
}
