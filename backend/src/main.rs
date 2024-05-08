use axum::{response::IntoResponse, routing::get, Json, Router};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "JWT Authentication in Rust using Axum, MySQL and sqlx.";
    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });
    Json(json_response)
}

#[tokio::main]
async fn main() {
    let app  = Router::new().route("/api/healthchecker", get(health_checker_handler));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}