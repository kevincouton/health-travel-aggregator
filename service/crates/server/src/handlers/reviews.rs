use crate::{extractors::CurrentUser, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use chassis::{
    clinics::{self, ClinicStatus},
    error::ApiError,
    reviews::{self, Review},
    users::UserRole,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReviewReq {
    pub inquiry_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
}

pub async fn create(
    Path(slug): Path<String>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CreateReviewReq>,
) -> Result<Json<Review>, ApiError> {
    if user.role != UserRole::Patient {
        return Err(ApiError::Forbidden);
    }

    let clinic = clinics::by_slug(&state.pool, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    if clinic.status != ClinicStatus::Approved {
        return Err(ApiError::NotFound);
    }

    let inquiry_id = reviews::can_review(&state.pool, user.id, clinic.id)
        .await?
        .ok_or(ApiError::Forbidden)?;

    if inquiry_id != req.inquiry_id {
        return Err(ApiError::BadRequest);
    }

    let review = reviews::create(
        &state.pool,
        clinic.id,
        user.id,
        inquiry_id,
        req.rating,
        req.comment.as_deref(),
    )
    .await?;

    Ok(Json(review))
}

pub async fn list_for_clinic(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Review>>, ApiError> {
    let clinic = clinics::by_slug(&state.pool, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    if clinic.status != ClinicStatus::Approved {
        return Err(ApiError::NotFound);
    }

    let reviews = reviews::list_for_clinic(&state.pool, clinic.id).await?;
    Ok(Json(reviews))
}
