use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts};
use chassis::{auth, error::ApiError, users::User};

use crate::state::AppState;

pub struct CurrentUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|c| c.split(';').find(|s| s.trim().starts_with("session=")))
            .and_then(|s| s.split_once('=').map(|(_, v)| v));
        match cookie {
            Some(token) => {
                let user = auth::resolve_session(&state.pool, token).await?;
                user.map(CurrentUser).ok_or(ApiError::Unauthorized)
            }
            None => Err(ApiError::Unauthorized),
        }
    }
}
