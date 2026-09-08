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
        cors_origin: "http://localhost:3000".into(),
        rate_limit_auth_per_minute: 100,
        rate_limit_write_per_minute: 100,
    }
}

fn state_with(pool: DbPool, cfg: Config) -> AppState {
    AppState {
        cfg,
        pool,
        email: Arc::new(MockEmailSender::new()),
    }
}

fn post_json(uri: &str, body: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

// --- CORS ---

#[sqlx::test]
async fn cors_preflight_allows_configured_origin(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/auth/magic-link")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "POST")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(
        resp.headers().get("access-control-allow-origin").unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        resp.headers()
            .get("access-control-allow-credentials")
            .unwrap(),
        "true"
    );
}

#[sqlx::test]
async fn cors_preflight_rejects_unknown_origin(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/auth/magic-link")
        .header("Origin", "https://evil.example.com")
        .header("Access-Control-Request-Method", "POST")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}

#[sqlx::test]
async fn cors_disabled_when_origin_unset(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let mut cfg = test_config();
    cfg.cors_origin = "".into();
    let router = app(state_with(pool, cfg));

    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/auth/magic-link")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "POST")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}

// --- CSRF origin check ---

#[sqlx::test]
async fn csrf_rejects_foreign_origin_on_mutation(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/magic-link")
        .header("Content-Type", "application/json")
        .header("Origin", "https://evil.example.com")
        .body(Body::from(r#"{"email":"patient@example.com"}"#))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn csrf_allows_configured_origin_on_mutation(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/magic-link")
        .header("Content-Type", "application/json")
        .header("Origin", "http://localhost:3000")
        .body(Body::from(r#"{"email":"patient@example.com"}"#))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test]
async fn csrf_ignores_safe_methods_with_foreign_origin(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/treatments")
        .header("Origin", "https://evil.example.com")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

// --- Rate limiting ---

#[sqlx::test]
async fn rate_limit_auth_endpoints_return_429(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let mut cfg = test_config();
    cfg.rate_limit_auth_per_minute = 3;
    let router = app(state_with(pool, cfg));

    for i in 0..4 {
        let resp = router
            .clone()
            .oneshot(post_json(
                "/auth/magic-link",
                r#"{"email":"a@example.com"}"#,
            ))
            .await
            .unwrap();
        if i < 3 {
            assert_eq!(resp.status(), 200, "request {i} should pass");
        } else {
            assert_eq!(resp.status(), 429, "request {i} should be throttled");
        }
    }
}

#[sqlx::test]
async fn rate_limit_write_endpoints_return_429(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let mut cfg = test_config();
    cfg.rate_limit_write_per_minute = 2;
    let router = app(state_with(pool, cfg));

    // Unauthenticated inquiries are 401 but still consume rate-limit budget.
    for i in 0..3 {
        let resp = router
            .clone()
            .oneshot(post_json("/inquiries", r#"{}"#))
            .await
            .unwrap();
        if i < 2 {
            assert_eq!(resp.status(), 401, "request {i} should reach handler");
        } else {
            assert_eq!(resp.status(), 429, "request {i} should be throttled");
        }
    }
}

#[sqlx::test]
async fn rate_limit_does_not_touch_unthrottled_routes(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let mut cfg = test_config();
    cfg.rate_limit_auth_per_minute = 1;
    cfg.rate_limit_write_per_minute = 1;
    let router = app(state_with(pool, cfg));

    for _ in 0..5 {
        let resp = router
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
}

// --- Session cookie hardening ---

#[sqlx::test]
async fn session_cookie_has_secure_attribute_under_https(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let mut cfg = test_config();
    cfg.app_url = "https://health-travel.lucanian.app".into();
    let router = app(state_with(pool, cfg));

    let resp = router
        .oneshot(post_json(
            "/auth/provider/register",
            r#"{"email":"provider@example.com","password":"password123"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let cookie = resp
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Secure"));
}

#[sqlx::test]
async fn logout_invalidates_session_server_side(pool: DbPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let router = app(state_with(pool, test_config()));

    let resp = router
        .clone()
        .oneshot(post_json(
            "/auth/provider/register",
            r#"{"email":"provider@example.com","password":"password123"}"#,
        ))
        .await
        .unwrap();
    let cookie = resp
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let session = cookie.split(';').next().unwrap().to_string();

    let resp = router
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header("Cookie", &session)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // The same session token must no longer authenticate.
    let resp = router
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/auth/me")
                .header("Cookie", &session)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
