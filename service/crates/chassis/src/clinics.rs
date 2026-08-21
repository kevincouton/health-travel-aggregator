use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ClinicFilters {
    pub treatment: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
    pub accreditation: Option<String>,
    pub q: Option<String>,
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

const SEARCH_SELECT: &str = "SELECT DISTINCT c.id, c.owner_user_id, c.name, c.slug, c.country_code, c.city, c.accreditations, c.description, c.status, c.created_at FROM clinics c";

fn push_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    filters: &'a ClinicFilters,
    q_pattern: Option<&'a str>,
) {
    builder.push(" WHERE c.status = 'approved'");

    if let Some(treatment) = &filters.treatment {
        builder.push(" AND EXISTS (SELECT 1 FROM packages p JOIN treatments t ON t.id = p.treatment_id WHERE p.clinic_id = c.id AND p.is_published = true AND t.slug = ");
        builder.push_bind(treatment);
        builder.push(")");
    }

    if let Some(country) = &filters.country {
        builder.push(" AND c.country_code = ");
        builder.push_bind(country);
    }

    if let Some(city) = &filters.city {
        builder.push(" AND c.city = ");
        builder.push_bind(city);
    }

    if filters.min_price.is_some() || filters.max_price.is_some() {
        builder.push(" AND EXISTS (SELECT 1 FROM packages p WHERE p.clinic_id = c.id AND p.is_published = true");
        if let Some(min_price) = filters.min_price {
            builder.push(" AND p.price_max >= ");
            builder.push_bind(min_price);
        }
        if let Some(max_price) = filters.max_price {
            builder.push(" AND p.price_min <= ");
            builder.push_bind(max_price);
        }
        builder.push(")");
    }

    if let Some(accreditation) = &filters.accreditation {
        builder.push(" AND c.accreditations && ARRAY[");
        builder.push_bind(accreditation);
        builder.push("]");
    }

    if let Some(pattern) = q_pattern {
        builder.push(" AND (c.name ILIKE ");
        builder.push_bind(pattern);
        builder.push(" OR c.description ILIKE ");
        builder.push_bind(pattern);
        builder.push(" OR c.city ILIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }
}

pub async fn search(
    pool: &DbPool,
    filters: ClinicFilters,
    page: i64,
    per_page: i64,
) -> Result<(Vec<Clinic>, i64), ApiError> {
    let per_page = per_page.clamp(1, 50);
    let page = page.max(1);
    let offset = (page - 1) * per_page;

    let q_pattern: Option<String> = filters.q.as_ref().map(|q| format!("%{q}%"));

    let mut builder = QueryBuilder::new(SEARCH_SELECT);
    push_filters(&mut builder, &filters, q_pattern.as_deref());
    builder.push(" ORDER BY c.created_at DESC LIMIT ");
    builder.push_bind(per_page);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let clinics = builder
        .build_query_as::<Clinic>()
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    let mut count_builder = QueryBuilder::new("SELECT COUNT(DISTINCT c.id) FROM clinics c");
    push_filters(&mut count_builder, &filters, q_pattern.as_deref());
    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok((clinics, total))
}

pub async fn list_by_status(pool: &DbPool, status: ClinicStatus) -> Result<Vec<Clinic>, ApiError> {
    sqlx::query_as::<_, Clinic>("SELECT * FROM clinics WHERE status = $1 ORDER BY created_at DESC")
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn update_status(
    pool: &DbPool,
    id: Uuid,
    status: ClinicStatus,
) -> Result<Option<Clinic>, ApiError> {
    sqlx::query_as::<_, Clinic>(
        "UPDATE clinics
         SET status = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, owner_user_id, name, slug, country_code, city, accreditations, description, status, created_at",
    )
    .bind(status)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}
