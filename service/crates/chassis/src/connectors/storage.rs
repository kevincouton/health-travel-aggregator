use async_trait::async_trait;
use crate::error::ApiError;

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<String, ApiError>;

    async fn delete_file(&self, bucket: &str, key: &str) -> Result<(), ApiError>;
}

pub struct MockFileStorage {
    pub calls: tokio::sync::Mutex<Vec<String>>,
}

impl MockFileStorage {
    pub fn new() -> Self {
        Self {
            calls: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MockFileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileStorage for MockFileStorage {
    async fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<String, ApiError> {
        self.calls
            .lock()
            .await
            .push(format!(
                "upload {}/{} ({} bytes, {})",
                bucket,
                key,
                content.len(),
                content_type
            ));
        Ok(format!("https://mock.example/{}/{}", bucket, key))
    }

    async fn delete_file(&self, bucket: &str, key: &str) -> Result<(), ApiError> {
        self.calls
            .lock()
            .await
            .push(format!("delete {}/{}", bucket, key));
        Ok(())
    }
}
