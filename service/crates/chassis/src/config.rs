#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub app_url: String,
    pub api_port: u16,
    pub session_signing_key: String,
    pub cors_origin: String,
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
        }
    }
}
