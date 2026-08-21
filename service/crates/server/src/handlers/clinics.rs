use crate::{extractors::CurrentUser, state::AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chassis::{
    auth,
    clinics::{self, ClinicStatus},
    error::ApiError,
    users::UserRole,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateClinicReq {
    name: String,
    slug: String,
    country_code: String,
    city: String,
    accreditations: Vec<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateClinicReq {
    name: String,
    slug: String,
    country_code: String,
    city: String,
    accreditations: Vec<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct PublicListQuery {
    country_code: Option<String>,
    city: Option<String>,
}

pub async fn create(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CreateClinicReq>,
) -> Result<Json<clinics::Clinic>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    let clinic = clinics::create(
        &state.pool,
        user.id,
        &req.name,
        &req.slug,
        &req.country_code,
        &req.city,
        &req.accreditations,
        req.description.as_deref(),
    )
    .await?;
    Ok(Json(clinic))
}

pub async fn update(
    Path(id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateClinicReq>,
) -> Result<Json<clinics::Clinic>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    let clinic = clinics::update(
        &state.pool,
        id,
        user.id,
        &req.name,
        &req.slug,
        &req.country_code,
        &req.city,
        &req.accreditations,
        req.description.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(clinic))
}

pub async fn list_for_owner(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<clinics::Clinic>>, ApiError> {
    Ok(Json(clinics::list_for_owner(&state.pool, user.id).await?))
}

pub async fn by_slug(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<clinics::Clinic>, ApiError> {
    let clinic = clinics::by_slug(&state.pool, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    if clinic.status != ClinicStatus::Approved {
        return Err(ApiError::NotFound);
    }
    Ok(Json(clinic))
}

pub async fn list_public(
    Query(query): Query<PublicListQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<clinics::Clinic>>, ApiError> {
    Ok(Json(
        clinics::list_public(
            &state.pool,
            query.country_code.as_deref(),
            query.city.as_deref(),
        )
        .await?,
    ))
}
