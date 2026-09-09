use crate::{db::DbPool, error::ApiError};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "subscription_interval", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionInterval {
    Month,
    Year,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trialing,
    Active,
    PastDue,
    Canceled,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub price_cents: i32,
    pub interval: SubscriptionInterval,
    pub max_clinics: i32,
    pub max_packages: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: SubscriptionStatus,
    pub current_period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn seed_plans(pool: &DbPool) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO subscription_plans (slug, name, price_cents, interval, max_clinics, max_packages)
         VALUES
           ('basic', 'Basic', 0, 'month', 1, 3),
           ('pro', 'Pro', 9900, 'month', 5, 20),
           ('enterprise', 'Enterprise', 29900, 'month', 1000, 10000)
         ON CONFLICT (slug) DO UPDATE SET
           name = EXCLUDED.name,
           price_cents = EXCLUDED.price_cents,
           interval = EXCLUDED.interval,
           max_clinics = EXCLUDED.max_clinics,
           max_packages = EXCLUDED.max_packages,
           is_active = EXCLUDED.is_active",
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(())
}

pub async fn default_plan(pool: &DbPool) -> Result<SubscriptionPlan, ApiError> {
    sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE slug = 'basic' AND is_active = true LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal)?
    .ok_or(ApiError::NotFound)
}

pub async fn create_subscription(
    pool: &DbPool,
    user_id: Uuid,
    plan_id: Uuid,
) -> Result<Subscription, ApiError> {
    let period_end = Utc::now() + Duration::days(14);

    sqlx::query_as::<_, Subscription>(
        "INSERT INTO subscriptions (user_id, plan_id, status, current_period_end)
         VALUES ($1, $2, 'trialing', $3)
         RETURNING id, user_id, plan_id, status, current_period_end, created_at",
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(period_end)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn get_active_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Option<Subscription>, ApiError> {
    sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions
         WHERE user_id = $1
           AND status IN ('trialing', 'active', 'past_due')
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

pub async fn plan_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<SubscriptionPlan>, ApiError> {
    sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE slug = $1 AND is_active = true LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal)
}

/// The plan currently governing the user: their active subscription's plan,
/// or the free default plan when they have no subscription.
pub async fn current_plan(pool: &DbPool, user_id: Uuid) -> Result<SubscriptionPlan, ApiError> {
    match get_active_for_user(pool, user_id).await? {
        Some(sub) => {
            sqlx::query_as::<_, SubscriptionPlan>(
                "SELECT * FROM subscription_plans WHERE id = $1",
            )
            .bind(sub.plan_id)
            .fetch_one(pool)
            .await
            .map_err(|_| ApiError::Internal)
        }
        None => default_plan(pool).await,
    }
}

/// Reject with 402 when the user is at their plan's clinic quota.
pub async fn enforce_clinic_limit(pool: &DbPool, user_id: Uuid) -> Result<(), ApiError> {
    let plan = current_plan(pool, user_id).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clinics WHERE owner_user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    if count >= i64::from(plan.max_clinics) {
        return Err(ApiError::PaymentRequired(format!(
            "the {} plan allows at most {} clinic(s); upgrade to add more",
            plan.name, plan.max_clinics
        )));
    }
    Ok(())
}

/// Reject with 402 when the user is at their plan's package quota (packages
/// are counted across all clinics the user owns).
pub async fn enforce_package_limit(pool: &DbPool, user_id: Uuid) -> Result<(), ApiError> {
    let plan = current_plan(pool, user_id).await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM packages p
         JOIN clinics c ON c.id = p.clinic_id
         WHERE c.owner_user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::Internal)?;
    if count >= i64::from(plan.max_packages) {
        return Err(ApiError::PaymentRequired(format!(
            "the {} plan allows at most {} package(s); upgrade to add more",
            plan.name, plan.max_packages
        )));
    }
    Ok(())
}

/// checkout.session.completed: point the user's subscription at the purchased
/// plan, mark it active, and record the Stripe subscription ID so later
/// subscription.updated/.deleted events find it.
pub async fn activate_from_checkout(
    pool: &DbPool,
    user_id: Uuid,
    plan_slug: &str,
    stripe_subscription_id: &str,
) -> Result<(), ApiError> {
    let plan = plan_by_slug(pool, plan_slug)
        .await?
        .ok_or_else(|| ApiError::Validation(format!("unknown plan slug {plan_slug:?}")))?;

    let updated = sqlx::query(
        "UPDATE subscriptions
         SET plan_id = $2, status = 'active', stripe_subscription_id = $3,
             current_period_end = NOW() + INTERVAL '30 days'
         WHERE id = (
             SELECT id FROM subscriptions WHERE user_id = $1
             ORDER BY created_at DESC LIMIT 1
         )",
    )
    .bind(user_id)
    .bind(plan.id)
    .bind(stripe_subscription_id)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO subscriptions (user_id, plan_id, status, current_period_end, stripe_subscription_id)
             VALUES ($1, $2, 'active', NOW() + INTERVAL '30 days', $3)",
        )
        .bind(user_id)
        .bind(plan.id)
        .bind(stripe_subscription_id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    }
    Ok(())
}

/// Map a Stripe subscription status onto our enum; `None` for statuses we
/// deliberately ignore (incomplete, incomplete_expired, unpaid stay as-is
/// until Stripe sends a terminal update).
fn map_stripe_status(status: &str) -> Option<SubscriptionStatus> {
    match status {
        "trialing" => Some(SubscriptionStatus::Trialing),
        "active" => Some(SubscriptionStatus::Active),
        "past_due" => Some(SubscriptionStatus::PastDue),
        "canceled" => Some(SubscriptionStatus::Canceled),
        _ => None,
    }
}

/// customer.subscription.updated: sync status and period end for the
/// subscription carrying this Stripe ID. Unknown IDs are ignored (Stripe
/// retries on 4xx/5xx; a subscription we never recorded is not an error).
pub async fn sync_from_stripe(
    pool: &DbPool,
    stripe_subscription_id: &str,
    status: &str,
    current_period_end: Option<i64>,
) -> Result<(), ApiError> {
    let Some(status) = map_stripe_status(status) else {
        return Ok(());
    };
    let period_end: Option<DateTime<Utc>> = current_period_end
        .and_then(|ts| DateTime::from_timestamp(ts, 0));
    sqlx::query(
        "UPDATE subscriptions
         SET status = $2,
             current_period_end = COALESCE($3, current_period_end)
         WHERE stripe_subscription_id = $1",
    )
    .bind(stripe_subscription_id)
    .bind(status)
    .bind(period_end)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(())
}
