use axum::{routing::post, Json, Router};
use serde::Deserialize;
use sqlx::types::Uuid;

use crate::state::{err, ok, AppState};

#[derive(Deserialize)]
pub struct ModerateUserInput { pub user_id: Uuid, pub action: String }

#[derive(Deserialize)]
pub struct ModerateProductInput { pub product_id: Uuid, pub action: String }

async fn moderate_user(state: axum::extract::State<AppState>, Json(input): Json<ModerateUserInput>) -> axum::response::Response {
	let status = match input.action.as_str() {
		"suspend" => "suspended",
		"activate" => "active",
		_ => return err(axum::http::StatusCode::BAD_REQUEST, "invalid action"),
	};

	let res = sqlx::query!(
		"UPDATE user_schema.users SET status = $2 WHERE user_id = $1",
		input.user_id,
		status
	).execute(&state.db).await;

	match res {
		Ok(_) => ok(serde_json::json!({"status": status})),
		Err(_) => err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "moderation failed"),
	}
}

async fn moderate_product(state: axum::extract::State<AppState>, Json(input): Json<ModerateProductInput>) -> axum::response::Response {
	let approved = matches!(input.action.as_str(), "approve");
	let status = if approved { "active" } else { "rejected" };
	let res = sqlx::query!(
		"UPDATE product_schema.products SET status = $2 WHERE product_id = $1",
		input.product_id,
		status
	).execute(&state.db).await;
	match res {
		Ok(_) => ok(serde_json::json!({"status": status})),
		Err(_) => err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "moderation failed"),
	}
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/users/moderate", post(moderate_user))
		.route("/products/moderate", post(moderate_product))
}

