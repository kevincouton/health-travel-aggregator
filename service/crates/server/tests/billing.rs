use axum::body::Body;
use chassis::{
    auth,
    config::Config,
    connectors::{
        email::MockEmailSender,
        payments::{MockPaymentProvider, PaymentProvider},
        stripe::{StripeConfig, StripePaymentProvider},
    },
    db::DbPool,
    subscriptions::{self, SubscriptionStatus},
    users::{self, UserRole},
};
use hmac::{Hmac, Mac};
use server::{router::app, state::AppState};
use sha2::Sha256;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const WEBHOOK_SECRET: &str = "whsec_test";

fn test_config() -> Config {
    Config {
        database_url: "postgres://localhost/test".into(),
        app_url: "http://localhost:3000".into(),
        api_port: 8080,
        session_signing_key: "super-secret-key-at-least-32-bytes-long".into(),
        cors_origin: "".into(),
        rate_limit_auth_per_minute: 100,
        rate_limit_write_per_minute: 100,
    }
}

async fn setup_state(pool: DbPool) -> (AppState, Arc<MockPaymentProvider>) {
    chassis::db::migrate(&pool).await.unwrap();
    subscriptions::seed_plans(&pool).await.unwrap();
    let payments = Arc::new(MockPaymentProvider::new());
    let state = AppState {
        cfg: test_config(),
        pool,
        email: Arc::new(MockEmailSender::new()),
        payments: payments.clone(),
        stripe: None,
    };
    (state, payments)
}

async fn setup_state_with_stripe(pool: DbPool) -> AppState {
    chassis::db::migrate(&pool).await.unwrap();
    subscriptions::seed_plans(&pool).await.unwrap();
    let stripe = Arc::new(StripePaymentProvider::with_base_url(
        StripeConfig {
            secret_key: "sk_test_123".into(),
            webhook_secret: Some(WEBHOOK_SECRET.into()),
            price_pro: Some("price_pro_123".into()),
            price_enterprise: Some("price_ent_123".into()),
        },
        // Never called by the webhook path.
        "http://127.0.0.1:1",
    ));
    AppState {
        cfg: test_config(),
        pool,
        email: Arc::new(MockEmailSender::new()),
        payments: stripe.clone(),
        stripe: Some(stripe),
    }
}

async fn provider_session(pool: &DbPool, email: &str) -> (Uuid, String) {
    let user = users::create(pool, email, UserRole::ProviderAdmin, None)
        .await
        .unwrap();
    let session = auth::create_session(pool, user.id, 1).await.unwrap();
    (user.id, session)
}

