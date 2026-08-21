use crate::error::ApiError;
use async_trait::async_trait;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), ApiError>;
    async fn send_inquiry_notification(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<(), ApiError>;
}

pub struct MockEmailSender {
    pub sent: tokio::sync::Mutex<Vec<String>>,
}

impl MockEmailSender {
    pub fn new() -> Self {
        Self {
            sent: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MockEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), ApiError> {
        self.sent
            .lock()
            .await
            .push(format!("magic-link to {}: {}", to, link));
        Ok(())
    }

    async fn send_inquiry_notification(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<(), ApiError> {
        self.sent
            .lock()
            .await
            .push(format!(
                "inquiry to {} for {} from {}",
                to, clinic_name, patient_email
            ));
        Ok(())
    }
}
