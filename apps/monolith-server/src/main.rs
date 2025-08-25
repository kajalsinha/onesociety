mod config;
mod routes;
mod state;

use axum::{routing::get, Router};
use routes::auth as auth_routes;
use routes::health::{healthz, readyz};
use routes::messaging as messaging_routes;
use routes::payment as payment_routes;
use routes::notification as notification_routes;
use routes::admin as admin_routes;
use routes::product as product_routes;
use routes::rental as rental_routes;
use routes::review as review_routes;
use routes::subscription as subscription_routes;
use routes::user as user_routes;
use state::AppState;
use tracing_subscriber::EnvFilter;
use tower_http::trace::TraceLayer;

use crate::observability;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod openapi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::AppConfig::from_env()?;
    let pool = config.make_db_pool().await?;

    let state = AppState { db: pool };

    let openapi = openapi::ApiDoc::openapi();
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .nest("/api/v1/auth", auth_routes::router())
        .nest("/api/v1/user", user_routes::router())
        .nest("/api/v1/product", product_routes::router())
        .nest("/api/v1/rental", rental_routes::router())
        .nest("/api/v1/payment", payment_routes::router())
        .nest("/api/v1/messaging", messaging_routes::router())
        .nest("/api/v1/review", review_routes::router())
        .nest("/api/v1/subscription", subscription_routes::router())
        .nest("/api/v1/notification", notification_routes::router())
        .nest("/api/v1/admin", admin_routes::router())
        .nest("/", observability::router())
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.bind_addr.parse()?;
    tracing::info!(%addr, "starting monolith server");
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}