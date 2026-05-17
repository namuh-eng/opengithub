use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = opengithub_api::config::AppConfig::from_env()?;

    let db = match opengithub_api::db::pool_from_env().await {
        Ok(pool) => pool,
        Err(error) => {
            tracing::warn!(%error, "starting with degraded database health");
            None
        }
    };
    let app = opengithub_api::build_app_with_config(db, config);

    let port = opengithub_api::config::api_port_from_env()?;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    tracing::info!("opengithub api listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
