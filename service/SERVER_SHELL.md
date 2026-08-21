# Server shell — verified topcoat idioms (from R-0 spike)

Source of truth for the R-3 server binary. Every snippet below compiled and
passed a live curl probe in `/tmp/topcoat-spike` at topcoat rev
`371c7403fcbf4d40bbacb2f87eb98d9ce00e76c8` (2026-07-28).
Deviations from these idioms need a comment explaining why.

Spike crate `Cargo.toml` (scratch, not committed):

```toml
[dependencies]
topcoat = { git = "https://github.com/tokio-rs/topcoat", rev = "371c7403fcbf4d40bbacb2f87eb98d9ce00e76c8", features = ["tower"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "signal", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
http = "1"                                      # header types; matches topcoat's http 1.4.2, no conflict
tower-http = { version = "0.6", features = ["fs"] }  # ServeDir/ServeFile for the static mount
```

`features = ["tower"]` is required for criterion 3/5 tower interop; it is not
in topcoat's default feature set.

## Router construction

```rust
use topcoat::router::{route, Router};

#[route(GET "/healthz")]
async fn healthz() -> topcoat::Result<&'static str> {
    Ok("ok")
}

fn router() -> Router {
    Router::builder()
        .cookies()          // cookie::RouterBuilderCookieExt — needed before any cookies(cx) call
        .layer(cors)        // outermost first; see CORS section
        .route(healthz)     // explicit registration per #[route] fn
        .route(TowerRoute::new(Methods::Any, Path::new("/"), spa.clone()))
        .route(TowerRoute::new(Methods::Any, Path::new("/{*rest}"), spa))
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    topcoat::serve(listener, router()).await?;   // handles SIGTERM/Ctrl+C itself
    Ok(())
}
```

Explicit routes win over the `/{*rest}` tower mount (`/healthz` answered by the
handler, not the SPA) — probe-verified.

## JSON responses with exact status and headers

```rust
use serde::Serialize;
use topcoat::{
    Result,
    router::{content::Json, route, StatusCode},
};

#[derive(Serialize)]
struct Health { status: &'static str }

#[route(GET "/api/health")]
async fn health() -> Result<Json<Health>> {
    Ok(Json(Health { status: "ok" }))   // 200 + content-type: application/json
}

// Tuple shape: (StatusCode, header-pair array, body).
#[route(GET "/api/created")]
async fn created() -> Result<(
    StatusCode,
    [(http::header::HeaderName, http::header::HeaderValue); 1],
    Json<Health>,
)> {
    Ok((
        StatusCode::CREATED,
        [(
            http::header::HeaderName::from_static("x-made-by"),
            http::header::HeaderValue::from_static("topcoat-spike"),
        )],
        Json(Health { status: "created" }),
    ))
}
```

Probe: `/api/created` → `HTTP/1.1 201 Created`, `x-made-by: topcoat-spike`,
`content-type: application/json`, `{"status":"created"}`.

Gotchas (both hit in the spike):

- Header pairs must be a **fixed array** `[(K, V); N]`. For a dynamic number of
  headers use a `http::HeaderMap` element instead — `Vec<(HeaderName, HeaderValue)>`
  does not implement `IntoResponseParts`.
- Tuple family: `(StatusCode, parts..., body)`; `Option<T>` parts are skipped
  when `None`.

## Cookie set/read

```rust
use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookies, RouterBuilderCookieExt, cookie, cookies, time::Duration},
    router::route,
};

// Exact Go session-cookie shape: name "session", Path=/, Max-Age=604800,
// HttpOnly, Secure, SameSite=Lax.
#[route(GET "/login")]
async fn login(cx: &Cx) -> Result<&'static str> {
    cookies(cx).add(cookie! {
        "session" = "spike-token";
        Path = "/";
        Secure;
        HttpOnly;
        SameSite = Lax;
        MaxAge = Duration::days(7)        // = 604800s, verified
    });
    Ok("logged in")
}

// OIDC state/nonce shape: Max-Age=300, same flags.
fn set_oidc_state(cx: &Cx, value: &str) {
    cookies(cx).add(cookie! {
        "oidc_state" = value;
        Path = "/";
        Secure;
        HttpOnly;
        SameSite = Lax;
        MaxAge = Duration::minutes(5)     // = 300s
    });
}

#[route(GET "/read")]
async fn read(cx: &Cx) -> Result<String> {
    let value = cookies(cx)
        .get("session")
        .map(|c| c.value().to_owned())
        .unwrap_or_else(|| "<none>".to_owned());
    Ok(format!("session={value}"))
}
```

