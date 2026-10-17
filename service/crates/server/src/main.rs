use chassis::{config::Config, db};
use server::{router::app, state::AppState};

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
    let cfg = Config::from_env();
    // Error tracking (GlitchTip, Sentry-compatible). Disabled cleanly when
    // SENTRY_DSN is empty so dev/CI is unaffected; the guard must outlive
    // main so queued events flush.
    let _sentry = chassis::observability::init(
        &cfg.sentry_dsn,
        Some(
            concat!(
                "health-travel-aggregator-server@",
                env!("CARGO_PKG_VERSION")
            )
            .to_string(),
        ),
    );
    // fmt layer (INFO default). With Sentry enabled, the sentry layer also
    // captures error-level events as Sentry errors and lower levels as
    // breadcrumbs; without it, keep the plain fmt subscriber.
    use tracing_subscriber::prelude::*;
    if _sentry.is_some() {
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(tracing_subscriber::fmt::layer())
            .with(sentry::integrations::tracing::layer())
            .init();
    } else {
        tracing_subscriber::fmt().init();
    }

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
