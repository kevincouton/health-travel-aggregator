//! Stripe implementation of [`PaymentProvider`] over Stripe's REST API
//! (reqwest 0.11, rustls), plus webhook signature verification.
//!
//! Env configuration: `STRIPE_SECRET_KEY` enables the real provider (unset =
//! mock mode for dev/CI); `STRIPE_WEBHOOK_SECRET` enables webhook ingestion;
//! `STRIPE_PRICE_PRO` / `STRIPE_PRICE_ENTERPRISE` map plan slugs to Stripe
//! price IDs for checkout.

use crate::connectors::payments::PaymentProvider;
use crate::error::ApiError;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;

const DEFAULT_BASE_URL: &str = "https://api.stripe.com";
/// Stripe's documented tolerance for webhook timestamps is 5 minutes.
pub const WEBHOOK_TOLERANCE_SECONDS: i64 = 300;

#[derive(Debug, Clone)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: Option<String>,
    pub price_pro: Option<String>,
    pub price_enterprise: Option<String>,
}

impl StripeConfig {
    /// `None` when `STRIPE_SECRET_KEY` is unset/empty (mock mode).
    pub fn from_env() -> Option<Self> {
        let secret_key =
            std::env::var("STRIPE_SECRET_KEY").ok().filter(|k| !k.is_empty())?;
        Some(Self {
            secret_key,
            webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
                .ok()
                .filter(|v| !v.is_empty()),
            price_pro: std::env::var("STRIPE_PRICE_PRO").ok().filter(|v| !v.is_empty()),
            price_enterprise: std::env::var("STRIPE_PRICE_ENTERPRISE")
                .ok()
                .filter(|v| !v.is_empty()),
        })
    }
}

/// Real [`PaymentProvider`] calling Stripe's REST API.
pub struct StripePaymentProvider {
    http: reqwest::Client,
    base_url: String,
    cfg: StripeConfig,
}

#[derive(Deserialize)]
struct StripeObject {
    id: String,
    #[serde(default)]
    url: Option<String>,
}

impl StripePaymentProvider {
    pub fn new(cfg: StripeConfig) -> Self {
        Self::with_base_url(cfg, DEFAULT_BASE_URL)
    }

    /// Test hook: point the client at a local stub instead of api.stripe.com.
    pub fn with_base_url(cfg: StripeConfig, base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            cfg,
        }
    }

    pub fn webhook_secret(&self) -> Option<&str> {
        self.cfg.webhook_secret.as_deref()
    }

    /// Stripe price ID for a plan slug, if configured.
    pub fn price_id_for(&self, plan_slug: &str) -> Option<&str> {
        match plan_slug {
            "pro" => self.cfg.price_pro.as_deref(),
            "enterprise" => self.cfg.price_enterprise.as_deref(),
            _ => None,
        }
    }

    /// Price ID for a plan expressed as (price_cents, interval), matching the
    /// seeded catalog: Pro $99/month, Enterprise $299/month.
    fn price_id_for_amount(&self, price_cents: u64, interval: &str) -> Result<&str, ApiError> {
        let slug = match (price_cents, interval) {
            (9900, "month") => "pro",
            (29900, "month") => "enterprise",
            _ => {
                return Err(ApiError::Validation(format!(
                    "no Stripe price configured for {price_cents} cents / {interval}"
                )))
            }
        };
        self.price_id_for(slug).ok_or_else(|| {
            ApiError::Validation(format!(
                "STRIPE_PRICE_{} is not configured",
                slug.to_uppercase()
            ))
        })
    }

    async fn post_form(&self, path: &str, form: &[(&str, String)]) -> Result<StripeObject, ApiError> {
        let resp = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.cfg.secret_key)
            .form(form)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, path, "Stripe request failed");
                ApiError::Internal
            })?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(%status, path, %body, "Stripe API error");
            return Err(ApiError::Internal);
        }
        resp.json::<StripeObject>().await.map_err(|e| {
            tracing::error!(error = %e, path, "Stripe response did not parse");
            ApiError::Internal
        })
    }
}

