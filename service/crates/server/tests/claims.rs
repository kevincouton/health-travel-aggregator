use axum::body::Body;
use chassis::{
    auth, clinics, config::Config, connectors::email::MockEmailSender, db::DbPool, users,
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

async fn setup_state(pool: DbPool) -> AppState {
    chassis::db::migrate(&pool).await.unwrap();
    AppState {
        cfg: test_config(),
        pool,
        email: Arc::new(MockEmailSender::new()),
    }
}

async fn session_for(pool: &DbPool, email: &str, role: UserRole) -> (uuid::Uuid, String) {
    let user = users::create(pool, email, role, None).await.unwrap();
    let session = auth::create_session(pool, user.id, 1).await.unwrap();
    (user.id, session)
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn authed_json(
    method: &str,
    uri: String,
    session: &str,
    body: &'static str,
) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Cookie", format!("session={session}"))
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

#[sqlx::test]
async fn provider_claim_intent_resolves_through_admin_queue(pool: DbPool) {
    let state = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (_admin_id, admin) = session_for(&pool, "admin@example.com", UserRole::PlatformAdmin).await;
    let (provider_id, provider) =
        session_for(&pool, "provider@example.com", UserRole::ProviderAdmin).await;
    let (_patient_id, patient) = session_for(&pool, "patient@example.com", UserRole::Patient).await;

    // Collector-ingested listing owned by the system user.
    let system_user = users::create(
        &pool,
        "collector@health-travel.internal",
        UserRole::ProviderAdmin,
        None,
    )
    .await
    .unwrap();
    let clinic_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO clinics
           (owner_user_id, name, slug, country_code, city, status, source, external_ref)
         VALUES ($1, 'Claimable Clinic', 'claimable-clinic', 'TR', 'Istanbul', 'pending', 'temos', '16725')
         RETURNING id",
    )
    .bind(system_user.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Patients cannot file claim intents.
    let req = authed_json(
        "POST",
        "/clinics/claimable-clinic/claim".into(),
        &patient,
        r#"{"message":"mine"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    // Provider files a claim intent.
    let req = authed_json(
        "POST",
        "/clinics/claimable-clinic/claim".into(),
        &provider,
        r#"{"message":"I am the clinic director"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let claim = body_json(resp).await;
    assert_eq!(claim["status"], "pending");
    assert_eq!(claim["clinic_id"], clinic_id.to_string());
    let claim_id = claim["id"].as_str().unwrap().to_string();

    // Duplicate open claim conflicts.
    let req = authed_json(
        "POST",
        "/clinics/claimable-clinic/claim".into(),
        &provider,
        r#"{"message":"again"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 409);

    // Admin sees the pending claim in the queue.
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/claims?status=pending")
        .header("Cookie", format!("session={admin}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], claim_id);

    // Approving the claim transfers clinic ownership to the claimant.
    let req = authed_json(
        "PATCH",
        format!("/admin/claims/{claim_id}/status"),
        &admin,
        r#"{"status":"approved"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["status"], "approved");
    assert!(json["resolved_at"].is_string());

    let clinic = clinics::by_id(&pool, clinic_id).await.unwrap().unwrap();
    assert_eq!(clinic.owner_user_id, provider_id);

    // A resolved claim leaves the pending queue and cannot be re-resolved.
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/admin/claims?status=pending")
        .header("Cookie", format!("session={admin}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(body_json(resp).await.as_array().unwrap().len(), 0);

    let req = authed_json(
        "PATCH",
        format!("/admin/claims/{claim_id}/status"),
        &admin,
        r#"{"status":"rejected"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn rejected_claim_keeps_system_ownership(pool: DbPool) {
    let state = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (_admin_id, admin) = session_for(&pool, "admin@example.com", UserRole::PlatformAdmin).await;
    let (_provider_id, provider) =
        session_for(&pool, "provider@example.com", UserRole::ProviderAdmin).await;
    let system_user = users::create(
        &pool,
        "collector@health-travel.internal",
        UserRole::ProviderAdmin,
        None,
    )
    .await
    .unwrap();
    let clinic_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO clinics
           (owner_user_id, name, slug, country_code, city, status, source, external_ref)
         VALUES ($1, 'Rejectable Clinic', 'rejectable-clinic', 'TH', 'Bangkok', 'pending', 'gha', 'x/y')
         RETURNING id",
    )
    .bind(system_user.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let req = authed_json(
        "POST",
        "/clinics/rejectable-clinic/claim".into(),
        &provider,
        r#"{"message":"please"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    let claim = body_json(resp).await;
    let claim_id = claim["id"].as_str().unwrap().to_string();

    let req = authed_json(
        "PATCH",
        format!("/admin/claims/{claim_id}/status"),
        &admin,
        r#"{"status":"rejected"}"#,
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let clinic = clinics::by_id(&pool, clinic_id).await.unwrap().unwrap();
    assert_eq!(clinic.owner_user_id, system_user.id);

    // After rejection the provider may file a fresh claim.
    let req = authed_json(
        "POST",
        "/clinics/rejectable-clinic/claim".into(),
        &provider,
        r#"{"message":"reconsider"}"#,
    );
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}
