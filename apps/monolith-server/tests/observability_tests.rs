use std::net::SocketAddr;

use axum::{routing::get, Router};
use monolith_server::{config::AppConfig, observability, routes::health::healthz, state::AppState};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn metrics_endpoint_works() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = AppConfig { bind_addr: addr.to_string(), database_url: std::env::var("DATABASE_URL").unwrap() };
    let pool = config.make_db_pool().await.unwrap();
    let state = AppState { db: pool };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .merge(observability::router())
        .with_state(state);

    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(50)).await;

    let url = format!("http://{}/metrics", addr);
    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());
    let text = resp.text().await.unwrap();
    assert!(text.contains("# HELP"));
}

