use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{err, ok, AppState};

// Request/Response Models
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub cancel_at_period_end: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionFilters {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionPlanResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub billing_cycle: String,
    pub features: serde_json::Value,
    pub is_active: bool,
    pub max_listings: Option<i32>,
    pub max_rentals_per_month: Option<i32>,
    pub priority_support: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub provider_subscription_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub plan: Option<SubscriptionPlanResponse>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionUsageResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub usage_type: String,
    pub usage_count: i32,
    pub usage_limit: Option<i32>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionPlanListResponse {
    pub plans: Vec<SubscriptionPlanResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<SubscriptionResponse>,
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

// Mock subscription provider functions (in real implementation, these would call Stripe/PayPal APIs)
async fn create_stripe_subscription(customer_id: &str, price_id: &str) -> Result<String, String> {
    // Mock implementation - in real app, this would call Stripe API
    Ok(format!("sub_mock_{}_{}", customer_id, price_id))
}

async fn cancel_stripe_subscription(subscription_id: &str) -> Result<(), String> {
    // Mock implementation - in real app, this would call Stripe API
    Ok(())
}

// Handlers
pub async fn list_subscription_plans(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let plans = sqlx::query!(
        r#"
        SELECT id, name, description, price_cents, currency, billing_cycle, features, 
               is_active, max_listings, max_rentals_per_month, priority_support, created_at, updated_at
        FROM subscription_schema.subscription_plans
        WHERE is_active = true
        ORDER BY price_cents ASC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = plans.len() as i64;

    let plan_responses: Vec<SubscriptionPlanResponse> = plans
        .into_iter()
        .map(|row| SubscriptionPlanResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            price_cents: row.price_cents,
            currency: row.currency,
            billing_cycle: row.billing_cycle,
            features: row.features,
            is_active: row.is_active,
            max_listings: row.max_listings,
            max_rentals_per_month: row.max_rentals_per_month,
            priority_support: row.priority_support,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    let response = SubscriptionPlanListResponse {
        plans: plan_responses,
        total,
        limit: total,
        offset: 0,
    };

    ok(response)
}

pub async fn get_subscription_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let plan = sqlx::query!(
        r#"
        SELECT id, name, description, price_cents, currency, billing_cycle, features, 
               is_active, max_listings, max_rentals_per_month, priority_support, created_at, updated_at
        FROM subscription_schema.subscription_plans
        WHERE id = $1 AND is_active = true
        "#,
        plan_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let plan = plan.ok_or((axum::http::StatusCode::NOT_FOUND, "Subscription plan not found"))?;

    let response = SubscriptionPlanResponse {
        id: plan.id,
        name: plan.name,
        description: plan.description,
        price_cents: plan.price_cents,
        currency: plan.currency,
        billing_cycle: plan.billing_cycle,
        features: plan.features,
        is_active: plan.is_active,
        max_listings: plan.max_listings,
        max_rentals_per_month: plan.max_rentals_per_month,
        priority_support: plan.priority_support,
        created_at: plan.created_at,
        updated_at: plan.updated_at,
    };

    ok(response)
}

pub async fn create_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateSubscriptionRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Check if plan exists and is active
    let plan = sqlx::query!(
        r#"
        SELECT id, name, price_cents, currency, billing_cycle
        FROM subscription_schema.subscription_plans
        WHERE id = $1 AND is_active = true
        "#,
        request.plan_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let plan = plan.ok_or((axum::http::StatusCode::NOT_FOUND, "Subscription plan not found"))?;

    // Check if user already has an active subscription
    let existing_subscription = sqlx::query!(
        "SELECT id FROM subscription_schema.subscriptions WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if existing_subscription.is_some() {
        return err(axum::http::StatusCode::CONFLICT, "User already has an active subscription");
    }

    // Calculate subscription period
    let now = Utc::now();
    let period_end = match plan.billing_cycle.as_str() {
        "monthly" => now + Duration::days(30),
        "yearly" => now + Duration::days(365),
        _ => return err(axum::http::StatusCode::BAD_REQUEST, "Invalid billing cycle"),
    };

    // Create subscription with provider (mock)
    let provider_subscription_id = create_stripe_subscription(&user_id.to_string(), &plan.id.to_string())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create subscription in database
    let subscription = sqlx::query!(
        r#"
        INSERT INTO subscription_schema.subscriptions 
        (user_id, plan_id, current_period_start, current_period_end, provider_subscription_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at, updated_at
        "#,
        user_id,
        request.plan_id,
        now,
        period_end,
        provider_subscription_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Initialize usage tracking
    sqlx::query!(
        r#"
        INSERT INTO subscription_schema.subscription_usage 
        (subscription_id, usage_type, usage_count, usage_limit, period_start, period_end)
        VALUES ($1, 'listings', 0, $2, $3, $4), ($1, 'rentals', 0, $3, $3, $4)
        "#,
        subscription.id,
        plan.max_listings,
        now,
        period_end
    )
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = SubscriptionResponse {
        id: subscription.id,
        user_id,
        plan_id: request.plan_id,
        status: "active".to_string(),
        current_period_start: now,
        current_period_end: period_end,
        cancel_at_period_end: false,
        canceled_at: None,
        provider_subscription_id: Some(provider_subscription_id),
        metadata: None,
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
        plan: Some(SubscriptionPlanResponse {
            id: plan.id,
            name: plan.name,
            description: None,
            price_cents: plan.price_cents,
            currency: plan.currency,
            billing_cycle: plan.billing_cycle,
            features: serde_json::json!([]),
            is_active: true,
            max_listings: plan.max_listings,
            max_rentals_per_month: None,
            priority_support: false,
            created_at: now,
            updated_at: now,
        }),
    };

    ok(response)
}

pub async fn get_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subscription_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let subscription = sqlx::query!(
        r#"
        SELECT s.id, s.user_id, s.plan_id, s.status, s.current_period_start, s.current_period_end,
               s.cancel_at_period_end, s.canceled_at, s.provider_subscription_id, s.metadata,
               s.created_at, s.updated_at,
               p.name as plan_name, p.description as plan_description, p.price_cents as plan_price_cents,
               p.currency as plan_currency, p.billing_cycle as plan_billing_cycle, p.features as plan_features,
               p.is_active as plan_is_active, p.max_listings as plan_max_listings,
               p.max_rentals_per_month as plan_max_rentals_per_month, p.priority_support as plan_priority_support
        FROM subscription_schema.subscriptions s
        JOIN subscription_schema.subscription_plans p ON s.plan_id = p.id
        WHERE s.id = $1 AND s.user_id = $2
        "#,
        subscription_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let subscription = subscription.ok_or((axum::http::StatusCode::NOT_FOUND, "Subscription not found"))?;

    let response = SubscriptionResponse {
        id: subscription.id,
        user_id: subscription.user_id,
        plan_id: subscription.plan_id,
        status: subscription.status,
        current_period_start: subscription.current_period_start,
        current_period_end: subscription.current_period_end,
        cancel_at_period_end: subscription.cancel_at_period_end,
        canceled_at: subscription.canceled_at,
        provider_subscription_id: subscription.provider_subscription_id,
        metadata: subscription.metadata,
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
        plan: Some(SubscriptionPlanResponse {
            id: subscription.plan_id,
            name: subscription.plan_name,
            description: subscription.plan_description,
            price_cents: subscription.plan_price_cents,
            currency: subscription.plan_currency,
            billing_cycle: subscription.plan_billing_cycle,
            features: subscription.plan_features,
            is_active: subscription.plan_is_active,
            max_listings: subscription.plan_max_listings,
            max_rentals_per_month: subscription.plan_max_rentals_per_month,
            priority_support: subscription.plan_priority_support,
            created_at: subscription.created_at,
            updated_at: subscription.updated_at,
        }),
    };

    ok(response)
}

pub async fn list_user_subscriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<SubscriptionFilters>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let limit = filters.limit.unwrap_or(20).min(100);
    let offset = filters.offset.unwrap_or(0);

    let subscriptions = sqlx::query!(
        r#"
        SELECT s.id, s.plan_id, s.status, s.current_period_start, s.current_period_end,
               s.cancel_at_period_end, s.canceled_at, s.provider_subscription_id, s.metadata,
               s.created_at, s.updated_at,
               p.name as plan_name, p.description as plan_description, p.price_cents as plan_price_cents,
               p.currency as plan_currency, p.billing_cycle as plan_billing_cycle, p.features as plan_features,
               p.is_active as plan_is_active, p.max_listings as plan_max_listings,
               p.max_rentals_per_month as plan_max_rentals_per_month, p.priority_support as plan_priority_support
        FROM subscription_schema.subscriptions s
        JOIN subscription_schema.subscription_plans p ON s.plan_id = p.id
        WHERE s.user_id = $1
        ORDER BY s.created_at DESC
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
        "SELECT COUNT(*) as count FROM subscription_schema.subscriptions WHERE user_id = $1",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let subscription_responses: Vec<SubscriptionResponse> = subscriptions
        .into_iter()
        .map(|row| SubscriptionResponse {
            id: row.id,
            user_id,
            plan_id: row.plan_id,
            status: row.status,
            current_period_start: row.current_period_start,
            current_period_end: row.current_period_end,
            cancel_at_period_end: row.cancel_at_period_end,
            canceled_at: row.canceled_at,
            provider_subscription_id: row.provider_subscription_id,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
            plan: Some(SubscriptionPlanResponse {
                id: row.plan_id,
                name: row.plan_name,
                description: row.plan_description,
                price_cents: row.plan_price_cents,
                currency: row.plan_currency,
                billing_cycle: row.plan_billing_cycle,
                features: row.plan_features,
                is_active: row.plan_is_active,
                max_listings: row.plan_max_listings,
                max_rentals_per_month: row.plan_max_rentals_per_month,
                priority_support: row.plan_priority_support,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
        })
        .collect();

    let response = SubscriptionListResponse {
        subscriptions: subscription_responses,
        total,
        limit,
        offset,
    };

    ok(response)
}

pub async fn cancel_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subscription_id): Path<Uuid>,
    Json(request): Json<UpdateSubscriptionRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Get subscription
    let subscription = sqlx::query!(
        r#"
        SELECT id, status, provider_subscription_id
        FROM subscription_schema.subscriptions
        WHERE id = $1 AND user_id = $2
        "#,
        subscription_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let subscription = subscription.ok_or((axum::http::StatusCode::NOT_FOUND, "Subscription not found"))?;

    if subscription.status != "active" {
        return err(axum::http::StatusCode::BAD_REQUEST, "Subscription is not active");
    }

    let cancel_at_period_end = request.cancel_at_period_end.unwrap_or(true);
    let now = Utc::now();

    if cancel_at_period_end {
        // Cancel at period end
        sqlx::query!(
            r#"
            UPDATE subscription_schema.subscriptions 
            SET cancel_at_period_end = true, updated_at = NOW()
            WHERE id = $1
            "#,
            subscription_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
    } else {
        // Cancel immediately
        if let Some(provider_subscription_id) = &subscription.provider_subscription_id {
            cancel_stripe_subscription(provider_subscription_id)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }

        sqlx::query!(
            r#"
            UPDATE subscription_schema.subscriptions 
            SET status = 'canceled', canceled_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            subscription_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    ok(serde_json::json!({
        "subscription_id": subscription_id,
        "cancel_at_period_end": cancel_at_period_end,
        "canceled_at": if cancel_at_period_end { None } else { Some(now) }
    }))
}

pub async fn get_subscription_usage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subscription_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Verify subscription belongs to user
    let subscription = sqlx::query!(
        "SELECT id FROM subscription_schema.subscriptions WHERE id = $1 AND user_id = $2",
        subscription_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if subscription.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Subscription not found");
    }

    let usage = sqlx::query!(
        r#"
        SELECT id, usage_type, usage_count, usage_limit, period_start, period_end, created_at, updated_at
        FROM subscription_schema.subscription_usage
        WHERE subscription_id = $1
        ORDER BY usage_type, period_start DESC
        "#,
        subscription_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let usage_responses: Vec<SubscriptionUsageResponse> = usage
        .into_iter()
        .map(|row| SubscriptionUsageResponse {
            id: row.id,
            subscription_id,
            usage_type: row.usage_type,
            usage_count: row.usage_count,
            usage_limit: row.usage_limit,
            period_start: row.period_start,
            period_end: row.period_end,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    ok(usage_responses)
}
