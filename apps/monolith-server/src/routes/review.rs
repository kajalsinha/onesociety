use axum::{routing::{get, post}, Router};
use crate::state::AppState;
use crate::review::{
    create_product_review,
    create_user_review,
    list_product_reviews,
    get_product_review_stats,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/product-reviews", post(create_product_review))
        .route("/user-reviews", post(create_user_review))
        .route("/product-reviews", get(list_product_reviews))
        .route("/products/:product_id/review-stats", get(get_product_review_stats))
}
