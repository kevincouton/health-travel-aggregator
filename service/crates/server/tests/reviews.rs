use axum::body::Body;
use chassis::{
    auth,
    config::Config,
    connectors::{email::MockEmailSender, payments::MockPaymentProvider},
    db::DbPool,
    inquiries::InquiryStatus,
    users,
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
        payments: Arc::new(MockPaymentProvider::new()),
        stripe: None,
    };
    (state, email)
}

async fn patient_session(pool: &DbPool, email: &str) -> (Uuid, String) {
    let user = users::create(pool, email, UserRole::Patient, None)
        .await
        .unwrap();
    let session = auth::create_session(pool, user.id, 1).await.unwrap();
    (user.id, session)
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

async fn create_converted_inquiry(
    pool: &DbPool,
    clinic_id: Uuid,
    owner_id: Uuid,
    patient_id: Uuid,
    contact_email: &str,
) -> Uuid {
    let inquiry =
        chassis::inquiries::create(pool, patient_id, clinic_id, None, None, None, contact_email)
            .await
            .unwrap();
    chassis::inquiries::update_status(pool, inquiry.id, owner_id, InquiryStatus::Converted)
        .await
        .unwrap()
        .unwrap()
        .id
}

fn review_body(inquiry_id: Uuid, rating: i32, comment: &str) -> serde_json::Value {
    serde_json::json!({
        "inquiry_id": inquiry_id,
        "rating": rating,
        "comment": comment
    })
}

#[sqlx::test]
async fn create_review_requires_auth(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/test-clinic/reviews")
        .header("Content-Type", "application/json")
        .body(Body::from(
            review_body(Uuid::new_v4(), 5, "Good").to_string(),
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test]
async fn patient_can_create_review_after_converted_inquiry(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-review@example.com", "review-clinic").await;
    let (patient_id, patient) = patient_session(&pool, "patient-review@example.com").await;
    let inquiry_id = create_converted_inquiry(
        &pool,
        clinic_id,
        owner_id,
        patient_id,
        "patient-review@example.com",
    )
    .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/review-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(
            review_body(inquiry_id, 5, "Excellent clinic").to_string(),
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["rating"], 5);
    assert_eq!(json["comment"], "Excellent clinic");
    assert_eq!(json["inquiry_id"], inquiry_id.to_string());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/clinics/review-clinic/reviews")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let list = json.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["rating"], 5);
}

#[sqlx::test]
async fn patient_cannot_review_without_converted_inquiry(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, _owner_id) =
        create_clinic(&pool, "owner-no-review@example.com", "no-review-clinic").await;
    let (patient_id, patient) = patient_session(&pool, "patient-no-review@example.com").await;

    let inquiry = chassis::inquiries::create(
        &pool,
        patient_id,
        clinic_id,
        None,
        None,
        None,
        "patient-no-review@example.com",
    )
    .await
    .unwrap();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/no-review-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(review_body(inquiry.id, 4, "Nice").to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn patient_cannot_review_with_wrong_inquiry_id(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-wrong-inq@example.com", "wrong-inq-clinic").await;
    let (patient_id, patient) = patient_session(&pool, "patient-wrong-inq@example.com").await;
    let _inquiry_id = create_converted_inquiry(
        &pool,
        clinic_id,
        owner_id,
        patient_id,
        "patient-wrong-inq@example.com",
    )
    .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/wrong-inq-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(
            review_body(Uuid::new_v4(), 4, "Nice").to_string(),
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[sqlx::test]
async fn review_requires_patient_role(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (_clinic_id, _owner_id) =
        create_clinic(&pool, "owner-role@example.com", "role-clinic").await;
    let provider = session_for(&pool, "provider-role@example.com", UserRole::ProviderAdmin).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/role-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(
            review_body(Uuid::new_v4(), 5, "Good").to_string(),
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn duplicate_review_is_conflict(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) = create_clinic(&pool, "owner-dup@example.com", "dup-clinic").await;
    let (patient_id, patient) = patient_session(&pool, "patient-dup@example.com").await;
    let inquiry_id = create_converted_inquiry(
        &pool,
        clinic_id,
        owner_id,
        patient_id,
        "patient-dup@example.com",
    )
    .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/dup-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(review_body(inquiry_id, 5, "First").to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/dup-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(review_body(inquiry_id, 4, "Second").to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 409);
}

#[sqlx::test]
async fn public_can_list_reviews(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-public@example.com", "public-clinic").await;
    let (patient_id, patient) = patient_session(&pool, "patient-public@example.com").await;
    let inquiry_id = create_converted_inquiry(
        &pool,
        clinic_id,
        owner_id,
        patient_id,
        "patient-public@example.com",
    )
    .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/clinics/public-clinic/reviews")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(review_body(inquiry_id, 3, "Okay").to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/clinics/public-clinic/reviews")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
}
