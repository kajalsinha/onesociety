use axum::{routing::{get, post, put, delete}, Router};
use crate::state::{ok, AppState};
use crate::product::{create_product, get_product, list_products, update_product, delete_product, create_category, list_categories};

async fn ping() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "service": "product", "status": "ok" }))
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/ping", get(ping))
		.route("/products", post(create_product))
		.route("/products", get(list_products))
		.route("/products/:product_id", get(get_product))
		.route("/products/:product_id", put(update_product))
		.route("/products/:product_id", delete(delete_product))
		.route("/categories", post(create_category))
		.route("/categories", get(list_categories))
}


