use crate::error::ApiError;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_charge(
        &self,
        amount_cents: u64,
        currency: &str,
        source_token: &str,
    ) -> Result<String, ApiError>;

    async fn refund_charge(&self, transaction_id: &str) -> Result<(), ApiError>;
}

pub struct MockPaymentProvider {
    pub calls: tokio::sync::Mutex<Vec<String>>,
    pub counter: AtomicU64,
}

impl MockPaymentProvider {
    pub fn new() -> Self {
        Self {
            calls: tokio::sync::Mutex::new(Vec::new()),
            counter: AtomicU64::new(1),
        }
    }
}

impl Default for MockPaymentProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    async fn create_charge(
        &self,
        amount_cents: u64,
        currency: &str,
        source_token: &str,
    ) -> Result<String, ApiError> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let tx_id = format!("tx-mock-{}", id);
        self.calls.lock().await.push(format!(
            "charge {} {} from {}",
            amount_cents, currency, source_token
        ));
        Ok(tx_id)
    }

    async fn refund_charge(&self, transaction_id: &str) -> Result<(), ApiError> {
        self.calls
            .lock()
            .await
            .push(format!("refund {}", transaction_id));
        Ok(())
    }
}
