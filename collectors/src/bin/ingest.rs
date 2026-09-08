//! Ingestion pipeline entrypoint: run all (or selected) collectors and upsert
//! results into Postgres.
//!
//! Usage:
//!   DATABASE_URL=postgres://user:pass@host:5432/db cargo run -p collectors --bin ingest [source...]
//!
//! With no positional args all collectors run. Each collector performs a
//! single listing-page fetch; re-runs are idempotent.

use anyhow::{Context, Result};
use collectors::{
    collector::{http_client, Collector},
    ingest::{ensure_system_user, ingest_batch, SourceStats},
    sources::{gha::GhaCollector, mhtc::MhtcCollector, temos::TemosCollector},
};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

static MIGRATOR: Migrator = sqlx::migrate!("../service/migrations");

fn registry() -> Vec<Box<dyn Collector>> {
    vec![
        Box::new(MhtcCollector),
        Box::new(TemosCollector),
        Box::new(GhaCollector),
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;
    let wanted: Vec<String> = std::env::args().skip(1).collect();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    MIGRATOR.run(&pool).await.context("migrations failed")?;

    let owner = ensure_system_user(&pool).await?;
    tracing::info!(%owner, "collector system user ready");

    let client = http_client().context("failed to build HTTP client")?;

    let mut all_stats: Vec<SourceStats> = Vec::new();
    for collector in registry() {
        let source = collector.source();
        if !wanted.is_empty() && !wanted.iter().any(|w| w == source) {
            continue;
        }
        tracing::info!(source, "running collector");
        match collector.collect(&client).await {
            Ok(clinics) => {
                tracing::info!(source, collected = clinics.len(), "collection complete");
                let stats = ingest_batch(&pool, owner, &clinics).await?;
                tracing::info!(
                    source,
                    inserted = stats.inserted,
                    updated = stats.updated,
                    skipped = stats.skipped,
                    "ingest complete"
                );
                all_stats.push(stats);
            }
            Err(e) => {
                // One source failing must not lose the others' data.
                tracing::error!(source, error = %e, "collector failed");
            }
        }
    }

    println!("\n=== ingest summary ===");
    for s in &all_stats {
        println!(
            "{:8} collected={:<4} inserted={:<4} updated={:<4} skipped={}",
            s.source, s.collected, s.inserted, s.updated, s.skipped
        );
    }
    Ok(())
}
