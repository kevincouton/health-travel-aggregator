//! Postgres ingestion: normalize/dedup and upsert collected records using
//! the platform schema. New listings land in `pending` moderation status;
//! re-runs are idempotent via the (source, external_ref) unique index and
//! never reset an already-moderated clinic's status.

use crate::{model::CollectedClinic, sources::slugify};
use sqlx::{Pool, Postgres, Row};
use thiserror::Error;
use uuid::Uuid;

/// Deterministic owner for collector-ingested clinics until a provider
/// claims the listing through the verification flow.
pub const SYSTEM_USER_EMAIL: &str = "collector@health-travel.internal";

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("record has empty fields: {0}")]
    Invalid(String),
}

#[derive(Debug, Default, Clone)]
pub struct SourceStats {
    pub source: String,
    pub collected: usize,
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
}

pub type DbPool = Pool<Postgres>;

/// Ensure the system user that owns collector-ingested clinics exists.
pub async fn ensure_system_user(pool: &DbPool) -> Result<Uuid, IngestError> {
    let row = sqlx::query(
        "INSERT INTO users (email, role, display_name)
         VALUES ($1, 'provider_admin', 'Data Collector (automated)')
         ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
         RETURNING id",
    )
    .bind(SYSTEM_USER_EMAIL)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

/// Upsert a city into the locations reference table.
pub async fn upsert_location(pool: &DbPool, clinic: &CollectedClinic) -> Result<(), IngestError> {
    sqlx::query(
        "INSERT INTO locations (country_code, country_name, city)
         VALUES ($1, $2, $3)
         ON CONFLICT (country_code, city) DO NOTHING",
    )
    .bind(&clinic.country_code)
    .bind(&clinic.country_name)
    .bind(&clinic.city)
    .execute(pool)
    .await?;
    Ok(())
}

/// Pick a slug not already used by a different clinic record.
async fn available_slug(pool: &DbPool, clinic: &CollectedClinic) -> Result<String, IngestError> {
    let base = slugify(&clinic.name);
    let base = if base.is_empty() {
        format!("clinic-{}", clinic.source)
    } else {
        base
    };
    for attempt in 0..100 {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            format!("{base}-{attempt}")
        };
        let taken: Option<(String, Option<String>)> =
            sqlx::query_as("SELECT source, external_ref FROM clinics WHERE slug = $1")
                .bind(&candidate)
                .fetch_optional(pool)
                .await?;
        match taken {
            None => return Ok(candidate),
            Some((src, ext)) => {
                let same_record =
                    src == clinic.source && ext.as_deref() == Some(clinic.external_ref.as_str());
                if same_record {
                    return Ok(candidate);
                }
            }
        }
    }
    Err(IngestError::Invalid(format!(
        "no free slug for {}",
        clinic.name
    )))
}

/// Upsert one record. Returns true when a new row was inserted.
pub async fn upsert_clinic(
    pool: &DbPool,
    owner_user_id: Uuid,
    clinic: &CollectedClinic,
) -> Result<bool, IngestError> {
    if clinic.name.is_empty() || clinic.city.is_empty() || clinic.country_code.len() != 2 {
        return Err(IngestError::Invalid(format!(
            "{}/{}",
            clinic.source, clinic.external_ref
        )));
    }

    let description = clinic
        .description
        .as_deref()
        .map(|d| format!("{d} Source: {} ({}).", clinic.source_url, clinic.source));

    // Existing record from this source/ref: refresh mutable fields only.
    // Status is deliberately untouched so moderation decisions survive
    // re-ingestion.
    let updated = sqlx::query(
        "UPDATE clinics
         SET name = $1, country_code = $2, city = $3, accreditations = $4,
             description = $5, updated_at = NOW()
         WHERE source = $6 AND external_ref = $7",
    )
    .bind(&clinic.name)
    .bind(&clinic.country_code)
    .bind(&clinic.city)
    .bind(&clinic.accreditations)
    .bind(description.as_deref())
    .bind(clinic.source)
    .bind(&clinic.external_ref)
    .execute(pool)
    .await?;
    if updated.rows_affected() > 0 {
        return Ok(false);
    }

    let slug = available_slug(pool, clinic).await?;
    sqlx::query(
        "INSERT INTO clinics
           (owner_user_id, name, slug, country_code, city, accreditations, description, status, source, external_ref)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9)",
    )
    .bind(owner_user_id)
    .bind(&clinic.name)
    .bind(&slug)
    .bind(&clinic.country_code)
    .bind(&clinic.city)
    .bind(&clinic.accreditations)
    .bind(description.as_deref())
    .bind(clinic.source)
    .bind(&clinic.external_ref)
    .execute(pool)
    .await?;
    Ok(true)
}

/// Ingest a batch of records from one collector.
pub async fn ingest_batch(
    pool: &DbPool,
    owner_user_id: Uuid,
    clinics: &[CollectedClinic],
) -> Result<SourceStats, IngestError> {
    let mut stats = SourceStats {
        source: clinics
            .first()
            .map(|c| c.source.to_string())
            .unwrap_or_default(),
        collected: clinics.len(),
        ..Default::default()
    };
    for clinic in clinics {
        if let Err(e) = upsert_location(pool, clinic).await {
            tracing::warn!(error = %e, r#ref = clinic.external_ref, "location upsert failed");
        }
        match upsert_clinic(pool, owner_user_id, clinic).await {
            Ok(true) => stats.inserted += 1,
            Ok(false) => stats.updated += 1,
            Err(e) => {
                tracing::warn!(error = %e, r#ref = clinic.external_ref, "clinic upsert skipped");
                stats.skipped += 1;
            }
        }
    }
    Ok(stats)
}
