use crate::state::AppState;
use axum::{extract::State, Json};
use chassis::{error::ApiError, treatments};
use serde::Serialize;

#[derive(Serialize)]
pub struct TreatmentsResponse {
    treatments: Vec<treatments::Treatment>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<TreatmentsResponse>, ApiError> {
    let treatments = treatments::list(&state.pool).await?;
    Ok(Json(TreatmentsResponse { treatments }))
}
