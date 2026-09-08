use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chassis::error::ApiError;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Fixed-window per-IP rate limiter, process-local. Adequate for the single
/// replica this service runs; swap for a shared store if replicas scale out.
#[derive(Clone)]
pub struct RateLimiter {
    max: u32,
    window: Duration,
    buckets: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>,
}

impl RateLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Self {
            max: max.max(1),
            window,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter poisoned");
        let entry = buckets.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) >= self.window {
            *entry = (0, now);
        }
        entry.0 += 1;
        entry.0 <= self.max
    }
}

pub fn client_ip(req: &Request<Body>) -> IpAddr {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

pub async fn rate_limit(req: Request<Body>, next: Next, limiter: RateLimiter) -> Response {
    if limiter.allow(client_ip(&req)) {
        next.run(req).await
    } else {
        ApiError::TooManyRequests.into_response()
    }
}

/// CSRF defense-in-depth on top of the SameSite=Lax session cookie: browsers
/// always send Origin on cross-site POSTs, so a mutating request whose Origin
/// is not the configured app/CORS origin is rejected. Requests without an
/// Origin header (non-browser API clients) pass through.
pub async fn origin_guard(req: Request<Body>, next: Next, allowed: Arc<Vec<String>>) -> Response {
    if !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        if let Some(origin) = req
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
        {
            if !allowed.iter().any(|o| o == origin) {
                return (
                    StatusCode::FORBIDDEN,
                    axum::Json(serde_json::json!({ "error": "forbidden origin" })),
                )
                    .into_response();
            }
        }
    }
    next.run(req).await
}