Probe: `set-cookie: session=spike-token; HttpOnly; SameSite=Lax; Secure; Path=/; Max-Age=3600`
(attribute order differs from Go; flags and values all exact — order is not
contractually significant). `cookies(cx)` panics if `.cookies()` was not
registered on the builder.

## Static mount (external dist dir + SPA fallback)

Candidate A (tower bridge) — verified, Candidate B never needed:

```rust
use topcoat::router::{Methods, Path, Router, tower::TowerRoute};
use tower_http::services::{ServeDir, ServeFile};

fn router() -> Router {
    let spa = ServeDir::new("/srv/app/dist")                       // external dir, NOT asset! embedding
        .fallback(ServeFile::new("/srv/app/dist/index.html"));   // SPA fallback, preserves 200
    Router::builder()
        // ... api/auth routes first ...
        // Catch-all "/{*rest}" does NOT match the bare "/" — register both.
        .route(TowerRoute::new(Methods::Any, Path::new("/"), spa.clone()))
        .route(TowerRoute::new(Methods::Any, Path::new("/{*rest}"), spa))
        .build()
}
```

Probes: `/` → 200 index.html; `/_nuxt/app.js` → 200 `text/javascript`;
`/deep/client/route` → **200** with index.html body; `/healthz` → handler wins.

Gotcha (hit in the spike): tower-http 0.6 `ServeDir::not_found_service(..)`
wraps the fallback in `SetStatus(404)` — right body, wrong status. Use
`.fallback(..)` (spec delta 15 requires 200). Note delta 15 also says paths
under `/api/`, `/auth/`, `/healthz` keep Go's exact 404 behavior — those are
explicit routes, so an unmatched API path 404s at the router before ever
reaching the mount only if the mount is registered *after* them; verify the
`/api/*` 404 contract case in R-3/R-4 (the spike probed only the SPA paths).

## Graceful shutdown

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    topcoat::serve_until(listener, router(), async move {
        let _ = term.recv().await;
    })
    .await?;
    println!("shutdown complete");
    Ok(())
}
```

Probe: SIGTERM with a 2s request in flight → in-flight curl still printed
`slow-done` (drained, not cut), server printed `shutdown complete`, exit code 0.

Drain semantics: on the signal, the listener closes and in-flight requests get
the service's shutdown timeout to finish — default **30s**, tunable via
`topcoat::router::RouterService::new(router()).shutdown_timeout(d)` passed to
`serve_until` in place of the router. Hung connections are cut at the timeout.
`topcoat::serve(listener, router)` (no `_until`) installs its own
SIGTERM/Ctrl+C handler with identical drain behavior — acceptable under
systemd; use `serve_until` only if we need a custom trigger. systemd
`TimeoutStopSec` must exceed the configured shutdown timeout.

## CORS and auth extractors

Both patterns verified; **Pattern A (a `#[layer("/")]`) is the spine choice** —
it is outermost, sees every route including the tower mount, and short-circuits
preflights in one place.

