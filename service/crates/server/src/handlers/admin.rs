use crate::{extractors::AdminUser, state::AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chassis::{
    claims::{self, ClaimStatus},
    clinics,
    clinics::ClinicStatus,
    error::ApiError,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AdminClinicsQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateStatusReq {
    status: ClinicStatus,
}

#[derive(Deserialize)]
pub struct FlagClinicReq {
    reason: String,
}

#[derive(Deserialize)]
pub struct AdminClaimsQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct ResolveClaimReq {
    status: ClaimStatus,
}

fn parse_status_query(status: Option<String>) -> ClinicStatus {
    match status.as_deref() {
        Some("draft") => ClinicStatus::Draft,
        Some("pending") => ClinicStatus::Pending,
        Some("approved") => ClinicStatus::Approved,
        Some("suspended") => ClinicStatus::Suspended,
        _ => ClinicStatus::Pending,
    }
}

pub async fn list_clinics(
    _admin: AdminUser,
    Query(query): Query<AdminClinicsQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<clinics::Clinic>>, ApiError> {
    let status = parse_status_query(query.status);
    Ok(Json(clinics::list_by_status(&state.pool, status).await?))
}

pub async fn update_clinic_status(
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateStatusReq>,
) -> Result<Json<clinics::Clinic>, ApiError> {
    let clinic = clinics::update_status(&state.pool, id, req.status)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(clinic))
}

/// Flag/reject a listing with a recorded reason: suspends the clinic so it
/// drops out of public search.
pub async fn flag_clinic(
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<FlagClinicReq>,
) -> Result<Json<clinics::Clinic>, ApiError> {
    let reason = req.reason.trim();
    if reason.is_empty() {
        return Err(ApiError::Validation("reason is required".into()));
    }
    let clinic = clinics::flag(&state.pool, id, reason)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(clinic))
}

fn parse_claim_status_query(status: Option<String>) -> ClaimStatus {
    match status.as_deref() {
        Some("approved") => ClaimStatus::Approved,
        Some("rejected") => ClaimStatus::Rejected,
        _ => ClaimStatus::Pending,
    }
}

pub async fn list_claims(
    _admin: AdminUser,
    Query(query): Query<AdminClaimsQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<claims::ClinicClaim>>, ApiError> {
    let status = parse_claim_status_query(query.status);
    Ok(Json(claims::list_by_status(&state.pool, status).await?))
}

pub async fn resolve_claim(
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<ResolveClaimReq>,
) -> Result<Json<claims::ClinicClaim>, ApiError> {
    let claim = claims::resolve(&state.pool, id, req.status)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(claim))
}
