use chassis::config::Config;
use std::sync::Mutex;

/// Env-mutating tests must not race each other under cargo's parallel harness.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const KEYS: [&str; 7] = [
    "DATABASE_URL",
    "APP_URL",
    "API_PORT",
    "SESSION_SIGNING_KEY",
    "CORS_ORIGIN",
    "RATE_LIMIT_AUTH_PER_MINUTE",
    "RATE_LIMIT_WRITE_PER_MINUTE",
];

fn clean_env() {
    for k in KEYS {
        std::env::remove_var(k);
    }
}

#[test]
fn from_env_uses_required_and_defaults() {
    let _g = ENV_LOCK.lock().unwrap();
    clean_env();
    std::env::set_var("DATABASE_URL", "postgres://localhost/db");
    std::env::set_var("SESSION_SIGNING_KEY", "secret");
    let cfg = Config::from_env();
    assert_eq!(cfg.database_url, "postgres://localhost/db");
    assert_eq!(cfg.session_signing_key, "secret");
    assert_eq!(cfg.app_url, "http://localhost:3000");
    assert_eq!(cfg.api_port, 8080);
    assert_eq!(cfg.cors_origin, "");
    assert_eq!(cfg.rate_limit_auth_per_minute, 10);
    assert_eq!(cfg.rate_limit_write_per_minute, 30);
    clean_env();
}

#[test]
fn from_env_parses_optional_overrides() {
    let _g = ENV_LOCK.lock().unwrap();
    clean_env();
    std::env::set_var("DATABASE_URL", "postgres://localhost/db");
    std::env::set_var("SESSION_SIGNING_KEY", "secret");
    std::env::set_var("APP_URL", "https://example.com");
    std::env::set_var("API_PORT", "9090");
    std::env::set_var("CORS_ORIGIN", "https://ui.example.com");
    std::env::set_var("RATE_LIMIT_AUTH_PER_MINUTE", "5");
    let cfg = Config::from_env();
    assert_eq!(cfg.app_url, "https://example.com");
    assert_eq!(cfg.api_port, 9090);
    assert_eq!(cfg.cors_origin, "https://ui.example.com");
    assert_eq!(cfg.rate_limit_auth_per_minute, 5);
    clean_env();
}
