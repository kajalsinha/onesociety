use std::net::SocketAddr;

use axum::Router;
use monolith_server::{routes::auth, state::{AppState}};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn auth_ping_works() {
	let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

	let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
	let state = AppState { db: pool };

	let app = Router::new()
		.nest("/api/v1/auth", auth::router())
		.with_state(state);

	let listener = std::net::TcpListener::bind(addr).unwrap();
	listener.set_nonblocking(true).unwrap();
	let addr = listener.local_addr().unwrap();
	tokio::spawn(async move {
		axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
	});

	sleep(Duration::from_millis(50)).await;

	let url = format!("http://{}/api/v1/auth/ping", addr);
	let resp = reqwest::get(url).await.unwrap();
	assert!(resp.status().is_success());
}


