use crate::{handlers::auth, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/auth/provider/register", post(auth::provider_register))
        .route("/auth/provider/login", post(auth::provider_login))
        .route("/auth/magic-link", post(auth::request_magic_link))
        .route("/auth/magic-link/verify", post(auth::verify_magic_link))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .with_state(state)
}
