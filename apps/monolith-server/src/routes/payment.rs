use axum::{
    routing::{get, post, put},
    Router,
};
use crate::state::AppState;
use crate::payment::{
    create_payment_method, list_payment_methods, create_payment_intent, confirm_payment_intent,
    get_payment_intent, list_payment_intents,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/payment-methods", post(create_payment_method))
        .route("/payment-methods", get(list_payment_methods))
        .route("/payment-intents", post(create_payment_intent))
        .route("/payment-intents", get(list_payment_intents))
        .route("/payment-intents/:id", get(get_payment_intent))
        .route("/payment-intents/:id/confirm", post(confirm_payment_intent))
}


