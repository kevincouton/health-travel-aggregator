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

fn admin_req(
    method: &str,
    uri: String,
    session: &str,
    body: Option<&'static str>,
) -> axum::http::Request<Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Cookie", format!("session={session}"));
    match body {
        Some(b) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(b))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// Simulate the collector pipeline: insert a clinic with source attribution
/// in pending moderation status, owned by the system user.
async fn ingest_clinic(pool: &DbPool, name: &str, slug: &str) -> uuid::Uuid {
    let system_user = users::create(
        pool,
        &format!("collector-{}@health-travel.internal", uuid::Uuid::new_v4()),
        UserRole::ProviderAdmin,
        None,
    )
    .await
    .unwrap();
    sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO clinics
           (owner_user_id, name, slug, country_code, city, accreditations, description, status, source, external_ref)
         VALUES ($1, $2, $3, 'MY', 'Kuala Lumpur', $4, 'ingested', 'pending', 'mhtc', $5)
         RETURNING id",
    )
    .bind(system_user.id)
    .bind(name)
    .bind(slug)
    .bind(vec!["MHTC Member".to_string()])
    .bind(format!("https://example.com/{slug}#wilayah-persekutuan-0d3rf"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn public_search_hits(router: &axum::Router, query: &str) -> Vec<serde_json::Value> {
    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/clinics?q={query}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    json["clinics"].as_array().unwrap().clone()
}

#[sqlx::test]
async fn ingested_clinic_flows_through_verification_queue(pool: DbPool) {
    let state = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let admin = session_for(&pool, "admin@example.com", UserRole::PlatformAdmin).await;
    let clinic_id = ingest_clinic(&pool, "Collector General Hospital", "collector-general").await;

    // 1. Ingested listing starts pending and appears in the admin queue with
    //    its source attribution.
    let req = admin_req("GET", "/admin/clinics?status=pending".into(), &admin, None);
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status"], "pending");
    assert_eq!(arr[0]["source"], "mhtc");
    assert!(arr[0]["external_ref"]
        .as_str()
        .unwrap()
        .contains("collector-general"));

    // 2. Pending listings are invisible in public search.
    assert!(public_search_hits(&router, "Collector").await.is_empty());

    // 3. Approve: the listing appears in public search.
    let req = admin_req(
        "PATCH",
        format!("/admin/clinics/{clinic_id}/status"),
        &admin,
        Some(r#"{"status":"approved"}"#),
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_json(resp).await["status"], "approved");

    let hits = public_search_hits(&router, "Collector").await;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["slug"], "collector-general");

    // 4. Flag with a reason: suspended again, excluded from public search,
    //    reason recorded.
    let req = admin_req(
        "POST",
        format!("/admin/clinics/{clinic_id}/flag"),
        &admin,
        Some(r#"{"reason":"duplicate of existing listing"}"#),
    );
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["status"], "suspended");
    assert_eq!(json["flag_reason"], "duplicate of existing listing");

    assert!(public_search_hits(&router, "Collector").await.is_empty());

    // 5. Flag requires a non-empty reason.
    let req = admin_req(
        "POST",
        format!("/admin/clinics/{clinic_id}/flag"),
        &admin,
        Some(r#"{"reason":"  "}"#),
    );
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[sqlx::test]
async fn flag_endpoint_requires_platform_admin(pool: DbPool) {
    let state = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let provider = session_for(&pool, "provider@example.com", UserRole::ProviderAdmin).await;
    let clinic_id = ingest_clinic(&pool, "Guarded Clinic", "guarded-clinic").await;

    // Unauthenticated.
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/admin/clinics/{clinic_id}/flag"))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"reason":"spam"}"#))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    // Provider (not platform admin).
    let req = admin_req(
        "POST",
        format!("/admin/clinics/{clinic_id}/flag"),
        &provider,
        Some(r#"{"reason":"spam"}"#),
    );
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    let clinic = clinics::by_id(&pool, clinic_id).await.unwrap().unwrap();
    assert_eq!(clinic.status, clinics::ClinicStatus::Pending);
}
