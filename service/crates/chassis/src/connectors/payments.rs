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

    async fn create_customer(&self, email: &str) -> Result<String, ApiError>;

    async fn create_subscription(
        &self,
        customer_id: &str,
        price_cents: u64,
        interval: &str,
    ) -> Result<String, ApiError>;
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

    async fn create_customer(&self, email: &str) -> Result<String, ApiError> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let customer_id = format!("cust-mock-{}", id);
        self.calls
            .lock()
            .await
            .push(format!("create_customer {}", email));
        Ok(customer_id)
    }

    async fn create_subscription(
        &self,
        customer_id: &str,
        price_cents: u64,
        interval: &str,
    ) -> Result<String, ApiError> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let subscription_id = format!("sub-mock-{}", id);
        self.calls.lock().await.push(format!(
            "create_subscription customer {} price {} interval {}",
            customer_id, price_cents, interval
        ));
        Ok(subscription_id)
    }
}
