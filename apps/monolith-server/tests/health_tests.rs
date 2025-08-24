use std::net::SocketAddr;

use axum::{routing::get, Router};
use monolith_server::{config::AppConfig, routes::health::{healthz, readyz}, state::AppState};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn healthz_returns_ok() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    // Minimal state: we don't need DB for healthz
    let config = AppConfig { bind_addr: addr.to_string(), database_url: std::env::var("DATABASE_URL").unwrap_or_default() };
    let pool = if !config.database_url.is_empty() {
        Some(config.make_db_pool().await.unwrap())
    } else {
        None
    };

    let state = AppState { db: pool.unwrap_or_else(|| {
        // create a dummy disconnected pool lazily; healthz handler does not use it
        sqlx::PgPool::connect_lazy("postgres://invalid").unwrap()
    }) };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .with_state(state);

    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    // give the server a moment to start
    sleep(Duration::from_millis(50)).await;

    let url = format!("http://{}/healthz", addr);
    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn readyz_handles_db_unavailable_quickly() {
    // Do not cause long running tests: use a pool that fails immediately
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let state = AppState { db: pool };

    let app = Router::new()
        .route("/readyz", get(readyz))
        .with_state(state);

    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    sleep(Duration::from_millis(50)).await;

    let url = format!("http://{}/readyz", addr);
    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_server_error());
}
