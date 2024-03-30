mod config;

use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use config::Config;
use dotenvy::dotenv;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "JWT Authentication in Rust using Axum, MySQL and sqlx.";
    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });
    Json(json_response)
}

pub struct AppState {
    db: Pool<MySql>,
    env: Config
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::init();

    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let app  = Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .with_state(Arc::new(AppState {
            db: pool.clone(),
            env: config.clone()
        }))
    ;

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}