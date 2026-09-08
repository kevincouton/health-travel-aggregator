//! Clinic-claim intents: a provider asserts ownership of an existing listing
//! (typically collector-ingested, owned by the system user). Admins resolve
//! claims; approving one transfers clinic ownership to the claimant.

use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "claim_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ClinicClaim {
    pub id: Uuid,
    pub clinic_id: Uuid,
    pub user_id: Uuid,
    pub message: Option<String>,
    pub status: ClaimStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

const CLAIM_COLS: &str = "id, clinic_id, user_id, message, status, created_at, resolved_at";

fn map_db_error(e: sqlx::Error) -> ApiError {
    match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => ApiError::Conflict,
        _ => ApiError::Internal,
    }
}

/// Record a claim intent. One open claim per (clinic, user) — duplicates
/// surface as Conflict.
pub async fn create(
    pool: &DbPool,
    clinic_id: Uuid,
    user_id: Uuid,
    message: Option<&str>,
) -> Result<ClinicClaim, ApiError> {
    sqlx::query_as::<_, ClinicClaim>(&format!(
        "INSERT INTO clinic_claims (clinic_id, user_id, message)
         VALUES ($1, $2, $3)
         RETURNING {CLAIM_COLS}"
    ))
    .bind(clinic_id)
    .bind(user_id)
    .bind(message)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_by_status(
    pool: &DbPool,
    status: ClaimStatus,
) -> Result<Vec<ClinicClaim>, ApiError> {
    sqlx::query_as::<_, ClinicClaim>(&format!(
        "SELECT {CLAIM_COLS} FROM clinic_claims WHERE status = $1 ORDER BY created_at DESC"
    ))
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

/// Resolve a pending claim. Approving transfers clinic ownership to the
/// claimant; both writes commit in one transaction.
pub async fn resolve(
    pool: &DbPool,
    id: Uuid,
    status: ClaimStatus,
) -> Result<Option<ClinicClaim>, ApiError> {
    if status == ClaimStatus::Pending {
        return Err(ApiError::Validation(
            "claim can only be resolved to approved or rejected".into(),
        ));
    }
    let mut tx = pool.begin().await.map_err(|_| ApiError::Internal)?;

    let claim = sqlx::query_as::<_, ClinicClaim>(&format!(
        "UPDATE clinic_claims
         SET status = $1, resolved_at = NOW()
         WHERE id = $2 AND status = 'pending'
         RETURNING {CLAIM_COLS}"
    ))
    .bind(status)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let Some(claim) = claim else {
        tx.rollback().await.map_err(|_| ApiError::Internal)?;
        return Ok(None);
    };

    if status == ClaimStatus::Approved {
        sqlx::query("UPDATE clinics SET owner_user_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(claim.user_id)
            .bind(claim.clinic_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
    }

    tx.commit().await.map_err(|_| ApiError::Internal)?;
    Ok(Some(claim))
}
