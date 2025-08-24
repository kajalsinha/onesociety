use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{err, ok, AppState};

// Request/Response Models
#[derive(Debug, Deserialize)]
pub struct CreateProductReviewRequest {
    pub product_id: Uuid,
    pub rental_id: Option<Uuid>,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserReviewRequest {
    pub reviewed_user_id: Uuid,
    pub rental_id: Uuid,
    pub review_type: String, // "renter" or "owner"
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewFilters {
    pub product_id: Option<Uuid>,
    pub reviewed_user_id: Option<Uuid>,
    pub reviewer_id: Option<Uuid>,
    pub rating: Option<i32>,
    pub review_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProductReviewResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub reviewer_id: Uuid,
    pub rental_id: Option<Uuid>,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_verified_rental: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reviewer_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserReviewResponse {
    pub id: Uuid,
    pub reviewed_user_id: Uuid,
    pub reviewer_id: Uuid,
    pub rental_id: Uuid,
    pub review_type: String,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reviewer_name: Option<String>,
    pub reviewed_user_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewListResponse {
    pub reviews: Vec<serde_json::Value>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct ReviewStatsResponse {
    pub average_rating: f64,
    pub total_reviews: i64,
    pub rating_distribution: Vec<RatingCount>,
}

#[derive(Debug, Serialize)]
pub struct RatingCount {
    pub rating: i32,
    pub count: i64,
}

// Helper function to extract user ID from JWT token
async fn extract_user_id_from_token(headers: &HeaderMap) -> Result<Uuid, (axum::http::StatusCode, String)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid authorization header format".to_string()));
    }

    let token = &auth_header[7..];
    let claims = crate::jwt::verify_token(token)
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid user ID in token".to_string()))
}

// Handlers
pub async fn create_product_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProductReviewRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Validate rating
    if request.rating < 1 || request.rating > 5 {
        return err(axum::http::StatusCode::BAD_REQUEST, "Rating must be between 1 and 5");
    }

    // Check if product exists
    let product = sqlx::query!(
        "SELECT id FROM product_schema.products WHERE id = $1 AND status = 'active'",
        request.product_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if product.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Product not found");
    }

    // Check if rental exists and user is the renter (if rental_id provided)
    let is_verified_rental = if let Some(rental_id) = request.rental_id {
        let rental = sqlx::query!(
            "SELECT renter_id FROM rental_schema.rentals WHERE id = $1 AND product_id = $2",
            rental_id,
            request.product_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

        rental.map(|r| r.renter_id == user_id).unwrap_or(false)
    } else {
        false
    };

    // Check if user already reviewed this product
    let existing_review = sqlx::query!(
        "SELECT id FROM review_schema.product_reviews WHERE product_id = $1 AND reviewer_id = $2 AND rental_id IS NOT DISTINCT FROM $3",
        request.product_id,
        user_id,
        request.rental_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if existing_review.is_some() {
        return err(axum::http::StatusCode::CONFLICT, "You have already reviewed this product");
    }

    // Create the review
    let review = sqlx::query!(
        r#"
        INSERT INTO review_schema.product_reviews 
        (product_id, reviewer_id, rental_id, rating, title, content, is_verified_rental)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, created_at, updated_at
        "#,
        request.product_id,
        user_id,
        request.rental_id,
        request.rating,
        request.title,
        request.content,
        is_verified_rental
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Get reviewer name
    let reviewer_name = sqlx::query!(
        "SELECT first_name, last_name FROM user_schema.user_profiles WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .map(|profile| format!("{} {}", profile.first_name.unwrap_or_default(), profile.last_name.unwrap_or_default()));

    let response = ProductReviewResponse {
        id: review.id,
        product_id: request.product_id,
        reviewer_id: user_id,
        rental_id: request.rental_id,
        rating: request.rating,
        title: request.title,
        content: request.content,
        is_verified_rental,
        status: "active".to_string(),
        created_at: review.created_at,
        updated_at: review.updated_at,
        reviewer_name,
    };

    ok(response)
}

pub async fn create_user_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateUserReviewRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Validate rating
    if request.rating < 1 || request.rating > 5 {
        return err(axum::http::StatusCode::BAD_REQUEST, "Rating must be between 1 and 5");
    }

    // Validate review type
    if request.review_type != "renter" && request.review_type != "owner" {
        return err(axum::http::StatusCode::BAD_REQUEST, "Review type must be 'renter' or 'owner'");
    }

    // Check if rental exists and user is involved
    let rental = sqlx::query!(
        "SELECT renter_id, product_id FROM rental_schema.rentals WHERE id = $1",
        request.rental_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let rental = rental.ok_or((axum::http::StatusCode::NOT_FOUND, "Rental not found"))?;

    // Get product owner
    let product = sqlx::query!(
        "SELECT owner_id FROM product_schema.products WHERE id = $1",
        rental.product_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Verify user is authorized to review
    let can_review = match request.review_type.as_str() {
        "renter" => {
            // Owner can review renter
            product.owner_id == user_id && rental.renter_id == request.reviewed_user_id
        }
        "owner" => {
            // Renter can review owner
            rental.renter_id == user_id && product.owner_id == request.reviewed_user_id
        }
        _ => false,
    };

    if !can_review {
        return err(axum::http::StatusCode::FORBIDDEN, "You are not authorized to review this user");
    }

    // Check if user already reviewed this user for this rental
    let existing_review = sqlx::query!(
        "SELECT id FROM review_schema.user_reviews WHERE reviewed_user_id = $1 AND reviewer_id = $2 AND rental_id = $3 AND review_type = $4",
        request.reviewed_user_id,
        user_id,
        request.rental_id,
        request.review_type
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if existing_review.is_some() {
        return err(axum::http::StatusCode::CONFLICT, "You have already reviewed this user for this rental");
    }

    // Create the review
    let review = sqlx::query!(
        r#"
        INSERT INTO review_schema.user_reviews 
        (reviewed_user_id, reviewer_id, rental_id, review_type, rating, title, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, created_at, updated_at
        "#,
        request.reviewed_user_id,
        user_id,
        request.rental_id,
        request.review_type,
        request.rating,
        request.title,
        request.content
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Get reviewer and reviewed user names
    let reviewer_name = sqlx::query!(
        "SELECT first_name, last_name FROM user_schema.user_profiles WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .map(|profile| format!("{} {}", profile.first_name.unwrap_or_default(), profile.last_name.unwrap_or_default()));

    let reviewed_user_name = sqlx::query!(
        "SELECT first_name, last_name FROM user_schema.user_profiles WHERE user_id = $1",
        request.reviewed_user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .map(|profile| format!("{} {}", profile.first_name.unwrap_or_default(), profile.last_name.unwrap_or_default()));

    let response = UserReviewResponse {
        id: review.id,
        reviewed_user_id: request.reviewed_user_id,
        reviewer_id: user_id,
        rental_id: request.rental_id,
        review_type: request.review_type,
        rating: request.rating,
        title: request.title,
        content: request.content,
        status: "active".to_string(),
        created_at: review.created_at,
        updated_at: review.updated_at,
        reviewer_name,
        reviewed_user_name,
    };

    ok(response)
}

pub async fn list_product_reviews(
    State(state): State<AppState>,
    Query(filters): Query<ReviewFilters>,
) -> impl axum::response::IntoResponse {
    let limit = filters.limit.unwrap_or(20).min(100);
    let offset = filters.offset.unwrap_or(0);
    let status = filters.status.unwrap_or_else(|| "active".to_string().to_string());

    let mut query = String::from(
        "SELECT r.id, r.product_id, r.reviewer_id, r.rental_id, r.rating, r.title, r.content, 
                r.is_verified_rental, r.status, r.created_at, r.updated_at,
                p.first_name, p.last_name
         FROM review_schema.product_reviews r
         LEFT JOIN user_schema.user_profiles p ON r.reviewer_id = p.user_id
         WHERE r.status = $1"
    );

    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(status)];
    let mut param_count = 1;

    if let Some(product_id) = filters.product_id {
        param_count += 1;
        query.push_str(&format!(" AND r.product_id = ${}", param_count));
        params.push(Box::new(product_id));
    }

    if let Some(reviewer_id) = filters.reviewer_id {
        param_count += 1;
        query.push_str(&format!(" AND r.reviewer_id = ${}", param_count));
        params.push(Box::new(reviewer_id));
    }

    if let Some(rating) = filters.rating {
        param_count += 1;
        query.push_str(&format!(" AND r.rating = ${}", param_count));
        params.push(Box::new(rating));
    }

    query.push_str(" ORDER BY r.created_at DESC LIMIT $");
    param_count += 1;
    query.push_str(&param_count.to_string());
    params.push(Box::new(limit));

    query.push_str(" OFFSET $");
    param_count += 1;
    query.push_str(&param_count.to_string());
    params.push(Box::new(offset));

    // For now, we'll use a simpler approach with raw SQL
    let reviews = sqlx::query_as!(
        ProductReviewResponse,
        r#"
        SELECT r.id, r.product_id, r.reviewer_id, r.rental_id, r.rating, r.title, r.content, 
               r.is_verified_rental, r.status, r.created_at, r.updated_at,
               CONCAT(p.first_name, ' ', p.last_name) as reviewer_name
        FROM review_schema.product_reviews r
        LEFT JOIN user_schema.user_profiles p ON r.reviewer_id = p.user_id
        WHERE r.status = $1
        ORDER BY r.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        status,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM review_schema.product_reviews WHERE status = $1",
        status
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let response = ReviewListResponse {
        reviews: reviews.into_iter().map(|r| serde_json::to_value(r).unwrap()).collect(),
        total,
        limit,
        offset,
    };

    ok(response)
}

pub async fn get_product_review_stats(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let stats = sqlx::query!(
        r#"
        SELECT 
            AVG(rating) as avg_rating,
            COUNT(*) as total_reviews,
            COUNT(CASE WHEN rating = 1 THEN 1 END) as rating_1,
            COUNT(CASE WHEN rating = 2 THEN 1 END) as rating_2,
            COUNT(CASE WHEN rating = 3 THEN 1 END) as rating_3,
            COUNT(CASE WHEN rating = 4 THEN 1 END) as rating_4,
            COUNT(CASE WHEN rating = 5 THEN 1 END) as rating_5
        FROM review_schema.product_reviews 
        WHERE product_id = $1 AND status = 'active'
        "#,
        product_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let rating_distribution = vec![
        RatingCount { rating: 1, count: stats.rating_1.unwrap_or(0) },
        RatingCount { rating: 2, count: stats.rating_2.unwrap_or(0) },
        RatingCount { rating: 3, count: stats.rating_3.unwrap_or(0) },
        RatingCount { rating: 4, count: stats.rating_4.unwrap_or(0) },
        RatingCount { rating: 5, count: stats.rating_5.unwrap_or(0) },
    ];

    let response = ReviewStatsResponse {
        average_rating: stats.avg_rating.unwrap_or(0.0),
        total_reviews: stats.total_reviews.unwrap_or(0),
        rating_distribution,
    };

    ok(response)
}
