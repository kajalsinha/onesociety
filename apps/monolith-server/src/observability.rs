use axum::{response::IntoResponse, routing::get, Router};
use once_cell::sync::Lazy;
use prometheus::{Encoder, IntCounterVec, Registry, TextEncoder};
use tower_http::trace::TraceLayer;

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);
pub static HTTP_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
	let c = IntCounterVec::new(
		prometheus::Opts::new("http_requests_total", "HTTP requests total"),
		&["method", "path", "status"],
	).unwrap();
	REGISTRY.register(Box::new(c.clone())).ok();
	c
});

async fn metrics() -> axum::response::Response {
	let encoder = TextEncoder::new();
	let metric_families = REGISTRY.gather();
	let mut buffer = Vec::new();
	encoder.encode(&metric_families, &mut buffer).unwrap();
	(
		axum::http::StatusCode::OK,
		[(axum::http::header::CONTENT_TYPE, encoder.format_type().to_string())],
		buffer,
	)
		.into_response()
}

pub fn layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
	TraceLayer::new_for_http()
}

pub fn router() -> Router {
	Router::new().route("/metrics", get(metrics))
}