#[async_trait]
impl PaymentProvider for StripePaymentProvider {
    /// `source_token` is a Stripe PaymentMethod ID (tok_/pm_); the charge is
    /// created as a confirmed PaymentIntent.
    async fn create_charge(
        &self,
        amount_cents: u64,
        currency: &str,
        source_token: &str,
    ) -> Result<String, ApiError> {
        let obj = self
            .post_form(
                "/v1/payment_intents",
                &[
                    ("amount", amount_cents.to_string()),
                    ("currency", currency.to_string()),
                    ("payment_method", source_token.to_string()),
                    ("confirm", "true".to_string()),
                ],
            )
            .await?;
        Ok(obj.id)
    }

    async fn refund_charge(&self, transaction_id: &str) -> Result<(), ApiError> {
        self.post_form(
            "/v1/refunds",
            &[("payment_intent", transaction_id.to_string())],
        )
        .await?;
        Ok(())
    }

    async fn create_customer(&self, email: &str) -> Result<String, ApiError> {
        let obj = self
            .post_form("/v1/customers", &[("email", email.to_string())])
            .await?;
        Ok(obj.id)
    }

    async fn create_subscription(
        &self,
        customer_id: &str,
        price_cents: u64,
        interval: &str,
    ) -> Result<String, ApiError> {
        let price_id = self.price_id_for_amount(price_cents, interval)?;
        let obj = self
            .post_form(
                "/v1/subscriptions",
                &[
                    ("customer", customer_id.to_string()),
                    ("items[0][price]", price_id.to_string()),
                ],
            )
            .await?;
        Ok(obj.id)
    }

    async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        metadata: &[(String, String)],
    ) -> Result<String, ApiError> {
        let mut form: Vec<(String, String)> = vec![
            ("mode".into(), "subscription".into()),
            ("customer".into(), customer_id.to_string()),
            ("line_items[0][price]".into(), price_id.to_string()),
            ("line_items[0][quantity]".into(), "1".into()),
            ("success_url".into(), success_url.to_string()),
            ("cancel_url".into(), cancel_url.to_string()),
        ];
        for (k, v) in metadata {
            form.push((format!("metadata[{k}]"), v.clone()));
            // client_reference_id is Stripe's canonical field for linking a
            // checkout back to an internal account.
            if k == "user_id" {
                form.push(("client_reference_id".into(), v.clone()));
            }
        }
        let form_refs: Vec<(&str, String)> =
            form.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        let obj = self.post_form("/v1/checkout/sessions", &form_refs).await?;
        obj.url.ok_or(ApiError::Internal)
    }
}

/// Connector selection: real Stripe provider when `STRIPE_SECRET_KEY` is
/// configured, mock otherwise so dev/CI keep working with no external
/// dependency. Returns the trait object plus the concrete handle (needed by
/// the webhook endpoint for its signing secret and price mapping).
pub fn provider_from_env() -> (
    Arc<dyn PaymentProvider + Send + Sync>,
    Option<Arc<StripePaymentProvider>>,
) {
    match StripeConfig::from_env() {
        Some(cfg) => {
            let stripe = Arc::new(StripePaymentProvider::new(cfg));
            (stripe.clone(), Some(stripe))
        }
        None => (
            Arc::new(crate::connectors::payments::MockPaymentProvider::new()),
            None,
        ),
    }
}

/// Verify a `Stripe-Signature` v1 header against the raw payload using the
/// current time; rejects timestamps older than [`WEBHOOK_TOLERANCE_SECONDS`].
pub fn verify_webhook_signature(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), ApiError> {
    verify_webhook_signature_at(
        payload,
        signature_header,
        secret,
        WEBHOOK_TOLERANCE_SECONDS,
        chrono::Utc::now().timestamp(),
    )
}

