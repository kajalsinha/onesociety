use axum::{extract::Path, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sqlx::types::Uuid;

use crate::state::{ok, AppState};

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct NotificationInput {
	pub user_id: Uuid,
	pub title: String,
	pub message: String,
	pub category: Option<String>,
	pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct NotificationItem {
	pub notification_id: Uuid,
	pub user_id: Uuid,
	pub title: String,
	pub message: String,
	pub category: Option<String>,
	pub data: Option<serde_json::Value>,
	pub is_read: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/notification/users/{user_id}/notifications",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List notifications", body = [NotificationItem])
    )
)]
async fn list_user_notifications(
	state: axum::extract::State<AppState>,
	Path(user_id): Path<Uuid>,
) -> axum::response::Response {
	let items: Vec<NotificationItem> = sqlx::query_as!(
		NotificationItem,
		"SELECT notification_id, user_id, title, message, category, data, is_read FROM notification_schema.notifications WHERE user_id = $1 ORDER BY created_at DESC",
		user_id
	)
	.fetch_all(&state.db)
	.await
	.unwrap_or_default();
	ok(items)
}

#[utoipa::path(
    post,
    path = "/api/v1/notification/notifications",
    request_body = NotificationInput,
    responses((status = 200, description = "Created notification"))
)]
async fn create_notification(
	state: axum::extract::State<AppState>,
	Json(input): Json<NotificationInput>,
) -> axum::response::Response {
	let rec = sqlx::query_scalar!(
		"INSERT INTO notification_schema.notifications (user_id, title, message, category, data) VALUES ($1, $2, $3, $4, $5) RETURNING notification_id",
		input.user_id,
		input.title,
		input.message,
		input.category,
		input.data
	)
	.fetch_one(&state.db)
	.await;

	match rec {
		Ok(id) => ok(serde_json::json!({"notification_id": id})),
		Err(_) => crate::state::err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "failed to create notification"),
	}
}

#[utoipa::path(
    post,
    path = "/api/v1/notification/users/{user_id}/notifications/{notification_id}/read",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("notification_id" = Uuid, Path, description = "Notification ID")
    ),
    responses((status = 200, description = "Marked as read"))
)]
async fn mark_read(
	state: axum::extract::State<AppState>,
	Path((user_id, notification_id)): Path<(Uuid, Uuid)>,
) -> axum::response::Response {
	let res = sqlx::query!(
		"UPDATE notification_schema.notifications SET is_read = true, read_at = NOW() WHERE user_id = $1 AND notification_id = $2",
		user_id,
		notification_id
	)
	.execute(&state.db)
	.await;

	match res {
		Ok(_) => ok(serde_json::json!({"status": "ok"})),
		Err(_) => crate::state::err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "failed to mark read"),
	}
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/users/:user_id/notifications", get(list_user_notifications))
		.route("/notifications", post(create_notification))
		.route("/users/:user_id/notifications/:notification_id/read", post(mark_read))
}

