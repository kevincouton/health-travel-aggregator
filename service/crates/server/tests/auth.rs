use axum::body::Body;
use chassis::config::Config;
use chassis::connectors::email::MockEmailSender;
use chassis::db::DbPool;
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

fn session_cookie_value(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get_all("set-cookie")
        .iter()
        .find(|v| v.to_str().ok().map_or(false, |s| s.starts_with("session=")))
        .and_then(|v| v.to_str().ok().map(|s| s.to_string()))
}

#[sqlx::test]
async fn provider_register_and_login(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/provider/register")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"email":"provider@example.com","password":"password123"}"#,
        ))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let cookie = session_cookie_value(resp.headers()).expect("session cookie set on register");
    assert!(cookie.starts_with("session="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["user"]["email"], "provider@example.com");
    assert_eq!(json["user"]["role"], "provider_admin");

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/provider/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"email":"provider@example.com","password":"password123"}"#,
        ))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let cookie = session_cookie_value(resp.headers()).expect("session cookie set on login");
    assert!(cookie.starts_with("session="));
}

#[sqlx::test]
async fn magic_link_flow(pool: DbPool) {
    let (state, email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/magic-link")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"email":"patient@example.com"}"#))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let sent = email.sent.lock().await;
    assert_eq!(sent.len(), 1);
    let token = sent[0]
        .split("token=")
        .nth(1)
        .expect("token in magic link")
        .to_string();
    drop(sent);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/magic-link/verify")
        .header("Content-Type", "application/json")
        .body(Body::from(format!(r#"{{"token":"{}"}}"#, token)))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let cookie = session_cookie_value(resp.headers()).expect("session cookie set on verify");
    assert!(cookie.starts_with("session="));

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["user"]["email"], "patient@example.com");
    assert!(json["user"]["email_verified_at"].is_string());

    let session_value = cookie.split(';').next().unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header("Cookie", session_value)
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["email"], "patient@example.com");

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("Cookie", session_value)
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let cleared = session_cookie_value(resp.headers()).expect("logout clears cookie");
    assert!(cleared.starts_with("session=;"));
    assert!(cleared.contains("Max-Age=0"));
}

#[sqlx::test]
async fn me_requires_session(pool: DbPool) {
    let (state, _email) = setup_state(pool).await;
    let router = app(state);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/auth/me")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
