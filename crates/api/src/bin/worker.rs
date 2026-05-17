use opengithub_api::jobs::email_delivery::enqueue_test_email;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = opengithub_api::jobs::worker::WorkerConfig::from_env();
    let pool = opengithub_api::db::pool_from_env()
        .await?
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required for the worker service"))?;

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

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(opengithub_api::jobs::worker::shutdown_signal(shutdown_tx));
    opengithub_api::jobs::worker::run_until_shutdown(pool, config, shutdown_rx).await?;
    Ok(())
}
