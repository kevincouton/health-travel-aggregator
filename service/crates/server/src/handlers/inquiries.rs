use crate::{extractors::CurrentUser, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use chassis::{
    auth,
    error::ApiError,
    inquiries::{self, Inquiry, InquiryStatus},
    users::UserRole,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateInquiryReq {
    clinic_id: Uuid,
    package_id: Option<Uuid>,
    medical_notes: Option<String>,
    preferred_dates: Option<String>,
    contact_email: String,
}

#[derive(Deserialize)]
pub struct UpdateStatusReq {
    status: InquiryStatus,
}

pub async fn create(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CreateInquiryReq>,
) -> Result<Json<Inquiry>, ApiError> {
    let inquiry = inquiries::create(
        &state.pool,
        user.id,
        req.clinic_id,
        req.package_id,
        req.medical_notes.as_deref(),
        req.preferred_dates.as_deref(),
        &req.contact_email,
    )
    .await?;
    Ok(Json(inquiry))
}

pub async fn list_for_me(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Inquiry>>, ApiError> {
    let list = match user.role {
        UserRole::Patient => inquiries::list_for_patient(&state.pool, user.id).await?,
        UserRole::ProviderAdmin | UserRole::PlatformAdmin => {
            inquiries::list_for_provider(&state.pool, user.id).await?
        }
    };
    Ok(Json(list))
}

pub async fn update_status(
    Path(id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateStatusReq>,
) -> Result<Json<Inquiry>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    let inquiry = inquiries::update_status(&state.pool, id, user.id, req.status)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(inquiry))
}
