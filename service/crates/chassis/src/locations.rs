//! Locations reference data.

use crate::error::ApiError;
use serde::Serialize;
use sqlx::FromRow;

/// A city/country location in the catalog.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Location {
    pub id: i32,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
}

const SEED_LOCATIONS: &[(char, char, &str, &str)] = &[
    ('U', 'S', "United States", "New York"),
    ('U', 'S', "United States", "Los Angeles"),
    ('G', 'B', "United Kingdom", "London"),
    ('G', 'B', "United Kingdom", "Manchester"),
    ('D', 'E', "Germany", "Berlin"),
    ('D', 'E', "Germany", "Munich"),
    ('F', 'R', "France", "Paris"),
    ('F', 'R', "France", "Lyon"),
    ('E', 'S', "Spain", "Barcelona"),
    ('E', 'S', "Spain", "Madrid"),
    ('T', 'H', "Thailand", "Bangkok"),
    ('T', 'H', "Thailand", "Phuket"),
    ('S', 'G', "Singapore", "Singapore"),
    ('I', 'N', "India", "New Delhi"),
    ('I', 'N', "India", "Mumbai"),
    ('M', 'X', "Mexico", "Mexico City"),
    ('M', 'X', "Mexico", "Guadalajara"),
    ('T', 'R', "Turkey", "Istanbul"),
    ('K', 'R', "South Korea", "Seoul"),
    ('C', 'Z', "Czechia", "Prague"),
];

/// Seed the reference locations table if not already present.
pub async fn seed(pool: &crate::db::DbPool) -> Result<(), ApiError> {
    for (a, b, country_name, city) in SEED_LOCATIONS {
        let country_code = format!("{}{}", a, b);
        sqlx::query(
            "INSERT INTO locations (country_code, country_name, city)
             VALUES ($1, $2, $3)
             ON CONFLICT (country_code, city) DO NOTHING",
        )
        .bind(&country_code)
        .bind(country_name)
        .bind(city)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    }
    Ok(())
}

/// List all seeded locations, ordered by country then city.
pub async fn list(pool: &crate::db::DbPool) -> Result<Vec<Location>, ApiError> {
    sqlx::query_as::<_, Location>(
        "SELECT id, country_code, country_name, city FROM locations ORDER BY country_name, city",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::Internal)
}
