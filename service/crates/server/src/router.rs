use crate::{
    handlers::{
        admin, auth, clinics, inquiries, packages, reviews, subscriptions, treatments, webhooks,
    },
    middleware::{origin_guard, rate_limit, RateLimiter},
    state::AppState,
};
use axum::{
    extract::Request,
    http::{header, HeaderValue, Method},
    middleware::{self, Next},
    routing::{get, patch, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;

/// Restrictive CORS: only the configured browser origin may call the API
/// cross-origin, with credentials for the session cookie. An empty
/// `cors_origin` disables cross-origin access entirely (same-origin only).
fn cors_layer(origin: &str) -> CorsLayer {
    if origin.is_empty() {
        return CorsLayer::new();
    }
    match origin.parse::<HeaderValue>() {
        Ok(origin) => CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::list([origin]))
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        Err(_) => {
            tracing::warn!("CORS_ORIGIN is not a valid header value; CORS disabled");
            CorsLayer::new()
        }
    }
}

pub fn app(state: AppState) -> Router {
    let admin_routes = Router::new()
        .route("/clinics", get(admin::list_clinics))
        .route("/clinics/:id/status", patch(admin::update_clinic_status))
        .route("/clinics/:id/flag", post(admin::flag_clinic))
        .route("/claims", get(admin::list_claims))
        .route("/claims/:id/status", patch(admin::resolve_claim));

    let auth_limiter = RateLimiter::new(
        state.cfg.rate_limit_auth_per_minute,
        Duration::from_secs(60),
    );
    let auth_limited = Router::new()
        .route("/auth/provider/register", post(auth::provider_register))
        .route("/auth/provider/login", post(auth::provider_login))
        .route("/auth/magic-link", post(auth::request_magic_link))
        .route("/auth/magic-link/verify", post(auth::verify_magic_link))
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            rate_limit(req, next, auth_limiter.clone())
        }));

    let write_limiter = RateLimiter::new(
        state.cfg.rate_limit_write_per_minute,
        Duration::from_secs(60),
    );
    let write_limited = Router::new()
        .route("/inquiries", post(inquiries::create))
        .route("/clinics/:slug/reviews", post(reviews::create))
        .route("/clinics/:slug/claim", post(clinics::claim))
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            rate_limit(req, next, write_limiter.clone())
        }));

    // Origins accepted on mutating requests (CSRF origin check): the app
    // itself plus the configured CORS origin.
    let mut allowed_origins = vec![state.cfg.app_url.clone()];
    if !state.cfg.cors_origin.is_empty() && state.cfg.cors_origin != state.cfg.app_url {
        allowed_origins.push(state.cfg.cors_origin.clone());
    }
    let allowed_origins = Arc::new(allowed_origins);
    let guard = middleware::from_fn(move |req: Request, next: Next| {
        origin_guard(req, next, allowed_origins.clone())
    });

    let cors = cors_layer(&state.cfg.cors_origin);

    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        // Stripe calls this server-to-server without an Origin header; the
        // origin guard passes it through and the signature check authenticates.
        .route("/webhooks/stripe", post(webhooks::stripe))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/me/subscription", get(subscriptions::current))
        .route("/me/subscription/checkout", post(subscriptions::checkout))
        .route(
            "/me/clinics",
            post(clinics::create).get(clinics::list_for_owner),
        )
        .route("/me/clinics/:id", patch(clinics::update))
        .route(
            "/me/clinics/:clinic_id/packages",
            post(packages::create).get(packages::list_for_owner),
        )
        .route(
            "/me/clinics/:clinic_id/packages/:id",
            patch(packages::update).delete(packages::delete),
        )
        .route("/treatments", get(treatments::list))
        .route("/me/inquiries", get(inquiries::list_for_me))
        .route("/me/inquiries/:id/status", patch(inquiries::update_status))
        .route("/clinics", get(clinics::list_public))
        .route("/clinics/:slug", get(clinics::by_slug))
        .route("/clinics/:slug/reviews", get(reviews::list_for_clinic))
        .route("/clinics/:slug/packages", get(packages::public_list))
        .route("/packages/:id", get(packages::public_detail))
        .nest("/admin", admin_routes)
        .merge(auth_limited)
        .merge(write_limited)
        .layer(guard)
        .layer(cors)
        .with_state(state)
}
