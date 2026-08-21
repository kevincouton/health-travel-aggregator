use crate::{clinics, db::DbPool, error::ApiError, packages};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "inquiry_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InquiryStatus {
    New,
    Contacted,
    Converted,
    Closed,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Inquiry {
    pub id: Uuid,
    pub patient_user_id: Uuid,
    pub clinic_id: Uuid,
    pub package_id: Option<Uuid>,
    pub status: InquiryStatus,
    pub medical_notes: Option<String>,
    pub preferred_dates: Option<String>,
    pub contact_email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn map_db_error(e: sqlx::Error) -> ApiError {
    match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => ApiError::Conflict,
        _ => ApiError::Internal,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &DbPool,
    patient_user_id: Uuid,
    clinic_id: Uuid,
    package_id: Option<Uuid>,
    medical_notes: Option<&str>,
    preferred_dates: Option<&str>,
    contact_email: &str,
) -> Result<Inquiry, ApiError> {
    let contact_email = contact_email.trim();
    if contact_email.is_empty() {
        return Err(ApiError::Validation("contact_email is required".into()));
    }

    let clinic = clinics::by_id(pool, clinic_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if let Some(package_id) = package_id {
        let package = packages::by_id(pool, package_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        if package.clinic_id != clinic.id {
            return Err(ApiError::BadRequest);
        }
    }

    sqlx::query_as::<_, Inquiry>(
        "INSERT INTO inquiries (patient_user_id, clinic_id, package_id, medical_notes, preferred_dates, contact_email)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, patient_user_id, clinic_id, package_id, status, medical_notes, preferred_dates, contact_email, created_at, updated_at",
    )
    .bind(patient_user_id)
    .bind(clinic_id)
    .bind(package_id)
    .bind(medical_notes)
    .bind(preferred_dates)
    .bind(contact_email)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_for_patient(pool: &DbPool, patient_user_id: Uuid) -> Result<Vec<Inquiry>, ApiError> {
    sqlx::query_as::<_, Inquiry>(
        "SELECT * FROM inquiries WHERE patient_user_id = $1 ORDER BY created_at DESC",
    )
    .bind(patient_user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn list_for_provider(
    pool: &DbPool,
    provider_user_id: Uuid,
) -> Result<Vec<Inquiry>, ApiError> {
    sqlx::query_as::<_, Inquiry>(
        "SELECT i.* FROM inquiries i
         WHERE i.clinic_id IN (SELECT c.id FROM clinics c WHERE c.owner_user_id = $1)
         ORDER BY i.created_at DESC",
    )
    .bind(provider_user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn update_status(
    pool: &DbPool,
    inquiry_id: Uuid,
    provider_user_id: Uuid,
    status: InquiryStatus,
) -> Result<Option<Inquiry>, ApiError> {
    sqlx::query_as::<_, Inquiry>(
        "UPDATE inquiries
         SET status = $1, updated_at = NOW()
         WHERE id = $2
           AND clinic_id IN (SELECT c.id FROM clinics c WHERE c.owner_user_id = $3)
         RETURNING id, patient_user_id, clinic_id, package_id, status, medical_notes, preferred_dates, contact_email, created_at, updated_at",
    )
    .bind(status)
    .bind(inquiry_id)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn by_id(pool: &DbPool, id: Uuid) -> Result<Option<Inquiry>, ApiError> {
    sqlx::query_as::<_, Inquiry>("SELECT * FROM inquiries WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}
