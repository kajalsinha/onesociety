use std::net::SocketAddr;

use axum::Router;
use monolith_server::{config::AppConfig, routes::notification as notification_routes, state::AppState};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn notification_crud_flow() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = AppConfig { bind_addr: addr.to_string(), database_url: std::env::var("DATABASE_URL").unwrap() };
    let pool = config.make_db_pool().await.unwrap();
    let state = AppState { db: pool };

    let app: Router<AppState> = notification_routes::router().with_state(state);

    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(50)).await;

    // Create notification
    let user_id = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let url = format!("http://{}/notifications", addr);
    let body = serde_json::json!({
        "user_id": user_id,
        "title": "Test",
        "message": "Hello",
        "category": "system"
    });
    let resp = reqwest::Client::new().post(url).json(&body).send().await.unwrap();
    assert!(resp.status().is_success());
    let created: serde_json::Value = resp.json().await.unwrap();
    let notif_id = created["data"]["notification_id"].as_str().unwrap().parse::<uuid::Uuid>().unwrap();

    // List notifications
    let url = format!("http://{}/users/{}/notifications", addr, user_id);
    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());
    let list: serde_json::Value = resp.json().await.unwrap();
    assert!(list["data"].is_array());

    // Mark read
    let url = format!("http://{}/users/{}/notifications/{}/read", addr, user_id, notif_id);
    let resp = reqwest::Client::new().post(url).send().await.unwrap();
    assert!(resp.status().is_success());
}

