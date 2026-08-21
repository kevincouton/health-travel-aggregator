use crate::{
    error::ApiError,
    users::{User, UserRole},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;
use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| ApiError::Internal)
}

pub fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash).map_err(|_| ApiError::Internal)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized)
}

pub fn generate_magic_link_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    ttl_days: i64,
) -> Result<String, ApiError> {
    let token = generate_magic_link_token();
    sqlx::query(
        "INSERT INTO sessions (token, user_id, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 day' * $3)",
    )
    .bind(&token)
    .bind(user_id)
    .bind(ttl_days)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(token)
}

pub async fn resolve_session(pool: &PgPool, token: &str) -> Result<Option<User>, ApiError> {
    sqlx::query_as::<_, User>(
        "SELECT u.id, u.email, u.role, u.display_name, u.email_verified_at, u.created_at
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token = $1 AND s.expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    Ok(())
}

pub fn require_role(user: &User, role: UserRole) -> Result<(), ApiError> {
    if user.role == role || user.role == UserRole::PlatformAdmin {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}
