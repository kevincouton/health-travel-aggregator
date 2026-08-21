use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Package {
    pub id: Uuid,
    pub clinic_id: Uuid,
    pub treatment_id: Uuid,
    pub name: String,
    pub price_min: Option<i32>,
    pub price_max: Option<i32>,
    pub duration_days: Option<i32>,
    pub inclusions: Vec<String>,
    pub exclusions: Vec<String>,
    pub is_published: bool,
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
    clinic_id: Uuid,
    treatment_id: Uuid,
    name: &str,
    price_min: Option<i32>,
    price_max: Option<i32>,
    duration_days: Option<i32>,
    inclusions: &[String],
    exclusions: &[String],
) -> Result<Package, ApiError> {
    sqlx::query_as::<_, Package>(
        "INSERT INTO packages (clinic_id, treatment_id, name, price_min, price_max, duration_days, inclusions, exclusions)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, clinic_id, treatment_id, name, price_min, price_max, duration_days, inclusions, exclusions, is_published, created_at, updated_at",
    )
    .bind(clinic_id)
    .bind(treatment_id)
    .bind(name)
    .bind(price_min)
    .bind(price_max)
    .bind(duration_days)
    .bind(inclusions)
    .bind(exclusions)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &DbPool,
    id: Uuid,
    name: &str,
    price_min: Option<i32>,
    price_max: Option<i32>,
    duration_days: Option<i32>,
    inclusions: &[String],
    exclusions: &[String],
    is_published: bool,
) -> Result<Option<Package>, ApiError> {
    sqlx::query_as::<_, Package>(
        "UPDATE packages
         SET name = $1, price_min = $2, price_max = $3, duration_days = $4, inclusions = $5, exclusions = $6, is_published = $7, updated_at = NOW()
         WHERE id = $8
         RETURNING id, clinic_id, treatment_id, name, price_min, price_max, duration_days, inclusions, exclusions, is_published, created_at, updated_at",
    )
    .bind(name)
    .bind(price_min)
    .bind(price_max)
    .bind(duration_days)
    .bind(inclusions)
    .bind(exclusions)
    .bind(is_published)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM packages WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_db_error)?;
    Ok(result.rows_affected() > 0)
}

pub async fn by_id(pool: &DbPool, id: Uuid) -> Result<Option<Package>, ApiError> {
    sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn list_for_clinic(pool: &DbPool, clinic_id: Uuid) -> Result<Vec<Package>, ApiError> {
    sqlx::query_as::<_, Package>(
        "SELECT * FROM packages WHERE clinic_id = $1 ORDER BY created_at DESC",
    )
    .bind(clinic_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn list_public_for_clinic(
    pool: &DbPool,
    clinic_id: Uuid,
) -> Result<Vec<Package>, ApiError> {
    sqlx::query_as::<_, Package>(
        "SELECT * FROM packages WHERE clinic_id = $1 AND is_published = true ORDER BY created_at DESC",
    )
    .bind(clinic_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn list_public_for_treatment(
    pool: &DbPool,
    treatment_id: Uuid,
) -> Result<Vec<Package>, ApiError> {
    sqlx::query_as::<_, Package>(
        "SELECT * FROM packages WHERE treatment_id = $1 AND is_published = true ORDER BY created_at DESC",
    )
    .bind(treatment_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}
