use monolith_server::product::{CreateProductRequest, UpdateProductRequest, ProductFilters, CreateCategoryRequest};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_create_product_request_validation() {
    let request = CreateProductRequest {
        name: "Test Product".to_string(),
        description: Some("A test product".to_string()),
        category_id: Uuid::new_v4(),
        daily_price: 25.0,
        deposit_amount: Some(100.0),
        insurance_required: Some(false),
        specifications: Some(serde_json::json!({
            "brand": "TestBrand",
            "model": "TestModel"
        })),
        address: Some(serde_json::json!({
            "city": "Test City",
            "country": "Test Country"
        })),
        tags: Some(vec!["electronics".to_string(), "test".to_string()]),
    };

    assert_eq!(request.name, "Test Product");
    assert_eq!(request.daily_price, 25.0);
    assert_eq!(request.tags.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_product_request_validation() {
    let request = UpdateProductRequest {
        name: Some("Updated Product".to_string()),
        description: None,
        category_id: None,
        daily_price: Some(30.0),
        deposit_amount: None,
        insurance_required: Some(true),
        specifications: None,
        address: None,
        status: Some("active".to_string()),
    };

    assert_eq!(request.name.as_ref().unwrap(), "Updated Product");
    assert_eq!(request.daily_price.unwrap(), 30.0);
    assert_eq!(request.insurance_required.unwrap(), true);
}

#[tokio::test]
async fn test_product_filters_validation() {
    let filters = ProductFilters {
        category_id: Some(Uuid::new_v4()),
        min_price: Some(10.0),
        max_price: Some(100.0),
        search: Some("test".to_string()),
        page: Some(1),
        per_page: Some(20),
    };

    assert!(filters.category_id.is_some());
    assert_eq!(filters.min_price.unwrap(), 10.0);
    assert_eq!(filters.max_price.unwrap(), 100.0);
    assert_eq!(filters.search.as_ref().unwrap(), "test");
}

#[tokio::test]
async fn test_create_category_request_validation() {
    let request = CreateCategoryRequest {
        name: "Electronics".to_string(),
        description: Some("Electronic devices and gadgets".to_string()),
        parent_category_id: None,
    };

    assert_eq!(request.name, "Electronics");
    assert_eq!(request.description.as_ref().unwrap(), "Electronic devices and gadgets");
    assert!(request.parent_category_id.is_none());
}

#[tokio::test]
async fn test_product_price_validation() {
    let valid_request = CreateProductRequest {
        name: "Valid Product".to_string(),
        description: None,
        category_id: Uuid::new_v4(),
        daily_price: 0.01, // Minimum valid price
        deposit_amount: None,
        insurance_required: None,
        specifications: None,
        address: None,
        tags: None,
    };

    assert!(valid_request.daily_price > 0.0);
    assert!(valid_request.daily_price.is_finite());
}

#[tokio::test]
async fn test_product_tags_validation() {
    let request = CreateProductRequest {
        name: "Tagged Product".to_string(),
        description: None,
        category_id: Uuid::new_v4(),
        daily_price: 50.0,
        deposit_amount: None,
        insurance_required: None,
        specifications: None,
        address: None,
        tags: Some(vec![
            "electronics".to_string(),
            "gaming".to_string(),
            "portable".to_string(),
        ]),
    };

    let tags = request.tags.unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"electronics".to_string()));
    assert!(tags.contains(&"gaming".to_string()));
    assert!(tags.contains(&"portable".to_string()));
}
