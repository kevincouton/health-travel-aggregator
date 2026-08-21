use crate::error::ApiError;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn index_document(
        &self,
        index: &str,
        doc_id: &str,
        document: Value,
    ) -> Result<(), ApiError>;

    async fn search(&self, index: &str, query: &str) -> Result<Vec<Value>, ApiError>;
}

pub struct MockSearchEngine {
    pub calls: tokio::sync::Mutex<Vec<String>>,
}

impl MockSearchEngine {
    pub fn new() -> Self {
        Self {
            calls: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MockSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchEngine for MockSearchEngine {
    async fn index_document(
        &self,
        index: &str,
        doc_id: &str,
        document: Value,
    ) -> Result<(), ApiError> {
        self.calls
            .lock()
            .await
            .push(format!("index {}/{}: {}", index, doc_id, document));
        Ok(())
    }

    async fn search(&self, index: &str, query: &str) -> Result<Vec<Value>, ApiError> {
        self.calls
            .lock()
            .await
            .push(format!("search {} for {}", index, query));
        Ok(Vec::new())
    }
}