/// `Stripe-Signature` scheme: `t=<unix_ts>,v1=<hex-hmac-sha256>[,v1=...]`
/// where the MAC input is `"{t}.{payload}"`. Comparison is constant-time and
/// the timestamp must be within `tolerance_seconds` of `now` (replay guard).
pub fn verify_webhook_signature_at(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
    tolerance_seconds: i64,
    now: i64,
) -> Result<(), ApiError> {
    let mut timestamp: Option<i64> = None;
    let mut signatures: Vec<&str> = Vec::new();
    for part in signature_header.split(',') {
        match part.split_once('=') {
            Some(("t", v)) => {
                timestamp = Some(
                    v.parse()
                        .map_err(|_| ApiError::Validation("bad stripe timestamp".into()))?,
                )
            }
            Some(("v1", v)) => signatures.push(v),
            _ => {}
        }
    }
    let t = timestamp.ok_or(ApiError::Unauthorized)?;
    if signatures.is_empty() {
        return Err(ApiError::Unauthorized);
    }
    if (now - t).abs() > tolerance_seconds {
        return Err(ApiError::Unauthorized);
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| ApiError::Internal)?;
    mac.update(t.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    let expected = hex::encode(mac.finalize().into_bytes());

    if signatures
        .iter()
        .any(|sig| constant_time_eq::constant_time_eq(sig.as_bytes(), expected.as_bytes()))
    {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign(payload: &[u8], secret: &str, t: i64) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(t.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        format!("t={t},v1={}", hex::encode(mac.finalize().into_bytes()))
    }

    const SECRET: &str = "whsec_test_secret";
    const PAYLOAD: &[u8] = br#"{"id":"evt_1","type":"checkout.session.completed"}"#;

    #[test]
    fn valid_signature_is_accepted() {
        let now = 1_700_000_000;
        let header = sign(PAYLOAD, SECRET, now);
        assert!(verify_webhook_signature_at(PAYLOAD, &header, SECRET, 300, now).is_ok());
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let now = 1_700_000_000;
        let header = sign(PAYLOAD, SECRET, now);
        let tampered = br#"{"id":"evt_1","type":"checkout.session.completed ","amount":0}"#;
        assert!(verify_webhook_signature_at(tampered, &header, SECRET, 300, now).is_err());
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let now = 1_700_000_000;
        let header = sign(PAYLOAD, "whsec_other", now);
        assert!(verify_webhook_signature_at(PAYLOAD, &header, SECRET, 300, now).is_err());
    }

    #[test]
    fn expired_timestamp_is_rejected() {
        let now = 1_700_000_000;
        let header = sign(PAYLOAD, SECRET, now - 301);
        assert!(verify_webhook_signature_at(PAYLOAD, &header, SECRET, 300, now).is_err());
    }

    #[test]
    fn future_timestamp_beyond_tolerance_is_rejected() {
        let now = 1_700_000_000;
        let header = sign(PAYLOAD, SECRET, now + 301);
        assert!(verify_webhook_signature_at(PAYLOAD, &header, SECRET, 300, now).is_err());
    }

    #[test]
    fn missing_v1_or_t_is_rejected() {
        assert!(verify_webhook_signature_at(PAYLOAD, "t=123", SECRET, 300, 123).is_err());
        assert!(verify_webhook_signature_at(PAYLOAD, "v1=abc", SECRET, 300, 123).is_err());
        assert!(verify_webhook_signature_at(PAYLOAD, "garbage", SECRET, 300, 123).is_err());
    }

    #[test]
    fn multiple_v1_signatures_accept_any_match() {
        let now = 1_700_000_000;
        let good = sign(PAYLOAD, SECRET, now);
        let header = format!("{good},v1=deadbeef");
        assert!(verify_webhook_signature_at(PAYLOAD, &header, SECRET, 300, now).is_ok());
    }

    #[test]
    fn price_id_mapping_matches_seeded_plans() {
        let cfg = StripeConfig {
            secret_key: "sk_test_x".into(),
            webhook_secret: None,
            price_pro: Some("price_pro".into()),
            price_enterprise: Some("price_ent".into()),
        };
        let stripe = StripePaymentProvider::new(cfg);
        assert_eq!(stripe.price_id_for("pro"), Some("price_pro"));
        assert_eq!(stripe.price_id_for("enterprise"), Some("price_ent"));
        assert_eq!(stripe.price_id_for("basic"), None);
        assert_eq!(stripe.price_id_for_amount(9900, "month").unwrap(), "price_pro");
        assert_eq!(stripe.price_id_for_amount(29900, "month").unwrap(), "price_ent");
        assert!(stripe.price_id_for_amount(100, "month").is_err());
    }
}
