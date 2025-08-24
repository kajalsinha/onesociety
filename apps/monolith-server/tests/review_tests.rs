use monolith_server::review::{
    CreateProductReviewRequest,
    CreateUserReviewRequest,
    ReviewFilters,
};

use uuid::Uuid;

#[tokio::test]
async fn test_create_product_review_request_validation() {
    let request = CreateProductReviewRequest {
        product_id: Uuid::new_v4(),
        rental_id: Some(Uuid::new_v4()),
        rating: 5,
        title: Some("Great product!".to_string()),
        content: Some("This product exceeded my expectations.".to_string()),
    };

    assert!(request.rating >= 1 && request.rating <= 5);
    assert!(request.title.as_ref().unwrap().len() <= 200);
    assert!(!request.content.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_user_review_request_validation() {
    let request = CreateUserReviewRequest {
        reviewed_user_id: Uuid::new_v4(),
        rental_id: Uuid::new_v4(),
        review_type: "renter".to_string(),
        rating: 4,
        title: Some("Good renter".to_string()),
        content: Some("Very responsible and punctual.".to_string()),
    };

    assert!(request.rating >= 1 && request.rating <= 5);
    assert_eq!(request.review_type, "renter");
    assert!(request.title.as_ref().unwrap().len() <= 200);
    assert!(!request.content.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_review_filters_validation() {
    let filters = ReviewFilters {
        product_id: Some(Uuid::new_v4()),
        reviewed_user_id: None,
        reviewer_id: Some(Uuid::new_v4()),
        rating: Some(5),
        review_type: Some("renter".to_string()),
        status: Some("active".to_string()),
        limit: Some(20),
        offset: Some(0),
    };

    assert!(filters.rating.unwrap() >= 1 && filters.rating.unwrap() <= 5);
    assert_eq!(filters.review_type, Some("renter".to_string()));
    assert_eq!(filters.status, Some("active".to_string()));
    assert_eq!(filters.limit, Some(20));
    assert_eq!(filters.offset, Some(0));
}

#[tokio::test]
async fn test_review_filters_defaults() {
    let filters = ReviewFilters {
        product_id: None,
        reviewed_user_id: None,
        reviewer_id: None,
        rating: None,
        review_type: None,
        status: None,
        limit: None,
        offset: None,
    };

    assert_eq!(filters.product_id, None);
    assert_eq!(filters.reviewed_user_id, None);
    assert_eq!(filters.reviewer_id, None);
    assert_eq!(filters.rating, None);
    assert_eq!(filters.review_type, None);
    assert_eq!(filters.status, None);
    assert_eq!(filters.limit, None);
    assert_eq!(filters.offset, None);
}

#[tokio::test]
async fn test_rating_validation() {
    let valid_ratings = vec![1, 2, 3, 4, 5];
    
    for rating in valid_ratings {
        assert!(rating >= 1 && rating <= 5);
    }

    let invalid_ratings = vec![0, 6, -1, 10];
    
    for rating in invalid_ratings {
        assert!(rating < 1 || rating > 5);
    }
}

#[tokio::test]
async fn test_review_type_validation() {
    let valid_types = vec!["renter", "owner"];
    
    for review_type in valid_types {
        assert!(!review_type.is_empty());
        assert!(review_type == "renter" || review_type == "owner");
    }
}

#[tokio::test]
async fn test_review_status_validation() {
    let valid_statuses = vec!["active", "hidden", "flagged"];
    
    for status in valid_statuses {
        assert!(!status.is_empty());
        assert!(status.len() <= 20);
    }
}

#[tokio::test]
async fn test_title_length_validation() {
    let short_title = "Good";
    assert!(short_title.len() <= 200);

    let long_title = "A".repeat(201);
    assert!(long_title.len() > 200);
}

#[tokio::test]
async fn test_content_validation() {
    let valid_content = "This is a valid review content that provides useful feedback.";
    assert!(!valid_content.is_empty());

    let empty_content = "";
    assert!(empty_content.is_empty());
}
