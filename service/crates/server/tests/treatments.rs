use axum::body::Body;
use chassis::{config::Config, connectors::email::MockEmailSender, db::DbPool};
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
    chassis::treatments::seed(&pool).await.unwrap();
    let email = Arc::new(MockEmailSender::new());
    let state = AppState {
        cfg: test_config(),
        pool,
        email: email.clone(),
    };
    (state, email)
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[sqlx::test]
async fn list_treatments_returns_seeded_data(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .uri("/treatments")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let json = body_json(resp).await;
    let treatments = json["treatments"].as_array().unwrap();
    assert!(!treatments.is_empty());
    assert!(treatments
        .iter()
        .all(|t| t["slug"].is_string() && t["name"].is_string()));
}
