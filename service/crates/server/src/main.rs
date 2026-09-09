use chassis::{config::Config, db};
use server::{router::app, state::AppState};

/// Error tracking (GlitchTip, Sentry-compatible). Disabled cleanly when
/// SENTRY_DSN is unset so dev/CI is unaffected; panic capture is on via the
/// "panic" feature. The guard must outlive main so queued events flush.
fn init_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").unwrap_or_default();
    if dsn.is_empty() {
        return None;
    }
    // ClientOptions is #[non_exhaustive]: mutate defaults, no FRU.
    let mut opts = sentry::ClientOptions::default();
    opts.release = sentry::release_name!();
    Some(sentry::init((dsn, opts)))
}

/// One-shot end-to-end verification hook: SENTRY_SELF_TEST=1 fires a single
/// error-level message shortly after startup. Never set in production.
fn start_sentry_self_test() {
    if std::env::var("SENTRY_SELF_TEST").as_deref() == Ok("1") {
        tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            sentry::capture_message("sentry wiring self-test", sentry::Level::Error);
        });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _sentry_guard = init_sentry();
    // fmt layer (INFO default) plus the sentry layer, which captures
    // error-level events as Sentry errors and lower levels as breadcrumbs.
    // No-op when the client is disabled.
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    let cfg = Config::from_env();
    let pool = db::connect(&cfg.database_url).await?;
    db::migrate(&pool).await?;
    // Idempotent; guarantees the Basic/Pro/Enterprise catalog exists for
    // plan-limit enforcement and checkout.
    chassis::subscriptions::seed_plans(&pool).await?;

    start_sentry_self_test();

    // Connector selection: real SMTP/Stripe when their env keys are set,
    // mocks otherwise so dev/CI run with no external dependencies.
    let email = chassis::connectors::email::sender_from_env();
    let (payments, stripe) = chassis::connectors::stripe::provider_from_env();

    let state = AppState {
        cfg,
        pool,
        email,
        payments,
        stripe,
    };

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", state.cfg.api_port)).await?;
    tracing::info!("server listening on {}", state.cfg.api_port);
    axum::serve(listener, app(state)).await?;
    Ok(())
}
