use axum::body::Body;
use chassis::{
    auth, clinics, config::Config, connectors::{email::MockEmailSender, payments::MockPaymentProvider}, db::DbPool, users,
    users::UserRole,
};
use server::{router::app, state::AppState};
use std::sync::Arc;
use tower::ServiceExt;

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

async fn setup_state(pool: DbPool) -> (AppState, Arc<MockEmailSender>) {
    chassis::db::migrate(&pool).await.unwrap();
    let email = Arc::new(MockEmailSender::new());
    let state = AppState {
        cfg: test_config(),
        pool,
        email: email.clone(),
        payments: Arc::new(MockPaymentProvider::new()),
        stripe: None,
    };
    (state, email)
}

async fn session_for(pool: &DbPool, email: &str, role: UserRole) -> String {
    let user = users::create(pool, email, role, None).await.unwrap();
    auth::create_session(pool, user.id, 1).await.unwrap()
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[sqlx::test]
async fn admin_can_list_and_approve_pending_clinics(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let admin = session_for(&pool, "admin@example.com", UserRole::PlatformAdmin).await;
    let provider_user = users::create(&pool, "provider@example.com", UserRole::ProviderAdmin, None)
        .await
        .unwrap();
    let clinic = clinics::create(
        &pool,
        provider_user.id,
        "Pending Clinic",
        "pending-clinic",
        "US",
        "Austin",
        &["JCI".to_string()],
        Some("A pending clinic"),
    )
    .await
    .unwrap();
    clinics::update_status(&pool, clinic.id, clinics::ClinicStatus::Pending)
        .await
        .unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics?status=pending")
        .header("Cookie", format!("session={}", admin))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["slug"], "pending-clinic");
    assert_eq!(arr[0]["status"], "pending");

    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/admin/clinics/{}/status", clinic.id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", admin))
        .body(Body::from(r#"{"status":"approved"}"#))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["id"], clinic.id.to_string());
    assert_eq!(json["status"], "approved");

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics?status=approved")
        .header("Cookie", format!("session={}", admin))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status"], "approved");
}

#[sqlx::test]
async fn non_admin_is_forbidden_from_admin_endpoints(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let provider = session_for(&pool, "provider@example.com", UserRole::ProviderAdmin).await;
    let patient = session_for(&pool, "patient@example.com", UserRole::Patient).await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics?status=pending")
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics")
        .header("Cookie", format!("session={}", patient))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri("/admin/clinics/00000000-0000-0000-0000-000000000000/status")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(r#"{"status":"approved"}"#))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn admin_clinics_defaults_to_pending_and_rejects_invalid_status(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let admin = session_for(&pool, "admin@example.com", UserRole::PlatformAdmin).await;
    let provider_user = users::create(&pool, "provider@example.com", UserRole::ProviderAdmin, None)
        .await
        .unwrap();
    let clinic = clinics::create(
        &pool,
        provider_user.id,
        "Pending Clinic",
        "pending-clinic-2",
        "US",
        "Austin",
        &[],
        None,
    )
    .await
    .unwrap();
    clinics::update_status(&pool, clinic.id, clinics::ClinicStatus::Pending)
        .await
        .unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics")
        .header("Cookie", format!("session={}", admin))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/clinics?status=not_a_status")
        .header("Cookie", format!("session={}", admin))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri("/admin/clinics/00000000-0000-0000-0000-000000000000/status")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", admin))
        .body(Body::from(r#"{"status":"not_a_status"}"#))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 422);
}
