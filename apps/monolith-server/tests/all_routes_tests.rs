use axum::{Router, body::Body, http::{Request, StatusCode}};
use monolith_server::{routes, state::AppState};
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn all_domain_pings_work() {
	let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
	let state = AppState { db: pool };

	let app = Router::new()
		.nest("/api/v1/auth", routes::auth::router())
		.nest("/api/v1/user", routes::user::router())
		.nest("/api/v1/product", routes::product::router())
		.nest("/api/v1/rental", routes::rental::router())
		.nest("/api/v1/payment", routes::payment::router())
		.nest("/api/v1/messaging", routes::messaging::router())
		.nest("/api/v1/review", routes::review::router())
		.nest("/api/v1/subscription", routes::subscription::router())
		.with_state(state);

	let endpoints = [
		"/api/v1/auth/ping",
		"/api/v1/user/ping",
		"/api/v1/product/ping",
		"/api/v1/rental/ping",
		"/api/v1/payment/ping",
		"/api/v1/messaging/ping",
		"/api/v1/review/ping",
		"/api/v1/subscription/ping",
	];

	for ep in endpoints {
		let req = Request::builder().uri(ep).body(Body::empty()).unwrap();
		let resp = app.clone().oneshot(req).await.unwrap();
		assert_eq!(resp.status(), StatusCode::OK, "endpoint {} failed", ep);
	}
}


