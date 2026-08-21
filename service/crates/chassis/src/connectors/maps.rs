use async_trait::async_trait;
use crate::error::ApiError;

#[async_trait]
pub trait MapsProvider: Send + Sync {
    async fn geocode(&self, address: &str) -> Result<(f64, f64), ApiError>;
    async fn distance_km(&self, from: (f64, f64), to: (f64, f64)) -> Result<f64, ApiError>;
}

pub struct MockMapsProvider {
    pub calls: tokio::sync::Mutex<Vec<String>>,
}

impl MockMapsProvider {
    pub fn new() -> Self {
        Self {
            calls: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MockMapsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MapsProvider for MockMapsProvider {
    async fn geocode(&self, address: &str) -> Result<(f64, f64), ApiError> {
        self.calls
            .lock()
            .await
            .push(format!("geocode {}", address));
        Ok((0.0, 0.0))
    }

    async fn distance_km(&self, from: (f64, f64), to: (f64, f64)) -> Result<f64, ApiError> {
        self.calls.lock().await.push(format!(
            "distance from {:?} to {:?}",
            from, to
        ));
        Ok(0.0)
    }
}
