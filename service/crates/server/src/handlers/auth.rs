use crate::{extractors::CurrentUser, state::AppState};
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    Json,
};
use chassis::{
    auth,
    error::ApiError,
    users::{self, UserRole},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterReq {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct MagicLinkReq {
    email: String,
}

#[derive(Deserialize)]
pub struct VerifyMagicLinkReq {
    token: String,
}

#[derive(Serialize)]
pub struct AuthResp {
    token: String,
    user: users::User,
}

fn session_cookie(token: &str, app_url: &str) -> String {
    let secure = app_url.starts_with("https://");
    format!(
        "session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}{}",
        30 * 24 * 60 * 60,
        if secure { "; Secure" } else { "" }
    )
}

fn clear_session_cookie() -> String {
    "session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0".to_string()
}

fn set_cookie_header(token: &str, app_url: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str(&session_cookie(token, app_url)).unwrap(),
    );
    headers
}

pub async fn provider_register(
    State(state): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> Result<(HeaderMap, Json<AuthResp>), ApiError> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || req.password.len() < 8 {
        return Err(ApiError::Validation("invalid email or password".into()));
    }
    let user = users::create(&state.pool, &email, UserRole::ProviderAdmin, None).await?;
    let hash = auth::hash_password(&req.password)?;
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&hash)
        .bind(user.id)
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    let token = auth::create_session(&state.pool, user.id, 30).await?;
    Ok((
        set_cookie_header(&token, &state.cfg.app_url),
        Json(AuthResp { token, user }),
    ))
}

pub async fn provider_login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<(HeaderMap, Json<AuthResp>), ApiError> {
    let email = req.email.trim().to_lowercase();
    let user = users::by_email(&state.pool, &email)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    auth::verify_password(&req.password, &hash)?;
    let token = auth::create_session(&state.pool, user.id, 30).await?;
    Ok((
        set_cookie_header(&token, &state.cfg.app_url),
        Json(AuthResp { token, user }),
    ))
}

pub async fn request_magic_link(
    State(state): State<AppState>,
    Json(req): Json<MagicLinkReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = req.email.trim().to_lowercase();
    let user = match users::by_email(&state.pool, &email).await? {
        Some(u) => u,
        None => users::create(&state.pool, &email, UserRole::Patient, None).await?,
    };
    let token = auth::generate_magic_link_token();
    sqlx::query(
        "INSERT INTO magic_links (token, user_id, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
    )
    .bind(&token)
    .bind(user.id)
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::Internal)?;
    state
        .email
        .send_magic_link(
            &email,
            &format!("{}/auth/verify?token={}", state.cfg.app_url, token),
        )
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn verify_magic_link(
    State(state): State<AppState>,
    Json(req): Json<VerifyMagicLinkReq>,
) -> Result<(HeaderMap, Json<AuthResp>), ApiError> {
    let row: Option<(uuid::Uuid,)> = sqlx::query_as(
        "DELETE FROM magic_links
         WHERE token = $1 AND expires_at > NOW()
         RETURNING user_id",
    )
    .bind(&req.token)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::Internal)?;
    let user_id = row.ok_or(ApiError::Unauthorized)?.0;
    users::mark_verified(&state.pool, user_id).await?;
    let user = users::by_id(&state.pool, user_id)
        .await?
        .ok_or(ApiError::Internal)?;
    let token = auth::create_session(&state.pool, user.id, 30).await?;
    Ok((
        set_cookie_header(&token, &state.cfg.app_url),
        Json(AuthResp { token, user }),
    ))
}

pub async fn logout(
    CurrentUser(_user): CurrentUser,
) -> Result<(HeaderMap, Json<serde_json::Value>), ApiError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str(&clear_session_cookie()).unwrap(),
    );
    Ok((headers, Json(serde_json::json!({ "ok": true }))))
}

pub async fn me(CurrentUser(user): CurrentUser) -> Result<Json<users::User>, ApiError> {
    Ok(Json(user))
}
