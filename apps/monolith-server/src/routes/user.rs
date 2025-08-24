use axum::{routing::get, Router};
use crate::state::{ok, AppState};

async fn ping() -> impl axum::response::IntoResponse {
	ok(serde_json::json!({ "service": "user", "status": "ok" }))
}

pub fn router() -> Router<AppState> {
	Router::new().route("/ping", get(ping))
}


