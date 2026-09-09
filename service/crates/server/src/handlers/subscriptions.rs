use crate::{extractors::CurrentUser, state::AppState};
use axum::{extract::State, Json};
use chassis::{
    auth,
    error::ApiError,
    subscriptions::{self, Subscription, SubscriptionPlan},
    users::UserRole,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SubscriptionResp {
    subscription: Option<Subscription>,
    plan: SubscriptionPlan,
}

#[derive(Deserialize)]
pub struct CheckoutReq {
    plan_slug: String,
}

#[derive(Serialize)]
pub struct CheckoutResp {
    url: String,
}

/// Current effective plan (free default when there is no subscription yet).
pub async fn current(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<SubscriptionResp>, ApiError> {
    let subscription = subscriptions::get_active_for_user(&state.pool, user.id).await?;
    let plan = subscriptions::current_plan(&state.pool, user.id).await?;
    Ok(Json(SubscriptionResp { subscription, plan }))
}

/// Create a hosted checkout session for a paid plan and return its URL. The
/// magic happens via the configured PaymentProvider — mock mode returns a
/// mock URL so the flow is exercisable without Stripe keys.
pub async fn checkout(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
    Json(req): Json<CheckoutReq>,
) -> Result<Json<CheckoutResp>, ApiError> {
    auth::require_role(&user, UserRole::ProviderAdmin)?;
    let plan = subscriptions::plan_by_slug(&state.pool, &req.plan_slug)
        .await?
        .ok_or_else(|| ApiError::Validation("unknown plan".into()))?;
    if plan.price_cents == 0 {
        return Err(ApiError::Validation(
            "the free plan needs no checkout".into(),
        ));
    }

    let price_id = match &state.stripe {
        Some(stripe) => stripe
            .price_id_for(&plan.slug)
            .ok_or_else(|| {
                ApiError::Validation(format!("no Stripe price configured for plan {}", plan.slug))
            })?
            .to_string(),
        None => format!("price-mock-{}", plan.slug),
    };

    // Get-or-create the billing customer for this user.
    let customer_id = match chassis::users::stripe_customer_id(&state.pool, user.id).await? {
        Some(id) => id,
        None => {
            let id = state.payments.create_customer(&user.email).await?;
            chassis::users::set_stripe_customer_id(&state.pool, user.id, &id).await?;
            id
        }
    };

    let base = state.cfg.app_url.trim_end_matches('/');
    let url = state
        .payments
        .create_checkout_session(
            &customer_id,
            &price_id,
            &format!("{base}/dashboard/billing?status=success"),
            &format!("{base}/dashboard/billing?status=cancel"),
            &[
                ("user_id".to_string(), user.id.to_string()),
                ("plan_slug".to_string(), plan.slug.clone()),
            ],
        )
        .await?;
    Ok(Json(CheckoutResp { url }))
}
