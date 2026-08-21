use crate::{handlers::{admin, auth, clinics}, state::AppState};
use axum::{
    routing::{get, patch, post},
    Router,
};

pub fn app(state: AppState) -> Router {
    let admin_routes = Router::new()
        .route("/clinics", get(admin::list_clinics))
        .route("/clinics/:id/status", patch(admin::update_clinic_status));

    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/auth/provider/register", post(auth::provider_register))
        .route("/auth/provider/login", post(auth::provider_login))
        .route("/auth/magic-link", post(auth::request_magic_link))
        .route("/auth/magic-link/verify", post(auth::verify_magic_link))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/me/clinics", post(clinics::create).get(clinics::list_for_owner))
        .route("/me/clinics/:id", patch(clinics::update))
        .route("/clinics", get(clinics::list_public))
        .route("/clinics/:slug", get(clinics::by_slug))
        .nest("/admin", admin_routes)
        .with_state(state)
}
