use axum::{response::IntoResponse, Json};

use crate::response::HealthCheckResponse;

#[doc = include_str!("../openapi_docs/en/operations/health_checker_handler.md")]
#[utoipa::path(
    get,
    path = "/api/healthchecker",
    tag = "Health",
    responses(
        (status = 200, description = "The service is reachable.", body = HealthCheckResponse)
    )
)]
#[tracing::instrument]
pub async fn health_checker_handler() -> impl IntoResponse {
    tracing::info!("health checked");
    const MESSAGE: &str = "JWT Authentication in Rust using Axum, Postgres, and SQLX";

    Json(HealthCheckResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    })
}
