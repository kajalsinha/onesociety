use std::net::SocketAddr;

use axum::Router;
use monolith_server::{config::AppConfig, routes::admin as admin_routes, state::AppState};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn admin_moderation_endpoints() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = AppConfig { bind_addr: addr.to_string(), database_url: std::env::var("DATABASE_URL").unwrap() };
    let pool = config.make_db_pool().await.unwrap();
    let state = AppState { db: pool };

    let app: Router<AppState> = admin_routes::router().with_state(state);

    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(50)).await;

    // Suspend user
    let user_id = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let url = format!("http://{}/users/moderate", addr);
    let resp = reqwest::Client::new().post(url).json(&serde_json::json!({"user_id": user_id, "action": "suspend"})).send().await.unwrap();
    assert!(resp.status().is_success());

    // Approve product: create a product directly by SQL for test
    let product_id = uuid::Uuid::new_v4();
    let insert = format!("INSERT INTO product_schema.products (product_id, owner_id, category_id, name, daily_price) VALUES ('{}','11111111-1111-1111-1111-111111111111','cccccccc-cccc-cccc-cccc-cccccccccccc','Tmp', 10.00)", product_id);
    let _ = sqlx::query(&insert).execute(&monolith_server::config::AppConfig { bind_addr: String::new(), database_url: std::env::var("DATABASE_URL").unwrap() }.make_db_pool().await.unwrap()).await.unwrap();

    let url = format!("http://{}/products/moderate", addr);
    let resp = reqwest::Client::new().post(url).json(&serde_json::json!({"product_id": product_id, "action": "approve"})).send().await.unwrap();
    assert!(resp.status().is_success());
}

