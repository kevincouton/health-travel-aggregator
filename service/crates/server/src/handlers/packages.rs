use crate::{
    extractors::{CurrentUser, MaybeUser},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use chassis::{
    auth,
    clinics::{self, ClinicStatus},
    error::ApiError,
    packages::{self, Package},
    users::UserRole,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreatePackageReq {
    treatment_id: Uuid,
    name: String,
    price_min: Option<i32>,
    price_max: Option<i32>,
    duration_days: Option<i32>,
    #[serde(default)]
    inclusions: Vec<String>,
    #[serde(default)]
    exclusions: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdatePackageReq {
    name: String,
    price_min: Option<i32>,
    price_max: Option<i32>,
    duration_days: Option<i32>,
    #[serde(default)]
    inclusions: Vec<String>,
    #[serde(default)]
    exclusions: Vec<String>,
    is_published: bool,
}

fn is_owner_or_platform_admin(user: &chassis::users::User, clinic: &clinics::Clinic) -> bool {
    user.role == UserRole::PlatformAdmin || clinic.owner_user_id == user.id
}

async fn require_clinic_ownership(
    pool: &chassis::db::DbPool,
    user: &chassis::users::User,
    clinic_id: Uuid,
) -> Result<clinics::Clinic, ApiError> {
    let clinic = clinics::by_id(pool, clinic_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if is_owner_or_platform_admin(user, &clinic) {
        Ok(clinic)
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn create(
    Path(clinic_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CreatePackageReq>,
) -> Result<Json<Package>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    require_clinic_ownership(&state.pool, &user, clinic_id).await?;

    let package = packages::create(
        &state.pool,
        clinic_id,
        req.treatment_id,
        &req.name,
        req.price_min,
        req.price_max,
        req.duration_days,
        &req.inclusions,
        &req.exclusions,
    )
    .await?;
    Ok(Json(package))
}

pub async fn update(
    Path((clinic_id, id)): Path<(Uuid, Uuid)>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<UpdatePackageReq>,
) -> Result<Json<Package>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    require_clinic_ownership(&state.pool, &user, clinic_id).await?;

    let package = packages::update(
        &state.pool,
        id,
        &req.name,
        req.price_min,
        req.price_max,
        req.duration_days,
        &req.inclusions,
        &req.exclusions,
        req.is_published,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    if package.clinic_id != clinic_id {
        return Err(ApiError::NotFound);
    }

    Ok(Json(package))
}

pub async fn delete(
    Path((clinic_id, id)): Path<(Uuid, Uuid)>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    require_clinic_ownership(&state.pool, &user, clinic_id).await?;

    let package = packages::by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if package.clinic_id != clinic_id {
        return Err(ApiError::NotFound);
    }

    packages::delete(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn list_for_owner(
    Path(clinic_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Package>>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    require_clinic_ownership(&state.pool, &user, clinic_id).await?;

    Ok(Json(
        packages::list_for_clinic(&state.pool, clinic_id).await?,
    ))
}

pub async fn public_list(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Package>>, ApiError> {
    let clinic = clinics::by_slug(&state.pool, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    if clinic.status != ClinicStatus::Approved {
        return Err(ApiError::NotFound);
    }

    Ok(Json(
        packages::list_public_for_clinic(&state.pool, clinic.id).await?,
    ))
}

pub async fn public_detail(
    Path(id): Path<Uuid>,
    MaybeUser(maybe_user): MaybeUser,
    State(state): State<AppState>,
) -> Result<Json<Package>, ApiError> {
    let package = packages::by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if package.is_published {
        return Ok(Json(package));
    }

    let can_view_unpublished = match maybe_user {
        Some(user) => {
            let clinic = clinics::by_id(&state.pool, package.clinic_id)
                .await?
                .ok_or(ApiError::NotFound)?;
            user.role == UserRole::PlatformAdmin || clinic.owner_user_id == user.id
        }
        None => false,
    };

    if can_view_unpublished {
        Ok(Json(package))
    } else {
        Err(ApiError::NotFound)
    }
}
