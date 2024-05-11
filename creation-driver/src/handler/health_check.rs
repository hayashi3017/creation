use axum::{response::IntoResponse, Json};

#[tracing::instrument]
pub async fn health_checker_handler() -> impl IntoResponse {
    tracing::info!("health checked");
    const MESSAGE: &str = "JWT Authentication in Rust using Axum, Postgres, and SQLX";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}
