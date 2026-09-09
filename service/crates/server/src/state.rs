use chassis::{
    config::Config,
    connectors::{
        email::EmailSender, payments::PaymentProvider, stripe::StripePaymentProvider,
    },
    db::DbPool,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub pool: DbPool,
    pub email: Arc<dyn EmailSender + Send + Sync>,
    pub payments: Arc<dyn PaymentProvider + Send + Sync>,
    /// Concrete Stripe handle when STRIPE_SECRET_KEY is configured; the
    /// webhook endpoint needs it for the signing secret and price mapping.
    /// `None` in mock mode (dev/CI).
    pub stripe: Option<Arc<StripePaymentProvider>>,
}
