use crate::{extractors::CurrentUser, state::AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chassis::{
    auth, claims,
    clinics::{self, ClinicStatus},
    error::ApiError,
    users::UserRole,
};
use serde::{Deserialize, Serialize};
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
pub struct SearchQuery {
    #[serde(flatten)]
    filters: clinics::ClinicFilters,
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_per_page")]
    per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

#[derive(Serialize)]
pub struct SearchResponse {
    clinics: Vec<clinics::Clinic>,
    total: i64,
    page: i64,
    per_page: i64,
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
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<SearchResponse>, ApiError> {
    let (clinics, total) =
        clinics::search(&state.pool, query.filters, query.page, query.per_page).await?;

    Ok(Json(SearchResponse {
        clinics,
        total,
        page: query.page.max(1),
        per_page: query.per_page.clamp(1, 50),
    }))
}

#[derive(Deserialize)]
pub struct ClaimClinicReq {
    message: Option<String>,
}

/// Provider claim intent on an existing listing (e.g. a collector-ingested
/// clinic owned by the system user). Resolves through admin moderation;
/// approving the claim transfers ownership.
pub async fn claim(
    Path(slug): Path<String>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<ClaimClinicReq>,
) -> Result<Json<claims::ClinicClaim>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    let clinic = clinics::by_slug(&state.pool, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    if clinic.owner_user_id == user.id {
        return Err(ApiError::Conflict);
    }
    let message = req
        .message
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty());
    let claim = claims::create(&state.pool, clinic.id, user.id, message).await?;
    Ok(Json(claim))
}
