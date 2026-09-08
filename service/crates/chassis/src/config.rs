#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub app_url: String,
    pub api_port: u16,
    pub session_signing_key: String,
    pub cors_origin: String,
    pub rate_limit_auth_per_minute: u32,
    pub rate_limit_write_per_minute: u32,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL"),
            app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            api_port: std::env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            session_signing_key: std::env::var("SESSION_SIGNING_KEY").expect("SESSION_SIGNING_KEY"),
            cors_origin: std::env::var("CORS_ORIGIN").unwrap_or_default(),
            rate_limit_auth_per_minute: env_u32("RATE_LIMIT_AUTH_PER_MINUTE", 10),
            rate_limit_write_per_minute: env_u32("RATE_LIMIT_WRITE_PER_MINUTE", 30),
        }
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