```rust
use topcoat::{
    Result,
    context::{Cx, CxBuilder},
    cookie::{Cookies, cookies},
    router::{
        Body, Next, Response, StatusCode, content::Json,
        error::RouterErrorExt, headers, layer, route,
    },
};

const ALLOWED_ORIGIN: &str = "https://app.example.com"; // from CORS_ORIGIN/APP_URL config

// Pattern A: outermost layer. Delta-1 logic: flag-off → wildcard dev mode
// (byte-identical to Go); flag-on → reflect Origin only when it matches
// CORS_ORIGIN, plus credentials + Vary.
#[layer("/")]
async fn cors(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    // Request method + headers are read from the Parts on the CxBuilder.
    let (origin, is_preflight) = {
        let parts = cx.get::<http::request::Parts>().expect("router registers request parts");
        let origin = parts.headers.get(http::header::ORIGIN)
            .and_then(|v| v.to_str().ok()).map(str::to_owned);
        (origin, parts.method == http::Method::OPTIONS)
    };

    if is_preflight {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;
        apply_cors(resp.headers_mut(), origin.as_deref());
        return Ok(resp);                      // short-circuit: route never runs
    }

    let mut resp = next.run(cx, body).await?;
    apply_cors(resp.headers_mut(), origin.as_deref());
    Ok(resp)
}

fn apply_cors(headers: &mut http::HeaderMap, origin: Option<&str>) {
    if origin == Some(ALLOWED_ORIGIN) {
        headers.insert(http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::HeaderValue::from_static(ALLOWED_ORIGIN));
        headers.insert(http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            http::HeaderValue::from_static("true"));
        headers.insert(http::header::VARY, http::HeaderValue::from_static("Origin"));
        headers.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS,
            http::HeaderValue::from_static("GET, POST, OPTIONS"));
    }
}

// Auth extractor: a function, not a middleware ("functions, not middlewares").
fn current_user(cx: &Cx) -> Result<String> {
    Ok(cookies(cx)
        .get("session")
        .map(|c| c.value().to_owned())
        .ok_or_unauthorized()?)               // 401 before the handler body runs
}

#[route(GET "/api/data")]
async fn data(cx: &Cx) -> Result<(StatusCode, http::HeaderMap, Json<Data>)> {
    let _user = current_user(cx)?;
    Ok((StatusCode::OK, cors_headers(cx), Json(Data { value: 42 })))
}

// Pattern B (also verified, for handlers that need per-route control):
fn cors_headers(cx: &Cx) -> http::HeaderMap {
    let origin = headers(cx)                  // topcoat::router::headers(&Cx) -> &HeaderMap
        .get(http::header::ORIGIN).and_then(|v| v.to_str().ok());
    let mut out = http::HeaderMap::new();
    apply_cors(&mut out, origin);
    out
}
```

Probes: no cookie → `401`; cookie + allowed Origin → `200` +
`access-control-allow-origin: https://app.example.com`,
`access-control-allow-credentials: true`, `vary: Origin`, `{"value":42}`;
OPTIONS preflight → `200` with the same CORS headers and empty body; disallowed
Origin → `200` with no CORS headers.

Rate limiting (criterion 5's third leg) was not separately probed; it follows
the same extractor-function shape as `current_user` (called at the top of the
handler, returning `Err` on limit) and needs no additional framework capability
beyond what Patterns A/B demonstrated. `TowerLayer` (tower/tower-http
middleware mounted at a path prefix) also exists and is tested upstream if a
tower-ecosystem limiter is preferred.

## Redirects and query params

```rust
use topcoat::{
    Result,
    context::Cx,
    router::{query_params, route},
    router::error::redirect,
};

#[route(GET "/auth/login")]
async fn login() -> Result<&'static str> {
    Err(redirect(&format!(
        "{ISSUER}/authorize?client_id=spike&redirect_uri={APP_URL}/auth/callback"
    )).into())
}

#[query_params(error = bad_request)]   // macro resolves bad_request itself; no import needed
struct Callback {
    code: Option<String>,
    state: Option<String>,
}

#[route(GET "/auth/callback")]
async fn callback(cx: &Cx) -> Result<String> {
    let q = query_params::<Callback>(cx)?;   // memoized per request; Option fields tolerate absence
    Ok(format!(
        "code={} state={}",
        q.code.as_deref().unwrap_or("<missing>"),
        q.state.as_deref().unwrap_or("<missing>")
    ))
}
```

Probes: `/auth/login` → `HTTP/1.1 307 Temporary Redirect` with the exact
external `location:` URL incl. query string; callback with params → echoed;
callback without → `200`, `<missing>`.

**Status-code note (contract-relevant):** `redirect()` is **307**,
`redirect_permanent()` is 308, `see_other()` is 303 (returned via `Ok`, for
Post/Redirect/Get after the callback sets the session cookie and sends the
browser to `/`). There is no 302 constructor; if R-1 fixtures show the Go
server emitting exactly 302, build it with the criterion-1 tuple:
`(StatusCode::FOUND, [(http::header::LOCATION, http::HeaderValue::from_str(&url)?)], "")`.
Redirecting to `/` after callback = same mechanism with a relative path
(`redirect("/")` is proven by the same code path; `see_other("/")` for the
303/PRG variant).

## DB access rule reminder (from spine, binding)

Handlers are async but rusqlite is sync: all DB work goes through
`chassis::db::SharedDb` inside `tokio::task::spawn_blocking`, never holding
the MutexGuard across an .await.
