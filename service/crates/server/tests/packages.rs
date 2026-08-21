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

async fn session_for_user(pool: &DbPool, user_id: Uuid) -> String {
    auth::create_session(pool, user_id, 1).await.unwrap()
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn package_body(treatment_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "treatment_id": treatment_id,
        "name": "Package",
        "price_min": 1000,
        "price_max": 2000,
        "duration_days": 7,
        "inclusions": ["consultation", "hotel"],
        "exclusions": ["flights"]
    })
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

#[sqlx::test]
async fn create_package_requires_provider_or_platform_admin(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) = create_clinic(
        &pool,
        "provider-create@example.com",
        "provider-create-clinic",
    )
    .await;
    let treatment_id = treatment_id(&pool, "dental-implants").await;

    let patient = session_for(&pool, "patient-create@example.com", UserRole::Patient).await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    let provider = session_for_user(&pool, owner_id).await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test]
async fn create_and_list_packages(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-list@example.com", "owner-list-clinic").await;
    let treatment_id = treatment_id(&pool, "hair-transplant").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Package");
    assert!(!json["is_published"].as_bool().unwrap());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn update_package(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-update@example.com", "owner-update-clinic").await;
    let treatment_id = treatment_id(&pool, "ivf").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let package_id = json["id"].as_str().unwrap();

    let body = serde_json::json!({
        "name": "Updated Package",
        "price_min": 500,
        "price_max": 900,
        "duration_days": 3,
        "inclusions": ["pickup"],
        "exclusions": ["meals"],
        "is_published": true
    });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}/packages/{}", clinic_id, package_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "Updated Package");
    assert!(json["is_published"].as_bool().unwrap());

    let other = session_for(&pool, "other-update@example.com", UserRole::ProviderAdmin).await;
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}/packages/{}", clinic_id, package_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", other))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn delete_package(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "owner-delete@example.com", "owner-delete-clinic").await;
    let treatment_id = treatment_id(&pool, "cosmetic-surgery").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let package_id = json["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri(format!("/me/clinics/{}/packages/{}", clinic_id, package_id))
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/packages/{}", package_id))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn public_list_only_published_for_approved_clinic(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "public-list@example.com", "public-list-clinic").await;
    let treatment_id = treatment_id(&pool, "hip-replacement").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let package_id = json["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .uri("/clinics/public-list-clinic/packages")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert!(json.as_array().unwrap().is_empty());

    let body = serde_json::json!({
        "name": "Published",
        "price_min": 1,
        "price_max": 2,
        "duration_days": 1,
        "inclusions": [],
        "exclusions": [],
        "is_published": true
    });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}/packages/{}", clinic_id, package_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .uri("/clinics/public-list-clinic/packages")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn public_detail_shows_published_or_owner(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "public-detail@example.com", "public-detail-clinic").await;
    let treatment_id = treatment_id(&pool, "cataract-surgery").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let package_id = json["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .uri(format!("/packages/{}", package_id))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);

    let req = axum::http::Request::builder()
        .uri(format!("/packages/{}", package_id))
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let other = session_for(&pool, "other-detail@example.com", UserRole::ProviderAdmin).await;
    let req = axum::http::Request::builder()
        .uri(format!("/packages/{}", package_id))
        .header("Cookie", format!("session={}", other))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);

    let body = serde_json::json!({
        "treatment_id": treatment_id,
        "name": "Published Detail",
        "price_min": 1,
        "price_max": 2,
        "duration_days": 1,
        "inclusions": [],
        "exclusions": [],
        "is_published": false
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let published_id = json["id"].as_str().unwrap();

    let body = serde_json::json!({
        "name": "Published Detail",
        "price_min": 1,
        "price_max": 2,
        "duration_days": 1,
        "inclusions": [],
        "exclusions": [],
        "is_published": true
    });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!(
            "/me/clinics/{}/packages/{}",
            clinic_id, published_id
        ))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .uri(format!("/packages/{}", published_id))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test]
async fn public_detail_includes_clinic_and_treatment_names(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let (clinic_id, owner_id) =
        create_clinic(&pool, "enriched-detail@example.com", "enriched-detail-clinic").await;
    let treatment_id = treatment_id(&pool, "dental-implants").await;
    let provider = session_for_user(&pool, owner_id).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/me/clinics/{}/packages", clinic_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(package_body(treatment_id).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let json = body_json(resp).await;
    let package_id = json["id"].as_str().unwrap();

    let body = serde_json::json!({
        "name": "Enriched Package",
        "price_min": 100,
        "price_max": 200,
        "duration_days": 5,
        "inclusions": ["consultation"],
        "exclusions": ["flights"],
        "is_published": true
    });
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}/packages/{}", clinic_id, package_id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let req = axum::http::Request::builder()
        .uri(format!("/packages/{}", package_id))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["clinic_name"], "Test Clinic");
    assert_eq!(json["clinic_slug"], "enriched-detail-clinic");
    assert_eq!(json["treatment_name"], "Dental Implants");
    assert_eq!(json["name"], "Enriched Package");
}
