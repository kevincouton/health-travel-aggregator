use crate::error::ApiError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Patient,
    ProviderAdmin,
    PlatformAdmin,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub display_name: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn create(
    pool: &crate::db::DbPool,
    email: &str,
    role: UserRole,
    display_name: Option<&str>,
) -> Result<User, ApiError> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (email, role, display_name)
         VALUES ($1, $2, $3)
         RETURNING id, email, role, display_name, email_verified_at, created_at",
    )
    .bind(email)
    .bind(role)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => ApiError::Conflict,
        _ => ApiError::Internal,
    })
}

pub async fn by_email(pool: &crate::db::DbPool, email: &str) -> Result<Option<User>, ApiError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn by_id(pool: &crate::db::DbPool, id: Uuid) -> Result<Option<User>, ApiError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

pub async fn mark_verified(pool: &crate::db::DbPool, id: Uuid) -> Result<(), ApiError> {
    sqlx::query("UPDATE users SET email_verified_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    Ok(())
}
