use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Review {
    pub id: Uuid,
    pub clinic_id: Uuid,
    pub patient_user_id: Uuid,
    pub inquiry_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

fn map_db_error(e: sqlx::Error) -> ApiError {
    match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => ApiError::Conflict,
        _ => ApiError::Internal,
    }
}

/// Returns the id of a converted inquiry that makes this patient eligible to
/// review the given clinic, if one exists.
pub async fn can_review(
    pool: &DbPool,
    patient_user_id: Uuid,
    clinic_id: Uuid,
) -> Result<Option<Uuid>, ApiError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM inquiries
         WHERE patient_user_id = $1
           AND clinic_id = $2
           AND status = 'converted'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(patient_user_id)
    .bind(clinic_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

/// Create a verified review for a clinic. The caller must have already
/// confirmed the patient is eligible via `can_review`.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &DbPool,
    clinic_id: Uuid,
    patient_user_id: Uuid,
    inquiry_id: Uuid,
    rating: i32,
    comment: Option<&str>,
) -> Result<Review, ApiError> {
    if !(1..=5).contains(&rating) {
        return Err(ApiError::Validation("rating must be between 1 and 5".into()));
    }

    sqlx::query_as::<_, Review>(
        "INSERT INTO reviews (clinic_id, patient_user_id, inquiry_id, rating, comment)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, clinic_id, patient_user_id, inquiry_id, rating, comment, created_at",
    )
    .bind(clinic_id)
    .bind(patient_user_id)
    .bind(inquiry_id)
    .bind(rating)
    .bind(comment)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

/// List all reviews for a clinic, newest first.
pub async fn list_for_clinic(pool: &DbPool, clinic_id: Uuid) -> Result<Vec<Review>, ApiError> {
    sqlx::query_as::<_, Review>(
        "SELECT id, clinic_id, patient_user_id, inquiry_id, rating, comment, created_at
         FROM reviews
         WHERE clinic_id = $1
         ORDER BY created_at DESC",
    )
    .bind(clinic_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

/// Compute the average rating for a clinic, if any reviews exist.
pub async fn average_rating(pool: &DbPool, clinic_id: Uuid) -> Result<Option<f64>, ApiError> {
    sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(rating)::float8 FROM reviews WHERE clinic_id = $1",
    )
    .bind(clinic_id)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::Internal)
}
