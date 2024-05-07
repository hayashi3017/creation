use std::sync::Arc;

use creation_adapter::{persistence::mysql::Db, repository::mysql::DatabaseImpl};
use creation_driver::{
    config::Config, middleware::cors::setup_cors, route::create_router, AppState,
};
use dotenvy::dotenv;

use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true)
                .json(),
        )
        .init();

    let pool = DatabaseImpl {
        pool: Db::new().await,
    };
    let cors = setup_cors();

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
    }))
    .layer(ServiceBuilder::new().layer(cors));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
