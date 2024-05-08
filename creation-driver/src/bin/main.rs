use std::sync::Arc;

use creation_driver::{
    config::Config, middleware::cors::setup_cors, route::create_router, AppModule, AppState,
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

    let module = AppModule::new().await;
    let cors = setup_cors();

    let app = create_router(Arc::new(AppState {
        driver: module.clone(),
        env: config.clone(),
    }))
    .layer(ServiceBuilder::new().layer(cors));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
