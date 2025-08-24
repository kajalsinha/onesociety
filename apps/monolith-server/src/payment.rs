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
pub struct CreatePaymentMethodRequest {
    pub payment_type: String,
    pub provider: String,
    pub provider_payment_method_id: String,
    pub is_default: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub rental_id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub amount_cents: i32,
    pub currency: Option<String>,
    pub payment_method_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentIntentRequest {
    pub payment_method_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentFilters {
    pub status: Option<String>,
    pub payment_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaymentMethodResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub payment_type: String,
    pub provider: String,
    pub provider_payment_method_id: String,
    pub is_default: bool,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub rental_id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub payment_method_id: Option<Uuid>,
    pub provider_payment_intent_id: Option<String>,
    pub provider_charge_id: Option<String>,
    pub failure_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaymentTransactionResponse {
    pub id: Uuid,
    pub payment_intent_id: Uuid,
    pub transaction_type: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub provider_transaction_id: Option<String>,
    pub failure_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaymentMethodListResponse {
    pub payment_methods: Vec<PaymentMethodResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentListResponse {
    pub payment_intents: Vec<PaymentIntentResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
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

// Mock payment provider functions (in real implementation, these would call Stripe/PayPal APIs)
async fn create_stripe_payment_method(provider_payment_method_id: &str) -> Result<String, String> {
    // Mock implementation - in real app, this would call Stripe API
    Ok(format!("pi_mock_{}", provider_payment_method_id))
}

async fn create_stripe_payment_intent(amount_cents: i32, currency: &str) -> Result<String, String> {
    // Mock implementation - in real app, this would call Stripe API
    Ok(format!("pi_mock_{}_{}", amount_cents, currency))
}

async fn confirm_stripe_payment_intent(payment_intent_id: &str, payment_method_id: &str) -> Result<String, String> {
    // Mock implementation - in real app, this would call Stripe API
    Ok(format!("ch_mock_{}_{}", payment_intent_id, payment_method_id))
}

// Handlers
pub async fn create_payment_method(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreatePaymentMethodRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Validate payment type
    let valid_payment_types = vec!["card", "bank_account", "digital_wallet"];
    if !valid_payment_types.contains(&request.payment_type.as_str()) {
        return err(axum::http::StatusCode::BAD_REQUEST, "Invalid payment type");
    }

    // Validate provider
    let valid_providers = vec!["stripe", "paypal", "square"];
    if !valid_providers.contains(&request.provider.as_str()) {
        return err(axum::http::StatusCode::BAD_REQUEST, "Invalid payment provider");
    }

    // If this is set as default, unset other default payment methods
    if request.is_default.unwrap_or(false) {
        sqlx::query!(
            "UPDATE payment_schema.payment_methods SET is_default = false WHERE user_id = $1",
            user_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    // Create payment method
    let payment_method = sqlx::query!(
        r#"
        INSERT INTO payment_schema.payment_methods 
        (user_id, payment_type, provider, provider_payment_method_id, is_default, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, created_at, updated_at
        "#,
        user_id,
        request.payment_type,
        request.provider,
        request.provider_payment_method_id,
        request.is_default.unwrap_or(false),
        request.metadata
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaymentMethodResponse {
        id: payment_method.id,
        user_id,
        payment_type: request.payment_type,
        provider: request.provider,
        provider_payment_method_id: request.provider_payment_method_id,
        is_default: request.is_default.unwrap_or(false),
        is_active: true,
        metadata: request.metadata,
        created_at: payment_method.created_at,
        updated_at: payment_method.updated_at,
    };

    ok(response)
}

pub async fn list_payment_methods(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<PaymentFilters>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let limit = filters.limit.unwrap_or(20).min(100);
    let offset = filters.offset.unwrap_or(0);

    let payment_methods = sqlx::query!(
        r#"
        SELECT id, payment_type, provider, provider_payment_method_id, is_default, is_active, metadata, created_at, updated_at
        FROM payment_schema.payment_methods
        WHERE user_id = $1 AND is_active = true
        ORDER BY is_default DESC, created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM payment_schema.payment_methods WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let payment_method_responses: Vec<PaymentMethodResponse> = payment_methods
        .into_iter()
        .map(|row| PaymentMethodResponse {
            id: row.id,
            user_id,
            payment_type: row.payment_type,
            provider: row.provider,
            provider_payment_method_id: row.provider_payment_method_id,
            is_default: row.is_default,
            is_active: row.is_active,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    let response = PaymentMethodListResponse {
        payment_methods: payment_method_responses,
        total,
        limit,
        offset,
    };

    ok(response)
}

pub async fn create_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreatePaymentIntentRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Validate amount
    if request.amount_cents <= 0 {
        return err(axum::http::StatusCode::BAD_REQUEST, "Amount must be greater than 0");
    }

    // Validate that either rental_id or subscription_id is provided, but not both
    if request.rental_id.is_some() && request.subscription_id.is_some() {
        return err(axum::http::StatusCode::BAD_REQUEST, "Cannot specify both rental_id and subscription_id");
    }

    if request.rental_id.is_none() && request.subscription_id.is_none() {
        return err(axum::http::StatusCode::BAD_REQUEST, "Must specify either rental_id or subscription_id");
    }

    // Validate rental exists and user has access (if rental_id provided)
    if let Some(rental_id) = request.rental_id {
        let rental = sqlx::query!(
            "SELECT renter_id FROM rental_schema.rentals WHERE id = $1",
            rental_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

        if rental.is_none() {
            return err(axum::http::StatusCode::NOT_FOUND, "Rental not found");
        }

        if rental.unwrap().renter_id != user_id {
            return err(axum::http::StatusCode::FORBIDDEN, "You can only create payment intents for your own rentals");
        }
    }

    // Validate subscription exists and user has access (if subscription_id provided)
    if let Some(subscription_id) = request.subscription_id {
        let subscription = sqlx::query!(
            "SELECT user_id FROM subscription_schema.subscriptions WHERE id = $1",
            subscription_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

        if subscription.is_none() {
            return err(axum::http::StatusCode::NOT_FOUND, "Subscription not found");
        }

        if subscription.unwrap().user_id != user_id {
            return err(axum::http::StatusCode::FORBIDDEN, "You can only create payment intents for your own subscriptions");
        }
    }

    let currency = request.currency.unwrap_or_else(|| "USD".to_string());

    // Create payment intent with provider
    let provider_payment_intent_id = create_stripe_payment_intent(request.amount_cents, &currency)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create payment intent in database
    let payment_intent = sqlx::query!(
        r#"
        INSERT INTO payment_schema.payment_intents 
        (user_id, rental_id, subscription_id, amount_cents, currency, payment_method_id, provider_payment_intent_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, created_at, updated_at
        "#,
        user_id,
        request.rental_id,
        request.subscription_id,
        request.amount_cents,
        currency,
        request.payment_method_id,
        provider_payment_intent_id,
        request.metadata
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaymentIntentResponse {
        id: payment_intent.id,
        user_id,
        rental_id: request.rental_id,
        subscription_id: request.subscription_id,
        amount_cents: request.amount_cents,
        currency,
        status: "pending".to_string(),
        payment_method_id: request.payment_method_id,
        provider_payment_intent_id: Some(provider_payment_intent_id),
        provider_charge_id: None,
        failure_reason: None,
        metadata: request.metadata,
        created_at: payment_intent.created_at,
        updated_at: payment_intent.updated_at,
    };

    ok(response)
}

pub async fn confirm_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(payment_intent_id): Path<Uuid>,
    Json(request): Json<ConfirmPaymentIntentRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Get payment intent
    let payment_intent = sqlx::query!(
        r#"
        SELECT id, user_id, status, provider_payment_intent_id, payment_method_id
        FROM payment_schema.payment_intents
        WHERE id = $1 AND user_id = $2
        "#,
        payment_intent_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let payment_intent = payment_intent.ok_or((axum::http::StatusCode::NOT_FOUND, "Payment intent not found"))?;

    if payment_intent.status != "pending" {
        return err(axum::http::StatusCode::BAD_REQUEST, "Payment intent is not in pending status");
    }

    // Get payment method
    let payment_method_id = request.payment_method_id.or(payment_intent.payment_method_id);
    if payment_method_id.is_none() {
        return err(axum::http::StatusCode::BAD_REQUEST, "Payment method is required");
    }

    let payment_method = sqlx::query!(
        "SELECT provider_payment_method_id FROM payment_schema.payment_methods WHERE id = $1 AND user_id = $2 AND is_active = true",
        payment_method_id.unwrap(),
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if payment_method.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Payment method not found");
    }

    // Confirm payment with provider
    let provider_charge_id = confirm_stripe_payment_intent(
        &payment_intent.provider_payment_intent_id.unwrap(),
        &payment_method.unwrap().provider_payment_method_id,
    )
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Update payment intent status
    let result = sqlx::query!(
        r#"
        UPDATE payment_schema.payment_intents 
        SET status = 'succeeded', provider_charge_id = $1, payment_method_id = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, user_id, rental_id, subscription_id, amount_cents, currency, status, metadata, created_at, updated_at
        "#,
        provider_charge_id,
        payment_method_id.unwrap(),
        payment_intent_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create payment transaction record
    sqlx::query!(
        r#"
        INSERT INTO payment_schema.payment_transactions 
        (payment_intent_id, transaction_type, amount_cents, currency, status, provider_transaction_id)
        VALUES ($1, 'charge', $2, $3, 'succeeded', $4)
        "#,
        payment_intent_id,
        result.amount_cents,
        result.currency,
        provider_charge_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaymentIntentResponse {
        id: result.id,
        user_id: result.user_id,
        rental_id: result.rental_id,
        subscription_id: result.subscription_id,
        amount_cents: result.amount_cents,
        currency: result.currency,
        status: result.status,
        payment_method_id,
        provider_payment_intent_id: payment_intent.provider_payment_intent_id,
        provider_charge_id: Some(provider_charge_id),
        failure_reason: None,
        metadata: result.metadata,
        created_at: result.created_at,
        updated_at: result.updated_at,
    };

    ok(response)
}

pub async fn get_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(payment_intent_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let payment_intent = sqlx::query!(
        r#"
        SELECT id, user_id, rental_id, subscription_id, amount_cents, currency, status, 
               payment_method_id, provider_payment_intent_id, provider_charge_id, failure_reason, 
               metadata, created_at, updated_at
        FROM payment_schema.payment_intents
        WHERE id = $1 AND user_id = $2
        "#,
        payment_intent_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let payment_intent = payment_intent.ok_or((axum::http::StatusCode::NOT_FOUND, "Payment intent not found"))?;

    let response = PaymentIntentResponse {
        id: payment_intent.id,
        user_id: payment_intent.user_id,
        rental_id: payment_intent.rental_id,
        subscription_id: payment_intent.subscription_id,
        amount_cents: payment_intent.amount_cents,
        currency: payment_intent.currency,
        status: payment_intent.status,
        payment_method_id: payment_intent.payment_method_id,
        provider_payment_intent_id: payment_intent.provider_payment_intent_id,
        provider_charge_id: payment_intent.provider_charge_id,
        failure_reason: payment_intent.failure_reason,
        metadata: payment_intent.metadata,
        created_at: payment_intent.created_at,
        updated_at: payment_intent.updated_at,
    };

    ok(response)
}

pub async fn list_payment_intents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<PaymentFilters>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let limit = filters.limit.unwrap_or(20).min(100);
    let offset = filters.offset.unwrap_or(0);

    let payment_intents = sqlx::query!(
        r#"
        SELECT id, rental_id, subscription_id, amount_cents, currency, status, 
               payment_method_id, provider_payment_intent_id, provider_charge_id, failure_reason, 
               metadata, created_at, updated_at
        FROM payment_schema.payment_intents
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM payment_schema.payment_intents WHERE user_id = $1",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let payment_intent_responses: Vec<PaymentIntentResponse> = payment_intents
        .into_iter()
        .map(|row| PaymentIntentResponse {
            id: row.id,
            user_id,
            rental_id: row.rental_id,
            subscription_id: row.subscription_id,
            amount_cents: row.amount_cents,
            currency: row.currency,
            status: row.status,
            payment_method_id: row.payment_method_id,
            provider_payment_intent_id: row.provider_payment_intent_id,
            provider_charge_id: row.provider_charge_id,
            failure_reason: row.failure_reason,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    let response = PaymentIntentListResponse {
        payment_intents: payment_intent_responses,
        total,
        limit,
        offset,
    };

    ok(response)
}
