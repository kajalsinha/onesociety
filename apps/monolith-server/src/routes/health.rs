use axum::{extract::State, response::IntoResponse};
use crate::state::{ok, AppState};

pub async fn healthz(State(_state): State<AppState>) -> impl IntoResponse {
    ok(serde_json::json!({ "status": "ok" }))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // Quick DB check without long-running queries
    let res = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db).await;
    match res {
        Ok(_) => ok(serde_json::json!({ "status": "ready" })),
        Err(_) => crate::state::err(axum::http::StatusCode::SERVICE_UNAVAILABLE, "db not ready"),
    }
}
