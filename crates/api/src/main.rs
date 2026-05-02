use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let db = match opengithub_api::db::pool_from_env().await {
        Ok(pool) => pool,
        Err(error) => {
            tracing::warn!(%error, "starting with degraded database health");
            None
        }
    };
    if let Some(pool) = db.clone() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(error) = opengithub_api::domain::webhooks::process_due_deliveries(&pool, 25).await {
                    tracing::warn!(%error, "webhook delivery worker tick failed");
                }
            }
        });
    }
    let app = opengithub_api::build_app(db);

    let addr: SocketAddr = "0.0.0.0:3016".parse()?;
    tracing::info!("opengithub api listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
