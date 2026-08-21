use chassis::connectors::{
    email::{EmailSender, MockEmailSender},
    maps::{MapsProvider, MockMapsProvider},
    payments::{MockPaymentProvider, PaymentProvider},
    search::{MockSearchEngine, SearchEngine},
    storage::{FileStorage, MockFileStorage},
};
use serde_json::json;

#[tokio::test]
async fn mock_email_sender_records_sent_messages() {
    let sender = MockEmailSender::new();

    sender
        .send_magic_link(
            "user@example.com",
            "https://app.example.com/magic?token=abc",
        )
        .await
        .unwrap();
    sender
        .send_inquiry_notification("user@example.com", "Acme Clinic")
        .await
        .unwrap();

    let sent = sender.sent.lock().await;
    assert_eq!(sent.len(), 2);
    assert!(sent[0].contains("magic-link to user@example.com"));
    assert!(sent[0].contains("https://app.example.com/magic?token=abc"));
    assert_eq!(sent[1], "inquiry to user@example.com for Acme Clinic");
}

#[tokio::test]
async fn mock_file_storage_records_upload_and_delete() {
    let storage = MockFileStorage::new();

    let url = storage
        .upload_file("assets", "report.pdf", b"hello", "application/pdf")
        .await
        .unwrap();
    assert_eq!(url, "https://mock.example/assets/report.pdf");

    storage.delete_file("assets", "report.pdf").await.unwrap();

    let calls = storage.calls.lock().await;
    assert_eq!(calls.len(), 2);
    assert!(calls[0].contains("upload assets/report.pdf"));
    assert!(calls[0].contains("5 bytes"));
    assert_eq!(calls[1], "delete assets/report.pdf");
}

#[tokio::test]
async fn mock_payment_provider_records_charge_and_refund() {
    let provider = MockPaymentProvider::new();

    let tx_id = provider
        .create_charge(10000, "usd", "tok_visa")
        .await
        .unwrap();
    assert!(tx_id.starts_with("tx-mock-"));

    provider.refund_charge(&tx_id).await.unwrap();

    let calls = provider.calls.lock().await;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], "charge 10000 usd from tok_visa");
    assert_eq!(calls[1], format!("refund {}", tx_id));
}

#[tokio::test]
async fn mock_maps_provider_records_geocode_and_distance() {
    let maps = MockMapsProvider::new();

    let coords = maps.geocode("1 Main St").await.unwrap();
    assert_eq!(coords, (0.0, 0.0));

    let distance = maps.distance_km((0.0, 0.0), (1.0, 1.0)).await.unwrap();
    assert_eq!(distance, 0.0);

    let calls = maps.calls.lock().await;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], "geocode 1 Main St");
    assert!(calls[1].contains("distance from (0.0, 0.0) to (1.0, 1.0)"));
}

#[tokio::test]
async fn mock_search_engine_records_index_and_search() {
    let engine = MockSearchEngine::new();

    engine
        .index_document("clinics", "clinic-1", json!({"name": "Acme Clinic"}))
        .await
        .unwrap();

    let results = engine.search("clinics", "acme").await.unwrap();
    assert!(results.is_empty());

    let calls = engine.calls.lock().await;
    assert_eq!(calls.len(), 2);
    assert!(calls[0].contains("index clinics/clinic-1"));
    assert_eq!(calls[1], "search clinics for acme");
}
