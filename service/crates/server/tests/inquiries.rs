use axum::body::Body;
use chassis::{
    auth, config::Config, connectors::email::MockEmailSender, db::DbPool, treatments, users,
    users::UserRole,
};
use server::{router::app, state::AppState};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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

async fn create_clinic(pool: &DbPool, owner_email: &str, slug: &str) -> (Uuid, Uuid) {
    let owner = users::create(pool, owner_email, UserRole::ProviderAdmin, None)
        .await
        .unwrap();
    let clinic = chassis::clinics::create(
        pool,
        owner.id,
        "Test Clinic",
        slug,
        "US",
        "Austin",
        &[],
        None,
    )
    .await
    .unwrap();
    sqlx::query("UPDATE clinics SET status = 'approved' WHERE id = $1")
        .bind(clinic.id)
        .execute(pool)
        .await
        .unwrap();
    (clinic.id, owner.id)
}

async fn treatment_id(pool: &DbPool, slug: &str) -> Uuid {
    treatments::seed(pool).await.unwrap();
    treatments::by_slug(pool, slug).await.unwrap().unwrap().id
}

async fn create_package(pool: &DbPool, clinic_id: Uuid, treatment_id: Uuid) -> Uuid {
    let package = chassis::packages::create(
        pool,
        clinic_id,
        treatment_id,
        "Test Package",
        Some(1_000),
        Some(2_000),
        Some(7),
        &["consultation".into()],
        &["flights".into()],
    )
    .await
    .unwrap();
    package.id
}

fn inquiry_body(clinic_id: Uuid, package_id: Option<Uuid>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "clinic_id": clinic_id,
        "contact_email": "patient@example.com",
        "preferred_dates": "March 2026",
        "medical_notes": "Notes"
    });
    if let Some(id) = package_id {
        body["package_id"] = serde_json::json!(id);
    }
    body
}

#[sqlx::test]
async fn create_inquiry_requires_auth(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .body(Body::from(inquiry_body(Uuid::new_v4(), None).to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test]
async fn patient_can_create_and_list_inquiries(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, _owner_id) =
        create_clinic(&pool, "owner-list@example.com", "list-clinic").await;
    let patient = session_for(&pool, "patient-list@example.com", UserRole::Patient).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(inquiry_body(clinic_id, None).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["clinic_id"], clinic_id.to_string());
    assert_eq!(json["status"], "new");

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/me/inquiries")
        .header("Cookie", format!("session={}", patient))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let list = json.as_array().unwrap();
    assert_eq!(list.len(), 1);
}

#[sqlx::test]
async fn provider_can_list_and_update_status(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-update@example.com", "update-clinic").await;
    let treatment_id = treatment_id(&pool, "dental-implants").await;
    let package_id = create_package(&pool, clinic_id, treatment_id).await;
    let patient = session_for(&pool, "patient-update@example.com", UserRole::Patient).await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(
            inquiry_body(clinic_id, Some(package_id)).to_string(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let inquiry_id = json["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/me/inquiries")
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json.as_array().unwrap().len(), 1);

    let body = serde_json::json!({ "status": "contacted" });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/inquiries/{}/status", inquiry_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["status"], "contacted");
}

#[sqlx::test]
async fn other_provider_cannot_update_status(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-secure@example.com", "secure-clinic").await;
    let patient = session_for(&pool, "patient-secure@example.com", UserRole::Patient).await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(inquiry_body(clinic_id, None).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let inquiry_id = json["id"].as_str().unwrap();

    let other = session_for(&pool, "other-secure@example.com", UserRole::ProviderAdmin).await;
    let body = serde_json::json!({ "status": "closed" });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/inquiries/{}/status", inquiry_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", other))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);

    let body = serde_json::json!({ "status": "closed" });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/inquiries/{}/status", inquiry_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test]
async fn patient_cannot_update_status(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, _owner_id) =
        create_clinic(&pool, "owner-patient@example.com", "patient-clinic").await;
    let patient = session_for(&pool, "patient-status@example.com", UserRole::Patient).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(inquiry_body(clinic_id, None).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let inquiry_id = json["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "closed" });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/inquiries/{}/status", inquiry_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn package_must_belong_to_clinic(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_a, owner_a) = create_clinic(&pool, "owner-a@example.com", "clinic-a").await;
    let (clinic_b, owner_b) = create_clinic(&pool, "owner-b@example.com", "clinic-b").await;
    let treatment_id = treatment_id(&pool, "hair-transplant").await;
    let package_b = create_package(&pool, clinic_b, treatment_id).await;
    let patient = session_for(&pool, "patient-bad@example.com", UserRole::Patient).await;
    let _provider_a = session_for_user(&pool, owner_a).await;
    let _provider_b = session_for_user(&pool, owner_b).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(
            inquiry_body(clinic_a, Some(package_b)).to_string(),
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[sqlx::test]
async fn inquiry_creation_sends_email_notification_to_clinic_owner(pool: DbPool) {
    let (state, email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let owner_email = "owner-notify@example.com";
    let (clinic_id, _owner_id) = create_clinic(&pool, owner_email, "notify-clinic").await;
    let treatment_id = treatment_id(&pool, "dental-implants").await;
    let package_id = create_package(&pool, clinic_id, treatment_id).await;
    let patient = session_for(&pool, "patient-notify@example.com", UserRole::Patient).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/inquiries")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(
            inquiry_body(clinic_id, Some(package_id)).to_string(),
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let sent = email.sent.lock().await;
    assert_eq!(sent.len(), 1);
    let expected = format!(
        "inquiry to {} for Test Clinic from patient@example.com",
        owner_email
    );
    assert_eq!(sent[0], expected);
}

async fn session_for_user(pool: &DbPool, user_id: Uuid) -> String {
    auth::create_session(pool, user_id, 1).await.unwrap()
}
