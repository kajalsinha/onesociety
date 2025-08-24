use axum::{
    routing::{get, post, put},
    Router,
};
use crate::state::AppState;
use crate::subscription::{
    list_subscription_plans, get_subscription_plan, create_subscription, get_subscription,
    list_user_subscriptions, cancel_subscription, get_subscription_usage,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/plans", get(list_subscription_plans))
        .route("/plans/:id", get(get_subscription_plan))
        .route("/subscriptions", post(create_subscription))
        .route("/subscriptions", get(list_user_subscriptions))
        .route("/subscriptions/:id", get(get_subscription))
        .route("/subscriptions/:id/cancel", put(cancel_subscription))
        .route("/subscriptions/:id/usage", get(get_subscription_usage))
}


