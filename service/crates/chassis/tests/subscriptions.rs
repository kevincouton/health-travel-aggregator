use chassis::{
    subscriptions::{self, SubscriptionInterval, SubscriptionStatus},
    users::{self, UserRole},
};
use sqlx::PgPool;

#[sqlx::test]
async fn seed_plans_and_default_plan(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();

    subscriptions::seed_plans(&pool).await.unwrap();
    let plan = subscriptions::default_plan(&pool).await.unwrap();

    assert_eq!(plan.slug, "basic");
    assert_eq!(plan.price_cents, 0);
    assert_eq!(plan.interval, SubscriptionInterval::Month);
    assert_eq!(plan.max_clinics, 1);
    assert_eq!(plan.max_packages, 3);

    // Seed is idempotent: running again should not change the basic plan.
    subscriptions::seed_plans(&pool).await.unwrap();
    let plan_again = subscriptions::default_plan(&pool).await.unwrap();
    assert_eq!(plan_again.id, plan.id);
}

#[sqlx::test]
async fn create_and_fetch_active_subscription(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    subscriptions::seed_plans(&pool).await.unwrap();

    let user = users::create(
        &pool,
        "subscriber@example.com",
        UserRole::ProviderAdmin,
        None,
    )
    .await
    .unwrap();
    let plan = subscriptions::default_plan(&pool).await.unwrap();

    let subscription = subscriptions::create_subscription(&pool, user.id, plan.id)
        .await
        .unwrap();

    assert_eq!(subscription.user_id, user.id);
    assert_eq!(subscription.plan_id, plan.id);
    assert_eq!(subscription.status, SubscriptionStatus::Trialing);

    let active = subscriptions::get_active_for_user(&pool, user.id)
        .await
        .unwrap()
        .expect("active subscription");
    assert_eq!(active.id, subscription.id);
}

#[sqlx::test]
async fn canceled_subscription_is_not_active(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    subscriptions::seed_plans(&pool).await.unwrap();

    let user = users::create(&pool, "canceled@example.com", UserRole::ProviderAdmin, None)
        .await
        .unwrap();
    let plan = subscriptions::default_plan(&pool).await.unwrap();

    let subscription = subscriptions::create_subscription(&pool, user.id, plan.id)
        .await
        .unwrap();

    sqlx::query("UPDATE subscriptions SET status = 'canceled' WHERE id = $1")
        .bind(subscription.id)
        .execute(&pool)
        .await
        .unwrap();

    let active = subscriptions::get_active_for_user(&pool, user.id)
        .await
        .unwrap();
    assert!(active.is_none());
}
