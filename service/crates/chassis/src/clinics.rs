use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "clinic_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ClinicStatus {
    Draft,
    Pending,
    Approved,
    Suspended,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Clinic {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub country_code: String,
    pub city: String,
    pub accreditations: Vec<String>,
    pub description: Option<String>,
    pub status: ClinicStatus,
    pub created_at: DateTime<Utc>,
}

const SELECT_CLINIC: &str =
    "SELECT id, owner_user_id, name, slug, country_code, city, accreditations, description, status, created_at FROM clinics";

fn map_db_error(e: sqlx::Error) -> ApiError {
    match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => ApiError::Conflict,
        _ => ApiError::Internal,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &DbPool,
    owner_user_id: Uuid,
    name: &str,
    slug: &str,
    country_code: &str,
    city: &str,
    accreditations: &[String],
    description: Option<&str>,
) -> Result<Clinic, ApiError> {
    sqlx::query_as::<_, Clinic>(
        "INSERT INTO clinics (owner_user_id, name, slug, country_code, city, accreditations, description)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, owner_user_id, name, slug, country_code, city, accreditations, description, status, created_at",
    )
    .bind(owner_user_id)
    .bind(name)
    .bind(slug)
    .bind(country_code)
    .bind(city)
    .bind(accreditations)
    .bind(description)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &DbPool,
    id: Uuid,
    owner_user_id: Uuid,
    name: &str,
    slug: &str,
    country_code: &str,
    city: &str,
    accreditations: &[String],
    description: Option<&str>,
) -> Result<Option<Clinic>, ApiError> {
    let row = sqlx::query_as::<_, Clinic>(
        "UPDATE clinics
         SET name = $1, slug = $2, country_code = $3, city = $4, accreditations = $5, description = $6, updated_at = NOW()
         WHERE id = $7 AND owner_user_id = $8
         RETURNING id, owner_user_id, name, slug, country_code, city, accreditations, description, status, created_at",
    )
    .bind(name)
    .bind(slug)
    .bind(country_code)
    .bind(city)
    .bind(accreditations)
    .bind(description)
    .bind(id)
    .bind(owner_user_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;

    match row {
        Some(clinic) => Ok(Some(clinic)),
        None => Err(ApiError::NotFound),
    }
}

pub async fn by_id(pool: &DbPool, id: Uuid) -> Result<Option<Clinic>, ApiError> {
    sqlx::query_as::<_, Clinic>("SELECT * FROM clinics WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn by_slug(pool: &DbPool, slug: &str) -> Result<Option<Clinic>, ApiError> {
    sqlx::query_as::<_, Clinic>("SELECT * FROM clinics WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn list_for_owner(pool: &DbPool, owner_user_id: Uuid) -> Result<Vec<Clinic>, ApiError> {
    sqlx::query_as::<_, Clinic>(
        "SELECT * FROM clinics WHERE owner_user_id = $1 ORDER BY created_at DESC",
    )
    .bind(owner_user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn list_public(
    pool: &DbPool,
    country_code: Option<&str>,
    city: Option<&str>,
) -> Result<Vec<Clinic>, ApiError> {
    let mut builder = sqlx::QueryBuilder::new(SELECT_CLINIC);
    builder.push(" WHERE status = 'approved'");

    if let Some(country_code) = country_code {
        builder.push(" AND country_code = ");
        builder.push_bind(country_code);
    }

    if let Some(city) = city {
        builder.push(" AND city = ");
        builder.push_bind(city);
    }

    builder.push(" ORDER BY created_at DESC");

    builder
        .build_query_as::<Clinic>()
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal)
}
