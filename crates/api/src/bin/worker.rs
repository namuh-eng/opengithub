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

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(opengithub_api::jobs::worker::shutdown_signal(shutdown_tx));
    opengithub_api::jobs::worker::run_until_shutdown(pool, config, shutdown_rx).await?;
    Ok(())
}
