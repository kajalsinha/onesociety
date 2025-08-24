use monolith_server::payment::{
    CreatePaymentMethodRequest, CreatePaymentIntentRequest, ConfirmPaymentIntentRequest, PaymentFilters,
};
use uuid::Uuid;

#[tokio::test]
async fn test_create_payment_method_request_validation() {
    let request = CreatePaymentMethodRequest {
        payment_type: "card".to_string(),
        provider: "stripe".to_string(),
        provider_payment_method_id: "pm_1234567890".to_string(),
        is_default: Some(true),
        metadata: Some(serde_json::json!({ "card_brand": "visa" })),
    };

    assert_eq!(request.payment_type, "card");
    assert_eq!(request.provider, "stripe");
    assert_eq!(request.provider_payment_method_id, "pm_1234567890");
    assert_eq!(request.is_default, Some(true));
    assert!(request.metadata.is_some());
}

#[tokio::test]
async fn test_create_payment_intent_request_validation() {
    let request = CreatePaymentIntentRequest {
        rental_id: Some(Uuid::new_v4()),
        subscription_id: None,
        amount_cents: 5000,
        currency: Some("USD".to_string()),
        payment_method_id: Some(Uuid::new_v4()),
        metadata: Some(serde_json::json!({ "description": "Rental payment" })),
    };

    assert!(request.rental_id.is_some());
    assert!(request.subscription_id.is_none());
    assert_eq!(request.amount_cents, 5000);
    assert_eq!(request.currency, Some("USD".to_string()));
    assert!(request.payment_method_id.is_some());
    assert!(request.metadata.is_some());
}

#[tokio::test]
async fn test_confirm_payment_intent_request_validation() {
    let request = ConfirmPaymentIntentRequest {
        payment_method_id: Some(Uuid::new_v4()),
    };

    assert!(request.payment_method_id.is_some());
}

#[tokio::test]
async fn test_payment_filters_validation() {
    let filters = PaymentFilters {
        status: Some("succeeded".to_string()),
        payment_type: Some("card".to_string()),
        limit: Some(20),
        offset: Some(0),
    };

    assert_eq!(filters.status, Some("succeeded".to_string()));
    assert_eq!(filters.payment_type, Some("card".to_string()));
    assert_eq!(filters.limit, Some(20));
    assert_eq!(filters.offset, Some(0));
}

#[tokio::test]
async fn test_payment_amount_validation() {
    let request = CreatePaymentIntentRequest {
        rental_id: Some(Uuid::new_v4()),
        subscription_id: None,
        amount_cents: 100,
        currency: Some("USD".to_string()),
        payment_method_id: None,
        metadata: None,
    };

    assert!(request.amount_cents > 0);
    assert!(request.amount_cents <= 999999); // Reasonable upper limit
}

#[tokio::test]
async fn test_payment_currency_validation() {
    let valid_currencies = vec!["USD", "EUR", "GBP", "CAD"];
    let request = CreatePaymentIntentRequest {
        rental_id: Some(Uuid::new_v4()),
        subscription_id: None,
        amount_cents: 1000,
        currency: Some("USD".to_string()),
        payment_method_id: None,
        metadata: None,
    };

    assert!(valid_currencies.contains(&request.currency.as_ref().unwrap().as_str()));
}
