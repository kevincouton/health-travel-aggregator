use chassis::{config::Config, db::DbPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub pool: DbPool,
    pub email: Arc<dyn chassis::connectors::email::EmailSender + Send + Sync>,
}
