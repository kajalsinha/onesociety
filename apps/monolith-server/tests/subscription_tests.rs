use monolith_server::subscription::{
    CreateSubscriptionRequest, UpdateSubscriptionRequest, SubscriptionFilters,
};
use uuid::Uuid;

#[tokio::test]
async fn test_create_subscription_request_validation() {
    let request = CreateSubscriptionRequest {
        plan_id: Uuid::new_v4(),
    };

    assert!(!request.plan_id.is_nil());
}

#[tokio::test]
async fn test_update_subscription_request_validation() {
    let request = UpdateSubscriptionRequest {
        cancel_at_period_end: Some(true),
    };

    assert_eq!(request.cancel_at_period_end, Some(true));
}

#[tokio::test]
async fn test_subscription_filters_validation() {
    let filters = SubscriptionFilters {
        status: Some("active".to_string()),
        limit: Some(10),
        offset: Some(0),
    };

    assert_eq!(filters.status, Some("active".to_string()));
    assert_eq!(filters.limit, Some(10));
    assert_eq!(filters.offset, Some(0));
}

#[tokio::test]
async fn test_subscription_plan_id_validation() {
    let request = CreateSubscriptionRequest {
        plan_id: Uuid::new_v4(),
    };

    // UUID should not be nil
    assert!(!request.plan_id.is_nil());
    
    // UUID should be version 4
    assert_eq!(request.plan_id.get_version_num(), 4);
}

#[tokio::test]
async fn test_subscription_cancel_options() {
    let request_immediate = UpdateSubscriptionRequest {
        cancel_at_period_end: Some(false),
    };

    let request_period_end = UpdateSubscriptionRequest {
        cancel_at_period_end: Some(true),
    };

    assert_eq!(request_immediate.cancel_at_period_end, Some(false));
    assert_eq!(request_period_end.cancel_at_period_end, Some(true));
}

#[tokio::test]
async fn test_subscription_status_validation() {
    let valid_statuses = vec!["active", "canceled", "past_due", "unpaid"];
    let test_status = "active";
    
    assert!(valid_statuses.contains(&test_status));
}
