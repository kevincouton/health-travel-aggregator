//! Stripe connector tests against a local stub HTTP server (httptest).
//! No real Stripe calls: every request/response is mocked.

use chassis::connectors::payments::PaymentProvider;
use chassis::connectors::stripe::{StripeConfig, StripePaymentProvider};
use httptest::{all_of, matchers::*, responders::*, Expectation, Server};

fn test_provider(server: &Server) -> StripePaymentProvider {
    let cfg = StripeConfig {
        secret_key: "sk_test_123".into(),
        webhook_secret: Some("whsec_123".into()),
        price_pro: Some("price_pro_123".into()),
        price_enterprise: Some("price_ent_123".into()),
    };
    StripePaymentProvider::with_base_url(cfg, &format!("http://{}", server.addr()))
}

#[tokio::test]
async fn create_customer_posts_email_with_bearer_auth() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/v1/customers"),
            request::headers(contains(("authorization", "Bearer sk_test_123"))),
            request::body(url_decoded(contains(("email", "billing@example.com")))),
        ])
        .respond_with(json_encoded(serde_json::json!({"id": "cus_123"}))),
    );
    let provider = test_provider(&server);

    let id = provider.create_customer("billing@example.com").await.unwrap();
    assert_eq!(id, "cus_123");
}

#[tokio::test]
async fn create_charge_confirms_a_payment_intent() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/v1/payment_intents"),
            request::body(url_decoded(all_of![
                contains(("amount", "9900")),
                contains(("currency", "usd")),
                contains(("payment_method", "pm_card_visa")),
                contains(("confirm", "true")),
            ])),
        ])
        .respond_with(json_encoded(serde_json::json!({"id": "pi_123"}))),
    );
    let provider = test_provider(&server);

    let id = provider
        .create_charge(9900, "usd", "pm_card_visa")
        .await
        .unwrap();
    assert_eq!(id, "pi_123");
}

#[tokio::test]
async fn refund_charge_posts_a_refund_for_the_payment_intent() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/v1/refunds"),
            request::body(url_decoded(contains(("payment_intent", "pi_123")))),
        ])
        .respond_with(json_encoded(serde_json::json!({"id": "re_123"}))),
    );
    let provider = test_provider(&server);

    provider.refund_charge("pi_123").await.unwrap();
}

#[tokio::test]
async fn create_subscription_maps_amount_to_configured_price() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/v1/subscriptions"),
            request::body(url_decoded(all_of![
                contains(("customer", "cus_123")),
                contains(("items[0][price]", "price_ent_123")),
            ])),
        ])
        .respond_with(json_encoded(serde_json::json!({"id": "sub_123"}))),
    );
    let provider = test_provider(&server);

    let id = provider
        .create_subscription("cus_123", 29900, "month")
        .await
        .unwrap();
    assert_eq!(id, "sub_123");
}

#[tokio::test]
async fn create_subscription_without_configured_price_fails_before_http() {
    let server = Server::run();
    let cfg = StripeConfig {
        secret_key: "sk_test_123".into(),
        webhook_secret: None,
        price_pro: None,
        price_enterprise: None,
    };
    let provider =
        StripePaymentProvider::with_base_url(cfg, &format!("http://{}", server.addr()));

    let err = provider
        .create_subscription("cus_123", 9900, "month")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        chassis::error::ApiError::Validation(_)
    ));
}

#[tokio::test]
async fn create_checkout_session_posts_subscription_mode_and_metadata() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/v1/checkout/sessions"),
            request::headers(contains(("authorization", "Bearer sk_test_123"))),
            request::body(url_decoded(all_of![
                contains(("mode", "subscription")),
                contains(("customer", "cus_123")),
                contains(("line_items[0][price]", "price_pro_123")),
                contains(("line_items[0][quantity]", "1")),
                contains((
                    "success_url",
                    "https://app.example.com/dashboard/billing?status=success"
                )),
                contains((
                    "cancel_url",
                    "https://app.example.com/dashboard/billing?status=cancel"
                )),
                contains(("metadata[plan_slug]", "pro")),
                contains((
                    "metadata[user_id]",
                    "018e2f2a-0000-7000-8000-000000000001"
                )),
                contains((
                    "client_reference_id",
                    "018e2f2a-0000-7000-8000-000000000001"
                )),
            ])),
        ])
        .respond_with(json_encoded(
            serde_json::json!({"id": "cs_123", "url": "https://checkout.stripe.com/c/pay/cs_123"}),
        )),
    );
    let provider = test_provider(&server);

    let url = provider
        .create_checkout_session(
            "cus_123",
            "price_pro_123",
            "https://app.example.com/dashboard/billing?status=success",
            "https://app.example.com/dashboard/billing?status=cancel",
            &[
                ("user_id".into(), "018e2f2a-0000-7000-8000-000000000001".into()),
                ("plan_slug".into(), "pro".into()),
            ],
        )
        .await
        .unwrap();
    assert_eq!(url, "https://checkout.stripe.com/c/pay/cs_123");
}

#[tokio::test]
async fn stripe_error_status_becomes_an_api_error() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("POST", "/v1/customers"))
            .respond_with(status_code(402).body(r#"{"error":{"message":"card declined"}}"#)),
    );
    let provider = test_provider(&server);

    let err = provider.create_customer("billing@example.com").await.unwrap_err();
    assert!(matches!(err, chassis::error::ApiError::Internal));
}
