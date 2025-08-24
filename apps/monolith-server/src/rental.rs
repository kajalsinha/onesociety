use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::state::{ok, err, AppState};
use crate::jwt::verify_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRentalRequest {
    pub product_id: Uuid,
    pub rental_period_start: DateTime<Utc>,
    pub rental_period_end: DateTime<Utc>,
    pub pickup_notes: Option<String>,
    pub return_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRentalRequest {
    pub status: Option<String>,
    pub pickup_notes: Option<String>,
    pub return_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RentalResponse {
    pub rental_id: Uuid,
    pub product_id: Uuid,
    pub renter_id: Uuid,
    pub rental_period_start: DateTime<Utc>,
    pub rental_period_end: DateTime<Utc>,
    pub status: String,
    pub pickup_notes: Option<String>,
    pub return_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub product: ProductInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInfo {
    pub product_id: Uuid,
    pub name: String,
    pub daily_price: f64,
    pub owner_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RentalListResponse {
    pub rentals: Vec<RentalResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RentalFilters {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailabilityRequest {
    pub product_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailabilityResponse {
    pub available: bool,
    pub conflicting_rentals: Vec<RentalConflict>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RentalConflict {
    pub rental_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: String,
}

fn extract_user_id_from_token(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    if claims.token_type != "access" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match Uuid::parse_str(&claims.sub) {
        Ok(id) => Ok(id),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn create_rental(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRentalRequest>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    // Verify product exists and is available
    let product = sqlx::query!(
        "SELECT product_id, owner_id, status FROM product_schema.products WHERE product_id = $1",
        req.product_id
    )
    .fetch_one(&state.db)
    .await;

    let product = match product {
        Ok(p) => p,
        Err(_) => return err(StatusCode::NOT_FOUND, "Product not found"),
    };

    if product.status != "active" {
        return err(StatusCode::BAD_REQUEST, "Product is not available for rental");
    }

    if product.owner_id == user_id {
        return err(StatusCode::BAD_REQUEST, "Cannot rent your own product");
    }

    // Check availability
    let conflicts = sqlx::query!(
        r#"
        SELECT rental_id, rental_period_start, rental_period_end, status
        FROM rental_schema.rentals
        WHERE product_id = $1 
        AND status IN ('requested', 'confirmed', 'active')
        AND (
            (rental_period_start <= $2 AND rental_period_end >= $2) OR
            (rental_period_start <= $3 AND rental_period_end >= $3) OR
            (rental_period_start >= $2 AND rental_period_end <= $3)
        )
        "#,
        req.product_id,
        req.rental_period_start,
        req.rental_period_end
    )
    .fetch_all(&state.db)
    .await;

    let conflicts = match conflicts {
        Ok(c) => c,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to check availability"),
    };

    if !conflicts.is_empty() {
        return err(StatusCode::CONFLICT, "Product is not available for the selected dates");
    }

    // Create rental
    let rental_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO rental_schema.rentals 
        (rental_id, product_id, renter_id, rental_period_start, rental_period_end, 
         pickup_notes, return_notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        rental_id,
        req.product_id,
        user_id,
        req.rental_period_start,
        req.rental_period_end,
        req.pickup_notes,
        req.return_notes
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create rental");
    }

    let response = serde_json::json!({
        "rental_id": rental_id,
        "message": "Rental request created successfully"
    });

    ok(response)
}

pub async fn get_rental(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(rental_id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    let rental = sqlx::query!(
        r#"
        SELECT r.*, p.name, p.daily_price, p.owner_id
        FROM rental_schema.rentals r
        JOIN product_schema.products p ON r.product_id = p.product_id
        WHERE r.rental_id = $1 AND (r.renter_id = $2 OR p.owner_id = $2)
        "#,
        rental_id,
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let rental = match rental {
        Ok(r) => r,
        Err(_) => return err(StatusCode::NOT_FOUND, "Rental not found"),
    };

    let response = RentalResponse {
        rental_id: rental.rental_id,
        product_id: rental.product_id,
        renter_id: rental.renter_id,
        rental_period_start: rental.rental_period_start,
        rental_period_end: rental.rental_period_end,
        status: rental.status,
        pickup_notes: rental.pickup_notes,
        return_notes: rental.return_notes,
        created_at: rental.created_at,
        updated_at: rental.updated_at,
        product: ProductInfo {
            product_id: rental.product_id,
            name: rental.name,
            daily_price: rental.daily_price,
            owner_id: rental.owner_id,
        },
    };

    ok(response)
}

pub async fn list_rentals(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<RentalFilters>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let rentals = sqlx::query!(
        r#"
        SELECT r.*, p.name, p.daily_price, p.owner_id
        FROM rental_schema.rentals r
        JOIN product_schema.products p ON r.product_id = p.product_id
        WHERE r.renter_id = $1 OR p.owner_id = $1
        ORDER BY r.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        per_page,
        offset
    )
    .fetch_all(&state.db)
    .await;

    let rentals = match rentals {
        Ok(r) => r,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch rentals"),
    };

    let total = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM rental_schema.rentals r
        JOIN product_schema.products p ON r.product_id = p.product_id
        WHERE r.renter_id = $1 OR p.owner_id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let total = match total {
        Ok(t) => t.count.unwrap_or(0),
        Err(_) => 0,
    };

    let rental_responses: Vec<RentalResponse> = rentals
        .into_iter()
        .map(|r| RentalResponse {
            rental_id: r.rental_id,
            product_id: r.product_id,
            renter_id: r.renter_id,
            rental_period_start: r.rental_period_start,
            rental_period_end: r.rental_period_end,
            status: r.status,
            pickup_notes: r.pickup_notes,
            return_notes: r.return_notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            product: ProductInfo {
                product_id: r.product_id,
                name: r.name,
                daily_price: r.daily_price,
                owner_id: r.owner_id,
            },
        })
        .collect();

    let response = RentalListResponse {
        rentals: rental_responses,
        total,
        page,
        per_page,
    };

    ok(response)
}

pub async fn update_rental(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(rental_id): Path<Uuid>,
    Json(req): Json<UpdateRentalRequest>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_token(&headers) {
        Ok(id) => id,
        Err(status) => return err(status, "Unauthorized"),
    };

    // Verify rental exists and user has permission
    let rental = sqlx::query!(
        r#"
        SELECT r.*, p.owner_id
        FROM rental_schema.rentals r
        JOIN product_schema.products p ON r.product_id = p.product_id
        WHERE r.rental_id = $1 AND (r.renter_id = $2 OR p.owner_id = $2)
        "#,
        rental_id,
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let rental = match rental {
        Ok(r) => r,
        Err(_) => return err(StatusCode::NOT_FOUND, "Rental not found"),
    };

    // Only product owner can update status
    if req.status.is_some() && rental.owner_id != user_id {
        return err(StatusCode::FORBIDDEN, "Only product owner can update rental status");
    }

    // Update rental
    let result = sqlx::query!(
        r#"
        UPDATE rental_schema.rentals 
        SET 
            status = COALESCE($2, status),
            pickup_notes = COALESCE($3, pickup_notes),
            return_notes = COALESCE($4, return_notes),
            updated_at = NOW()
        WHERE rental_id = $1
        "#,
        rental_id,
        req.status,
        req.pickup_notes,
        req.return_notes
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update rental");
    }

    ok(serde_json::json!({ "message": "Rental updated successfully" }))
}

pub async fn check_availability(
    State(state): State<AppState>,
    Json(req): Json<AvailabilityRequest>,
) -> impl IntoResponse {
    // Verify product exists
    let product = sqlx::query!(
        "SELECT product_id, status FROM product_schema.products WHERE product_id = $1",
        req.product_id
    )
    .fetch_one(&state.db)
    .await;

    let product = match product {
        Ok(p) => p,
        Err(_) => return err(StatusCode::NOT_FOUND, "Product not found"),
    };

    if product.status != "active" {
        return ok(AvailabilityResponse {
            available: false,
            conflicting_rentals: vec![],
        });
    }

    // Check for conflicts
    let conflicts = sqlx::query!(
        r#"
        SELECT rental_id, rental_period_start, rental_period_end, status
        FROM rental_schema.rentals
        WHERE product_id = $1 
        AND status IN ('requested', 'confirmed', 'active')
        AND (
            (rental_period_start <= $2 AND rental_period_end >= $2) OR
            (rental_period_start <= $3 AND rental_period_end >= $3) OR
            (rental_period_start >= $2 AND rental_period_end <= $3)
        )
        ORDER BY rental_period_start
        "#,
        req.product_id,
        req.start_date,
        req.end_date
    )
    .fetch_all(&state.db)
    .await;

    let conflicts = match conflicts {
        Ok(c) => c,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to check availability"),
    };

    let conflicting_rentals: Vec<RentalConflict> = conflicts
        .into_iter()
        .map(|c| RentalConflict {
            rental_id: c.rental_id,
            start_date: c.rental_period_start,
            end_date: c.rental_period_end,
            status: c.status,
        })
        .collect();

    let available = conflicting_rentals.is_empty();

    let response = AvailabilityResponse {
        available,
        conflicting_rentals,
    };

    ok(response)
}
