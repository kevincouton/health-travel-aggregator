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
