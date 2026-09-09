use crate::state::AppState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chassis::{connectors::stripe, error::ApiError, subscriptions};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct StripeEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: StripeEventData,
}

#[derive(Deserialize)]
struct StripeEventData {
    object: serde_json::Value,
}

/// POST /webhooks/stripe — Stripe event sink. Verifies the Stripe-Signature
/// v1 header against the raw body before touching the database; unverified
/// events are rejected, unknown event types acknowledged and ignored.
pub async fn stripe(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let provider = state
        .stripe
        .as_ref()
        .ok_or(ApiError::Validation("stripe is not configured".into()))?;
    let secret = provider
        .webhook_secret()
        .ok_or(ApiError::Validation("stripe webhook secret is not configured".into()))?;
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::BadRequest)?;

    stripe::verify_webhook_signature(&body, signature, secret)?;

    let event: StripeEvent = serde_json::from_slice(&body)
        .map_err(|_| ApiError::Validation("invalid stripe event payload".into()))?;

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let obj = &event.data.object;
            let user_id: Uuid = obj
                .pointer("/metadata/user_id")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("client_reference_id").and_then(|v| v.as_str()))
                .and_then(|s| s.parse().ok())
                .ok_or(ApiError::Validation("checkout session has no user reference".into()))?;
            let plan_slug = obj
                .pointer("/metadata/plan_slug")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::Validation("checkout session has no plan reference".into()))?;
            let stripe_subscription_id = obj
                .get("subscription")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::Validation("checkout session has no subscription".into()))?;
            subscriptions::activate_from_checkout(
                &state.pool,
                user_id,
                plan_slug,
                stripe_subscription_id,
            )
            .await?;
        }
        "customer.subscription.updated" => {
            let obj = &event.data.object;
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::Validation("subscription event has no id".into()))?;
            let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let period_end = obj.get("current_period_end").and_then(|v| v.as_i64());
            subscriptions::sync_from_stripe(&state.pool, id, status, period_end).await?;
        }
        "customer.subscription.deleted" => {
            let obj = &event.data.object;
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::Validation("subscription event has no id".into()))?;
            subscriptions::sync_from_stripe(&state.pool, id, "canceled", None).await?;
        }
        _ => {
            // Unknown/unhandled events are acknowledged so Stripe stops
            // retrying them.
        }
    }
    Ok(StatusCode::OK)
}
