use axum::{routing::{get, post}, Router};
use crate::state::{ok, AppState};
// use crate::auth::{signup, login, refresh, get_profile, update_profile};

async fn ping() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "service": "auth", "status": "ok" }))
}

// Stub functions for testing
async fn signup() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "message": "signup stub" }))
}

async fn login() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "message": "login stub" }))
}

async fn refresh() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "message": "refresh stub" }))
}

async fn get_profile() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "message": "get_profile stub" }))
}

async fn update_profile() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "message": "update_profile stub" }))
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/ping", get(ping))
		.route("/signup", post(signup))
		.route("/login", post(login))
		.route("/refresh", post(refresh))
		.route("/profile", get(get_profile))
		.route("/profile", post(update_profile))
}


