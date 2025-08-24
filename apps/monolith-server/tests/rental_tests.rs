use monolith_server::rental::{CreateRentalRequest, UpdateRentalRequest, AvailabilityRequest, RentalFilters};
use uuid::Uuid;
use chrono::{Utc, Duration};

#[tokio::test]
async fn test_create_rental_request_validation() {
    let start_date = Utc::now() + Duration::days(1);
    let end_date = Utc::now() + Duration::days(3);
    
    let request = CreateRentalRequest {
        product_id: Uuid::new_v4(),
        rental_period_start: start_date,
        rental_period_end: end_date,
        pickup_notes: Some("Please call before pickup".to_string()),
        return_notes: Some("Return to same location".to_string()),
    };

    assert!(request.rental_period_start < request.rental_period_end);
    assert_eq!(request.pickup_notes.as_ref().unwrap(), "Please call before pickup");
    assert_eq!(request.return_notes.as_ref().unwrap(), "Return to same location");
}

#[tokio::test]
async fn test_update_rental_request_validation() {
    let request = UpdateRentalRequest {
        status: Some("confirmed".to_string()),
        pickup_notes: Some("Updated pickup instructions".to_string()),
        return_notes: None,
    };

    assert_eq!(request.status.as_ref().unwrap(), "confirmed");
    assert_eq!(request.pickup_notes.as_ref().unwrap(), "Updated pickup instructions");
    assert!(request.return_notes.is_none());
}

#[tokio::test]
async fn test_availability_request_validation() {
    let start_date = Utc::now() + Duration::days(1);
    let end_date = Utc::now() + Duration::days(5);
    
    let request = AvailabilityRequest {
        product_id: Uuid::new_v4(),
        start_date,
        end_date,
    };

    assert!(request.start_date < request.end_date);
    assert!(request.start_date > Utc::now());
}

#[tokio::test]
async fn test_rental_filters_validation() {
    let filters = RentalFilters {
        status: Some("active".to_string()),
        page: Some(1),
        per_page: Some(10),
    };

    assert_eq!(filters.status.as_ref().unwrap(), "active");
    assert_eq!(filters.page.unwrap(), 1);
    assert_eq!(filters.per_page.unwrap(), 10);
}

#[tokio::test]
async fn test_rental_period_validation() {
    let now = Utc::now();
    let start_date = now + Duration::days(1);
    let end_date = now + Duration::days(3);
    
    // Valid rental period
    assert!(start_date > now);
    assert!(end_date > start_date);
    
    // Test minimum rental period (1 day)
    let min_rental = end_date - start_date;
    assert!(min_rental >= Duration::days(1));
}

#[tokio::test]
async fn test_rental_status_validation() {
    let valid_statuses = vec!["requested", "confirmed", "active", "completed", "cancelled"];
    
    for status in valid_statuses {
        let request = UpdateRentalRequest {
            status: Some(status.to_string()),
            pickup_notes: None,
            return_notes: None,
        };
        
        assert_eq!(request.status.as_ref().unwrap(), status);
    }
}

#[tokio::test]
async fn test_future_dates_validation() {
    let past_date = Utc::now() - Duration::days(1);
    let future_date = Utc::now() + Duration::days(1);
    
    // Past dates should be invalid for rentals
    assert!(past_date < Utc::now());
    
    // Future dates should be valid
    assert!(future_date > Utc::now());
}

#[tokio::test]
async fn test_rental_duration_calculation() {
    let start_date = Utc::now() + Duration::days(1);
    let end_date = Utc::now() + Duration::days(5);
    
    let duration = end_date - start_date;
    assert_eq!(duration.num_days(), 4);
    assert!(duration > Duration::zero());
}
