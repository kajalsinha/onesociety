use axum::{routing::{get, post, put}, Router};
use crate::state::{ok, AppState};
use crate::rental::{create_rental, get_rental, list_rentals, update_rental, check_availability};

async fn ping() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "service": "rental", "status": "ok" }))
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/ping", get(ping))
		.route("/rentals", post(create_rental))
		.route("/rentals", get(list_rentals))
		.route("/rentals/:rental_id", get(get_rental))
		.route("/rentals/:rental_id", put(update_rental))
		.route("/availability", post(check_availability))
}


