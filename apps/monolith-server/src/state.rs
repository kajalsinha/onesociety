use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Serialize)]
pub struct Envelope<T> {
    pub data: Option<T>,
    pub error: Option<String>,
}

pub fn ok<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, axum::Json(Envelope { data: Some(data), error: None })).into_response()
}

pub fn err(status: StatusCode, message: &str) -> Response {
    (status, axum::Json(Envelope::<()> { data: None, error: Some(message.to_string()) })).into_response()
}