fn post_json(
    uri: &str,
    session: Option<&str>,
    body: serde_json::Value,
) -> axum::http::Request<Body> {
    let mut builder = axum::http::Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json");
    if let Some(session) = session {
        builder = builder.header("Cookie", format!("session={session}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn stripe_signature(payload: &[u8], secret: &str, t: i64) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(t.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    format!("t={t},v1={}", hex::encode(mac.finalize().into_bytes()))
}

fn webhook_request(
    payload: serde_json::Value,
    signature: Option<String>,
) -> axum::http::Request<Body> {
    let mut builder = axum::http::Request::builder()
        .method("POST")
        .uri("/webhooks/stripe")
        .header("Content-Type", "application/json");
    if let Some(sig) = signature {
        builder = builder.header("Stripe-Signature", sig);
    }
    builder.body(Body::from(payload.to_string())).unwrap()
}

// --- Checkout endpoint -----------------------------------------------------

#[sqlx::test]
async fn checkout_returns_mock_url_in_mock_mode(pool: DbPool) {
    let (state, payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (user_id, session) = provider_session(&pool, "checkout-mock@example.com").await;

    let resp = router
        .oneshot(post_json(
            "/me/subscription/checkout",
            Some(&session),
            serde_json::json!({"plan_slug": "pro"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = body_json(resp).await;
    assert!(body["url"]
        .as_str()
        .unwrap()
        .starts_with("https://mock-checkout.example/"));

    // The provider recorded the checkout with APP_URL-derived redirect URLs
    // and the user/plan metadata the webhook relies on.
    let calls = payments.calls.lock().await;
    assert_eq!(calls.len(), 2); // create_customer + create_checkout_session
    assert!(calls[1].contains("success http://localhost:3000/dashboard/billing?status=success"));
    assert!(calls[1].contains("cancel http://localhost:3000/dashboard/billing?status=cancel"));
    assert!(calls[1].contains(&user_id.to_string()));
    assert!(calls[1].contains("pro"));
    drop(calls);

    // The mock customer id was persisted on the user.
    let customer = users::stripe_customer_id(&pool, user_id).await.unwrap();
    assert_eq!(customer.as_deref(), Some("cust-mock-1"));
}

#[sqlx::test]
async fn checkout_rejects_free_and_unknown_plans(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (_user_id, session) = provider_session(&pool, "checkout-basic@example.com").await;

    for slug in ["basic", "nonexistent"] {
        let resp = router
            .clone()
            .oneshot(post_json(
                "/me/subscription/checkout",
                Some(&session),
                serde_json::json!({"plan_slug": slug}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "slug {slug}");
    }
}

#[sqlx::test]
async fn checkout_requires_provider_role(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let patient = users::create(&pool, "patient@example.com", UserRole::Patient, None)
        .await
        .unwrap();
    let session = auth::create_session(&pool, patient.id, 1).await.unwrap();

    let resp = router
        .oneshot(post_json(
            "/me/subscription/checkout",
            Some(&session),
            serde_json::json!({"plan_slug": "pro"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// --- Stripe webhook ---------------------------------------------------------

fn checkout_completed_payload(user_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "id": "evt_1",
        "type": "checkout.session.completed",
        "data": {"object": {
            "id": "cs_1",
            "client_reference_id": user_id.to_string(),
            "metadata": {"user_id": user_id.to_string(), "plan_slug": "pro"},
            "subscription": "sub_123"
        }}
    })
}

#[sqlx::test]
async fn webhook_checkout_completed_activates_pro_subscription(pool: DbPool) {
    let state = setup_state_with_stripe(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (user_id, _session) = provider_session(&pool, "webhook-user@example.com").await;

    let payload = checkout_completed_payload(user_id);
    let sig = stripe_signature(
        payload.to_string().as_bytes(),
        WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let resp = router
        .oneshot(webhook_request(payload, Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let sub = subscriptions::get_active_for_user(&pool, user_id)
        .await
        .unwrap()
        .expect("active subscription");
    assert_eq!(sub.status, SubscriptionStatus::Active);
    let plan = sqlx::query_scalar::<_, String>("SELECT slug FROM subscription_plans WHERE id = $1")
        .bind(sub.plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plan, "pro");
}

#[sqlx::test]
async fn webhook_rejects_tampered_and_expired_signatures(pool: DbPool) {
    let state = setup_state_with_stripe(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (user_id, _session) = provider_session(&pool, "webhook-tamper@example.com").await;

    // Missing signature header.
    let payload = checkout_completed_payload(user_id);
    let resp = router
        .clone()
        .oneshot(webhook_request(payload.clone(), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Signature over a different payload (tampering).
    let other = serde_json::json!({"id": "evt_other"}).to_string();
    let sig = stripe_signature(
        other.as_bytes(),
        WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let resp = router
        .clone()
        .oneshot(webhook_request(payload.clone(), Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Expired timestamp.
    let sig = stripe_signature(
        payload.to_string().as_bytes(),
        WEBHOOK_SECRET,
        chrono::Utc::now().timestamp() - 3600,
    );
    let resp = router
        .clone()
        .oneshot(webhook_request(payload.clone(), Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Wrong secret.
    let sig = stripe_signature(
        payload.to_string().as_bytes(),
        "whsec_wrong",
        chrono::Utc::now().timestamp(),
    );
    let resp = router
        .oneshot(webhook_request(payload, Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Nothing was written.
    assert!(subscriptions::get_active_for_user(&pool, user_id)
        .await
        .unwrap()
        .is_none());
}

#[sqlx::test]
async fn webhook_subscription_updated_and_deleted_sync_status(pool: DbPool) {
    let state = setup_state_with_stripe(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (user_id, _session) = provider_session(&pool, "webhook-sync@example.com").await;
    subscriptions::activate_from_checkout(&pool, user_id, "pro", "sub_123")
        .await
        .unwrap();

    let updated = serde_json::json!({
        "id": "evt_2",
        "type": "customer.subscription.updated",
        "data": {"object": {
            "id": "sub_123",
            "status": "past_due",
            "current_period_end": 1_800_000_000
        }}
    });
    let sig = stripe_signature(
        updated.to_string().as_bytes(),
        WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let resp = router
        .clone()
        .oneshot(webhook_request(updated, Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let sub = subscriptions::get_active_for_user(&pool, user_id)
        .await
        .unwrap()
        .expect("past_due is still active");
    assert_eq!(sub.status, SubscriptionStatus::PastDue);
    assert_eq!(sub.current_period_end.timestamp(), 1_800_000_000);

    let deleted = serde_json::json!({
        "id": "evt_3",
        "type": "customer.subscription.deleted",
        "data": {"object": {"id": "sub_123"}}
    });
    let sig = stripe_signature(
        deleted.to_string().as_bytes(),
        WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let resp = router
        .oneshot(webhook_request(deleted, Some(sig)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(subscriptions::get_active_for_user(&pool, user_id)
        .await
        .unwrap()
        .is_none());
}

#[sqlx::test]
async fn webhook_requires_stripe_configuration(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await; // mock mode: no stripe
    let router = app(state);
    let resp = router
        .oneshot(webhook_request(serde_json::json!({"type": "ping"}), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// --- Plan-limit enforcement --------------------------------------------------

fn clinic_body(slug: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "Clinic",
        "slug": slug,
        "country_code": "US",
        "city": "Austin",
        "accreditations": [],
        "description": null
    })
}

#[sqlx::test]
async fn basic_plan_allows_one_clinic_then_rejects_with_402(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (_user_id, session) = provider_session(&pool, "limit-clinics@example.com").await;

    let resp = router
        .clone()
        .oneshot(post_json(
            "/me/clinics",
            Some(&session),
            clinic_body("clinic-one"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = router
        .oneshot(post_json(
            "/me/clinics",
            Some(&session),
            clinic_body("clinic-two"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 402);
    let body = body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Basic"));
}

#[sqlx::test]
async fn pro_plan_raises_the_clinic_limit(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (user_id, session) = provider_session(&pool, "limit-pro@example.com").await;
    subscriptions::activate_from_checkout(&pool, user_id, "pro", "sub_pro")
        .await
        .unwrap();

    for slug in ["p-one", "p-two", "p-three"] {
        let resp = router
            .clone()
            .oneshot(post_json("/me/clinics", Some(&session), clinic_body(slug)))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "slug {slug}");
    }
}

#[sqlx::test]
async fn basic_plan_allows_three_packages_then_rejects_with_402(pool: DbPool) {
    let (state, _payments) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);
    let (_user_id, session) = provider_session(&pool, "limit-packages@example.com").await;

    let resp = router
        .clone()
        .oneshot(post_json(
            "/me/clinics",
            Some(&session),
            clinic_body("pkg-clinic"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let clinic = body_json(resp).await;
    let clinic_id = clinic["id"].as_str().unwrap();

    chassis::treatments::seed(&pool).await.unwrap();
    let treatment_id = chassis::treatments::by_slug(&pool, "dental-implants")
        .await
        .unwrap()
        .unwrap()
        .id;
    let package_body = serde_json::json!({
        "treatment_id": treatment_id,
        "name": "Package",
        "price_min": 1000,
        "price_max": 2000,
        "duration_days": 7,
        "inclusions": [],
        "exclusions": []
    });

    let uri = format!("/me/clinics/{clinic_id}/packages");
    for i in 0..3 {
        let resp = router
            .clone()
            .oneshot(post_json(&uri, Some(&session), package_body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "package {i}");
    }
    let resp = router
        .oneshot(post_json(&uri, Some(&session), package_body))
        .await
        .unwrap();
    assert_eq!(resp.status(), 402);
}

/// The trait stays object-safe and the webhook plumbing is reachable through
/// the same provider handle the checkout endpoint uses.
#[allow(dead_code)]
fn assert_object_safe(_: Arc<dyn PaymentProvider>) {}
