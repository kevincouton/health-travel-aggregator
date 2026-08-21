//! Treatments reference data.

use crate::error::ApiError;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// A medical-tourism treatment in the catalog.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Treatment {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

const SEED_TREATMENTS: &[(&str, &str, &str, Option<&str>)] = &[
    (
        "dental-implants",
        "Dental Implants",
        "Dental",
        Some("Titanium implant restoration for missing teeth."),
    ),
    (
        "hair-transplant",
        "Hair Transplant",
        "Cosmetic",
        Some("Follicular unit extraction or transplantation for hair restoration."),
    ),
    (
        "ivf",
        "IVF",
        "Fertility",
        Some("In vitro fertilization and related assisted-reproduction services."),
    ),
    (
        "hip-replacement",
        "Hip Replacement",
        "Orthopedic",
        Some("Total or partial hip arthroplasty."),
    ),
    (
        "cataract-surgery",
        "Cataract Surgery",
        "Ophthalmology",
        Some("Lens replacement to restore vision impaired by cataracts."),
    ),
    (
        "cosmetic-surgery",
        "Cosmetic Surgery",
        "Cosmetic",
        Some("Aesthetic surgical procedures including rhinoplasty and liposuction."),
    ),
    (
        "cardiac-bypass",
        "Cardiac Bypass Surgery",
        "Cardiac",
        Some("Coronary artery bypass grafting for advanced heart disease."),
    ),
    (
        "bariatric-surgery",
        "Bariatric Surgery",
        "Weight Loss",
        Some("Gastric bypass, sleeve, or other weight-loss procedures."),
    ),
    (
        "orthopedic-surgery",
        "Orthopedic Surgery",
        "Orthopedic",
        Some("Knee, shoulder, spine, and sports-medicine procedures."),
    ),
    (
        "stem-cell-therapy",
        "Stem Cell Therapy",
        "Regenerative",
        Some("Regenerative treatments using stem cells for joints and tissues."),
    ),
];

/// Seed the reference treatments table if not already present.
pub async fn seed(pool: &crate::db::DbPool) -> Result<(), ApiError> {
    for (slug, name, category, description) in SEED_TREATMENTS {
        sqlx::query(
            "INSERT INTO treatments (slug, name, category, description)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (slug) DO NOTHING",
        )
        .bind(slug)
        .bind(name)
        .bind(category)
        .bind(description)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    }
    Ok(())
}

/// List all seeded treatments, ordered by category then name.
pub async fn list(pool: &crate::db::DbPool) -> Result<Vec<Treatment>, ApiError> {
    sqlx::query_as::<_, Treatment>("SELECT * FROM treatments ORDER BY category, name")
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

/// Fetch a single treatment by its unique slug.
pub async fn by_slug(pool: &crate::db::DbPool, slug: &str) -> Result<Option<Treatment>, ApiError> {
    sqlx::query_as::<_, Treatment>("SELECT * FROM treatments WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

/// Fetch a single treatment by its id.
pub async fn by_id(pool: &crate::db::DbPool, id: Uuid) -> Result<Option<Treatment>, ApiError> {
    sqlx::query_as::<_, Treatment>("SELECT * FROM treatments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}
