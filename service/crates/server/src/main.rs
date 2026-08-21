use chassis::{config::Config, db};
use server::{router::app, state::AppState};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::from_env();
    let pool = db::connect(&cfg.database_url).await?;
    db::migrate(&pool).await?;

    let email: Arc<dyn chassis::connectors::email::EmailSender + Send + Sync> =
        Arc::new(chassis::connectors::email::MockEmailSender::new());

    let state = AppState { cfg, pool, email };

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", state.cfg.api_port)).await?;
    tracing::info!("server listening on {}", state.cfg.api_port);
    axum::serve(listener, app(state)).await?;
    Ok(())
}
