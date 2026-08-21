use axum::body::Body;
use chassis::{
    auth, config::Config, connectors::email::MockEmailSender, db::DbPool, users,
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

fn clinic_body(name: &str, slug: &str, country: &str, city: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "slug": slug,
        "country_code": country,
        "city": city,
        "accreditations": ["JCI"],
        "description": "description"
    })
}

#[sqlx::test]
async fn create_clinic_requires_provider_or_platform_admin(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let patient = session_for(&pool, "patient@example.com", UserRole::Patient).await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/me/clinics")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", patient))
        .body(Body::from(clinic_body("Clinic", "clinic", "US", "Austin").to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);

    let provider = session_for(&pool, "provider@example.com", UserRole::ProviderAdmin).await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/me/clinics")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(clinic_body("Clinic", "clinic", "US", "Austin").to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test]
async fn create_and_list_own_clinics(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let provider = session_for(&pool, "owner@example.com", UserRole::ProviderAdmin).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/me/clinics")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(clinic_body("Mine", "mine", "US", "Denver").to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["slug"], "mine");
    assert_eq!(json["status"], "draft");

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/me/clinics")
        .header("Cookie", format!("session={}", provider))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn update_own_clinic(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let provider = session_for(&pool, "updater@example.com", UserRole::ProviderAdmin).await;
    let clinic = chassis::clinics::create(
        &pool,
        users::by_email(&pool, "updater@example.com")
            .await
            .unwrap()
            .unwrap()
            .id,
        "Old",
        "old-slug",
        "US",
        "Phoenix",
        &[],
        None,
    )
    .await
    .unwrap();

    let body = clinic_body("New", "new-slug", "CA", "Toronto");
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}", clinic.id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", provider))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "New");
    assert_eq!(json["slug"], "new-slug");

    let other = session_for(&pool, "other@example.com", UserRole::ProviderAdmin).await;
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri(format!("/me/clinics/{}", clinic.id))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("session={}", other))
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn public_detail_only_approved(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let owner_id = users::create(&pool, "public-owner@example.com", UserRole::ProviderAdmin, None)
        .await
        .unwrap()
        .id;
    let clinic = chassis::clinics::create(&pool, owner_id, "Public", "public-clinic", "US", "Miami", &[], None)
        .await
        .unwrap();

    let req = axum::http::Request::builder()
        .uri("/clinics/public-clinic")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);

    sqlx::query("UPDATE clinics SET status = 'approved' WHERE id = $1")
        .bind(clinic.id)
        .execute(&pool)
        .await
        .unwrap();

    let req = axum::http::Request::builder()
        .uri("/clinics/public-clinic")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert_eq!(json["slug"], "public-clinic");
    assert_eq!(json["status"], "approved");
}

#[sqlx::test]
async fn public_list_filters_and_approval(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let pool = state.pool.clone();
    let router = app(state);

    let owner_id = users::create(&pool, "list-owner@example.com", UserRole::ProviderAdmin, None)
        .await
        .unwrap()
        .id;
    let miami = chassis::clinics::create(&pool, owner_id, "Miami", "miami-clinic", "US", "Miami", &[], None)
        .await
        .unwrap();
    let _draft = chassis::clinics::create(&pool, owner_id, "Draft", "draft-clinic", "US", "Miami", &[], None)
        .await
        .unwrap();

    sqlx::query("UPDATE clinics SET status = 'approved' WHERE id = $1")
        .bind(miami.id)
        .execute(&pool)
        .await
        .unwrap();

    let req = axum::http::Request::builder()
        .uri("/clinics?country_code=US&city=Miami")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["slug"], "miami-clinic");

    let req = axum::http::Request::builder()
        .uri("/clinics?country_code=US&city=Boston")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let json = body_json(resp).await;
    assert!(json.as_array().unwrap().is_empty());
}
